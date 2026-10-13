use truck_geometry::prelude::*;

#[test]
fn cubic_segments_preserve_requested_parameter_endpoints() {
    let curve = Line(Point3::new(13., -7., 23.), Point3::new(17., 2., -5.));
    for range in [
        (-0.014358461080567665, -0.005752014283787985),
        (0.011460879309771374, 0.028673772903330733),
        (-0.005752014283787985, 0.0028544325129916943),
    ] {
        let fitted = BSplineCurve::cubic_approximation(&curve, range, 1e-10, 1e-8, 0).unwrap();
        assert_eq!(fitted.range_tuple(), range);
        assert_eq!(fitted.front(), curve.subs(range.0));
        assert_eq!(fitted.back(), curve.subs(range.1));
        for i in 0..=64 {
            let t = range.0 + (range.1 - range.0) * i as f64 / 64.;
            assert!(fitted.subs(t).distance(curve.subs(t)) < 1e-10);
            assert!((fitted.der(t) - curve.der(t)).magnitude() < 1e-8);
        }
    }
}

#[test]
fn adaptive_cubic_pieces_share_exact_parameter_boundaries() {
    let (a, b) = (-0.022964907877347344, 0.5278476871165522);
    let curve = BSplineCurve::new(
        vec![a, a, a, a, a, b, b, b, b, b].into(),
        vec![
            Point3::new(-30., 0., 0.),
            Point3::new(-30., 30., 3.),
            Point3::new(0., 30., 10.),
            Point3::new(30., 30., 3.),
            Point3::new(30., 0., 0.),
        ],
    );
    for scale in [1., 0.02] {
        let matrix = if scale == 1. {
            Matrix4::identity()
        } else {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale)
        };
        let transformed = curve.transformed(matrix);
        for curve in [transformed.clone(), transformed.inverse()] {
            let range = curve.range_tuple();
            let tol = 0.00005 * scale;
            let d_tol = 0.00005_f64.sqrt() * scale;
            let fitted = BSplineCurve::cubic_approximation(&curve, range, tol / 4., d_tol, 16)
                .expect("adjacent adaptive pieces share their boundary exactly");
            assert_eq!(fitted.range_tuple(), range);
            assert_eq!(*fitted.control_point(0), curve.subs(range.0));
            assert_eq!(
                *fitted.control_points().last().unwrap(),
                curve.subs(range.1)
            );
            assert!(fitted.front().distance(curve.front()) < 1e-12);
            assert!(fitted.back().distance(curve.back()) < 1e-12);
            let (knots, multiplicities) = fitted.knot_vec().to_single_multi();
            assert!(knots.len() > 2);
            assert!(multiplicities[1..multiplicities.len() - 1]
                .iter()
                .all(|&m| m == 3));
            let samples = (0..=4096)
                .map(|i| range.0 + (range.1 - range.0) * i as f64 / 4096.)
                .chain(knots.windows(2).flat_map(|span| {
                    (0..=16).map(move |i| span[0] + (span[1] - span[0]) * i as f64 / 16.)
                }));
            for t in samples {
                assert!(fitted.subs(t).distance(curve.subs(t)) <= tol);
                assert!((fitted.der(t) - curve.der(t)).magnitude() <= d_tol);
            }
        }
    }
}

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
