use truck_modeling::*;

#[test]
fn transformed_revolved_boundaries_use_the_surface_coordinate_system() {
    let line = Curve::Line(Line(Point3::new(5., 0., 0.), Point3::new(3., 0., 8.)));
    let profiles = [
        line.clone(),
        Curve::BSplineCurve(BSplineCurve::from(Line(
            Point3::new(5., 0., 0.),
            Point3::new(3., 0., 8.),
        ))),
        Curve::NurbsCurve(NurbsCurve::try_from(&line).unwrap()),
        Curve::Conic(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (-0.7, 0.9)),
            Matrix4::from_translation(Vector3::new(3., 0., 0.))
                * Matrix4::from_angle_x(Rad(std::f64::consts::FRAC_PI_2)),
        )),
    ];
    for (index, profile) in profiles.into_iter().enumerate() {
        for scale in [0.1, 1., 10.] {
            for affine in [false, true] {
                let mut distortion = Matrix4::from_scale(scale);
                if affine {
                    distortion.x.y = 0.2 * scale;
                    distortion.y.z = 0.3 * scale;
                }
                let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * distortion;
                for reversed in [false, true] {
                    let mut surface = Surface::RevolutedCurve(Processor::with_transform(
                        RevolutedCurve::by_revolution(
                            profile.clone(),
                            Point3::origin(),
                            Vector3::unit_z(),
                        ),
                        transform,
                    ));
                    if reversed {
                        surface.invert();
                    }
                    for angle in [0., 1.9, 6.2] {
                        let local = profile.transformed(Matrix4::from_angle_z(Rad(angle)));
                        let boundary = local.transformed(transform);
                        assert!(
                            surface.include(&boundary),
                            "profile={index}, scale={scale}, affine={affine}, reversed={reversed}"
                        );
                        assert!(
                            !surface.include(&local),
                            "local coordinates are not world-space boundary points"
                        );
                        let outside = local.transformed(
                            transform * Matrix4::from_translation(Vector3::new(20., 0., 0.)),
                        );
                        assert!(!surface.include(&outside));
                    }
                    let (a, b) = profile.range_tuple();
                    for t in [a, a.midpoint(b), b] {
                        let p = profile.subs(t);
                        let circle = Curve::Conic(Processor::with_transform(
                            TrimmedCurve::new(UnitCircle::new(), (0.2, 1.9)),
                            transform
                                * Matrix4::from_translation(Vector3::new(0., 0., p.z))
                                * Matrix4::from_nonuniform_scale(
                                    p.x.hypot(p.y),
                                    p.x.hypot(p.y),
                                    1.,
                                ),
                        ));
                        assert!(surface.include(&circle), "cross-section profile={index}, scale={scale}, affine={affine}, reversed={reversed}");
                    }
                }
            }
        }
    }
}

#[test]
fn singular_revolved_transform_rejects_inclusion_without_panicking() {
    let curve = Curve::Line(Line(Point3::new(5., 0., 0.), Point3::new(3., 0., 8.)));
    let surface = Surface::RevolutedCurve(Processor::with_transform(
        RevolutedCurve::by_revolution(curve.clone(), Point3::origin(), Vector3::unit_z()),
        Matrix4::from_scale(0.),
    ));
    assert!(!surface.include(&curve));
}
