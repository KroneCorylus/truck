use truck_geometry::prelude::*;

#[test]
fn concatenation_removes_only_identical_join_controls() {
    let first = BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Point3::new(0., 0., 0.),
            Point3::new(1., 1., 0.),
            Point3::new(2., 1., 1.),
            Point3::new(3., 0., 1.),
        ],
    );
    for gap in [0., 0.25 * TOLERANCE] {
        let mut second = first.transformed(Matrix4::from_translation(Vector3::new(3., gap, 1.)));
        second.knot_translate(1.);
        let joined = first.try_concat(&second).unwrap();
        let (_, multiplicities) = joined.knot_vec().to_single_multi();
        assert_eq!(multiplicities, vec![4, if gap == 0. { 3 } else { 4 }, 4]);
        for i in 0..=256 {
            let t = 2. * i as f64 / 256.;
            let original = if t < 1. { &first } else { &second };
            assert!(joined.subs(t).distance(original.subs(t)) < 1e-12);
            assert!((joined.der(t) - original.der(t)).magnitude() < 1e-12);
        }
    }
}

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
            let (_, multiplicities) = fitted.knot_vec().to_single_multi();
            assert!(multiplicities.len() > 2);
            assert!(multiplicities[1..multiplicities.len() - 1]
                .iter()
                .all(|&m| m <= fitted.degree()));
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
