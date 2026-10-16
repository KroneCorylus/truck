mod common;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances};
const TOL: f64 = 0.001;
fn fixture() -> Solid {
    let c = (25_f64 - 2.5_f64.powi(2)).sqrt();
    let v = builder::vertices([
        Point3::new(-2.5, -10., 0.),
        Point3::new(2.5, -10., 0.),
        Point3::new(2.5, -c, 0.),
        Point3::new(-2.5, -c, 0.),
    ]);
    let wire = wire![
        builder::line(&v[0], &v[1]),
        builder::line(&v[1], &v[2]),
        builder::circle_arc(&v[2], &v[3], Point3::new(0., 5., 0.)),
        builder::line(&v[3], &v[0]),
    ];
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::new(0., 0., 5.),
    )
}
fn volume(d: f64, e: f64) -> f64 {
    let a: f64 = 2.5;
    let b: f64 = 10.;
    let radius: f64 = 5.;
    let x0 = -a;
    let x1 = -a + d;
    let circular = |x: f64| {
        (x * (radius * radius - x * x).sqrt() + radius * radius * (x / radius).asin()) / 2.
    };
    let weighted = |x: f64| -(radius * radius - x * x).powf(1.5) / 3.;
    let removed = e / d
        * ((d - a) * (b * d - circular(x1) + circular(x0))
            - (b * (x1 * x1 - x0 * x0) / 2. - weighted(x1) + weighted(x0)));
    5. * (5. * std::f64::consts::PI * radius * radius / 6. + 2. * a * b
        - a * (radius * radius - a * a).sqrt())
        - removed
}

fn check(equal: bool, moved: bool, reversed: bool) {
    let scale: f64 = if moved { 0.2 } else { 1. };
    let transform = if moved {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&fixture(), transform);
    if reversed {
        let mut compressed = input.compress();
        for shell in &mut compressed.boundaries {
            for (index, edge) in shell.edges.iter_mut().enumerate() {
                if matches!(edge.curve, Curve::Conic(_)) {
                    edge.curve.invert();
                    edge.vertices = (edge.vertices.1, edge.vertices.0);
                    for face in &mut shell.faces {
                        for member in face.boundaries.iter_mut().flatten() {
                            if member.index == index {
                                member.orientation = !member.orientation;
                            }
                        }
                    }
                }
            }
        }
        input = Solid::extract(compressed).unwrap();
    }
    if moved {
        input = Solid::new(
            input
                .boundaries()
                .iter()
                .map(|s| s.iter().cloned().rev().collect())
                .collect(),
        );
    }
    let initial = 5. * (5. * std::f64::consts::PI * 25. / 6. + 50. - 2.5 * (25_f64 - 6.25).sqrt());
    common::assert_solid(&input, initial * scale.powi(3), &[0], TOL);
    let edge = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(-2.5, -7., 5.)),
    )
    .id();
    let (top, side) = if equal { (0.25, 0.25) } else { (0.4, 0.2) };
    let distances: [f64; 2] = input
        .face_iter()
        .filter(|f| f.edge_iter().any(|e| e.id() == edge))
        .map(|f| {
            let n = inverse
                .transform_vector(f.oriented_surface().normal(0., 0.))
                .normalize();
            if n.z > 0.9 {
                top * scale
            } else {
                side * scale
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = if equal {
        try_chamfer_solid_edges(&input, &[edge], top * scale, TOL)
    } else {
        try_chamfer_solid_edges_with_distances(&input, &[(edge, distances)], TOL)
    }
    .unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert_eq!(result.generated_faces.len(), 1);
    assert_eq!(result.solid.face_iter().count(), 7);
    let expected = volume(top, side) * scale.powi(3);
    common::assert_solid(&result.solid, expected, &[0], TOL);
    common::blend::assert_step(&result.solid, expected, TOL);
    for target in [
        Point3::new(-2.5 + top, -(25. - (-2.5 + top).powi(2)).sqrt(), 5.),
        Point3::new(-2.5, -(25_f64 - 6.25).sqrt(), 5. - side),
    ] {
        assert!(result
            .solid
            .vertex_iter()
            .any(|v| inverse.transform_point(v.point()).distance(target) < 1e-6));
    }
    let normal = Vector3::new(-side, 0., top).normalize();
    for v in result.generated_faces[0].vertex_iter() {
        let p = inverse.transform_point(v.point());
        assert!((p - Point3::new(-2.5 + top, 0., 5.)).dot(normal).abs() < 1e-6);
    }
    assert!(
        inverse
            .transform_vector(result.generated_faces[0].oriented_surface().normal(0., 0.))
            .normalize()
            .dot(normal)
            > 1. - 1e-8
    );
}
#[test]
fn equal_chamfer_extends_the_terminal_circle() {
    for moved in [false, true] {
        for reversed in [false, true] {
            check(true, moved, reversed);
        }
    }
}
#[test]
fn unequal_chamfer_extends_the_terminal_circle() {
    for moved in [false, true] {
        for reversed in [false, true] {
            check(false, moved, reversed);
        }
    }
}
#[test]
fn oversized_stem_chamfers_reject_without_input_mutation() {
    let input = fixture();
    let edge = common::blend::edge_through(&input.boundaries()[0], Point3::new(-2.5, -7., 5.)).id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for distance in [5., 10., 20.] {
        assert!(try_chamfer_solid_edges(&input, &[edge], distance, TOL).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn extended_cylinder_mesh_preserves_the_verified_step_material() {
    let compressed = serde_json::from_str(include_str!(
        "fixtures/conic-contact-extension/trimmed-solid.json"
    ))
    .unwrap();
    let solid = Solid::extract(compressed).unwrap();
    assert!(solid
        .boundaries()
        .iter()
        .all(|s| s.is_geometric_consistent()));
    common::assert_solid(&solid, volume(0.25, 0.25), &[0], TOL);
    common::blend::assert_step(&solid, volume(0.25, 0.25), TOL);
}
