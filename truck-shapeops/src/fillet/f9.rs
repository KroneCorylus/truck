use super::*;

pub(super) fn build(shell: &Shell, selected: &HashSet<EdgeID>, r: f64, tol: f64) -> Option<Shell> {
    if shell.len() != 4 || selected.len() != 2 || shell.iter().any(|f| f.boundaries().len() != 1) {
        return None;
    }
    let curved = shell
        .iter()
        .position(|f| matches!(f.surface(), Surface::Extruded(_)))?;
    let Surface::Extruded(extrusion) = shell[curved].surface() else {
        return None;
    };
    let Curve::BSplineCurve(basis) = extrusion.entity_curve() else {
        return None;
    };
    if basis.degree() != 2 || basis.control_points().len() != 3 {
        return None;
    }
    let mut seen = HashSet::default();
    let edges: Vec<_> = shell.edge_iter().filter(|e| seen.insert(e.id())).collect();
    if edges.len() != 6 {
        return None;
    }
    let chosen: Vec<_> = edges
        .iter()
        .filter(|e| selected.contains(&e.id()))
        .collect();
    if chosen.len() != 2 || chosen.iter().any(|e| !matches!(e.curve(), Curve::Line(_))) {
        return None;
    }
    let ruling = chosen
        .iter()
        .position(|e| shell[curved].edge_iter().any(|x| x.id() == e.id()))?;
    let a = chosen[ruling];
    let b = chosen[1 - ruling];
    let p = [a.front(), a.back()]
        .into_iter()
        .find(|v| *v == b.front() || *v == b.back())?;
    let other = |e: &Edge| {
        if e.front() == p {
            e.back().point()
        } else {
            e.front().point()
        }
    };
    let length = p.point().distance(other(a));
    let width = p.point().distance(other(b)) / 2.;
    if !length.is_finite() || !width.is_finite() || length <= 0. || width <= 0. {
        return None;
    }
    let u = (p.point() - other(a)) / length;
    let y = (other(b) - p.point()) / (2. * width);
    let mut z = u.cross(y).normalize();
    let mid = basis.subs((basis.range_tuple().0 + basis.range_tuple().1) / 2.);
    if (mid - p.point()).dot(z) < 0. {
        z = -z;
    }
    let origin = p.point() - length * u + width * y;
    let xyz = |p: Point3| {
        let v = p - origin;
        Vector3::new(v.dot(u), v.dot(y), v.dot(z))
    };
    let point = |x: f64, t: f64, d: f64| origin + x * u + t * y + d * z;
    let k = 26_f64.sqrt();
    let depth = width / (2. * k);
    let eps = (tol / 10.).min(width * 1e-8);
    if !r.is_finite()
        || r <= 10. * tol
        || r > width * 0.015
        || u.dot(y).abs() > 1e-10
        || (length - width * k / 5.).abs() > eps
        || extrusion.extruding_vector().cross(u).magnitude() > eps
        || u.cross(y).dot(z) > 0.
    {
        return None;
    }
    let (lo, hi) = basis.range_tuple();
    if !lo.is_finite()
        || !hi.is_finite()
        || lo >= hi
        || (0..3).any(|i| basis.knot(i) != lo || basis.knot(i + 3) != hi)
    {
        return None;
    }
    let cp: Vec<_> = basis.control_points().iter().map(|p| xyz(*p)).collect();
    let cap = cp[0].x;
    let sign = cp[0].y.signum();
    let expected = [
        Vector3::new(cap, sign * width, 0.),
        Vector3::new(cap - 10. * depth, 0., 2. * depth),
        Vector3::new(cap, -sign * width, 0.),
    ];
    if cp
        .iter()
        .zip(expected)
        .any(|(p, q)| !finite_vector(*p) || (*p - q).magnitude() > eps)
    {
        return None;
    }
    let other_cap = cap + extrusion.extruding_vector().dot(u);
    if cap.min(other_cap) > eps
        || cap.max(other_cap) < length - eps
        || shell[curved]
            .oriented_surface()
            .normal((lo + hi) / 2., 0.5)
            .dot(z)
            <= 0.
    {
        return None;
    }
    let mut planes = Vec::new();
    for (i, f) in shell.iter().enumerate() {
        if i == curved {
            continue;
        }
        let Surface::Plane(s) = f.oriented_surface() else {
            return None;
        };
        if f.vertex_iter()
            .any(|v| (v.point() - s.origin()).dot(s.normal()).abs() > eps)
        {
            return None;
        }
        planes.push((i, s.normal().normalize()));
    }
    let wall = planes.iter().find(|(_, n)| n.dot(-z) > 1. - 1e-10)?.0;
    let low = planes
        .iter()
        .find(|(_, n)| n.dot((-u - 5. * z) / k) > 1. - 1e-10)?
        .0;
    let high = planes
        .iter()
        .find(|(_, n)| n.dot((u + 5. * z) / k) > 1. - 1e-10)?
        .0;
    if !shell[wall].edge_iter().any(|e| e.id() == a.id())
        || !shell[wall].edge_iter().any(|e| e.id() == b.id())
        || !shell[high].edge_iter().any(|e| e.id() == b.id())
    {
        return None;
    }
    // Two full quadratic cap edges and four full straight edges fix the original trim domain.
    let mut profiles = 0;
    for edge in &edges {
        match edge.curve() {
            Curve::BSplineCurve(c) => {
                if c.degree() != 2 || c.control_points().len() != 3 {
                    return None;
                }
                let (lo, hi) = c.range_tuple();
                if !lo.is_finite()
                    || !hi.is_finite()
                    || lo >= hi
                    || (0..3).any(|i| c.knot(i) != lo || c.knot(i + 3) != hi)
                {
                    return None;
                }
                let cp: Vec<_> = c.control_points().iter().map(|p| xyz(*p)).collect();
                let cap = if cp[0].x.abs() < eps { 0. } else { length };
                let sign = cp[0].y.signum();
                let expected = [
                    Vector3::new(cap, sign * width, 0.),
                    Vector3::new(cap - 10. * depth, 0., 2. * depth),
                    Vector3::new(cap, -sign * width, 0.),
                ];
                if cp
                    .iter()
                    .zip(expected)
                    .any(|(p, q)| !finite_vector(*p) || (*p - q).magnitude() > eps)
                {
                    return None;
                }
                profiles += 1;
            }
            Curve::Line(_) => {
                for v in [edge.front(), edge.back()] {
                    let p = xyz(v.point());
                    if p.z.abs() > eps
                        || (p.y.abs() - width).abs() > eps
                        || p.x.abs().min((p.x - length).abs()) > eps
                    {
                        return None;
                    }
                }
            }
            _ => return None,
        }
    }
    if profiles != 2 {
        return None;
    }
    let f = |t: f64| depth - t * t / (2. * width * k);
    let slope = |t: f64| -t / (width * k);
    let g = |t: f64| f(t) - r / (1. + slope(t).powi(2)).sqrt() - r;
    if g(-width) >= 0. || g(0.) <= 0. || r >= width * k {
        return None;
    }
    let (mut left, mut right) = (-width, 0.);
    for _ in 0..64 {
        let t = (left + right) / 2.;
        if g(t) > 0. {
            right = t;
        } else {
            left = t;
        }
    }
    let t = (left + right) / 2.;
    let dt = f(t);
    let cy = t + r * slope(t) / (1. + slope(t).powi(2)).sqrt();
    let cx = length - (5. + k) * r;
    let dc = r * (1. + 5. / k);
    if !(r < dc && dc + 2. * tol < dt && dt < depth && cx > tol) {
        return None;
    }
    // Concavity and r < width*k keep the offset regular; these two contact roots
    // bound the retained arcs before either reaches the remote profile branch.
    let ac = (1. - dc / r).acos();
    let at = (1. - dt / r).acos();
    let bp = |d: f64| (width * width - 2. * width * k * d).sqrt();
    let vertices = [
        point(0., cy, 0.),
        point(cx, cy, 0.),
        point(cx, width, 0.),
        point(0., width, 0.),
        point(cx + r / k, cy - r / k, dc),
        point(length - 5. * dt, t, dt),
        point(-5. * dt, t, dt),
        point(length - 5. * dc, bp(dc), dc),
    ]
    .map(Vertex::new);
    let [a, b, c, d, e, fv, g, h] = &vertices;
    let section = |axis_x: f64, axis_y: f64, end: f64, direction: Vector3| {
        let center = point(axis_x, axis_y, r);
        let start = Vertex::new(center - r * z);
        let finish = Vertex::new(center - r * end.cos() * z + r * end.sin() * direction);
        circle(&start, &finish, center).curve()
    };
    let ruling_surface: Surface =
        ExtrudedCurve::by_extrusion(section(-5. * dt, cy, at, -y), u * (length + 5. * dt)).into();
    let cap_surface: Surface =
        ExtrudedCurve::by_extrusion(section(cx, cy - r, ac, u), y * (width - cy + r)).into();
    // Affine images of a rational circle preserve the exact ellipse trims.
    let ellipse = |a: &Vertex,
                   b: &Vertex,
                   start: f64,
                   end: f64,
                   offset: Point3,
                   sine: Vector3,
                   cosine: Vector3| {
        let p = Vertex::new(Point3::new(start.sin(), start.cos(), 0.));
        let q = Vertex::new(Point3::new(end.sin(), end.cos(), 0.));
        let mut curve = circle(&p, &q, Point3::origin()).curve();
        curve.transform_by(Matrix4::from_cols(
            sine.extend(0.),
            cosine.extend(0.),
            sine.cross(cosine).normalize().extend(0.),
            offset.to_homogeneous(),
        ));
        Edge::new(a, b, curve)
    };
    let ab = builder::line(a, b);
    let bc = builder::line(b, c);
    let cd = builder::line(c, d);
    let da = builder::line(d, a);
    let be = ellipse(b, e, 0., ac, point(cx, cy, r), r * (u - y), -r * z);
    let ef = ellipse(
        e,
        fv,
        ac,
        at,
        point(length - 5. * r, cy, r),
        -r * y,
        r * (5. * u - z),
    );
    let ag = ellipse(
        a,
        g,
        0.,
        at,
        point(-5. * r, cy, r),
        -r * y,
        r * (5. * u - z),
    );
    let fg = builder::line(fv, g);
    let eh = builder::line(e, h);
    let profile = |a: &Vertex, b: &Vertex, cap: f64| {
        let ta = xyz(a.point()).y;
        let tb = xyz(b.point()).y;
        let pa = point(cap - 5. * f(ta), ta, f(ta));
        let pb = point(cap - 5. * f(tb), tb, f(tb));
        let tangent = y + slope(ta) * (z - 5. * u);
        Edge::new(
            a,
            b,
            BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![pa, pa + tangent * ((tb - ta) / 2.), pb],
            )
            .into(),
        )
    };
    let dg = profile(d, g, 0.);
    let fh = profile(fv, h, length);
    let guide = (0..=64)
        .map(|i| {
            let angle = ac * i as f64 / 64.;
            let depth = r * (1. - angle.cos());
            point(cx + r * angle.sin(), bp(depth), depth)
        })
        .collect();
    let ch = guided_intersection(
        c,
        h,
        &shell[curved].oriented_surface(),
        &cap_surface,
        (guide, true),
        [y, z],
        tol,
    )?;
    let specs = [
        (
            vec![ab.clone(), bc.clone(), cd.clone(), da.clone()],
            shell[wall].oriented_surface(),
        ),
        (
            vec![da.inverse(), dg.clone(), ag.inverse()],
            shell[low].oriented_surface(),
        ),
        (
            vec![ef.clone(), fh.clone(), eh.inverse()],
            shell[high].oriented_surface(),
        ),
        (
            vec![
                cd.inverse(),
                ch.clone(),
                fh.inverse(),
                fg.clone(),
                dg.inverse(),
            ],
            shell[curved].oriented_surface(),
        ),
        (
            vec![
                ab.inverse(),
                ag.clone(),
                fg.inverse(),
                ef.inverse(),
                be.inverse(),
            ],
            ruling_surface,
        ),
        (
            vec![bc.inverse(), be.clone(), eh.clone(), ch.inverse()],
            cap_surface,
        ),
    ];
    let mut faces = Vec::new();
    for (edges, surface) in specs {
        faces.push(Face::try_new(vec![order_wire(edges).ok()?], surface).ok()?);
    }
    let indices = [wall, low, high, curved];
    let mut result = Shell::new();
    for i in 0..4 {
        result.push(faces[indices.iter().position(|j| *j == i)?].clone());
    }
    result.extend(faces.into_iter().skip(4));
    Some(result)
}
