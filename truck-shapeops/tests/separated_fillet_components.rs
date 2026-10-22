mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
fn body() -> Solid {
    let v = builder::vertices([
        Point3::new(0., -2., 0.),
        Point3::new(3., -2., 0.),
        Point3::new(3., -1., 0.),
        Point3::new(3., 1., 0.),
        Point3::new(3., 2., 0.),
        Point3::new(0., 2., 0.),
    ]);
    let wire = vec![
        builder::line(&v[0], &v[1]),
        builder::line(&v[1], &v[2]),
        builder::circle_arc(&v[2], &v[3], Point3::new(4., 0., 0.)),
        builder::line(&v[3], &v[4]),
        builder::line(&v[4], &v[5]),
        builder::line(&v[5], &v[0]),
    ]
    .into();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 2.,
    )
}
#[test]
fn separated_planar_corners_and_curved_rims_preserve_material() {
    for moved in [false, true] {
        let scale = if moved { 2.5 } else { 1. };
        let matrix = if moved {
            Matrix4::from_translation(Vector3::new(7., -4., 11.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.61))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        for reversed_faces in [false, true] {
            let mut input = builder::transformed(&body(), matrix);
            if reversed_faces {
                input = Solid::new(vec![input.boundaries()[0].iter().cloned().rev().collect()]);
            }
            let before = serde_json::to_string(&input.compress()).unwrap();
            common::assert_solid(&input, (24. + PI) * scale.powi(3), &[0], 0.001 * scale);
            let selected: Vec<_> = [
                Point3::new(0., -2., 1.),
                Point3::new(0., 0., 0.),
                Point3::new(4., 0., 2.),
            ]
            .map(|p| {
                common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p)).id()
            })
            .to_vec();
            for radius in [0.1_f64, 0.2] {
                for order in [[0, 1, 2], [2, 1, 0], [1, 2, 0]] {
                    let ids = order.map(|i| selected[i]);
                    let result =
                        try_fillet_solid_edges(&input, &ids, radius * scale, 0.001 * scale)
                            .unwrap();
                    // Integrate the overlap of the two quarter-cylinder corner cuts and the toroidal rim cut.
                    let corner = 6. * radius * radius * (1. - PI / 4.)
                        - radius.powi(3) * (5. / 3. - PI / 2.);
                    let rim = PI / 2.
                        * (radius * radius * (2. - PI / 2.) + radius.powi(3) * (PI / 2. - 5. / 3.));
                    let expected = (24. + PI - corner - rim) * scale.powi(3);
                    common::assert_solid(&result.solid, expected, &[0], 0.001 * scale);
                    common::blend::assert_step(&result.solid, expected, 0.001 * scale);
                    assert!(result.solid.is_geometric_consistent());
                    assert_eq!(result.generated_faces.len(), 3);
                    for side in [-1., 1.] {
                        for (y, z) in [(side, 2. - radius), (side * (1. - radius), 2.)] {
                            let point = matrix.transform_point(Point3::new(3., y, z));
                            assert!(result
                                .generated_faces
                                .iter()
                                .flat_map(|f| f.vertex_iter())
                                .any(|v| v.point().near(&point)));
                        }
                    }
                    for original in input.face_iter() {
                        let id = result
                            .modified_faces
                            .iter()
                            .find(|(id, _)| *id == original.id())
                            .map(|(_, f)| f.id())
                            .unwrap_or(original.id());
                        assert!(result.solid.face_iter().any(|f| f.id() == id));
                    }
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}
#[test]
fn separated_fillet_components_reject_oversized_cuts_without_mutation() {
    let input = body();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let selected = [
        Point3::new(0., -2., 1.),
        Point3::new(0., 0., 0.),
        Point3::new(4., 0., 2.),
    ]
    .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id());
    assert!(try_fillet_solid_edges(&input, &selected, 2., 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
