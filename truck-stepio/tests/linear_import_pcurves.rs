use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

#[test]
fn planar_and_cylindrical_generator_pcurves_preserve_exact_spatial_lines() {
    let matrix = Matrix4::from_translation(Vector3::new(13., -7., 23.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    for reversed in [false, true] {
        let plane = Surface::ElementarySurface(ElementarySurface::Plane(
            Plane::new(
                Point3::origin(),
                Point3::new(1., 0., 0.),
                Point3::new(0., 1., 0.),
            )
            .transformed(matrix),
        ));
        let mut cylinder = Processor::new(RevolutedCurve::by_revolution(
            Line(Point3::new(2., 0., 0.), Point3::new(2., 0., 1.)),
            Point3::origin(),
            Vector3::unit_z(),
        ))
        .transformed(matrix);
        if reversed {
            cylinder.invert();
        }
        let uv = if reversed {
            Line(Point2::new(0.73, -3.), Point2::new(0.73, 20.))
        } else {
            Line(Point2::new(-3., 0.73), Point2::new(20., 0.73))
        };
        for (surface, line) in [
            (plane, Line(Point2::new(-3., 5.), Point2::new(7., 11.))),
            (
                Surface::ElementarySurface(ElementarySurface::CylindricalSurface(cylinder)),
                uv,
            ),
        ] {
            let mut source = Curve3D::PCurve(PCurve::new(
                Box::new(Curve2D::Line(line)),
                Box::new(surface),
            ));
            if reversed {
                source.invert();
            }
            let mapped = Curve::try_from(&source).unwrap();
            assert!(matches!(mapped, Curve::Line(_)));
            for i in 0..=64 {
                let t = i as f64 / 64.;
                assert!(mapped.subs(t).distance(source.subs(t)) < 1e-10);
                assert!(mapped.der(t).distance(source.der(t)) < 1e-10);
                assert!(mapped.der2(t).distance(source.der2(t)) < 1e-10);
            }
        }
    }
}

#[test]
fn numerical_seam_offsets_preserve_generators_and_curved_pcurves_stay_exact() {
    let cylinder = Surface::ElementarySurface(ElementarySurface::CylindricalSurface(
        Processor::new(RevolutedCurve::by_revolution(
            Line(Point3::new(10., 0., 0.), Point3::new(10., 0., 20.)),
            Point3::origin(),
            Vector3::unit_z(),
        )),
    ));
    for delta in [4. * f64::EPSILON, 0.01] {
        let source = Curve3D::PCurve(PCurve::new(
            Box::new(Curve2D::Line(Line(
                Point2::new(0., 3.),
                Point2::new(1., 3. + delta),
            ))),
            Box::new(cylinder.clone()),
        ));
        let mapped = Curve::try_from(&source);
        if delta < 1e-12 {
            let mapped = mapped.unwrap();
            for i in 0..=64 {
                let t = i as f64 / 64.;
                assert!(mapped.subs(t).distance(source.subs(t)) < 1e-12);
                assert!(mapped.der(t).distance(source.der(t)) < 1e-12);
            }
        } else {
            let mapped = mapped.unwrap();
            assert!(matches!(mapped, Curve::PCurve(_)));
            for i in 0..=64 {
                let t = i as f64 / 64.;
                assert!(mapped.subs(t).distance(source.subs(t)) < 1e-12);
                assert!(mapped.der(t).distance(source.der(t)) < 1e-12);
                assert!(mapped.der2(t).distance(source.der2(t)) < 1e-12);
            }
        }
    }
}
