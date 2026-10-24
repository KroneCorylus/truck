use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;
#[test]
fn clustered_spline_pcurve_trim_preserves_its_image() {
    let vk = KnotVec::from(vec![0., 0., 0., 0.5, 0.5, 0.5000001, 0.5000001, 1., 1., 1.]);
    let points = (0..2)
        .map(|i| {
            (0..7)
                .map(|j| Point3::new(i as f64, j as f64, 2.))
                .collect()
        })
        .collect();
    let surface = BSplineSurface::new((KnotVec::bezier_knot(1), vk), points);
    let raw = Curve3D::PCurve(PCurve::new(
        Box::new(Curve2D::Line(Line(
            Point2::new(0.3, 0.1),
            Point2::new(0.3, 0.50000015),
        ))),
        Box::new(Surface::BSplineSurface(surface)),
    ));
    let modeled = Curve::try_from(&raw).unwrap();
    for t in [0., 0.25, 0.5, 0.75, 1.] {
        assert!(raw.subs(t).distance(modeled.subs(t)) < 1e-10);
    }
    assert!(raw.back().distance(modeled.back()) < 1e-10);
}
