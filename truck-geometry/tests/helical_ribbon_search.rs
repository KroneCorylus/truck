use truck_geometry::prelude::*;

#[test]
fn multi_turn_ribbon_search_stays_on_the_bounded_sheet() {
    let mut controls = Vec::new();
    let mut knots = vec![0.0; 4];
    let point = |t: f64| {
        Point3::new(
            47.5 * t.cos(),
            47.5 * t.sin(),
            t * 2.0 / std::f64::consts::PI,
        )
    };
    let tangent =
        |t: f64| Vector3::new(-47.5 * t.sin(), 47.5 * t.cos(), 2.0 / std::f64::consts::PI);
    let step = std::f64::consts::FRAC_PI_2;
    controls.push(point(0.0));
    for i in 0..48 {
        let a = i as f64 * step;
        let b = a + step;
        controls.extend([
            point(a) + tangent(a) * (step / 3.0),
            point(b) - tangent(b) * (step / 3.0),
            point(b),
        ]);
        knots.extend(std::iter::repeat_n(
            (i + 1) as f64,
            if i == 47 { 4 } else { 3 },
        ));
    }
    let curve = BSplineCurve::new(KnotVec::from(knots), controls);
    let upper = curve.transformed(Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.8)));
    let surface = BSplineSurface::homotopy(curve, upper);
    for u in [30.0, 33.3, 34.0, 36.7, 41.0] {
        let u = u / 48.0;
        for v in [0.0, 0.4, 1.0] {
            let point = surface.subs(u, v);
            for found in [
                surface.search_parameter(point, None, 100),
                surface.search_nearest_parameter(point, None, 100),
            ] {
                let (x, y) = found.unwrap_or_else(|| panic!("missing ({u},{v})"));
                assert!(
                    (x - u).abs() < 1e-6 && (y - v).abs() < 1e-6,
                    "({u},{v}) -> ({x},{y})"
                );
            }
        }
    }
}
