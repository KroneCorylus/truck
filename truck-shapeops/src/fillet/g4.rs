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
    if basis.degree() != 4 || basis.control_points().len() != 5 {
        return None;
    }
    let (lo, hi) = basis.range_tuple();
    if !lo.is_finite()
        || !hi.is_finite()
        || lo >= hi
        || !(hi - lo).is_finite()
        || (0..5).any(|i| basis.knot(i) != lo || basis.knot(i + 5) != hi)
    {
        return None;
    }
    let mut seen = HashSet::default();
    let edges: Vec<_> = shell.edge_iter().filter(|e| seen.insert(e.id())).collect();
    let vertices: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    if edges.len() != 6
        || vertices.len() != 4
        || !finite_vector(extrusion.extruding_vector())
        || shell.vertex_iter().any(|v| !finite_point(v.point()))
    {
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
    let chord = other(b) - p.point();
    let y = (chord - chord.dot(u) * u).normalize();
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
    let scale = width / 500_f64.sqrt();
    let width = chord.dot(y);
    let alpha = (26_f64 / 819.).sqrt();
    let beta = -(26_f64 / 126.).sqrt() / 5.;
    let k = (1. + alpha * alpha).sqrt();
    let n = (k * k + beta * beta).sqrt();
    let depth = scale * profile_depth(0.5);
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
        || r > 1.5 * scale
        || u.dot(y).abs() > 1e-10
        || (length - scale * 2. * 26_f64.sqrt()).abs() > eps
        || extrusion.extruding_vector().cross(u).magnitude() > eps
        || u.cross(y).dot(z) > 0.
        || (chord.dot(u) - alpha * width).abs() > eps
    {
        return None;
    }
    let cp: Vec<_> = basis.control_points().iter().map(|p| xyz(*p)).collect();
    let cap = cp[0].x - alpha * cp[0].y - beta * cp[0].z;
    let expected_controls = |cap: f64| {
        [
            (0., 0., 0.),
            (20., 0., 0.),
            (20., 5., 0.),
            (25., 10., 0.),
            (10., 20., 0.),
        ]
        .map(|(x, y, z)| {
            Vector3::new(
                cap + scale * (y + 5. * z) / 26_f64.sqrt(),
                scale * (13. * x + 25. * y - 5. * z) / 819_f64.sqrt(),
                scale * (10. * x - 5. * y + z) / 126_f64.sqrt(),
            )
        })
    };
    let mut expected = expected_controls(cap);
    if cp[0].y > width / 2. {
        expected.reverse();
    }
    if cp
        .iter()
        .zip(expected)
        .any(|(p, q)| !finite_vector(*p) || (*p - q).magnitude() > arithmetic)
    {
        return None;
    }
    let sweep = extrusion.extruding_vector();
    let axial = sweep.dot(u);
    if !axial.is_finite() || axial == 0. {
        return None;
    }
    let heights = [-cap / axial, (length - cap) / axial];
    let height_bound = heights[0].abs().max(heights[1].abs());
    let transverse = (sweep - axial * u).magnitude();
    let axial_residual = cp
        .iter()
        .map(|p| (p.x - alpha * p.y - beta * p.z - cap).abs())
        .fold(0., f64::max);
    // Convexity bounds the base curve; linearity bounds every admitted sweep height.
    let sweep_error = axial_residual + height_bound * transverse * (1. + alpha.abs() + beta.abs());
    if !sweep_error.is_finite() || sweep_error + arithmetic > eps / 16. {
        return None;
    }
    let other_cap = cap + axial;
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
        .find(|(_, n)| n.dot((-u + alpha * y + beta * z).normalize()) > 1. - 1e-10)?
        .0;
    let high = planes
        .iter()
        .find(|(_, n)| n.dot((u - alpha * y - beta * z).normalize()) > 1. - 1e-10)?
        .0;
    if !shell[wall].edge_iter().any(|e| e.id() == a.id())
        || !shell[wall].edge_iter().any(|e| e.id() == b.id())
        || !shell[high].edge_iter().any(|e| e.id() == b.id())
    {
        return None;
    }
    // Two full quartic cap edges and four full straight edges fix the original trim domain.
    let axes = [u, y, z];
    let mut gram_error = 0_f64;
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1. } else { 0. };
            let residual = (axes[i].dot(axes[j]) - expected).abs()
                + 8. * f64::EPSILON * axes[i].magnitude() * axes[j].magnitude();
            gram_error = gram_error.max(residual);
        }
    }
    // ||A^T A-I||_2 <= 3 max|entry|. Polar decomposition gives
    // ||A-Q||_2 <= delta/(1+sqrt(1-delta)) <= delta for delta<1.
    let delta = 3. * gram_error;
    let frame_error = delta * (length + width + depth + 4. * r);
    if !frame_error.is_finite() || delta >= 0.5 || frame_error + arithmetic > eps / 16. {
        return None;
    }
    let mut profiles = 0;
    let mut representation = sweep_error + frame_error;
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
                if c.degree() != 4 || c.control_points().len() != 5 {
                    return None;
                }
                let (lo, hi) = c.range_tuple();
                if !lo.is_finite()
                    || !hi.is_finite()
                    || lo >= hi
                    || (0..5).any(|i| c.knot(i) != lo || c.knot(i + 5) != hi)
                {
                    return None;
                }
                let cp: Vec<_> = c.control_points().iter().map(|p| xyz(*p)).collect();
                let intercept = cp[0].x - alpha * cp[0].y - beta * cp[0].z;
                let cap = if intercept.abs() < eps { 0. } else { length };
                let mut expected = expected_controls(cap);
                if cp[0].y > width / 2. {
                    expected.reverse();
                }
                let deviation = cp.iter().zip(expected).try_fold(0_f64, |bound, (p, q)| {
                    finite_vector(*p).then_some(bound.max((*p - q).magnitude()))
                })?;
                representation += deviation;
                if !representation.is_finite() || representation + arithmetic > eps / 16. {
                    return None;
                }
                profiles += 1;
            }
            Curve::Line(_) => {
                for v in [edge.front(), edge.back()] {
                    let p = xyz(v.point());
                    if p.z.abs() > eps
                        || p.y.abs().min((p.y - width).abs()) > eps
                        || (p.x - alpha * p.y)
                            .abs()
                            .min((p.x - alpha * p.y - length).abs())
                            > eps
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
    // A plane residual is affine, so corners bound the entire retained trim domain.
    let xmax = cp.iter().map(|p| p.y.abs()).fold(width, f64::max);
    let dmax = cp.iter().map(|p| p.z.abs()).fold(depth, f64::max);
    let mut plane_error = 0_f64;
    for (index, level) in [(low, 0.), (high, length), (wall, 0.)] {
        let Surface::Plane(plane) = shell[index].oriented_surface() else {
            return None;
        };
        let normal = plane.normal().normalize();
        let a = plane.u_axis();
        let b = plane.v_axis();
        let cross_norm = a.cross(b).magnitude();
        // Subtraction and cross-product forward error, including cancellation.
        // Normalization is Lipschitz with constant 2/(|cross|-cross_error).
        let cross_error = 16. * f64::EPSILON * a.magnitude() * b.magnitude();
        if !cross_norm.is_finite() || !cross_error.is_finite() || cross_norm <= cross_error {
            return None;
        }
        let normal_error = 2. * cross_error / (cross_norm - cross_error) + 16. * f64::EPSILON;
        for x in [-xmax, xmax] {
            for d in [-dmax, dmax] {
                let p = if index == wall {
                    point(d / dmax * (length + alpha.abs() * xmax), x, 0.)
                } else {
                    point(level + alpha * x + beta * d, x, d)
                };
                let distance = (p - plane.origin()).magnitude();
                let residual = (p - plane.origin()).dot(normal).abs()
                    + normal_error * distance
                    + 16.
                        * f64::EPSILON
                        * (p.to_vec().magnitude() + plane.origin().to_vec().magnitude());
                if !residual.is_finite() {
                    return None;
                }
                plane_error = plane_error.max(residual);
            }
        }
    }
    representation += plane_error;
    if !representation.is_finite() || representation + arithmetic > eps / 8. {
        return None;
    }
    let mut profile_controls: Vec<_> = cp.iter().map(|p| *p / scale).collect();
    if profile_controls[0].y > width / (2. * scale) {
        profile_controls.reverse();
    }
    let x_coefficients = bezier_power(
        profile_controls
            .iter()
            .map(|p| p.y)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?,
    );
    let f_coefficients = bezier_power(
        profile_controls
            .iter()
            .map(|p| p.z)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?,
    );
    let contact = disk_clearance(r / scale, x_coefficients, f_coefficients)?;
    let t = contact.0 + (contact.1 - contact.0) / 2.;
    let dx = derivative(&x_coefficients);
    let df = derivative(&f_coefficients);
    let contact_x = polynomial(&x_coefficients, contact).scale(scale);
    let contact_f = polynomial(&f_coefficients, contact).scale(scale);
    let contact_slope = polynomial(&df, contact).div(polynomial(&dx, contact));
    let contact_center = contact_x.add(
        contact_slope
            .scale(r)
            .div(Bound::exact(1.).add(contact_slope.pow(2)).sqrt()),
    );
    let sine_distance = contact_f.mul(Bound::exact(2. * r).sub(contact_f)).sqrt();
    if !sine_distance.0.is_finite() || sine_distance.0 <= 0. {
        return None;
    }
    // Coordinate interval diameters cover the contact point and cylinder center.
    // r*d(acos(1-F/r))/dF = r/sqrt(F*(2r-F)) bounds arc-end displacement.
    let root_error = (1. + alpha.abs())
        * (contact_x.1 - contact_x.0 + contact_center.1 - contact_center.0)
        + (1. + beta.abs() + r / sine_distance.0) * (contact_f.1 - contact_f.0);
    if !root_error.is_finite() || root_error + arithmetic > eps / 16. {
        return None;
    }
    let f = |t: f64| scale * polynomial_midpoint(&f_coefficients, t);
    let x = |t: f64| scale * polynomial_midpoint(&x_coefficients, t);
    let slope = |t: f64| polynomial_midpoint(&df, t) / polynomial_midpoint(&dx, t);
    let dt = f(t);
    let cy = x(t) + r * slope(t) / (1. + slope(t).powi(2)).sqrt();
    let cx = length + r * (beta - n);
    let dc = r * (1. - beta / n);
    if !(r < dc && dc < dt && dt < depth && cx > tol && cy - r > 10. * tol && cy + r < x(0.35)) {
        return None;
    }
    let ac = (1. - dc / r).acos();
    let at = (1. - dt / r).acos();
    let remote = |d: f64| {
        let (mut a, mut b) = (0.5, 1.);
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
        point(alpha * cy, cy, 0.),
        point(alpha * cy + cx, cy, 0.),
        point(alpha * width + cx, width, 0.),
        point(alpha * width, width, 0.),
        point(
            alpha * (cy - r * ac.sin()) + cx + k * r * ac.sin(),
            cy - r * ac.sin(),
            dc,
        ),
        point(length + alpha * x(t) + beta * dt, x(t), dt),
        point(alpha * x(t) + beta * dt, x(t), dt),
        point(length + alpha * bp(dc) + beta * dc, bp(dc), dc),
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
        ExtrudedCurve::by_extrusion(section(0., cy, at, -y), u * (length + alpha * width)).into();
    let axis = (alpha * u + y) / k;
    let normal = (u - alpha * y) / k;
    let center = point(cx + alpha * (cy - 2. * r), cy - 2. * r, r);
    let start = Vertex::new(center - r * z);
    let end = Vertex::new(center - r * ac.cos() * z + r * ac.sin() * normal);
    let mut cap_surface: Surface = ExtrudedCurve::by_extrusion(
        circle(&start, &end, center).curve(),
        axis * (width + 4. * r),
    )
    .into();
    for surface in [&mut ruling_surface, &mut cap_surface] {
        if surface.normal(0., 0.).dot(-z) < 0. {
            surface.invert();
        }
    }
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
    let be = ellipse(
        b,
        e,
        0.,
        ac,
        point(alpha * cy + cx, cy, r),
        r * ((k - alpha) * u - y),
        -r * z,
    );
    let ef = ellipse(
        e,
        fv,
        ac,
        at,
        point(length + alpha * cy + beta * r, cy, r),
        -r * (alpha * u + y),
        -r * (beta * u + z),
    );
    let ag = ellipse(
        a,
        g,
        0.,
        at,
        point(alpha * cy + beta * r, cy, r),
        -r * (alpha * u + y),
        -r * (beta * u + z),
    );
    let fg = builder::line(fv, g);
    let eh = builder::line(e, h);
    let profile = |a: &Vertex, b: &Vertex, cap: f64, ta: f64, tb: f64| {
        let height = (cap - (cp[0].x - alpha * cp[0].y - beta * cp[0].z)) / axial;
        let mut controls = basis
            .control_points()
            .iter()
            .map(|p| *p + height * sweep)
            .collect::<Vec<_>>();
        if cp[0].y > width / 2. {
            controls.reverse();
        }
        Edge::new(
            a,
            b,
            BSplineCurve::new(KnotVec::bezier_knot(4), bezier_interval(&controls, ta, tb)).into(),
        )
    };
    let dg = profile(d, g, 0., 1., t);
    let fh = profile(fv, h, length, t, remote(dc));
    // Reserve half the STEP default edge precision (1e-7) for spatial support
    // agreement; the existing whole-span control/evaluation certificate still applies.
    let fit_budget = (eps / 2.).min(5e-8);
    let (fourth, inverse_lower) =
        remote_fourth_bound(&x_coefficients, &f_coefficients, r / scale, k)?;
    let fourth = fourth * scale;
    if f(0.95) <= dc || arithmetic > fit_budget / 16. {
        return None;
    }
    let mut count = 1usize;
    while fourth * (ac / count as f64).powi(4) / 384. > fit_budget / 2. {
        count *= 2;
        if count > 16384 {
            return None;
        }
    }
    let value = |angle: f64| {
        let depth = r * (1. - angle.cos());
        let t = remote(depth);
        let x = x(t);
        let parameter = t;
        let (sin, cos) = trig_bounds(angle);
        let target = Bound::exact(1.).sub(cos).scale(r / scale);
        let residual = polynomial(&f_coefficients, Bound::exact(parameter)).sub(target);
        let inverse_error = residual.0.abs().max(residual.1.abs()) / inverse_lower;
        let parameter = Bound(
            (parameter - inverse_error).next_down(),
            (parameter + inverse_error).next_up(),
        );
        if !parameter.0.is_finite()
            || !parameter.1.is_finite()
            || parameter.0 < 0.949
            || parameter.1 > 1.001
        {
            return None;
        }
        let bx = polynomial(&x_coefficients, parameter).scale(scale);
        let bd = Bound::exact(1.).sub(cos).scale(r);
        let bu = bx.scale(alpha).add(Bound::exact(cx)).add(sin.scale(k * r));
        let velocity = polynomial(&dx, parameter)
            .div(polynomial(&df, parameter))
            .mul(sin)
            .scale(r);
        let position_bounds = std::array::from_fn::<_, 3, _>(|j| {
            Bound::exact(origin[j])
                .add(bu.scale(u[j]))
                .add(bx.scale(y[j]))
                .add(bd.scale(z[j]))
        });
        let derivative_bounds = std::array::from_fn::<_, 3, _>(|j| {
            velocity
                .scale(alpha * u[j] + y[j])
                .add(cos.scale(k * r * u[j]))
                .add(sin.scale(r * z[j]))
        });
        Some((
            point(alpha * x + cx + k * r * angle.sin(), x, depth),
            (alpha * u + y)
                * (polynomial_midpoint(&dx, t) / polynomial_midpoint(&df, t) * r * angle.sin())
                + u * (k * r * angle.cos())
                + z * (r * angle.sin()),
            position_bounds,
            derivative_bounds,
        ))
    };
    let mut controls = vec![c.point()];
    let mut knots = vec![0.; 4];
    for i in 0..count {
        let a = ac * i as f64 / count as f64;
        let b = ac * (i + 1) as f64 / count as f64;
        if fourth * (b - a).powi(4) / 384. + root_error + arithmetic + representation > fit_budget {
            return None;
        }
        let (pa, va, pav, vad) = value(a)?;
        let (pb, vb, pbv, vbd) = value(b)?;
        let stored = [
            *controls.last()?,
            pa + va * ((b - a) / 3.),
            pb - vb * ((b - a) / 3.),
            if i + 1 == count { h.point() } else { pb },
        ];
        let span = Bound::exact(b).sub(Bound::exact(a)).div(Bound::exact(3.));
        let ideal = [
            pav,
            std::array::from_fn(|j| pav[j].add(vad[j].mul(span))),
            std::array::from_fn(|j| pbv[j].sub(vbd[j].mul(span))),
            pbv,
        ];
        let mut construction_error = 0_f64;
        for (stored, ideal) in stored.iter().zip(ideal) {
            let error = (0..3)
                .map(|j| {
                    let residual = Bound::exact(stored[j]).sub(ideal[j]);
                    residual.0.abs().max(residual.1.abs())
                })
                .sum::<f64>();
            construction_error = construction_error.max(error);
        }
        // Bernstein convexity propagates endpoint, inverse and control roundoff errors.
        // Cubic de Boor evaluation has three convex-interpolation levels. gamma(32)
        // covers their weights and vector operations; use the actual stored hull.
        let hull = stored
            .iter()
            .map(|p| p.to_vec().magnitude())
            .fold(0., f64::max);
        let evaluation_error = Bound::exact(32. * f64::EPSILON)
            .div(Bound::exact(1.).sub(Bound::exact(32. * f64::EPSILON)))
            .scale(hull)
            .1;
        let total = fourth * (b - a).powi(4) / 384.
            + construction_error
            + evaluation_error
            + root_error
            + representation
            + arithmetic;
        if !total.is_finite() || total > fit_budget {
            return None;
        }
        controls.extend_from_slice(&stored[1..]);
        knots.extend(std::iter::repeat_n(b, if i + 1 == count { 4 } else { 3 }));
    }
    let ch = Edge::new(c, h, BSplineCurve::new(knots.into(), controls).into());
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

fn profile_depth(t: f64) -> f64 {
    (800. * t - 1350. * t * t + 1100. * t.powi(3) - 550. * t.powi(4)) / 126_f64.sqrt()
}
#[cfg(test)]
fn profile_dx(t: f64) -> f64 {
    (1040. - 1620. * t + 2400. * t * t - 1600. * t.powi(3)) / 819_f64.sqrt()
}
#[cfg(test)]
fn profile_df(t: f64) -> f64 {
    (800. - 2700. * t + 3300. * t * t - 2200. * t.powi(3)) / 126_f64.sqrt()
}

// Outward interval arithmetic for the quartic contact and degree-six quotient.
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

fn polynomial(c: &[Bound], t: Bound) -> Bound {
    c.iter()
        .rev()
        .fold(Bound::exact(0.), |v, c| v.mul(t).add(*c))
}

fn polynomial_midpoint(c: &[Bound], t: f64) -> f64 {
    c.iter()
        .rev()
        .fold(0., |v, c| v * t + (c.0 + (c.1 - c.0) / 2.))
}

fn derivative(c: &[Bound]) -> Vec<Bound> {
    c.iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| c.scale(i as f64))
        .collect()
}

fn bezier_power(c: [f64; 5]) -> [Bound; 5] {
    let mut differences = c.map(Bound::exact);
    let mut power = [Bound::exact(0.); 5];
    for (i, factor) in [1., 4., 6., 4., 1.].into_iter().enumerate() {
        power[i] = differences[0].scale(factor);
        for j in 0..4 - i {
            differences[j] = differences[j + 1].sub(differences[j]);
        }
    }
    power
}

// Taylor remainder <= |angle|^26/26! for both series on this bounded domain.
fn trig_bounds(angle: f64) -> (Bound, Bound) {
    let a = Bound::exact(angle);
    let square = a.pow(2).neg();
    let mut sin = a;
    let mut cos = Bound::exact(1.);
    let mut st = a;
    let mut ct = cos;
    for i in 1..=12 {
        st = st
            .mul(square)
            .div(Bound::exact((2 * i * (2 * i + 1)) as f64));
        ct = ct
            .mul(square)
            .div(Bound::exact((2 * i * (2 * i - 1)) as f64));
        sin = sin.add(st);
        cos = cos.add(ct);
    }
    let factorial = (1..=26).fold(Bound::exact(1.), |v, i| v.scale(i as f64));
    let remainder = a.pow(26).div(factorial).1;
    (
        sin.add(Bound(-remainder, remainder)),
        cos.add(Bound(-remainder, remainder)),
    )
}

fn remote_fourth_bound(x: &[Bound; 5], f: &[Bound; 5], r: f64, k: f64) -> Option<(f64, f64)> {
    let mut dx = x.to_vec();
    let mut df = f.to_vec();
    let mut xb = [0_f64; 4];
    let mut fb = [0_f64; 4];
    let mut lower = f64::INFINITY;
    for order in 0..4 {
        dx = derivative(&dx);
        df = derivative(&df);
        for i in 0..64 {
            let t = Bound(
                (0.949 + 0.052 * i as f64 / 64.).next_down(),
                (0.949 + 0.052 * (i + 1) as f64 / 64.).next_up(),
            );
            let x = polynomial(&dx, t);
            let f = polynomial(&df, t);
            xb[order] = xb[order].max(x.0.abs()).max(x.1.abs());
            fb[order] = fb[order].max(f.0.abs()).max(f.1.abs());
            if order == 0 {
                lower = lower.min(-f.1);
            }
        }
    }
    if !lower.is_finite() || lower <= 50. {
        return None;
    }
    let r = Bound::exact(r);
    let m = Bound::exact(lower);
    let f = fb.map(Bound::exact);
    let x = xb.map(Bound::exact);
    let t1 = r.div(m);
    let t2 = r.add(f[1].mul(t1.pow(2))).div(m);
    let t3 = r
        .add(f[1].mul(t1).mul(t2).scale(3.))
        .add(f[2].mul(t1.pow(3)))
        .div(m);
    let t4 = r
        .add(f[1].mul(t1.mul(t3).scale(4.).add(t2.pow(2).scale(3.))))
        .add(f[2].mul(t1.pow(2)).mul(t2).scale(6.))
        .add(f[3].mul(t1.pow(4)))
        .div(m);
    let fourth = x[0]
        .mul(t4)
        .add(x[1].mul(t1.mul(t3).scale(4.).add(t2.pow(2).scale(3.))))
        .add(x[2].mul(t1.pow(2)).mul(t2).scale(6.))
        .add(x[3].mul(t1.pow(4)))
        .scale(k)
        .add(r.scale(k + 1.));
    (fourth.1.is_finite() && fourth.1 <= 10000.).then_some((fourth.1, lower))
}

fn disk_clearance(r: f64, x: [Bound; 5], f: [Bound; 5]) -> Option<Bound> {
    if !r.is_finite() || r <= 0. || r > 1.5 {
        return None;
    }
    // Restrict this certificate to a small neighborhood of the intended family.
    // Admission of the complete represented surface remains a separate gate.
    let canonical_x = [0., 1040., -810., 800., -400.].map(|v| v / 819_f64.sqrt());
    let canonical_f = [0., 800., -1350., 1100., -550.].map(|v| v / 126_f64.sqrt());
    for (c, ideal) in x.iter().zip(canonical_x).chain(f.iter().zip(canonical_f)) {
        if !c.0.is_finite()
            || !c.1.is_finite()
            || c.0 > c.1
            || (c.0 - ideal).abs().max((c.1 - ideal).abs()) > 1e-6
        {
            return None;
        }
    }
    let one = Bound::exact(1.);
    let eval = polynomial;
    let dx = derivative(&x);
    let df = derivative(&f);
    let ddx = derivative(&dx);
    let ddf = derivative(&df);
    // Verify uniqueness on the actual coefficient enclosure, not the ideal curve.
    for i in 0..64 {
        let t = Bound(i as f64 * 0.05 / 64., (i + 1) as f64 * 0.05 / 64.);
        let vx = eval(&dx, t);
        let vf = eval(&df, t);
        let p = vf.div(vx);
        let dp = eval(&ddf, t)
            .mul(vx)
            .sub(vf.mul(eval(&ddx, t)))
            .div(vx.pow(2));
        let q = one.add(p.pow(2)).sqrt();
        let dg = vf.add(p.mul(dp).scale(r).div(q.pow(3)));
        if vx.0 <= 0. || !dg.0.is_finite() || dg.0 <= 0. {
            return None;
        }
    }
    let slope = |t: Bound| eval(&df, t).div(eval(&dx, t));
    let r = Bound(r.next_down(), r.next_up());
    let contact = |t: Bound| {
        eval(&f, t)
            .sub(r)
            .sub(r.div(one.add(slope(t).pow(2)).sqrt()))
    };
    let (mut lo, mut hi) = (0., 0.05);
    if contact(Bound::exact(lo)).1 >= 0. || contact(Bound::exact(hi)).0 <= 0. {
        return None;
    }
    for _ in 0..60 {
        let mid = (lo + hi) / 2.;
        let v = contact(Bound::exact(mid));
        if v.0 > 0. {
            hi = mid;
        } else if v.1 < 0. {
            lo = mid;
        } else {
            break;
        }
    }
    if hi - lo > 1e-10 {
        return None;
    }
    let t = Bound(lo, hi);
    let p = slope(t);
    let q = one.add(p.pow(2)).sqrt();
    let center = eval(&x, t).add(r.mul(p).div(q));
    let (mut a, mut b) = (0., 0.05);
    for _ in 0..60 {
        let mid = (a + b) / 2.;
        let depth = eval(&f, Bound::exact(mid));
        if depth.0 > r.1 {
            b = mid;
        } else if depth.1 < r.0 {
            a = mid;
        } else {
            break;
        }
    }
    if eval(&x, Bound(a, b)).1 >= center.0 || eval(&x, Bound::exact(0.5)).0 <= center.1 {
        return None;
    }

    let choose =
        |n: usize, k: usize| -> f64 { (0..k).fold(1., |v, i| v * (n - i) as f64 / (i + 1) as f64) };
    let shifted = |c: [Bound; 5]| -> Vec<Bound> {
        (1..5)
            .map(|i| {
                (i..5).fold(Bound::exact(0.), |v, j| {
                    v.add(c[j].scale(choose(j, i)).mul(t.pow(j - i)))
                })
            })
            .collect()
    };
    let a = shifted(x);
    let b = shifted(f);
    // For each represented polynomial, the root above satisfies F-r=r/q
    // and the center is X+r*p/q. Thus P(t)=P'(t)=0 at its own contact.
    // Enclose its quotient directly, without dividing by the double root.
    let mut power = [Bound::exact(0.); 7];
    for i in 0..4 {
        for j in 0..4 {
            power[i + j] = power[i + j].add(a[i].mul(a[j])).add(b[i].mul(b[j]));
        }
    }
    for i in 0..3 {
        power[i] = power[i].add(r.scale(2.).div(q).mul(b[i + 1].sub(p.mul(a[i + 1]))));
    }
    let contained = (0..64).all(|interval| {
        let start = Bound::exact(interval as f64 / 64.).sub(t);
        let span = Bound::exact(1. / 64.);
        let transformed: Vec<_> = (0..7)
            .map(|k| {
                (k..7).fold(Bound::exact(0.), |v, j| {
                    v.add(
                        power[j]
                            .scale(choose(j, k))
                            .mul(start.pow(j - k))
                            .mul(span.pow(k)),
                    )
                })
            })
            .collect();
        (0..7).all(|i| {
            let v = (0..=i).fold(Bound::exact(0.), |v, k| {
                v.add(
                    transformed[k]
                        .scale(choose(i, k))
                        .div(Bound::exact(choose(6, k))),
                )
            });
            v.0.is_finite() && v.1.is_finite() && v.0 > 0.
        })
    });
    contained.then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical() -> ([Bound; 5], [Bound; 5]) {
        (
            [0., 1040., -810., 800., -400.].map(|v| Bound::exact(v).div(Bound::exact(819.).sqrt())),
            [0., 800., -1350., 1100., -550.]
                .map(|v| Bound::exact(v).div(Bound::exact(126.).sqrt())),
        )
    }

    #[test]
    fn g4_contact_and_whole_profile_certificate() {
        let (x, f) = canonical();
        for r in [0.0001, 0.5, 1., 1.5] {
            let root = disk_clearance(r, x, f).unwrap();
            assert!(root.0 < root.1 && root.1 - root.0 < 1e-10);
            let residual = |t: f64| {
                let p = profile_df(t) / profile_dx(t);
                profile_depth(t) - r - r / (1. + p * p).sqrt()
            };
            assert!(residual(root.0) < 0.);
            assert!(residual(root.1) > 0.);
        }
        for r in [0., -1., 1.5_f64.next_up(), f64::NAN, f64::INFINITY] {
            assert!(disk_clearance(r, x, f).is_none());
        }
    }

    #[test]
    fn g4_represented_contact_moves_with_profile() {
        let (x, mut f) = canonical();
        let original = disk_clearance(1., x, f).unwrap();
        // Translate toward the old disk at its contact: the ideal quotient stays
        // positive, but the old disk is no longer contained in this profile.
        let delta = 1e-7;
        f[0] = f[0].sub(Bound::exact(delta));
        let moved = disk_clearance(1., x, f).unwrap();
        assert!(moved.0 > original.1);
        let t = (original.0 + original.1) / 2.;
        let p = profile_df(t) / profile_dx(t);
        let q = (1. + p * p).sqrt();
        let old_disk_clearance = (-p / q).powi(2) + (1. / q - delta).powi(2) - 1.;
        assert!(old_disk_clearance < 0.);
        let mut invalid = f;
        invalid[2] = Bound(f64::NAN, f64::NAN);
        assert!(disk_clearance(1., x, invalid).is_none());
        f[0] = Bound::exact(-1e-4);
        assert!(disk_clearance(1., x, f).is_none());
    }
}
