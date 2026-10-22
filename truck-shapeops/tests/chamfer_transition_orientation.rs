mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;
fn bracket() -> Solid {
    let vertices = builder::vertices(
        [
            [0., 0.],
            [20., 0.],
            [20., 10.],
            [10., 10.],
            [10., 20.],
            [0., 20.],
        ]
        .map(|p| Point3::new(p[0], p[1], 0.)),
    );
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}
#[test]
fn convex_chamfer_transitions_on_nonconvex_solids_preserve_material() {
    for moved in [false, true] {
        let scale = if moved { 2.5 } else { 1. };
        let matrix = if moved {
            Matrix4::from_translation(Vector3::new(7., -4., 11.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.61))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let input = builder::transformed(&bracket(), matrix);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for bottom in [false, true] {
            for sideways in [false, true] {
                let local = |p: Point3| {
                    let p = if sideways {
                        Point3::new(p.y, p.x, p.z)
                    } else {
                        p
                    };
                    if bottom {
                        Point3::new(p.x, p.y, 10. - p.z)
                    } else {
                        p
                    }
                };
                let ids = [Point3::new(0., 0., 5.), Point3::new(15., 0., 10.)].map(|p| {
                    common::blend::edge_through(
                        &input.boundaries()[0],
                        matrix.transform_point(local(p)),
                    )
                    .id()
                });
                for (a, b) in [(0.08, 0.1), (0.1, 0.08), (0.2, 0.3), (0.3, 0.2)] {
                    let mut selected = vec![(ids[0], [a * scale; 2]), (ids[1], [b * scale; 2])];
                    for _ in 0..2 {
                        let result = try_chamfer_solid_edges_with_distances(
                            &input,
                            &selected,
                            0.001 * scale,
                        )
                        .unwrap();
                        // The transition separates the two cut prisms by z=b*x/a.
                        let expected = (3000. - 10. * a * a / 2. - 20. * b * b / 2.
                            + a * b * (a + b) / 6.)
                            * scale.powi(3);
                        assert!(result.solid.is_geometric_consistent());
                        common::assert_solid(&result.solid, expected, &[0], 0.001 * scale);
                        common::blend::assert_step(&result.solid, expected, 0.001 * scale);
                        assert_eq!(result.generated_faces.len(), 3);
                        for p in [
                            Point3::new(a, 0., 10. - b),
                            Point3::new(0., a, 10.),
                            Point3::new(0., b, 10.),
                        ] {
                            assert!(result
                                .solid
                                .vertex_iter()
                                .any(|v| v.point().near(&matrix.transform_point(local(p)))));
                        }
                        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                        selected.reverse();
                    }
                }
            }
        }
    }
}
