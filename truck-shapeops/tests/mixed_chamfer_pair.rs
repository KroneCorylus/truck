mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;
const TOL: f64 = 0.001;
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
fn mixed_chamfer_pairs_have_a_planar_corner_transition() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(7., -4., 11.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.61)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let inverse = transform.invert().unwrap();
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
                let ids = [Point3::new(15., 10., 10.), Point3::new(10., 10., 5.)].map(|p| {
                    common::blend::edge_through(
                        &input.boundaries()[0],
                        transform.transform_point(local(p)),
                    )
                    .id()
                });
                let before = serde_json::to_string(&input.compress()).unwrap();
                for [a, b, c, e] in [
                    [0.0002; 4],
                    [0.2; 4],
                    [0.2, 0.3, 0.2, 0.3],
                    [0.3, 0.2, 0.4, 0.1],
                ] {
                    let mut selected: Vec<_> = ids
                        .iter()
                        .enumerate()
                        .map(|(k, &id)| {
                            let sizes = input
                                .face_iter()
                                .filter(|f| f.edge_iter().any(|edge| edge.id() == id))
                                .map(|f| {
                                    let n = inverse
                                        .transform_vector(f.oriented_surface().normal(0., 0.));
                                    if k == 0 {
                                        if n.z.abs() > 0.9 {
                                            a
                                        } else {
                                            b
                                        }
                                    } else if (if sideways { n.x } else { n.y }) > 0.9 {
                                        c
                                    } else {
                                        e
                                    }
                                })
                                .collect::<Vec<_>>();
                            (id, [sizes[0], sizes[1]])
                        })
                        .collect();
                    let result =
                        try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
                    // Integrate the concave fill, convex cut, and tapered corner cross sections.
                    let expected =
                        3000. + 10. * c * e / 2. - 10. * a * b / 2. + c * b * (a - e) / 6.;
                    assert!(result.solid.is_geometric_consistent());
                    common::assert_solid(&result.solid, expected, &[0], TOL);
                    common::blend::assert_step(&result.solid, expected, TOL);
                    assert_eq!(result.generated_faces.len(), 3);
                    for p in [
                        Point3::new(10., 10. - a, 10.),
                        Point3::new(10., 10. + e, 10.),
                        Point3::new(10. + c, 10., 10. - b),
                    ] {
                        assert!(result
                            .solid
                            .vertex_iter()
                            .any(|v| v.point().near(&transform.transform_point(local(p)))));
                    }
                    selected.reverse();
                    let reversed =
                        try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
                    assert_eq!(
                        serde_json::to_string(&result.solid.compress()).unwrap(),
                        serde_json::to_string(&reversed.solid.compress()).unwrap()
                    );
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}

#[test]
fn transitions_at_both_ends_share_the_vertical_bevel_and_reject_collisions() {
    let input = bracket();
    let ids = [
        Point3::new(15., 10., 10.),
        Point3::new(15., 10., 0.),
        Point3::new(10., 10., 5.),
    ]
    .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id());
    let before = serde_json::to_string(&input.compress()).unwrap();
    let selected = ids.map(|id| (id, [0.2; 2]));
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
    assert!(result.solid.is_geometric_consistent());
    common::assert_solid(&result.solid, 2999.8, &[0], TOL);
    common::blend::assert_step(&result.solid, 2999.8, TOL);
    assert_eq!(result.generated_faces.len(), 5);
    for d in [10., 20.] {
        let selected = ids.map(|id| (id, [d; 2]));
        let error = try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap_err();
        assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
