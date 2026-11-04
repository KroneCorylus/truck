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
    if basis.degree() != 3 || basis.control_points().len() != 4 {
        return None;
    }
    let (lo, hi) = basis.range_tuple();
    if !lo.is_finite()
        || !hi.is_finite()
        || lo >= hi
        || !(hi - lo).is_finite()
        || (0..4).any(|i| basis.knot(i) != lo || basis.knot(i + 4) != hi)
    {
        return None;
    }
    let mut seen = HashSet::default();
    let edges: Vec<_> = shell.edge_iter().filter(|e| seen.insert(e.id())).collect();
    if edges.len() != 6 || shell.vertex_iter().any(|v| !finite_point(v.point())) {
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
    let width = p.point().distance(other(b));
    if !length.is_finite() || !width.is_finite() || length <= 0. || width <= 0. {
        return None;
    }
    let u = (p.point() - other(a)) / length;
    let y = (other(b) - p.point()) / width;
    let mut z = u.cross(y).normalize();
    let mid = basis.subs(lo + (hi - lo) / 2.);
    if (mid - p.point()).dot(z) < 0. {
        z = -z;
    }
    let origin = p.point() - length * u;
    let xyz = |p: Point3| {
        let v = p - origin;
        Vector3::new(v.dot(u), v.dot(y), v.dot(z))
    };
    let point = |x: f64, t: f64, d: f64| origin + x * u + t * y + d * z;
    let k = 26_f64.sqrt();
    let scale = width / 10.;
    let depth = scale * profile_depth((7. - 19_f64.sqrt()) / 6.);
    let eps = (tol / 10.).min(width * 1e-8);
    let coordinate_bound = basis
        .control_points()
        .iter()
        .map(|p| p.to_vec().magnitude())
        .fold(origin.to_vec().magnitude(), f64::max)
        + extrusion.extruding_vector().magnitude()
        + width
        + length;
    let arithmetic = 256. * f64::EPSILON * coordinate_bound;
    if !arithmetic.is_finite() || arithmetic > eps / 16. {
        return None;
    }
    if !r.is_finite()
        || r <= 10. * tol
        || r > width * 0.075
        || u.dot(y).abs() > 1e-10
        || (length - width * k / 5.).abs() > eps
        || extrusion.extruding_vector().cross(u).magnitude() > eps
        || u.cross(y).dot(z) < 0.
    {
        return None;
    }
    let cp: Vec<_> = basis.control_points().iter().map(|p| xyz(*p)).collect();
    let cap = cp[0].x;
    let mut expected = [
        Vector3::new(cap, 0., 0.),
        Vector3::new(cap + 10. * scale / k, 2. * scale, 50. * scale / k),
        Vector3::new(cap + 6. * scale / k, 4. * scale, 30. * scale / k),
        Vector3::new(cap, width, 0.),
    ];
    if cp[0].y > width / 2. {
        expected.reverse();
    }
    if cp
        .iter()
        .zip(expected)
        .any(|(p, q)| !finite_vector(*p) || (*p - q).magnitude() > eps)
    {
        return None;
    }
    let other_cap = cap + extrusion.extruding_vector().dot(u);
    let normal = shell[curved]
        .oriented_surface()
        .normal(lo + (hi - lo) / 2., 0.5);
    if !finite_vector(normal) || !normal.magnitude().is_finite() {
        return None;
    }
    if cap.min(other_cap) > eps || cap.max(other_cap) < length - eps || normal.dot(z) <= 0. {
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
        .find(|(_, n)| n.dot((-u + 0.2 * z).normalize()) > 1. - 1e-10)?
        .0;
    let high = planes
        .iter()
        .find(|(_, n)| n.dot((u - 0.2 * z).normalize()) > 1. - 1e-10)?
        .0;
    if !shell[wall].edge_iter().any(|e| e.id() == a.id())
        || !shell[wall].edge_iter().any(|e| e.id() == b.id())
        || !shell[high].edge_iter().any(|e| e.id() == b.id())
    {
        return None;
    }
    // Two full cubic cap edges and four full straight edges fix the original trim domain.
    let mut profiles = 0;
    for edge in &edges {
        let curve = match edge.curve() {
            Curve::PCurve(p) => {
                let uv = p.curve();
                let Surface::Extruded(e) = p.surface().as_ref() else {
                    return None;
                };
                let Curve::BSplineCurve(c) = e.entity_curve() else {
                    return None;
                };
                let cp = uv.control_points();
                let (a, b) = uv.range_tuple();
                let (lo, hi) = c.range_tuple();
                if uv.degree() != 1
                    || cp.len() != 2
                    || !uv.is_clamped()
                    || !a.is_finite()
                    || !b.is_finite()
                    || a >= b
                    || cp[0].y != cp[1].y
                    || !cp[0].y.is_finite()
                    || !((cp[0].x == lo && cp[1].x == hi) || (cp[0].x == hi && cp[1].x == lo))
                {
                    return None;
                }
                let mut c = c.clone();
                c.transform_by(Matrix4::from_translation(e.extruding_vector() * cp[0].y));
                if cp[0].x > cp[1].x {
                    c.invert();
                }
                Curve::BSplineCurve(c)
            }
            c => c,
        };
        let (start, end) = curve.range_tuple();
        if !start.is_finite()
            || !end.is_finite()
            || start >= end
            || curve.subs(start).distance(edge.absolute_front().point()) > eps
            || curve.subs(end).distance(edge.absolute_back().point()) > eps
        {
            return None;
        }
        match curve {
            Curve::BSplineCurve(c) => {
                if c.degree() != 3 || c.control_points().len() != 4 {
                    return None;
                }
                let (lo, hi) = c.range_tuple();
                if !lo.is_finite()
                    || !hi.is_finite()
                    || lo >= hi
                    || (0..4).any(|i| c.knot(i) != lo || c.knot(i + 4) != hi)
                {
                    return None;
                }
                let cp: Vec<_> = c.control_points().iter().map(|p| xyz(*p)).collect();
                let cap = if cp[0].x.abs() < eps { 0. } else { length };
                let mut expected = [
                    Vector3::new(cap, 0., 0.),
                    Vector3::new(cap + 10. * scale / k, 2. * scale, 50. * scale / k),
                    Vector3::new(cap + 6. * scale / k, 4. * scale, 30. * scale / k),
                    Vector3::new(cap, width, 0.),
                ];
                if cp[0].y > width / 2. {
                    expected.reverse();
                }
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
                        || p.y.abs().min((p.y - width).abs()) > eps
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
    let f = |t: f64| scale * profile_depth(t);
    let x = |t: f64| scale * (6. * t + 4. * t.powi(3));
    let slope = |t: f64| (30. - 84. * t + 36. * t * t) * 5. / k / (6. + 12. * t * t);
    let g = |t: f64| f(t) - r / (1. + slope(t).powi(2)).sqrt() - r;
    let (mut left, mut right) = (0., 0.07);
    if g(left) >= 0. || g(right) <= 0. {
        return None;
    }
    for _ in 0..60 {
        let t = (left + right) / 2.;
        if g(t) > 0. {
            right = t;
        } else {
            left = t;
        }
    }
    let t = (left + right) / 2.;
    let dt = f(t);
    let cy = x(t) + r * slope(t) / (1. + slope(t).powi(2)).sqrt();
    let cx = length + r * (0.2 - 1.04_f64.sqrt());
    let dc = r * (1. - 0.2 / 1.04_f64.sqrt());
    if !(dc < r && r < dt && dt < depth && cx > tol && cy - r > 10. * tol && cy + r < x(0.35)) {
        return None;
    }
    if !disk_clearance(r / scale, left, right) {
        return None;
    }
    let ac = (1. - dc / r).acos();
    let at = (1. - dt / r).acos();
    let remote = |d: f64| {
        let (mut a, mut b) = ((7. - 19_f64.sqrt()) / 6., 1.);
        for _ in 0..60 {
            let t = (a + b) / 2.;
            if f(t) > d {
                a = t;
            } else {
                b = t;
            }
        }
        (a + b) / 2.
    };
    let bp = |d: f64| x(remote(d));
    let vertices = [
        point(0., cy, 0.),
        point(cx, cy, 0.),
        point(cx, width, 0.),
        point(0., width, 0.),
        point(cx + r * ac.sin(), cy - r * ac.sin(), dc),
        point(length + 0.2 * dt, x(t), dt),
        point(0.2 * dt, x(t), dt),
        point(length + 0.2 * dc, bp(dc), dc),
    ]
    .map(Vertex::new);
    let [a, b, c, d, e, fv, g, h] = &vertices;
    let section = |axis_x: f64, axis_y: f64, end: f64, direction: Vector3| {
        let center = point(axis_x, axis_y, r);
        let start = Vertex::new(center - r * z);
        let finish = Vertex::new(center - r * end.cos() * z + r * end.sin() * direction);
        circle(&start, &finish, center).curve()
    };
    let mut ruling_surface: Surface =
        ExtrudedCurve::by_extrusion(section(0., cy, at, -y), u * (length + 0.2 * dt)).into();
    let mut cap_surface: Surface =
        ExtrudedCurve::by_extrusion(section(cx, cy - r, ac, u), y * (width - cy + r)).into();
    ruling_surface.invert();
    cap_surface.invert();
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
        point(length + 0.2 * r, cy, r),
        -r * y,
        -r * (0.2 * u + z),
    );
    let ag = ellipse(
        a,
        g,
        0.,
        at,
        point(0.2 * r, cy, r),
        -r * y,
        -r * (0.2 * u + z),
    );
    let fg = builder::line(fv, g);
    let eh = builder::line(e, h);
    let profile = |a: &Vertex, b: &Vertex, cap: f64, ta: f64, tb: f64| {
        let h = (tb - ta) / 3.;
        let value = |t| point(cap + 0.2 * f(t), x(t), f(t));
        let derivative = |t: f64| {
            scale
                * ((6. + 12. * t * t) * y + (30. - 84. * t + 36. * t * t) * 5. / k * (z + 0.2 * u))
        };
        Edge::new(
            a,
            b,
            BSplineCurve::new(
                KnotVec::bezier_knot(3),
                vec![
                    value(ta),
                    value(ta) + h * derivative(ta),
                    value(tb) - h * derivative(tb),
                    value(tb),
                ],
            )
            .into(),
        )
    };
    let dg = profile(d, g, 0., 1., t);
    let fh = profile(fv, h, length, t, remote(dc));
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
        faces.push(
            Face::try_new(
                vec![order_wire(edges.into_iter().rev().map(|e| e.inverse()).collect()).ok()?],
                surface,
            )
            .ok()?,
        );
    }
    let indices = [wall, low, high, curved];
    let mut result = Shell::new();
    for i in 0..4 {
        result.push(faces[indices.iter().position(|j| *j == i)?].clone());
    }
    result.extend(faces.into_iter().skip(4));
    Some(result)
}

fn profile_depth(t: f64) -> f64 { (30. * t - 42. * t * t + 12. * t.powi(3)) * 5. / 26_f64.sqrt() }

// Local interval arithmetic for the G8 contact and degree-four quotient only.
#[derive(Clone, Copy)]
struct Bound(f64, f64);
impl Bound {
    fn exact(x: f64) -> Self { Self(x, x) }
    fn add(self, b: Self) -> Self { Self((self.0 + b.0).next_down(), (self.1 + b.1).next_up()) }
    fn neg(self) -> Self { Self(-self.1, -self.0) }
    fn sub(self, b: Self) -> Self { self.add(b.neg()) }
    fn mul(self, b: Self) -> Self {
        let v = [self.0 * b.0, self.0 * b.1, self.1 * b.0, self.1 * b.1];
        Self(
            v.into_iter().fold(f64::INFINITY, f64::min).next_down(),
            v.into_iter().fold(f64::NEG_INFINITY, f64::max).next_up(),
        )
    }
    fn div(self, b: Self) -> Self {
        if b.0 <= 0. && b.1 >= 0. {
            return Self(f64::NEG_INFINITY, f64::INFINITY);
        }
        self.mul(Self((1. / b.1).next_down(), (1. / b.0).next_up()))
    }
    fn sqrt(self) -> Self { Self(self.0.sqrt().next_down(), self.1.sqrt().next_up()) }
    fn scale(self, x: f64) -> Self { self.mul(Self::exact(x)) }
    fn pow(self, n: usize) -> Self { (0..n).fold(Self::exact(1.), |a, _| a.mul(self)) }
}

fn disk_clearance(r: f64, low: f64, high: f64) -> bool {
    let one = Bound::exact(1.);
    let c = Bound::exact(5.).div(Bound::exact(26.).sqrt());
    // Include the rounded normalization in the radius enclosure.
    let r = Bound(r.next_down(), r.next_up());
    let slope = |t: Bound| {
        c.mul(Bound::exact(30.).sub(t.scale(84.)).add(t.pow(2).scale(36.)))
            .div(Bound::exact(6.).add(t.pow(2).scale(12.)))
    };
    let contact = |t: Bound| {
        let f = c.mul(
            t.scale(30.)
                .sub(t.pow(2).scale(42.))
                .add(t.pow(3).scale(12.)),
        );
        f.sub(r).sub(r.div(one.add(slope(t).pow(2)).sqrt()))
    };
    let (mut lo, mut hi) = (0., 0.07);
    if contact(Bound::exact(lo)).1 >= 0. || contact(Bound::exact(hi)).0 <= 0. {
        return false;
    }
    // g'=F'(1-r*kappa)>0: kappa<1.28 on the ascending branch,
    // normalized r<=.75. The bracket therefore encloses the unique contact.
    for _ in 0..60 {
        let mid = (lo + hi) / 2.;
        let value = contact(Bound::exact(mid));
        if value.0 > 0. {
            hi = mid;
        } else if value.1 < 0. {
            lo = mid;
        } else {
            break;
        }
    }
    if hi - lo > 1e-10 || low < lo - 1e-12 || high > hi + 1e-12 {
        return false;
    }
    let t = Bound(lo, hi);
    let p = slope(t);
    let q = one.add(p.pow(2)).sqrt();
    // R(t+h)-R(t)=h(A0+h*A1+h²*A2). Exact tangency cancels
    // P's constant/linear terms, leaving Q=A·A+2(R-center)·(A1+h*A2).
    let a = [
        Bound::exact(6.).add(t.pow(2).scale(12.)),
        t.scale(12.),
        Bound::exact(4.),
    ];
    let b = [
        c.mul(Bound::exact(30.).sub(t.scale(84.)).add(t.pow(2).scale(36.))),
        c.mul(Bound::exact(-42.).add(t.scale(36.))),
        c.scale(12.),
    ];
    let mut power = [Bound::exact(0.); 5];
    for i in 0..3 {
        for j in 0..3 {
            power[i + j] = power[i + j].add(a[i].mul(a[j])).add(b[i].mul(b[j]));
        }
    }
    for i in 0..2 {
        power[i] = power[i].add(r.scale(2.).div(q).mul(b[i + 1].sub(p.mul(a[i + 1]))));
    }
    let choose =
        |n: usize, k: usize| -> f64 { (0..k).fold(1., |v, i| v * (n - i) as f64 / (i + 1) as f64) };
    (0..64).all(|interval| {
        let start = Bound::exact(interval as f64 / 64.).sub(t);
        let span = Bound::exact(1. / 64.);
        let transformed: Vec<_> = (0..5)
            .map(|k| {
                (k..5).fold(Bound::exact(0.), |v, j| {
                    v.add(
                        power[j]
                            .scale(choose(j, k))
                            .mul(start.pow(j - k))
                            .mul(span.pow(k)),
                    )
                })
            })
            .collect();
        (0..5).all(|i| {
            let bound = (0..=i).fold(Bound::exact(0.), |v, k| {
                v.add(
                    transformed[k]
                        .scale(choose(i, k))
                        .div(Bound::exact(choose(4, k))),
                )
            });
            bound.0.is_finite() && bound.1.is_finite() && bound.0 > 0.
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact_bracket(r: f64) -> (f64, f64) {
        let residual = |t: f64| {
            let dx = 6. + 12. * t * t;
            let df = (30. - 84. * t + 36. * t * t) * 5. / 26_f64.sqrt();
            profile_depth(t) - r - r * dx / dx.hypot(df)
        };
        let (mut lo, mut hi) = (0., 0.07);
        assert!(residual(lo) < 0. && residual(hi) > 0.);
        for _ in 0..60 {
            let mid = (lo + hi) / 2.;
            if residual(mid) > 0. {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        (lo, hi)
    }

    #[test]
    fn g8_disk_certificate_accepts_contact_and_rejects_uncertain_bounds() {
        for r in [0.25, 0.5, 0.75_f64.next_down(), 0.75] {
            let (lo, hi) = contact_bracket(r);
            assert!(disk_clearance(r, lo, hi), "valid contact at {r}");
            assert!(!disk_clearance(r, 0., 0.07), "unresolved contact interval");
            assert!(!disk_clearance(r, lo - 1e-6, hi), "uncertain lower bound");
            assert!(!disk_clearance(r, lo, hi + 1e-6), "uncertain upper bound");
            assert!(
                !disk_clearance(r, lo + 1e-4, hi + 1e-4),
                "wrong branch location"
            );
        }
        for r in [0., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(!disk_clearance(r, 0., 0.07));
        }
    }
}
