mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances};

const TOL: f64 = 0.001;

fn fixture() -> Solid {
    let vertices = builder::vertices([
        Point3::new(0., 0.25, 0.),
        Point3::new(1.2, 0.25, 0.),
        Point3::new(1.2, 0.95, 0.),
        Point3::new(1., 1., 0.),
        Point3::new(0., 1., 0.),
    ]);
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::rsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Point3::origin(),
        Vector3::unit_x(),
        Rad(std::f64::consts::FRAC_PI_2),
        1,
    )
}

fn check(equal: bool, moved: bool) {
    let scale: f64 = if moved { 0.2 } else { 1. };
    let transform = if moved {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&fixture(), transform);
    if moved {
        input = Solid::new(
            input
                .boundaries()
                .iter()
                .map(|s| s.iter().cloned().rev().collect())
                .collect(),
        );
    }
    common::assert_solid(&input, 0.8758498518820544 * scale.powi(3), &[0], TOL);
    let edge = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(0.5, 1., 0.)),
    )
    .id();
    let consumed = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(1.1, 0.975, 0.)),
    )
    .id();
    // Exact section integrals and their high-precision evaluations are archived with the proof.
    let (planar, tangent, expected): (f64, f64, f64) = if equal {
        (0.2, 0.2, 0.8524577031471756)
    } else {
        (0.1, 0.20305866063400407, 0.8639619850312715)
    };
    let distances: Vec<_> = input
        .face_iter()
        .filter(|f| f.edge_iter().any(|e| e.id() == edge))
        .map(|f| match f.oriented_surface().elementary().unwrap().0 {
            Elementary::Plane(_) => planar * scale,
            Elementary::Cylinder { .. } => tangent * scale,
            _ => panic!("unexpected support"),
        })
        .collect();
    let distances: [f64; 2] = distances.try_into().unwrap();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = if equal {
        try_chamfer_solid_edges(&input, &[edge], 0.2 * scale, TOL)
    } else {
        try_chamfer_solid_edges_with_distances(&input, &[(edge, distances)], &[], TOL)
    }
    .unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert_eq!(result.generated_faces.len(), 1);
    assert_eq!(
        result.solid.face_iter().count(),
        input.face_iter().count() + 1
    );
    assert!(!result.solid.edge_iter().any(|e| e.id() == consumed));
    common::assert_solid(&result.solid, expected * scale.powi(3), &[0], TOL);
    common::blend::assert_step(&result.solid, expected * scale.powi(3), TOL);
    let psi = tangent.atan();
    let a = 1. - planar;
    let slope = psi.sin() / (psi.cos() - a);
    for v in result.generated_faces[0].vertex_iter() {
        let p = inverse.transform_point(v.point());
        assert!((p.z - slope * (p.y - a)).abs() < 1e-6);
    }
    let alpha = slope.atan();
    let t = -a * alpha.cos() + (0.95_f64.powi(2) - a * a * alpha.sin().powi(2)).sqrt();
    let crossing = Point3::new(1.2, a + t * alpha.cos(), t * alpha.sin());
    assert!(result
        .solid
        .vertex_iter()
        .any(|v| inverse.transform_point(v.point()).distance(crossing) < 1e-6));
}

#[test]
fn equal_chamfer_crosses_a_curved_terminal_rim() {
    for moved in [false, true] {
        check(true, moved);
    }
}

#[test]
fn unequal_chamfer_crosses_a_curved_terminal_rim() {
    for moved in [false, true] {
        check(false, moved);
    }
}

#[test]
fn curved_terminal_chamfers_reject_consumed_supports_without_mutation() {
    let input = fixture();
    let edge = common::blend::edge_through(&input.boundaries()[0], Point3::new(0.5, 1., 0.)).id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for distance in [1., 2., 10.] {
        assert!(try_chamfer_solid_edges(&input, &[edge], distance, TOL).is_err());
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &[(edge, [distance; 2])], &[], TOL)
                .is_err()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
