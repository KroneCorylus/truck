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
                            &[],
                            0.001 * scale,
                        )
                        .unwrap();
                        // Integrate the overlap of the removed wedges: (a-y)*(b-y).
                        let m = a.min(b);
                        let expected = (3000. - 10. * a * a / 2. - 20. * b * b / 2. + a * b * m
                            - (a + b) * m * m / 2.
                            + m.powi(3) / 3.)
                            * scale.powi(3);
                        assert!(result.solid.is_geometric_consistent());
                        common::assert_solid(&result.solid, expected, &[0], 0.001 * scale);
                        common::blend::assert_step(&result.solid, expected, 0.001 * scale);
                        assert_eq!(result.generated_faces.len(), 2);
                        for p in [
                            Point3::new(a, 0., 10. - b),
                            Point3::new(0., a.max(b), 10.),
                            Point3::new((a - b).max(0.), m, 10. - (b - a).max(0.)),
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

#[test]
fn two_edge_box_cuts_equal_the_union_of_two_analytic_wedges() {
    for scale in [0.1_f64, 1., 2.5] {
        let matrix = Matrix4::from_translation(Vector3::new(7., -4., 11.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.61))
            * Matrix4::from_scale(scale);
        let inverse = matrix.invert().unwrap();
        let input = builder::transformed(
            &common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.)),
            matrix,
        );
        let before = serde_json::to_string(&input.compress()).unwrap();
        let ids = [Point3::new(0., 0., 5.), Point3::new(5., 0., 10.)].map(|p| {
            common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p)).id()
        });
        for [u, v, w, t] in [
            [1., 1., 1., 1.],
            [1., 1., 0.5, 0.5],
            [0.5, 0.5, 1., 1.],
            [0.3, 0.8, 0.7, 0.4],
            [0.7, 0.4, 0.3, 0.8],
        ] {
            let mut selected: Vec<_> = ids
                .iter()
                .enumerate()
                .map(|(i, &id)| {
                    let distances: Vec<_> = input
                        .face_iter()
                        .filter(|face| face.edge_iter().any(|edge| edge.id() == id))
                        .map(|face| {
                            let n = inverse
                                .transform_vector(face.oriented_surface().normal(0., 0.))
                                .normalize();
                            scale
                                * if i == 0 {
                                    if n.x < -0.9 {
                                        v
                                    } else {
                                        u
                                    }
                                } else if n.z > 0.9 {
                                    t
                                } else {
                                    w
                                }
                        })
                        .collect();
                    (id, [distances[0], distances[1]])
                })
                .collect();
            let m = v.min(t);
            // Integrate u*w*(1-y/v)*(1-y/t) over the shared wedge depth.
            let overlap = u * w * (m - (1. / v + 1. / t) * m * m / 2. + m.powi(3) / (3. * v * t));
            let expected = (1000. - 5. * (u * v + w * t) + overlap) * scale.powi(3);
            for _ in 0..2 {
                let output =
                    try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale)
                        .unwrap();
                assert_eq!(output.generated_faces.len(), 2);
                assert!(output.solid.is_geometric_consistent());
                common::assert_solid(&output.solid, expected, &[0], 0.001 * scale);
                common::blend::assert_step(&output.solid, expected, 0.001 * scale);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                selected.reverse();
            }
        }
        let oversized = [(ids[0], [20. * scale; 2]), (ids[1], [0.5 * scale; 2])];
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &oversized, &[], 0.001 * scale).is_err()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
