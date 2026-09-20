use truck_modeling::*;

#[test]
fn exact_cylinder_boundaries_include_at_small_and_large_scales() {
    let rotation = Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    for radius in [0.002, 0.2, 20.] {
        for reversed in [false, true] {
            let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * rotation
                * Matrix4::from_scale(radius);
            let mut circle = Processor::with_transform(
                TrimmedCurve::new(UnitCircle::new(), (0.1, 1.7)),
                transform,
            );
            if reversed {
                circle.invert();
            }
            let curve = Curve::Conic(circle);
            let vector = rotation.transform_vector(Vector3::new(0., 0., radius * 50.));
            let raw = ExtrudedCurve::by_extrusion(curve.clone(), vector);
            let surface = Surface::Extruded(raw.clone());
            for u in [0.1, 0.5, 1.7] {
                let line = Curve::Line(Line(raw.subs(u, 0.), raw.subs(u, 1.)));
                assert!(
                    surface.include(&line),
                    "radius={radius}, reversed={reversed}, u={u}"
                );
                let outside = Curve::Line(Line(raw.subs(u, -0.1), raw.subs(u, 0.9)));
                assert!(!surface.include(&outside));
                let outside = Curve::Line(Line(raw.subs(u, 0.1), raw.subs(u, 1.1)));
                assert!(!surface.include(&outside));
                let radial = raw.normal(u, 0.5).normalize() * radius * 0.01;
                assert!(!surface.include(&line.transformed(Matrix4::from_translation(radial))));
            }
            assert!(surface.include(&curve));
            assert!(surface.include(&curve.transformed(Matrix4::from_translation(vector))));
            for u in [0., 1.8] {
                assert!(!surface.include(&Curve::Line(Line(raw.subs(u, 0.), raw.subs(u, 1.)))));
            }
        }
    }
}
