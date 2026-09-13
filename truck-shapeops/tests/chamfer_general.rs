mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances};

const TOL: f64 = 0.001;

fn bracket() -> Solid {
    let vertices = builder::vertices(
        [
            (0.0, 0.0),
            (20.0, 0.0),
            (20.0, 10.0),
            (10.0, 10.0),
            (10.0, 20.0),
            (0.0, 20.0),
        ]
        .map(|(x, y)| Point3::new(x, y, 0.0)),
    );
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.0,
    )
}

fn selected(solid: &Solid) -> [EdgeID; 2] {
    [Point3::new(15.0, 10.0, 10.0), Point3::new(10.0, 15.0, 10.0)]
        .map(|p| common::blend::edge_through(&solid.boundaries()[0], p).id())
}

#[test]
fn isolated_inset_edges_have_valid_planar_terminations() {
    let input = bracket();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for edge in selected(&input) {
        let result = try_chamfer_solid_edges(&input, &[edge], 0.2, TOL).unwrap();
        common::assert_solid(&result.solid, 3000.0 - 10.0 * 0.2 * 0.2 / 2.0, &[0], TOL);
        common::blend::assert_step(&result.solid, 2999.8, TOL);
        let single =
            truck_shapeops::fillet::try_chamfer_solid_edge(&input, edge, 0.2, 0.2, TOL).unwrap();
        common::assert_solid(&single.solid, 2999.8, &[0], TOL);
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn holes_elsewhere_do_not_disable_a_planar_chamfer() {
    for open_top in [false, true] {
        let hole = common::modeling::cylinder(
            Point3::new(4.0, 4.0, 5.0),
            Vector3::unit_z(),
            1.0,
            if open_top { 10.0 } else { 3.0 },
        );
        let input = truck_shapeops::subtract(&bracket(), &hole, TOL).unwrap();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let depth = if open_top { 5.0 } else { 3.0 };
        for count in [1, 2] {
            let result =
                try_chamfer_solid_edges(&input, &selected(&input)[..count], 0.2, TOL).unwrap();
            let removed = count as f64 * 0.2
                + if count == 2 {
                    0.2_f64.powi(3) / 3.0
                } else {
                    0.0
                };
            let volume = 3000.0 - depth * std::f64::consts::PI - removed;
            common::assert_solid(
                &result.solid,
                volume,
                if open_top { &[0] } else { &[0, 0] },
                TOL,
            );
            common::blend::assert_step(&result.solid, volume, TOL);
            for edge in input
                .edge_iter()
                .filter(|e| !matches!(e.curve(), Curve::Line(_)))
            {
                assert!(result.solid.edge_iter().any(|e| e.id() == edge.id()));
            }
            for face in input
                .face_iter()
                .filter(|f| !matches!(f.surface(), Surface::Plane(_)))
            {
                assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
            }
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn chamfers_reject_hole_collisions_without_changing_the_body() {
    for y in [9.7, 9.65] {
        let hole =
            common::modeling::cylinder(Point3::new(15.0, y, 9.0), Vector3::unit_z(), 0.15, 2.0);
        let input = truck_shapeops::subtract(&bracket(), &hole, TOL).unwrap();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let error = try_chamfer_solid_edges(&input, &selected(&input)[..1], 0.2, TOL)
            .expect_err("reject intersecting or touching hole");
        assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
        let error = truck_shapeops::fillet::try_chamfer_solid_edge(
            &input,
            selected(&input)[0],
            0.2,
            0.2,
            TOL,
        )
        .expect_err("single-edge entry point also rejects the collision");
        assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

fn distances_on_top(solid: &Solid, ids: &[EdgeID], a: f64, b: f64) -> Vec<(EdgeID, [f64; 2])> {
    ids.iter()
        .map(|&id| {
            let first = solid
                .face_iter()
                .find(|f| f.edge_iter().any(|e| e.id() == id))
                .unwrap();
            let normal = first.oriented_surface().normal(0.0, 0.0);
            (id, if normal.z > 0.9 { [a, b] } else { [b, a] })
        })
        .collect()
}

#[test]
fn asymmetric_inset_chamfers_keep_their_contact_distances_and_order() {
    for (a, b) in [(0.2, 0.3), (0.3, 0.2)] {
        let input = bracket();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for count in [1, 2] {
            let mut chosen = distances_on_top(&input, &selected(&input)[..count], a, b);
            let result = try_chamfer_solid_edges_with_distances(&input, &chosen, TOL).unwrap();
            let expected = 3000.0
                - count as f64 * 10.0 * a * b / 2.0
                - if count == 2 { a * a * b / 3.0 } else { 0.0 };
            common::assert_solid(&result.solid, expected, &[0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            chosen.reverse();
            let reverse = try_chamfer_solid_edges_with_distances(&input, &chosen, TOL).unwrap();
            assert_eq!(
                serde_json::to_string(&result.solid.compress()).unwrap(),
                serde_json::to_string(&reverse.solid.compress()).unwrap()
            );
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn three_edge_corners_do_not_require_a_globally_convex_body() {
    let input = bracket();
    let ids: Vec<_> = [
        Point3::new(10.0, 0.0, 10.0),
        Point3::new(0.0, 10.0, 10.0),
        Point3::new(0.0, 0.0, 5.0),
    ]
    .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
    .into();
    let result = try_chamfer_solid_edges(&input, &ids, 0.2, TOL).unwrap();
    let expected = 3000.0 - 50.0 * 0.2_f64.powi(2) / 2.0 + 3.0 * 0.2_f64.powi(3) / 4.0;
    common::assert_solid(&result.solid, expected, &[0], TOL);
    common::blend::assert_step(&result.solid, expected, TOL);
    assert_eq!(result.generated_faces.len(), 3);
}

#[test]
fn asymmetric_three_edge_corner_matches_an_affine_scaled_reference() {
    let base = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    let transform = Matrix4::from_nonuniform_scale(1.5, 1.0, 1.0);
    let input = builder::transformed(&base, transform);
    let points = [
        Point3::new(5.0, 0.0, 10.0),
        Point3::new(0.0, 5.0, 10.0),
        Point3::new(0.0, 0.0, 5.0),
    ];
    let ids: Vec<_> = points
        .map(|p| common::blend::edge_through(&base.boundaries()[0], p).id())
        .into();
    let reference = try_chamfer_solid_edges(&base, &ids, 0.2, TOL).unwrap();
    let reference = builder::transformed(&reference.solid, transform);
    let chosen: Vec<_> = points
        .map(|p| {
            let edge =
                common::blend::edge_through(&input.boundaries()[0], transform.transform_point(p));
            let axis = (edge.back().point() - edge.front().point()).normalize();
            let sizes: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .map(|f| {
                    let into = f
                        .oriented_surface()
                        .normal(0.0, 0.0)
                        .cross(axis)
                        .normalize();
                    0.2 * transform.transform_vector(into).magnitude()
                })
                .collect();
            (edge.id(), sizes.try_into().unwrap())
        })
        .into();
    let result = try_chamfer_solid_edges_with_distances(&input, &chosen, TOL).unwrap();
    common::assert_solid(&result.solid, 1499.109, &[0], TOL);
    common::blend::assert_step(&result.solid, 1499.109, TOL);
    for vertex in reference.vertex_iter() {
        assert!(result
            .solid
            .vertex_iter()
            .any(|v| v.point().near(&vertex.point())));
    }
    assert_eq!(result.generated_faces.len(), 3);
}

#[test]
fn oversized_unequal_distances_preserve_the_input() {
    let input = bracket();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for count in [1, 2] {
        for (a, b) in [(20.0, 0.2), (0.2, 20.0), (f64::MAX, 0.2)] {
            let chosen = distances_on_top(&input, &selected(&input)[..count], a, b);
            let error = try_chamfer_solid_edges_with_distances(&input, &chosen, TOL)
                .expect_err("oversized chamfer");
            assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn an_untouched_four_edge_vertex_does_not_block_a_chamfer() {
    let vertices = builder::vertices([
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [5.0, 5.0, 10.0],
    ]);
    let mut edges = std::collections::HashMap::new();
    let shell = [
        vec![3, 2, 1, 0],
        vec![0, 1, 4],
        vec![1, 2, 4],
        vec![2, 3, 4],
        vec![3, 0, 4],
    ]
    .into_iter()
    .map(|indices| {
        let wire: Wire = (0..indices.len())
            .map(|i| {
                let (a, b) = (indices[i], indices[(i + 1) % indices.len()]);
                let edge = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| builder::line(&vertices[a.min(b)], &vertices[a.max(b)]));
                if a < b {
                    edge.clone()
                } else {
                    edge.inverse()
                }
            })
            .collect();
        builder::try_attach_plane(&[wire]).unwrap()
    })
    .collect();
    let input = Solid::new(vec![shell]);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges(&input, &[edges[&(0, 1)].id()], 0.2, TOL).unwrap();
    let expected =
        1000.0 / 3.0 - 10.0 * 0.2_f64.powi(2) / 5.0_f64.sqrt() + 2.0 * 0.2_f64.powi(3) / 15.0;
    common::assert_solid(&result.solid, expected, &[0], TOL);
    common::blend::assert_step(&result.solid, expected, TOL);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert!(result
        .solid
        .vertex_iter()
        .any(|v| v.id() == vertices[4].id()));
}

#[test]
fn concave_chamfers_add_material_and_preserve_distance_sides() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13.0, -7.0, 23.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            transform.transform_point(Point3::new(10.0, 10.0, 5.0)),
        );
        for (a, b) in [(0.2, 0.2), (0.2, 0.3), (0.3, 0.2)] {
            let result =
                truck_shapeops::fillet::try_chamfer_solid_edge(&input, edge.id(), a, b, TOL)
                    .unwrap();
            let expected = 3000.0 + 10.0 * a * b / 2.0;
            common::assert_solid(&result.solid, expected, &[0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            assert_eq!(result.generated_faces.len(), 1);
            let generated = &result.generated_faces[0];
            let adjacent: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .collect();
            for (face, distance) in adjacent.iter().zip([a, b]) {
                let replacement = &result
                    .modified_faces
                    .iter()
                    .find(|(id, _)| *id == face.id())
                    .unwrap()
                    .1;
                let contact = generated
                    .edge_iter()
                    .find(|e| replacement.edge_iter().any(|other| other.id() == e.id()))
                    .unwrap();
                let axis = (edge.back().point() - edge.front().point()).normalize();
                for vertex in [contact.front(), contact.back()] {
                    assert!(
                        ((vertex.point() - edge.front().point())
                            .cross(axis)
                            .magnitude()
                            - distance)
                            .abs()
                            < TOLERANCE
                    );
                }
            }
            let cutter = builder::transformed(
                &common::modeling::cuboid(
                    Point3::new(-1.0, -1.0, -1.0),
                    Point3::new(21.0, 21.0, 4.3),
                ),
                transform,
            );
            let control = truck_shapeops::try_and(&input, &cutter, TOL);
            let cut = truck_shapeops::try_and(&result.solid, &cutter, TOL);
            match control {
                Ok(control) => {
                    common::assert_solid(&control, 1290.0, &[0], TOL);
                    common::assert_solid(&cut.unwrap(), expected * 0.43, &[0], TOL);
                }
                Err(control) => {
                    // The unchanged rotated bracket already fails in Boolean face division.
                    assert_eq!(
                        control.code,
                        truck_base::diagnostics::Code::InvalidOutputTopology
                    );
                    assert_eq!(control.stage, "divide_faces");
                    let error = cut.unwrap_err();
                    assert_eq!(error.code, control.code);
                    assert_eq!(error.stage, control.stage);
                }
            }
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn concave_cavity_edges_and_mixed_disjoint_selections_preserve_volume() {
    let outer =
        common::modeling::cuboid(Point3::new(-2.0, -2.0, -2.0), Point3::new(12.0, 12.0, 12.0));
    let void = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    let input = truck_shapeops::subtract(&outer, &void, TOL).unwrap();
    let cavity = input
        .boundaries()
        .iter()
        .find(|shell| shell.vertex_iter().all(|v| v.point().x >= 0.0))
        .unwrap();
    let ids: Vec<_> = [
        Point3::new(5.0, 0.0, 10.0),
        Point3::new(0.0, 5.0, 10.0),
        Point3::new(0.0, 0.0, 5.0),
    ]
    .map(|p| common::blend::edge_through(cavity, p).id())
    .into();
    for count in [1, 2, 3] {
        let overlap = match count {
            1 => 0.0,
            2 => 0.2_f64.powi(3) / 3.0,
            _ => 3.0 * 0.2_f64.powi(3) / 4.0,
        };
        let result = try_chamfer_solid_edges(&input, &ids[..count], 0.2, TOL).unwrap();
        let expected = 1744.0 + count as f64 * 0.2 - overlap;
        common::assert_solid(&result.solid, expected, &[0, 0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        let mut chosen = ids[..count].to_vec();
        chosen.reverse();
        let reverse = try_chamfer_solid_edges(&input, &chosen, 0.2, TOL).unwrap();
        assert_eq!(
            serde_json::to_string(&result.solid.compress()).unwrap(),
            serde_json::to_string(&reverse.solid.compress()).unwrap()
        );
    }
    let input = bracket();
    let mut chosen = vec![
        common::blend::edge_through(&input.boundaries()[0], Point3::new(10.0, 10.0, 5.0)).id(),
        common::blend::edge_through(&input.boundaries()[0], Point3::new(0.0, 0.0, 5.0)).id(),
    ];
    let result = try_chamfer_solid_edges(&input, &chosen, 0.2, TOL).unwrap();
    common::assert_solid(&result.solid, 3000.0, &[0], TOL);
    chosen.reverse();
    let reverse = try_chamfer_solid_edges(&input, &chosen, 0.2, TOL).unwrap();
    assert_eq!(
        serde_json::to_string(&result.solid.compress()).unwrap(),
        serde_json::to_string(&reverse.solid.compress()).unwrap()
    );
}

#[test]
fn concave_chamfers_reject_collapsed_neighbours() {
    let input = bracket();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let id = common::blend::edge_through(&input.boundaries()[0], Point3::new(10.0, 10.0, 5.0)).id();
    for distances in [[10.0, 10.0], [10.0, 0.2], [0.2, 20.0], [f64::MAX, 0.2]] {
        let error =
            try_chamfer_solid_edges_with_distances(&input, &[(id, distances)], TOL).unwrap_err();
        assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn concave_chamfers_measure_distances_on_oblique_support_faces() {
    for shear in [-0.4, 0.6] {
        let transform = Matrix4::from_cols(
            Vector4::unit_x(),
            Vector4::new(shear, 1.0, 0.0, 0.0),
            Vector4::unit_z(),
            Vector4::unit_w(),
        );
        let input = builder::transformed(&bracket(), transform);
        let id = common::blend::edge_through(
            &input.boundaries()[0],
            transform.transform_point(Point3::new(10.0, 10.0, 5.0)),
        )
        .id();
        let result =
            try_chamfer_solid_edges_with_distances(&input, &[(id, [0.2, 0.3])], TOL).unwrap();
        let expected = 3000.0 + 10.0 * 0.2 * 0.3 / (2.0 * (1.0 + shear * shear).sqrt());
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
    }
}

#[test]
fn a_flat_face_seam_is_not_a_concave_chamfer() {
    let vertices = builder::vertices(
        [
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ]
        .map(|(x, y)| Point3::new(x, y, 0.0)),
    );
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    let input = builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.0,
    );
    let before = serde_json::to_string(&input.compress()).unwrap();
    let id = common::blend::edge_through(&input.boundaries()[0], Point3::new(5.0, 0.0, 5.0)).id();
    assert!(try_chamfer_solid_edges(&input, &[id], 0.2, TOL).is_err());
    assert!(truck_shapeops::fillet::try_chamfer_solid_edge(&input, id, 0.2, 0.2, TOL).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn mixed_three_way_chamfer_has_a_transition_face_above_the_inside_bevel() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13.0, -7.0, 23.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut ids: Vec<_> = [
            Point3::new(15.0, 10.0, 10.0),
            Point3::new(10.0, 15.0, 10.0),
            Point3::new(10.0, 10.0, 5.0),
        ]
        .map(|p| {
            common::blend::edge_through(&input.boundaries()[0], transform.transform_point(p)).id()
        })
        .into();
        let d = 0.2_f64;
        let result = try_chamfer_solid_edges(&input, &ids, d, TOL).unwrap();
        assert_eq!(
            result.generated_faces.len(),
            4,
            "three edge bevels and one corner transition"
        );
        let k = 2.0 - 2.0_f64.sqrt();
        let expected = 3000.0 - 10.0 * d * d - d.powi(3) / 3.0
            + 5.0 * d * d
            + k * d.powi(3) / 2.0
            + k * k * d.powi(3) / 6.0;
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        let transition = result.generated_faces.last().unwrap();
        assert_eq!(transition.edge_iter().count(), 4);
        let points = [
            Point3::new(10.0 + d, 10.0, 10.0 - d),
            Point3::new(10.0, 10.0 + d, 10.0 - d),
            Point3::new(10.0 - d, 10.0 + k * d, 10.0),
            Point3::new(10.0 + k * d, 10.0 - d, 10.0),
        ]
        .map(|p| transform.transform_point(p));
        for point in points {
            assert!(
                transition.vertex_iter().any(|v| v.point().near(&point)),
                "missing transition vertex {point:?}"
            );
        }
        ids.reverse();
        let reversed = try_chamfer_solid_edges(&input, &ids, d, TOL).unwrap();
        assert_eq!(
            serde_json::to_string(&result.solid.compress()).unwrap(),
            serde_json::to_string(&reversed.solid.compress()).unwrap()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn mixed_corner_transitions_support_distinct_setbacks_and_both_ends() {
    let input = bracket();
    let shell = &input.boundaries()[0];
    for ends in [1, 2] {
        for (d, depth, c) in [(0.2_f64, 0.35, 0.4), (3.0, 2.0, 1.0)] {
            let mut ids =
                vec![common::blend::edge_through(shell, Point3::new(10.0, 10.0, 5.0)).id()];
            for z in [10.0, 0.0].into_iter().take(ends) {
                ids.extend(
                    [Point3::new(15.0, 10.0, z), Point3::new(10.0, 15.0, z)]
                        .map(|p| common::blend::edge_through(shell, p).id()),
                );
            }
            let request: Vec<_> = ids
                .iter()
                .enumerate()
                .map(|(i, &id)| {
                    let distances: Vec<_> = shell
                        .iter()
                        .filter(|f| f.edge_iter().any(|e| e.id() == id))
                        .map(|f| {
                            if i == 0 {
                                c
                            } else if f.oriented_surface().normal(0.0, 0.0).z.abs() > 0.99 {
                                d
                            } else {
                                depth
                            }
                        })
                        .collect();
                    (id, [distances[0], distances[1]])
                })
                .collect();
            let result = try_chamfer_solid_edges_with_distances(&input, &request, TOL).unwrap();
            assert_eq!(result.generated_faces.len(), 1 + 3 * ends);
            let k = 2.0 - 2.0_f64.sqrt();
            let expected = 3000.0
                + 5.0 * c * c
                + ends as f64
                    * (-10.0 * d * depth - d * d * depth / 3.0
                        + c * k * d * depth / 2.0
                        + k * k * d * d * depth / 6.0);
            common::assert_solid(&result.solid, expected, &[0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
        }
    }
}
