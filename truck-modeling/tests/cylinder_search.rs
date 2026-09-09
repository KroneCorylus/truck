use std::f64::consts::{PI, TAU};
use truck_modeling::*;

fn extrusion(
    transform: Matrix4,
    range: (f64, f64),
    reversed: bool,
    vector: Vector3,
) -> ExtrudedCurve<Curve, Vector3> {
    let mut circle =
        Processor::with_transform(TrimmedCurve::new(UnitCircle::new(), range), transform);
    if reversed {
        circle.invert();
    }
    ExtrudedCurve::by_extrusion(Curve::Conic(circle), vector)
}

#[test]
fn round_cylinders_match_general_projection() {
    let rotation = Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, -1.0).normalize(), Rad(0.7));
    for scale in [0.01, 1.0, 100.0] {
        let transform = Matrix4::from_translation(Vector3::new(3.0, -2.0, 1.0))
            * rotation
            * Matrix4::from_scale(scale);
        let vector = rotation.transform_vector(Vector3::new(0.0, 0.0, 2.0 * scale));
        for range in [(-1.2, 0.9), (0.0, TAU), (PI, 1.8 * PI), (12.0, 13.0)] {
            for reversed in [false, true] {
                let raw = extrusion(transform, range, reversed, vector);
                let surface = Surface::Extruded(raw.clone());
                for t in [0.001, 0.2, 0.5, 0.8, 0.999] {
                    let u = range.0 + (range.1 - range.0) * t;
                    for v in [-0.4, 0.2, 0.9, 1.7] {
                        for offset in [-0.3, 0.0, 0.3] {
                            let expected = raw.subs(u, v);
                            let point = expected + raw.normal(u, v) * offset * scale;
                            for hint_u in [u - 0.1, u + 0.1, u - TAU, u + TAU] {
                                let hint = SPHint2D::Parameter(hint_u, v + 0.2);
                                if offset == 0.0 {
                                    let (found_u, found_v) = surface
                                        .search_parameter(point, hint, 100)
                                        .expect("hinted point lies on cylinder");
                                    assert_near!(surface.subs(found_u, found_v), point);
                                    let (old_u, old_v) =
                                        raw.search_parameter(point, hint, 100).unwrap();
                                    assert_near!(found_u, old_u);
                                    assert_near!(found_v, old_v);
                                } else {
                                    assert!(surface.search_parameter(point, hint, 100).is_none());
                                }
                            }
                            let exact = surface.search_parameter(point, None, 100);
                            if offset == 0.0 {
                                let (u, v) = exact.expect("point lies on cylinder");
                                assert_near!(surface.subs(u, v), point);
                            } else {
                                assert!(exact.is_none());
                            }
                            let optimized =
                                surface.search_nearest_parameter(point, None, 100).unwrap();
                            assert_near!(surface.subs(optimized.0, optimized.1), expected);
                            if let Some(general) = raw.search_nearest_parameter(point, None, 100) {
                                assert_near!(
                                    surface.subs(optimized.0, optimized.1),
                                    raw.subs(general.0, general.1)
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_hints_and_other_extrusions_keep_general_search() {
    let mut shear = Matrix4::identity();
    shear.y.x = 0.2;
    for transform in [
        Matrix4::identity(),
        Matrix4::from_nonuniform_scale(2.0, 1.0, 1.0),
        shear,
    ] {
        for vector in [Vector3::unit_z(), Vector3::new(0.3, 0.1, 1.0)] {
            let raw = extrusion(transform, (0.2, 2.0), false, vector);
            let surface = Surface::Extruded(raw.clone());
            for point in [
                Point3::new(1.2, 0.6, 0.7),
                Point3::new(-1.0, -1.0, 2.0),
                Point3::origin(),
            ] {
                for hint in [
                    SPHint2D::Parameter(0.5, 0.2),
                    SPHint2D::Range((0.3, 1.0), (0.0, 0.5)),
                ] {
                    assert_eq!(
                        surface.search_parameter(point, hint, 100),
                        raw.search_parameter(point, hint, 100)
                    );
                    assert_eq!(
                        surface.search_nearest_parameter(point, hint, 100),
                        raw.search_nearest_parameter(point, hint, 100)
                    );
                }
                if transform != Matrix4::identity() || vector != Vector3::unit_z() {
                    assert_eq!(
                        surface.search_parameter(point, None, 100),
                        raw.search_parameter(point, None, 100)
                    );
                    assert_eq!(
                        surface.search_nearest_parameter(point, None, 100),
                        raw.search_nearest_parameter(point, None, 100)
                    );
                }
            }
        }
    }
}

#[test]
fn unsupported_cylinder_seeds_fall_back() {
    let mut projective = Matrix4::identity();
    projective.x.w = 0.1;
    for (transform, vector) in [
        (Matrix4::identity(), Vector3::unit_z()),
        (Matrix4::identity(), Vector3::zero()),
        (
            Matrix4::from_nonuniform_scale(0.0, 1.0, 1.0),
            Vector3::unit_z(),
        ),
        (projective, Vector3::unit_z()),
    ] {
        let raw = extrusion(transform, (0.2, 1.0), false, vector);
        let surface = Surface::Extruded(raw.clone());
        for point in [Point3::origin(), Point3::new(-2.0, -1.0, 0.5)] {
            assert_eq!(
                surface.search_nearest_parameter(point, None, 100),
                raw.search_nearest_parameter(point, None, 100)
            );
            assert_eq!(
                surface.search_parameter(point, None, 100),
                raw.search_parameter(point, None, 100)
            );
        }
    }
}
