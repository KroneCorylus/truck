use std::f64::consts::PI;
use truck_modeling::*;

const N: usize = 64;

fn conic(edge: &Edge) -> Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    match edge.oriented_curve() {
        Curve::Conic(curve) => curve,
        curve => panic!("circle_arc did not yield Curve::Conic: {curve:?}"),
    }
}

/// Asserts that the NURBS version traces the same arc as the conic, in the same direction.
///
/// Each NURBS sample is placed by its angle about the center, and the conic is evaluated at that
/// angle, so the comparison needs no parameter search.
fn assert_matches_nurbs(
    conic: &Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>,
    center: Point3,
    radius: f64,
) {
    let nurbs: NurbsCurve<Vector4> = conic.to_same_geometry();
    let (t0, t1) = conic.range_tuple();
    let start = conic.front() - center;
    let axis = start
        .cross(conic.subs((t0 + t1) / 2.0) - center)
        .normalize();
    let (s0, s1) = nurbs.range_tuple();
    let mut last = 0.0;
    for i in 0..=N {
        let q = nurbs.subs(s0 + (s1 - s0) * i as f64 / N as f64);
        assert!(
            (q.distance(center) - radius).abs() < 1e-12,
            "NURBS sample off the circle: {q:?}"
        );
        let v = q - center;
        let mut angle = f64::atan2(axis.dot(start.cross(v)), start.dot(v));
        if angle < -1e-9 {
            angle += 2.0 * PI;
        }
        assert!(angle >= last - 1e-9, "NURBS turns back at angle {angle}");
        last = angle;
        let d = conic.subs(t0 + angle).distance(q);
        assert!(
            d < 1e-12,
            "conic and NURBS disagree by {d} at angle {angle}"
        );
    }
    assert!(
        (last - (t1 - t0)).abs() < 1e-12,
        "NURBS sweeps {last}, conic {}",
        t1 - t0
    );
}

#[test]
fn circle_arc_by_transit_is_conic() {
    let center = Point3::new(1.0, -2.0, 3.0);
    let radius = 2.5;
    let vertex0 = builder::vertex(center + Vector3::new(radius, 0.0, 0.0));
    let vertex1 = builder::vertex(center + Vector3::new(0.0, radius, 0.0));
    let transit = center + Vector3::new(-radius, 0.0, 0.0);
    let edge: Edge = builder::circle_arc(&vertex0, &vertex1, transit);
    let conic = conic(&edge);
    assert!(conic.front().distance(vertex0.point()) < 1e-12);
    assert!(conic.back().distance(vertex1.point()) < 1e-12);
    let (t0, t1) = conic.range_tuple();
    assert!(
        (t1 - t0 - 1.5 * PI).abs() < 1e-12,
        "arc through the transit spans 270 degrees"
    );
    assert!(conic.search_parameter(transit, None, 10).is_some());
    assert_matches_nurbs(&conic, center, radius);
}

#[test]
fn circle_arc_by_tangent_is_conic() {
    let vertex0 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let vertex1 = builder::vertex(Point3::new(0.0, 1.0, 0.0));
    let edge: Edge = builder::circle_arc(&vertex0, &vertex1, Vector3::new(0.0, 1.0, 0.0));
    let conic = conic(&edge);
    let (t0, t1) = conic.range_tuple();
    assert!((t1 - t0 - 0.5 * PI).abs() < 1e-12);
    assert_matches_nurbs(&conic, Point3::origin(), 1.0);
}

#[test]
fn inverted_conic_reverses_and_stays_exact() {
    let vertex0 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let vertex1 = builder::vertex(Point3::new(-1.0, 0.0, 0.0));
    let edge: Edge = builder::circle_arc(&vertex0, &vertex1, Point3::new(0.0, 1.0, 0.0));
    let conic = conic(&edge.inverse());
    assert!(conic.front().distance(vertex1.point()) < 1e-12);
    assert!(conic.back().distance(vertex0.point()) < 1e-12);
    assert_matches_nurbs(&conic, Point3::origin(), 1.0);
}
