use truck_modeling::*;

#[test]
fn revolved_cylinder_inverse_preserves_membership_and_orientation() {
    let curve: Curve = Line(Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 0.0, 4.0)).into();
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(7.0, -3.0, 2.0)) * Matrix4::from_angle_x(Rad(0.7)),
        Matrix4::from_nonuniform_scale(2.0, 1.0, 0.5),
    ] {
        for inverted in [false, true] {
            let mut processor = Processor::with_transform(
                RevolutedCurve::by_revolution(curve.clone(), Point3::origin(), Vector3::unit_z()),
                transform,
            );
            if inverted {
                processor.invert();
            }
            let surface = Surface::RevolutedCurve(processor.clone());
            for u in [-0.2, 0.0, 0.25, 0.9, 1.0, 1.2] {
                for v in [0.0, 0.1, 1.0, 3.0, 6.2] {
                    let (u, v) = if inverted { (v, u) } else { (u, v) };
                    for offset in [-0.1, 0.0, 0.1] {
                        let point = surface.subs(u, v) + surface.normal(u, v) * offset;
                        let expected = processor.search_parameter(point, None, 100);
                        let actual = surface.search_parameter(point, None, 100);
                        assert_eq!(actual.is_some(), expected.is_some());
                        if let Some((u, v)) = actual {
                            assert_near!(surface.subs(u, v), point);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn native_revolved_nearest_matches_grid_search_under_similarity_transforms() {
    let curves: Vec<Curve> = vec![
        Line(Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 0.0, 4.0)).into(),
        Line(Point3::new(1.0, 0.0, 0.0), Point3::new(3.0, 0.0, 4.0)).into(),
        BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(3.0, 0.0, 2.0),
                Point3::new(2.0, 0.0, 4.0),
            ],
        )
        .into(),
    ];
    for curve in curves {
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(7.0, -3.0, 2.0))
                * Matrix4::from_angle_x(Rad(0.7))
                * Matrix4::from_scale(2.0),
            Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0),
            Matrix4::from_nonuniform_scale(2.0, 1.0, 1.0),
        ] {
            for inverted in [false, true] {
                let mut processor = Processor::with_transform(
                    RevolutedCurve::by_revolution(
                        curve.clone(),
                        Point3::origin(),
                        Vector3::unit_z(),
                    ),
                    transform,
                );
                if inverted {
                    processor.invert();
                }
                let surface = Surface::RevolutedCurve(processor.clone());
                for (u, v) in [(0.1, 0.05), (0.3, 1.0), (0.8, 3.0), (0.95, 6.2)] {
                    let (u, v) = if inverted { (v, u) } else { (u, v) };
                    for offset in [-0.1, 0.0, 0.1] {
                        let point = surface.subs(u, v) + surface.normal(u, v) * offset;
                        let seed = algo::surface::presearch(
                            &processor,
                            point,
                            processor.range_tuple(),
                            100,
                        );
                        let expected =
                            algo::surface::search_nearest_parameter(&processor, point, seed, 100)
                                .unwrap();
                        let actual = surface.search_nearest_parameter(point, None, 100).unwrap();
                        assert_near!(
                            surface.subs(actual.0, actual.1),
                            surface.subs(expected.0, expected.1)
                        );
                    }
                }
            }
        }
    }
}
