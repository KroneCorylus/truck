use truck_geometry::prelude::*;

#[test]
fn cubic_fitting_preserves_tolerances_below_default_knot_removal_precision() {
    for scale in [1.0, 0.01] {
        let curve = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., 1.7)),
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale),
        );
        for curve in [curve, curve.inverse()] {
            let p_tol = scale * 1e-7;
            let d_tol = scale * 1e-4;
            let fitted =
                BSplineCurve::cubic_approximation(&curve, curve.range_tuple(), p_tol, d_tol, 16)
                    .unwrap();
            for i in 0..=1024 {
                let (a, b) = curve.range_tuple();
                let t = a + (b - a) * i as f64 / 1024.;
                let point_error = curve.subs(t).distance(fitted.subs(t));
                let derivative_error = (curve.der(t) - fitted.der(t)).magnitude();
                assert!(
                    point_error <= p_tol,
                    "scale {scale}, t {t}: {point_error} > {p_tol}"
                );
                assert!(
                    derivative_error <= d_tol,
                    "scale {scale}, t {t}: {derivative_error} > {d_tol}"
                );
            }
        }
    }
}
