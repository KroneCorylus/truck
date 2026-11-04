mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{
    try_chamfer_solid_edge, try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances,
};

const TOL: f64 = 0.001;

fn check(vertical: f64, scale: f64, reversed_faces: bool) {
    let block = common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.));
    let first = common::blend::edge_through(&block.boundaries()[0], Point3::new(10., 10., 5.)).id();
    let input = try_chamfer_solid_edges(&block, &[first], 1., TOL)
        .unwrap()
        .solid;
    let transform = if reversed_faces {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&input, transform);
    if reversed_faces {
        input = Solid::new(
            input
                .boundaries()
                .iter()
                .map(|shell| shell.iter().cloned().rev().collect())
                .collect(),
        );
    }
    common::assert_solid(&input, 995. * scale.powi(3), &[0], TOL);
    assert_eq!(input.face_iter().count(), 7);
    let edge = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(4., 10., 0.)),
    )
    .id();
    let consumed = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(9.5, 9.5, 0.)),
    )
    .id();
    let distances: Vec<_> = input
        .face_iter()
        .filter(|face| face.edge_iter().any(|e| e.id() == edge))
        .map(|face| {
            let normal = inverse
                .transform_vector(face.oriented_surface().normal(0., 0.))
                .normalize();
            if normal.z < -0.9 {
                2. * scale
            } else {
                assert!(normal.y > 0.9);
                vertical * scale
            }
        })
        .collect();
    let distances: [f64; 2] = distances.try_into().unwrap();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = if vertical == 2. {
        try_chamfer_solid_edges(&input, &[edge], 2. * scale, TOL)
    } else {
        try_chamfer_solid_edges_with_distances(&input, &[(edge, distances)], &[], TOL)
    }
    .unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert_eq!(result.solid.face_iter().count(), 8);
    assert_eq!(result.generated_faces.len(), 1);
    assert!(!result.solid.edge_iter().any(|e| e.id() == consumed));
    // Subtract the second wedge, excluding its overlap with the first corner cut.
    let removed = 10. * 2. * vertical / 2. - vertical * (1. / 2. - 1. / 12.);
    let expected = (995. - removed) * scale.powi(3);
    common::assert_solid(&result.solid, expected, &[0], TOL);
    common::blend::assert_step(&result.solid, expected, TOL);
    let face = &result.generated_faces[0];
    for edge in face.edge_iter() {
        let p = inverse.transform_point(edge.front().point());
        assert!((p.y - 2. / vertical * p.z - 8.).abs() * scale < 1e-6);
    }
    let normal = inverse
        .transform_vector(face.oriented_surface().normal(0., 0.))
        .normalize();
    assert!(normal.dot(Vector3::new(0., 1., -2. / vertical).normalize()) > 1. - 1e-8);
    let single = try_chamfer_solid_edge(&input, edge, distances[0], distances[1], TOL).unwrap();
    common::assert_solid(&single.solid, expected, &[0], TOL);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn equal_chamfer_consumes_a_short_neighbor_without_removing_its_support() {
    for reversed in [false, true] {
        check(2., 1., reversed);
    }
}

#[test]
fn unequal_chamfer_consumes_a_short_neighbor_and_preserves_setbacks() {
    for scale in [1., 0.2] {
        for reversed in [false, true] {
            check(1., scale, reversed);
        }
    }
}
