use std::f64::consts::PI;
use truck_geometry::prelude::*;

/// Parameter searches on a trimmed periodic curve answer inside the trimmed range.
#[test]
fn trimmed_circle_searches_stay_in_range() {
    let arc = TrimmedCurve::new(UnitCircle::<Point3>::new(), (PI, 2.0 * PI));

    let t = arc
        .search_parameter(Point3::new(0.0, -1.0, 0.0), None, 0)
        .unwrap();
    assert_near!(t, 1.5 * PI);
    let t = arc
        .search_parameter(Point3::new(1.0, 0.0, 0.0), None, 0)
        .unwrap();
    assert_near!(
        t,
        2.0 * PI,
        "the end point is reported at the end of the range"
    );
    assert!(arc
        .search_parameter(Point3::new(0.0, 1.0, 0.0), None, 0)
        .is_none());

    let t = arc
        .search_nearest_parameter(Point3::new(0.5, 0.5, 0.0), None, 0)
        .unwrap();
    assert_near!(
        t,
        2.0 * PI,
        "beyond the end, the nearest point is the end vertex"
    );

    let ellipse = Processor::new(arc).transformed(Matrix4::from_nonuniform_scale(2.0, 1.0, 1.0));
    let t = ellipse
        .search_nearest_parameter(Point3::new(1.0, 1.0, 0.0), None, 100)
        .unwrap();
    assert!(
        (PI..=2.0 * PI).contains(&t),
        "nearest parameter {t} left the range"
    );
    let t = ellipse
        .search_nearest_parameter(Point3::new(0.0, -3.0, 0.0), None, 100)
        .unwrap();
    assert_near!(t, 1.5 * PI);
}
