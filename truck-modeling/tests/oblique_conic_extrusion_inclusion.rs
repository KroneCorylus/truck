use truck_modeling::*;

#[test]
fn affine_conic_extrusions_preserve_exact_inclusion_and_bounded_trims() {
    let rotation = Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    let mut shear = Matrix4::identity();
    shear.y.x = 0.7;
    for scale in [0.002, 1., 20.] {
        for reversed in [false, true] {
            for oblique in [false, true] {
                let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * rotation
                    * Matrix4::from_nonuniform_scale(2. * scale, scale, scale)
                    * shear;
                let mut conic = Processor::with_transform(
                    TrimmedCurve::new(UnitCircle::new(), (0.1, 1.7)),
                    transform,
                );
                if reversed {
                    conic.invert();
                }
                let axis = if oblique {
                    Vector3::new(3., 2., 40.)
                } else {
                    Vector3::new(0., 0., 40.)
                };
                let vector = rotation.transform_vector(axis * scale);
                let base = Curve::Conic(conic);
                let raw = ExtrudedCurve::by_extrusion(base.clone(), vector);
                let surface = Surface::Extruded(raw.clone());
                for u in [0.1, 0.5, 1.7] {
                    let line = Curve::Line(Line(raw.subs(u, 0.), raw.subs(u, 1.)));
                    assert!(
                        surface.include(&line),
                        "scale={scale}, reversed={reversed}, oblique={oblique}, u={u}"
                    );
                    assert!(
                        !surface.include(&Curve::Line(Line(raw.subs(u, -0.1), raw.subs(u, 0.9))))
                    );
                    assert!(
                        !surface.include(&Curve::Line(Line(raw.subs(u, 0.1), raw.subs(u, 1.1))))
                    );
                    let normal = surface.normal(u, 0.5) * scale * 0.01;
                    assert!(!surface.include(&line.transformed(Matrix4::from_translation(normal))));
                    for v in [0., 0.3, 1.] {
                        let p = raw.subs(u, v);
                        let uv = surface.search_parameter(p, None, 100).unwrap();
                        assert!(surface.subs(uv.0, uv.1).distance(p) < 1e-8);
                        let uv = surface.search_nearest_parameter(p, None, 100).unwrap();
                        assert!(surface.subs(uv.0, uv.1).distance(p) < 1e-8);
                    }
                }
                assert!(surface.include(&base));
                assert!(surface.include(&base.transformed(Matrix4::from_translation(vector))));
                for u in [0., 1.8] {
                    assert!(!surface.include(&Curve::Line(Line(raw.subs(u, 0.), raw.subs(u, 1.)))));
                }
            }
        }
    }
}
