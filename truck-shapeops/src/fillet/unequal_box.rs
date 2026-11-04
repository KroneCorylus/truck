use truck_modeling::*;

/// Only unsplit rectangular boxes. Local u/v/d point inward from the corner;
/// a is the radius along v and b along u. The retained section at depth d is
/// [a-sqrt(2ad-d²),U] × [b-sqrt(2bd-d²),V], clipping each inset to zero at d=r.
pub(in crate::fillet) fn build(
    shell: &Shell,
    selected: &[(EdgeID, f64)],
    tol: f64,
) -> Option<Shell> {
    if selected.len() != 2 || shell.len() != 6 || !shell.is_geometric_consistent() {
        return None;
    }
    let mut selected = selected.to_vec();
    selected.sort_by(|a, b| b.1.total_cmp(&a.1));
    let (a, b) = (selected[0].1, selected[1].1);
    let clearance = (8. * tol).max(100. * TOLERANCE);
    if b <= clearance || a - b <= clearance {
        return None;
    }
    let edges: Vec<_> = selected
        .iter()
        .map(|(id, _)| shell.edge_iter().find(|e| e.id() == *id))
        .collect::<Option<_>>()?;
    let corner = [edges[0].front(), edges[0].back()]
        .into_iter()
        .find(|v| *v == edges[1].front() || *v == edges[1].back())?;
    let away = |e: &Edge| {
        if e.front() == corner {
            e.back().point() - corner.point()
        } else {
            e.front().point() - corner.point()
        }
    };
    let (dv, du) = (away(&edges[0]), away(&edges[1]));
    let (u_len, v_len) = (du.magnitude(), dv.magnitude());
    let (u, v) = (du / u_len, dv / v_len);
    if u.dot(v).abs() > TOLERANCE {
        return None;
    }
    let top = shell.iter().find(|f| {
        selected
            .iter()
            .all(|(id, _)| f.edge_iter().any(|e| e.id() == *id))
    })?;
    let (Elementary::Plane(plane), forward) = top.oriented_surface().elementary()? else {
        return None;
    };
    let d = plane.normal() * if forward { -1. } else { 1. };
    if d.dot(u).abs() > TOLERANCE || d.dot(v).abs() > TOLERANCE {
        return None;
    }
    let origin = corner.point();
    let local = |p: Point3| {
        let x = p - origin;
        Point3::new(x.dot(u), x.dot(v), x.dot(d))
    };
    let world = |p: Point3| origin + u * p.x + v * p.y + d * p.z;
    let w = shell
        .vertex_iter()
        .map(|p| local(p.point()).z)
        .fold(0_f64, f64::max);
    if a + clearance >= u_len || b + clearance >= v_len || a + clearance >= w {
        return None;
    }
    let lengths = [u_len, v_len, w];
    let mut masks = std::collections::HashSet::new();
    for vertex in shell.vertex_iter() {
        let p = local(vertex.point());
        let mut mask = 0;
        for k in 0..3 {
            if (p[k] - lengths[k]).abs() <= TOLERANCE {
                mask |= 1 << k;
            } else if p[k].abs() > TOLERANCE {
                return None;
            }
        }
        masks.insert(mask);
    }
    if masks.len() != 8 {
        return None;
    }
    let mut face_keys = Vec::new();
    for face in shell {
        let boundaries = face.boundaries();
        if boundaries.len() != 1 || boundaries[0].len() != 4 {
            return None;
        }
        let (Elementary::Plane(plane), forward) = face.oriented_surface().elementary()? else {
            return None;
        };
        let n = plane.normal() * if forward { 1. } else { -1. };
        let key = (0..6).find(|&key| {
            let k = key / 2;
            let axis = [u, v, d][k];
            let value = if key % 2 == 0 { 0. } else { lengths[k] };
            n.dot(axis) * if key % 2 == 0 { -1. } else { 1. } > 1. - TOLERANCE
                && face
                    .vertex_iter()
                    .all(|p| (local(p.point())[k] - value).abs() <= TOLERANCE)
        })?;
        if face_keys.contains(&key) {
            return None;
        }
        for edge in face.edge_iter() {
            let curve = edge.curve();
            // Only exact linear geometry, including degree-one imported splines.
            let straight = match curve {
                Curve::Line(_) => true,
                Curve::BSplineCurve(ref c) => c.degree() == 1 && c.control_points().len() == 2,
                Curve::NurbsCurve(ref c) => {
                    let points = c.control_points();
                    let (lo, hi) = c.range_tuple();
                    c.degree() == 1
                        && points.len() == 2
                        && c.is_clamped()
                        && c.knot_vec().iter().all(|t| t.is_finite())
                        && hi > lo
                        && (hi - lo).is_finite()
                        && (1. / (hi - lo)).is_finite()
                        && points.iter().all(|p| {
                            p.w != 0.
                                && (0..4).all(|i| p[i].is_finite())
                                && (0..3).all(|i| (p[i] / p.w).is_finite())
                        })
                        && points[0].w.signum() == points[1].w.signum()
                }
                _ => false,
            };
            if !straight {
                return None;
            }
            let p = local(edge.front().point());
            let q = local(edge.back().point());
            if (0..3).filter(|&k| (p[k] - q[k]).abs() > TOLERANCE).count() != 1 {
                return None;
            }
        }
        face_keys.push(key);
    }
    let seam = seam(a, b, origin, u, v, d, tol)?;
    let q = a - (2. * a * b - b * b).sqrt();
    let points = [
        Point3::new(0., 0., w),
        Point3::new(u_len, 0., w),
        Point3::new(u_len, v_len, w),
        Point3::new(0., v_len, w),
        Point3::new(0., 0., a),
        Point3::new(0., v_len, a),
        Point3::new(a, b, 0.),
        Point3::new(u_len, b, 0.),
        Point3::new(u_len, v_len, 0.),
        Point3::new(a, v_len, 0.),
        Point3::new(q, 0., b),
        Point3::new(u_len, 0., b),
    ];
    let vertices: Vec<_> = points.iter().map(|&p| Vertex::new(world(p))).collect();
    let arc = |i: usize, j: usize, center: Point3| {
        let middle = center
            + ((points[i] - center) + (points[j] - center)).normalize()
                * (points[i] - center).magnitude();
        builder::circle_arc(&vertices[i], &vertices[j], world(middle))
    };
    let a_arc = arc(9, 5, Point3::new(a, v_len, a));
    let b_arc = arc(7, 11, Point3::new(u_len, b, b));
    let mut sa: Surface = ExtrudedCurve::by_extrusion(a_arc.oriented_curve(), -v * v_len).into();
    let mut sb: Surface = ExtrudedCurve::by_extrusion(b_arc.oriented_curve(), -u * u_len).into();
    if sa.normal(0.5, 0.5).dot(-u - d) < 0. {
        sa.invert();
    }
    if sb.normal(0.5, 0.5).dot(-v - d) < 0. {
        sb.invert();
    }
    let mut cache = std::collections::HashMap::new();
    cache.insert((9, 5), a_arc);
    cache.insert((7, 11), b_arc);
    cache.insert((10, 4), arc(10, 4, Point3::new(a, 0., a)));
    cache.insert((6, 10), Edge::new(&vertices[6], &vertices[10], seam));
    let mut wire = |ids: &[usize]| {
        let mut edges = Vec::new();
        for i in 0..ids.len() {
            let (a, b) = (ids[i], ids[(i + 1) % ids.len()]);
            let edge = if let Some(e) = cache.get(&(a, b)) {
                e.clone()
            } else if let Some(e) = cache.get(&(b, a)) {
                e.inverse()
            } else {
                let e = builder::line(&vertices[a], &vertices[b]);
                cache.insert((a, b), e.clone());
                e
            };
            edges.push(edge);
        }
        Wire::from(edges)
    };
    let loops: [&[usize]; 6] = [
        &[0, 3, 5, 4],
        &[1, 11, 7, 8, 2],
        &[0, 4, 10, 11, 1],
        &[3, 2, 8, 9, 5],
        &[6, 9, 8, 7],
        &[0, 1, 2, 3],
    ];
    let mut result = Shell::new();
    for (face, key) in shell.iter().zip(face_keys) {
        let mut boundary = wire(loops[key]);
        let surface = face.oriented_surface();
        let p: Vec<_> = boundary.iter().map(|e| e.front().point()).collect();
        let normal = (p[1] - p[0]).cross(p[2] - p[1]);
        if normal.dot(surface.normal(0.5, 0.5)) < 0. {
            boundary.invert();
        }
        result.push(Face::try_new(vec![boundary], surface).ok()?);
    }
    for (ids, surface) in [(&[6, 10, 4, 5, 9][..], sa), (&[6, 7, 11, 10][..], sb)] {
        let boundary = wire(ids);
        // Oppose the already assembled support uses; this also handles either frame handedness.
        let first = boundary.iter().find_map(|e| {
            result
                .edge_iter()
                .find(|other| other.id() == e.id())
                .map(|other| (e.orientation(), other.orientation()))
        })?;
        let boundary = if first.0 == first.1 {
            boundary.inverse()
        } else {
            boundary
        };
        result.push(Face::try_new(vec![boundary], surface).ok()?);
    }
    Some(result)
}

fn seam(a: f64, b: f64, o: Point3, u: Vector3, v: Vector3, d: Vector3, tol: f64) -> Option<Curve> {
    let eps = (tol * 1e-5).min(TOLERANCE * 0.01);
    // Keep products, square roots and reciprocals away from overflow/underflow.
    if !tol.is_finite()
        || !eps.is_finite()
        || eps <= 0.
        || !(1e-50..=1e50).contains(&b)
        || !(b..=1e50).contains(&a)
        || !(0..3).all(|i| [o[i], u[i], v[i], d[i]].iter().all(|x| x.is_finite()))
    {
        return None;
    }
    let axes = [u, v, d];
    let defect = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| (axes[i].dot(axes[j]) - if i == j { 1. } else { 0. }).abs())
                .sum::<f64>()
        })
        .fold(0_f64, f64::max)
        + 64. * f64::EPSILON;
    if defect >= 0.01 {
        return None;
    }
    // For F=[u v d], ||F-Q||₂ <= ||FᵀF-I||₂ <= defect, where Q is
    // its orthogonal polar factor. Local positions have norm < 2a; 8a
    // also covers projection/reconstruction through the admitted frame.
    let frame_error = 8. * a * defect;
    // Absolute forward-error allowance, using EPSILON (twice unit roundoff).
    // Local point/derivative/control arithmetic costs < 128 eps*a; world
    // conversion < 16 eps*(|o|₁+8a). The cubic basis evaluator has three
    // recurrence levels (including `1-b`), then four products/additions.
    // On triple-knot spans its nonzero denominators are the span width;
    // basis absolute errors, including joins/endpoints, cost < 128 eps.
    // 1024 covers these errors, frame extraction arithmetic and bound rounding.
    // This is a conservative FP error model, not formal interval certification.
    let arithmetic_error = 1024. * f64::EPSILON * (o.x.abs() + o.y.abs() + o.z.abs() + 8. * a);
    if frame_error + arithmetic_error >= eps * 0.5 {
        return None;
    }
    // d=b*t² removes the square-root singularity. For g=t*sqrt(A-Bt²),
    // max |g''''|=15*A²*B²/(A-B)^(7/2) <= 60b for either coordinate
    // when a>=b. ||F||₂<2 gives M<=120*sqrt(2)*b in world space.
    // Reserve half the unchanged target for arithmetic/frame error. Actual
    // rounded knot widths are checked below; triple knots preserve each
    // piece's Hermite bound M*h⁴/384 without a continuity solve.
    let bound = 120. * 2_f64.sqrt() * b;
    let truncation = eps * 0.5;
    let count = (bound / (384. * truncation)).powf(0.25).ceil() + 1.;
    if !(1. ..=4096.).contains(&count) {
        return None;
    }
    let count = count as usize;
    let point = |t: f64| {
        Point3::new(
            a - t * (2. * a * b - b * b * t * t).sqrt(),
            b - b * t * (2. - t * t).sqrt(),
            b * t * t,
        )
    };
    let derivative = |t: f64| {
        Vector3::new(
            -(2. * a * b - 2. * b * b * t * t) / (2. * a * b - b * b * t * t).sqrt(),
            -b * (2. - 2. * t * t) / (2. - t * t).sqrt(),
            2. * b * t,
        )
    };
    let mut controls = Vec::new();
    let mut knots = vec![0.; 4];
    for i in 0..count {
        let (t, s) = (i as f64 / count as f64, (i + 1) as f64 / count as f64);
        let h = s - t;
        if bound * h.powi(4) / 384. >= truncation {
            return None;
        }
        let p = [
            point(t),
            point(t) + derivative(t) * h / 3.,
            point(s) - derivative(s) * h / 3.,
            point(s),
        ];
        if p.windows(2)
            .any(|p| p[1].x > p[0].x + eps || p[1].y > p[0].y + eps || p[1].z < p[0].z - eps)
        {
            return None;
        }
        if i == 0 {
            controls.push(o + u * p[0].x + v * p[0].y + d * p[0].z);
        }
        controls.extend(p[1..].iter().map(|p| o + u * p.x + v * p.y + d * p.z));
        knots.extend(std::iter::repeat_n(s, if i + 1 == count { 4 } else { 3 }));
    }
    Some(BSplineCurve::new(KnotVec::from(knots), controls).into())
}

#[cfg(test)]
mod numerical_tests {
    use super::*;
    #[test]
    fn translated_seam_requires_roundoff_budget() {
        let c = seam(
            1.,
            0.5,
            Point3::new(1e9, 0., 0.),
            Vector3::unit_x(),
            Vector3::unit_y(),
            Vector3::unit_z(),
            0.001,
        );
        if let Some(c) = c {
            // 80-digit Decimal cylinder intersection at the exact binary64 t=25/53.
            let reference = 0.541_608_631_717_976_2_f64;
            let error = ((c.subs(25. / 53.).x - 1e9) - reference).abs();
            eprintln!("production spline knot error={error:e}; target=1e-8");
            assert!(error <= 1e-8);
        }
    }
    #[test]
    fn numerical_admission_controls() {
        let make = |x, v| {
            seam(
                1.,
                0.5,
                Point3::new(x, 0., 0.),
                Vector3::unit_x(),
                v,
                Vector3::unit_z(),
                0.001,
            )
        };
        // Same frame and radii: only the world arithmetic allowance changes.
        for x in [0., 10., 21_900.] {
            let c = make(x, Vector3::unit_y()).unwrap();
            for i in 0..=8192 {
                let t = i as f64 / 8192.;
                let depth = 0.5 * t * t;
                let expected = Point3::new(
                    1. - (depth * (2. - depth)).sqrt(),
                    0.5 - (depth * (1. - depth)).sqrt(),
                    depth,
                );
                assert!((c.subs(t) - Vector3::new(x, 0., 0.)).distance(expected) < 1e-8);
            }
            if let Curve::BSplineCurve(c) = c {
                for &t in c.knot_vec().iter() {
                    for t in [t, t.next_down().max(0.), t.next_up().min(1.)] {
                        let depth = 0.5 * t * t;
                        let expected = Point3::new(
                            1. - (depth * (2. - depth)).sqrt(),
                            0.5 - (depth * (1. - depth)).sqrt(),
                            depth,
                        );
                        assert!((c.subs(t) - Vector3::new(x, 0., 0.)).distance(expected) < 1e-8);
                    }
                }
            }
        }
        assert!(make(22_000., Vector3::unit_y()).is_none());
        assert!(make(1e9, Vector3::unit_y()).is_none());
        // Below the old 1e-6 orthogonality gate; only frame budget changes.
        assert!(make(0., Vector3::new(6e-10, 1., 0.).normalize()).is_some());
        assert!(make(0., Vector3::new(7e-10, 1., 0.).normalize()).is_none());
        assert!(make(0., Vector3::new(0., 1. + 4e-10, 0.)).is_none());
    }
}
