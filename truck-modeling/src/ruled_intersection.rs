use crate::*;

pub(super) fn exact(
    first: &Surface,
    second: &Surface,
    leader: &BSplineCurve<Point3>,
) -> Option<Curve> {
    for (ruled, other) in [(first, second), (second, first)] {
        let Surface::NurbsSurface(surface) = ruled else {
            continue;
        };
        if surface.vdegree() != 1
            || surface
                .control_points()
                .iter()
                .any(|row| row.len() != 2 || (row[0].w - row[1].w).abs() > TOLERANCE2)
        {
            continue;
        }
        let (a, b) = leader.range_tuple();
        let start = leader.subs(a);
        let end = leader.subs(b);
        let uv0 = ruled.search_parameter(start, None, 100)?;
        let uv1 = ruled.search_parameter(end, None, 100)?;
        let full = if let Some((Elementary::Plane(plane), _)) = other.elementary() {
            plane_section(surface, plane)?
        } else if (uv0.1 - uv1.1).abs() < TOLERANCE {
            let (v0, v1) = surface.range_tuple().1;
            let v = (uv0.1 - v0) / (v1 - v0);
            NurbsCurve::new(BSplineCurve::new(
                surface.uknot_vec().clone(),
                surface
                    .control_points()
                    .iter()
                    .map(|row| row[0] * (1.0 - v) + row[1] * v)
                    .collect(),
            ))
        } else {
            continue;
        };
        let (low, high) = (uv0.0.min(uv1.0), uv0.0.max(uv1.0));
        let (lo, hi) = full.range_tuple();
        if low < lo - TOLERANCE || high > hi + TOLERANCE || high - low <= TOLERANCE2 {
            continue;
        }
        let mut curve = full;
        if high < hi {
            curve.cut(high);
        }
        if low > lo {
            curve = curve.cut(low);
        }
        if uv1.0 < uv0.0 {
            curve.invert();
        }
        if !curve.front().near(&start) || !curve.back().near(&end) {
            continue;
        }
        let (a, b) = curve.range_tuple();
        if (0..=32).all(|i| {
            let p = curve.subs(a + (b - a) * i as f64 / 32.0);
            first.search_parameter(p, None, 100).is_some()
                && second.search_parameter(p, None, 100).is_some()
        }) {
            return Some(curve.into());
        }
    }
    None
}

fn choose(n: usize, k: usize) -> f64 {
    (0..k.min(n - k)).fold(1.0, |v, i| v * (n - i) as f64 / (i + 1) as f64)
}

fn plane_section(surface: &NurbsSurface<Vector4>, plane: Plane) -> Option<NurbsCurve<Vector4>> {
    let c0 = surface
        .row_curve(0)
        .non_rationalized()
        .bezier_decomposition();
    let c1 = surface
        .row_curve(1)
        .non_rationalized()
        .bezier_decomposition();
    let n = plane.normal();
    let origin = plane.origin();
    let signed = |p: Vector4| n.dot(p.truncate() - origin.to_vec() * p.w);
    let mut result: Option<BSplineCurve<Vector4>> = None;
    for (c0, c1) in c0.iter().zip(c1.iter()) {
        let degree = c0.degree();
        if degree > 8 {
            return None;
        }
        let a = c0.control_points();
        let d: Vec<_> = c1
            .control_points()
            .iter()
            .zip(a)
            .map(|(b, a)| b - a)
            .collect();
        let mut points = vec![Vector4::zero(); 2 * degree + 1];
        // Multiply Bernstein polynomials: C0*plane(D) - D*plane(C0).
        for i in 0..=degree {
            for j in 0..=degree {
                points[i + j] += (a[i] * signed(d[j]) - d[i] * signed(a[j]))
                    * (choose(degree, i) * choose(degree, j) / choose(2 * degree, i + j));
            }
        }
        if points
            .iter()
            .any(|p| !p.is_finite() || p.w.abs() < TOLERANCE2)
        {
            return None;
        }
        if points[0].w < 0.0 {
            for p in &mut points {
                *p = -*p;
            }
        }
        if points.iter().any(|p| p.w <= 0.0) {
            return None;
        }
        let (a, b) = c0.range_tuple();
        let knots = KnotVec::from([vec![a; 2 * degree + 1], vec![b; 2 * degree + 1]].concat());
        let next = BSplineCurve::new(knots, points);
        match &mut result {
            Some(curve) => {
                *curve = curve.try_concat(&next).ok()?;
            }
            None => result = Some(next),
        }
    }
    result.map(NurbsCurve::new)
}
