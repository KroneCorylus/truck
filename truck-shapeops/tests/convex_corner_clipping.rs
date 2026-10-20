mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

const TOL: f64 = 0.001;

fn check_case(index: usize) {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("convex-corner-clipping.json")).unwrap();
    let case = &cases[index];
    let d: Vec<_> = case["distances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let angle: Vec<_> = case["angles"]
        .as_array()
        .map(|v| v.iter().map(|v| v.as_f64().unwrap().to_radians()).collect())
        .unwrap_or_else(|| vec![std::f64::consts::FRAC_PI_4; 3]);
    let pairs = [
        [d[0] * angle[0].tan(), d[0]],
        [d[1] * angle[1].tan(), d[1]],
        [d[2] * angle[2].tan(), d[2]],
    ];
    for transformed in [false, true] {
        let scale = if transformed { 0.2 } else { 1.0 };
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(2.0, -3.0, 4.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let mut input = builder::transformed(
            &common::modeling::cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0)),
            matrix,
        );
        if transformed {
            let mut faces = input.boundaries()[0].iter().cloned().collect::<Vec<_>>();
            faces.reverse();
            input = Solid::new(vec![faces.into()]);
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mids = [
            Point3::new(0.0, 0.5, 1.0),
            Point3::new(0.0, 1.0, 0.5),
            Point3::new(0.5, 1.0, 1.0),
        ];
        let face_normals = [Vector3::unit_z(), Vector3::unit_y(), Vector3::unit_z()];
        let mut selected: Vec<_> = mids
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let id =
                    common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p))
                        .id();
                let distances: Vec<_> = input
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == id))
                    .map(|f| {
                        let normal = f.oriented_surface().normal(0.0, 0.0).normalize();
                        let first =
                            normal.dot(matrix.transform_vector(face_normals[i]).normalize()) > 0.9;
                        pairs[i][usize::from(!first)] * scale
                    })
                    .collect();
                (id, distances.try_into().unwrap())
            })
            .collect();
        if transformed {
            selected.reverse();
        }
        let result = try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
        let volume = case["expected_volume"].as_f64().unwrap() * scale.powi(3);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
        assert!(result
            .solid
            .boundaries()
            .iter()
            .all(|s| s.is_geometric_consistent()));
        assert_eq!(result.generated_faces.len(), 3);
        let expected = case["halfspace_vertices"].as_array().unwrap();
        let actual: Vec<_> = result.solid.vertex_iter().map(|v| v.point()).collect();
        for p in expected {
            let p = matrix.transform_point(Point3::new(
                p[0].as_f64().unwrap(),
                p[1].as_f64().unwrap(),
                p[2].as_f64().unwrap(),
            ));
            assert!(
                actual.iter().any(|q| p.distance(*q) < 1e-7),
                "missing contact {p:?}"
            );
        }
        for p in &actual {
            assert!(
                expected.iter().any(|q| matrix
                    .transform_point(Point3::new(
                        q[0].as_f64().unwrap(),
                        q[1].as_f64().unwrap(),
                        q[2].as_f64().unwrap()
                    ))
                    .distance(*p)
                    < 1e-7),
                "unexpected contact {p:?}"
            );
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn balanced_convex_corner_retains_three_plane_miter() { check_case(0); }
#[test]
fn dominant_xz_bevel_clips_to_original_face() { check_case(1); }
#[test]
fn dominant_xy_bevel_clips_to_original_face() { check_case(2); }
#[test]
fn dominant_yz_bevel_clips_to_original_face() { check_case(3); }
#[test]
fn unequal_angle_corner_clips_to_original_face() { check_case(4); }
