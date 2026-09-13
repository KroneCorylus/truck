mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges;

const TOL: f64 = 0.001;

fn bracket() -> Solid {
    let vertices = builder::vertices(
        [
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 1.0),
            (1.0, 2.0),
            (0.0, 2.0),
        ]
        .map(|(x, y)| Point3::new(x, y, 0.0)),
    );
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z(),
    )
}

#[test]
fn chamfers_meet_at_a_reentrant_planar_corner() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13.0, -7.0, 23.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected: Vec<_> = [Point3::new(1.5, 1.0, 1.0), Point3::new(1.0, 1.5, 1.0)]
            .map(|p| {
                common::blend::edge_through(&input.boundaries()[0], transform.transform_point(p))
                    .id()
            })
            .into();
        let expected = 3.0 - 0.2_f64.powi(2) - 0.2_f64.powi(3) / 3.0;
        let result = try_chamfer_solid_edges(&input, &selected, 0.2, TOL).unwrap();
        common::assert_solid(&result.solid, expected, &[0], TOL);
        assert_eq!(result.generated_faces.len(), 2);
        assert_eq!(result.modified_faces.len(), 8);
        let [a, b] = result.generated_faces.as_slice() else {
            unreachable!()
        };
        let seam = a
            .edge_iter()
            .find(|e| b.edge_iter().any(|f| e.id() == f.id()))
            .unwrap();
        let endpoints = [Point3::new(0.8, 0.8, 1.0), Point3::new(1.0, 1.0, 0.8)]
            .map(|p| transform.transform_point(p));
        assert!(endpoints
            .iter()
            .all(|p| seam.front().point().near(p) || seam.back().point().near(p)));
        common::blend::assert_step(&result.solid, expected, TOL);
        let cutter = builder::transformed(
            &common::modeling::cuboid(Point3::new(-1.0, -1.0, -1.0), Point3::new(3.0, 3.0, 0.9)),
            transform,
        );
        let cut = truck_shapeops::and(&result.solid, &cutter, TOL).unwrap();
        common::assert_solid(
            &cut,
            2.7 - 0.1_f64.powi(2) - 0.1_f64.powi(3) / 3.0,
            &[0],
            TOL,
        );
        selected.reverse();
        let reversed = try_chamfer_solid_edges(&input, &selected, 0.2, TOL).unwrap();
        assert_eq!(
            serde_json::to_string(&result.solid.compress()).unwrap(),
            serde_json::to_string(&reversed.solid.compress()).unwrap()
        );
        for distance in [1.0, 1.1, 3.0] {
            assert!(try_chamfer_solid_edges(&input, &selected, distance, TOL).is_err());
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn nonconvex_body_supports_outer_corners_and_a_complete_rim() {
    let input = bracket();
    let d: f64 = 0.2;
    for (points, expected) in [
        (vec![Point3::new(1.0, 0.0, 1.0)], 3.0 - d * d),
        (
            vec![Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0)],
            3.0 - 2.0 * d * d + d.powi(3) / 3.0,
        ),
        (
            vec![
                Point3::new(1.0, 0.0, 1.0),
                Point3::new(2.0, 0.5, 1.0),
                Point3::new(1.5, 1.0, 1.0),
                Point3::new(1.0, 1.5, 1.0),
                Point3::new(0.5, 2.0, 1.0),
                Point3::new(0.0, 1.0, 1.0),
            ],
            3.0 - 4.0 * d * d + 4.0 * d.powi(3) / 3.0,
        ),
    ] {
        let selected: Vec<_> = points
            .iter()
            .map(|&p| common::blend::edge_through(&input.boundaries()[0], p).id())
            .collect();
        let result = try_chamfer_solid_edges(&input, &selected, d, TOL).unwrap();
        common::assert_solid(&result.solid, expected, &[0], TOL);
        if points.len() == 6 {
            for distance in [0.5, 0.6, 1.5] {
                assert!(try_chamfer_solid_edges(&input, &selected, distance, TOL).is_err());
            }
        }
    }
    let concave = common::blend::edge_through(&input.boundaries()[0], Point3::new(1.0, 1.0, 0.5));
    let result = try_chamfer_solid_edges(&input, &[concave.id()], d, TOL).unwrap();
    common::assert_solid(&result.solid, 3.0 + d * d / 2.0, &[0], TOL);
    common::blend::assert_step(&result.solid, 3.0 + d * d / 2.0, TOL);
}
