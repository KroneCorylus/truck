use proptest::{prelude::*, property_test};
use std::f64::consts::{PI, TAU};
use truck_geometry::prelude::*;

#[property_test]
fn search_parameter(#[strategy = 0.0..=TAU] t: f64) {
    let circle = UnitCircle::<Point2>::new();
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, None, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter(#[strategy = 0.0..=TAU] t: f64, #[strategy = 0.1..=5f64] a: f64) {
    let circle = UnitCircle::<Point2>::new();
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, None, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_parameter_with_parameter_hint(
    #[strategy = -100.0..=100.0] t: f64,
    #[strategy = -0.2..=0.2] d: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, t + d, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter_with_parameter_hint(
    #[strategy = -100.0..=100.0] t: f64,
    #[strategy = -0.2..=0.2] d: f64,
    #[strategy = 0.1..=5.0] a: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, t + d, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_parameter_with_range(
    #[strategy = -100.0..=100.0] start: f64,
    #[strategy = 0.01..=PI] length: f64,
    #[strategy = -0.1..=1.1] lerp_ratio: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let range = (start, start + length);
    let t = start + lerp_ratio * length;
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, range, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter_with_range(
    #[strategy = -100.0..=100.0] start: f64,
    #[strategy = 0.01..=PI] length: f64,
    #[strategy = -0.1..=1.1] lerp_ratio: f64,
    #[strategy = 0.1..=5.0] a: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let range = (start, start + length);
    let t = start + lerp_ratio * length;
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, range, 1).unwrap();
    prop_assert_near!(s, t);
}

#[property_test]
fn to_nurbs(#[strategy = 0.0..=PI] t0: f64, #[strategy = PI..=TAU] t1: f64) {
    let circle = UnitCircle::<Point2>::new();
    let arc = TrimmedCurve::new(circle, (t0, t1));
    let bsp: NurbsCurve<_> = arc.to_same_geometry();
    prop_assert_near!(bsp.front(), arc.front());
    prop_assert_near!(bsp.back(), arc.back());
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        let p = bsp.subs(t).to_vec();
        let der = bsp.der(t);
        prop_assert_near!(p.magnitude2(), 1.0);
        prop_assert!(der.dot(p).so_small());
    }
}

#[test]
fn parameter_division() {
    let c = UnitCircle::<Point2>::new();
    let (_div, pts) = c.parameter_division(c.range_tuple(), 0.05);
    for a in pts.windows(2) {
        let p = a[0].midpoint(a[1]);
        assert!(p.to_vec().magnitude() > 0.95);
    }
}

#[test]
fn inverse_near_axes_preserves_parameter_precision() {
    let circle = UnitCircle::<Point2>::new();
    for t in [
        31.41596780328447,
        -31.41596780328447,
        1.0e-10,
        -1.0e-10,
        PI - 1.0e-10,
        PI + 1.0e-10,
        TAU - 1.0e-10,
        TAU + 1.0e-10,
    ] {
        let point = circle.subs(t);
        for hint in [
            SPHint1D::Parameter(t),
            SPHint1D::Parameter(t - 0.1),
            SPHint1D::Parameter(t + 0.1),
            SPHint1D::Range(t - 0.1, t + 0.1),
        ] {
            assert_near2!(circle.search_parameter(point, hint, 1).unwrap(), t);
            for radius in [0.1, 1.0, 5.0] {
                let parameter = circle
                    .search_nearest_parameter(point * radius, hint, 1)
                    .unwrap();
                assert_near2!(parameter, t);
            }
        }
    }
}

#[test]
fn inverse_axes_and_signed_zero_stay_in_one_revolution() {
    let circle = UnitCircle::<Point2>::new();
    for zero in [0.0, -0.0] {
        for (point, expected) in [
            (Point2::new(1.0, zero), TAU),
            (Point2::new(-1.0, zero), PI),
            (Point2::new(zero, 1.0), PI / 2.0),
            (Point2::new(zero, -1.0), 3.0 * PI / 2.0),
        ] {
            assert_eq!(circle.search_parameter(point, None, 1), Some(expected));
            assert_eq!(
                circle.search_nearest_parameter(point, None, 1),
                Some(expected)
            );
            for turn in [-2.0, 0.0, 2.0] {
                let hint = expected + turn * TAU;
                assert_near2!(circle.search_parameter(point, hint, 1).unwrap(), hint);
                assert_near2!(
                    circle.search_nearest_parameter(point, hint, 1).unwrap(),
                    hint
                );
            }
        }
    }
}

#[test]
fn scaled_circle_accepts_submicron_local_chord_tolerance() {
    let curve = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, PI / 2.0)),
        Matrix4::from_scale(100.0),
    );
    let (parameters, points) = curve.parameter_division((0.0, PI / 2.0), 1e-6);
    assert!(parameters.len() > 100);
    for (ts, ps) in parameters.windows(2).zip(points.windows(2)) {
        let middle = curve.subs((ts[0] + ts[1]) / 2.0);
        assert!(middle.distance(ps[0].midpoint(ps[1])) <= 1e-6);
    }
    assert_near!(points[0], Point3::new(100.0, 0.0, 0.0));
    assert_near!(*points.last().unwrap(), Point3::new(0.0, 100.0, 0.0));
}

#[test]
fn complete_and_nearly_complete_circles_have_regular_nurbs_images() {
    for start in [0., 0.37, -9.1] {
        for span in [TAU, TAU - 1e-8, 1.5 * PI, PI, 1e-9, 0.]
            .into_iter()
            .flat_map(|span| [span, -span])
        {
            let arc = TrimmedCurve::new(UnitCircle::<Point2>::new(), (start, start + span));
            let curve: NurbsCurve<Vector3> = arc.to_same_geometry();
            assert!(curve.front().distance(arc.front()) < 1e-12);
            assert!(curve.back().distance(arc.back()) < 1e-12);
            let mut previous = curve.front().to_vec();
            let mut turn = 0.;
            for i in 0..=128 {
                let t = i as f64 / 128.;
                let point = curve.subs(t).to_vec();
                assert!(
                    (point.magnitude2() - 1.).abs() < 1e-12,
                    "{start} {span} {t}: {point:?}"
                );
                let tangent = curve.der(t);
                assert!(point.dot(tangent).abs() < 1e-11);
                turn += (previous.x * point.y - previous.y * point.x).atan2(previous.dot(point));
                previous = point;
            }
            assert!((turn - span).abs() < 1e-11, "turn {turn} span {span}");
            assert!(curve
                .non_rationalized()
                .control_points()
                .iter()
                .all(|p| p.z > 0.));
        }
    }
}

#[test]
fn full_circle_images_survive_spatial_affine_maps_and_reversal() {
    let transform = Matrix4::from_cols(
        Vector4::new(150., 20., -30., 0.),
        Vector4::new(-50., 70., 90., 0.),
        Vector4::new(0., 0., 1., 0.),
        Vector4::new(13., -7., 23., 1.),
    );
    let inverse = transform.invert().unwrap();
    let arc = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.37, 0.37 + TAU)),
        transform,
    );
    for arc in [arc, arc.inverse()] {
        let curve: NurbsCurve<Vector4> = arc.to_same_geometry();
        assert!(curve.front().distance(arc.front()) < 1e-10);
        assert!(curve.back().distance(arc.back()) < 1e-10);
        for i in 0..=128 {
            let p = inverse.transform_point(curve.subs(i as f64 / 128.));
            assert!((p.x * p.x + p.y * p.y - 1.).abs() < 1e-12);
            assert!(p.z.abs() < 1e-10);
        }
        let (knots, counts) = curve.knot_vec().to_single_multi();
        assert!(counts[1..knots.len() - 1].iter().all(|&n| n <= 2));
        assert!(curve
            .non_rationalized()
            .control_points()
            .iter()
            .all(|p| p.w > 0.));
    }
}
