use truck_stepio::r#in::step_geometry::*;

#[test]
fn spline_traces_on_cones_and_cylinders_keep_their_parameterization() {
    for cylinder in [false, true] {
        for reversed in [false, true] {
            let line = Line(
                Point3::new(3.0, 0.0, 0.0),
                Point3::new(if cylinder { 3.0 } else { 4.0 }, 0.0, 2.0),
            );
            let mut swept = Processor::with_transform(
                RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_z()),
                Matrix4::from_translation(Vector3::new(12.0, -7.0, 8.0))
                    * Matrix4::from_angle_x(Rad(0.4)),
            );
            if reversed {
                swept.invert();
            }
            let surface = Surface::ElementarySurface(if cylinder {
                ElementarySurface::CylindricalSurface(swept)
            } else {
                ElementarySurface::ConicalSurface(swept)
            });
            let trace = BSplineCurve::new(
                KnotVec::bezier_knot(3),
                vec![
                    Point2::new(0.2, 0.3),
                    Point2::new(0.4, 0.6),
                    Point2::new(0.7, 0.2),
                    Point2::new(0.9, 0.7),
                ],
            );
            let source = Curve3D::PCurve(PCurve::new(
                Box::new(Curve2D::BSplineCurve(trace)),
                Box::new(surface),
            ));
            let mapped = truck_modeling::Curve::try_from(&source).unwrap();
            for i in 0..=32 {
                let t = i as f64 / 32.0;
                assert!(mapped.subs(t).distance(source.subs(t)) < 1e-10);
                assert!(mapped.der(t).distance(source.der(t)) < 1e-10);
                assert!(mapped.der2(t).distance(source.der2(t)) < 1e-10);
            }
        }
    }
}
