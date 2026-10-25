//! Unequal chamfers at a unit-cube corner follow OCCT's three-corner rule. Each case records
//! the OCCT 7.9.3 `BRepFilletAPI_MakeChamfer` volume of the same request and the corner face
//! derived from the rule: a pivot bevel is crossed by the other two no closer to the vertex
//! than they cross each other; those two end at their sections through the pivot crossings.

mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

const TOL: f64 = 0.001;

struct Case {
    distances: [f64; 3],
    angles: [f64; 3],
    occt_volume: f64,
    corner: Vec<[f64; 3]>,
}

fn check(case: Case) {
    let [d, angle] = [case.distances, case.angles.map(f64::to_radians)];
    // Setbacks on the face named first and on the other face of each edge.
    let pairs = [0, 1, 2].map(|i| [d[i] * angle[i].tan(), d[i]]);
    let corner: Vec<_> = case.corner.iter().map(|&p| Point3::from(p)).collect();
    let planar = {
        let (n, e) = common::polytope::plane(&corner, Point3::new(0.5, 0.5, 0.5));
        corner.iter().all(|p| (n.dot(p.to_vec()) - e).abs() < 1e-12)
    };
    let mut planes = vec![
        (-Vector3::unit_x(), 0.0),
        (Vector3::unit_x(), 1.0),
        (-Vector3::unit_y(), 0.0),
        (Vector3::unit_y(), 1.0),
        (-Vector3::unit_z(), 0.0),
        (Vector3::unit_z(), 1.0),
    ];
    // Bevels through their contact lines: along y on x = 0 and z = 1, along z on x = 0 and
    // y = 1, and along x on y = 1 and z = 1.
    let contacts = [
        [
            Point3::new(0.0, 0.5, 1.0 - pairs[0][1]),
            Point3::new(pairs[0][0], 0.5, 1.0),
        ],
        [
            Point3::new(0.0, 1.0 - pairs[1][1], 0.5),
            Point3::new(pairs[1][0], 1.0, 0.5),
        ],
        [
            Point3::new(0.5, 1.0, 1.0 - pairs[2][1]),
            Point3::new(0.5, 1.0 - pairs[2][0], 1.0),
        ],
    ];
    let axes = [Vector3::unit_y(), Vector3::unit_z(), Vector3::unit_x()];
    for ([p, q], axis) in contacts.into_iter().zip(axes) {
        let mut n = (q - p).cross(axis).normalize();
        if n.dot(p - Point3::new(0.5, 0.5, 0.5)) < 0.0 {
            n = -n;
        }
        planes.push((n, n.dot(p.to_vec())));
    }
    let expected_vertices = if planar {
        planes.push(common::polytope::plane(&corner, Point3::new(0.5, 0.5, 0.5)));
        let volume = common::polytope::volume(&planes);
        assert!(
            (volume - case.occt_volume).abs() < 1e-11,
            "{volume} vs OCCT"
        );
        Some(common::polytope::vertices(&planes))
    } else {
        None
    };
    let volume = case.occt_volume;
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
        common::assert_solid(&result.solid, volume * scale.powi(3), &[0], TOL);
        common::blend::assert_step(&result.solid, volume * scale.powi(3), TOL);
        assert!(result
            .solid
            .boundaries()
            .iter()
            .all(|s| s.is_geometric_consistent()));
        assert_eq!(result.generated_faces.len(), 4);
        let face = result
            .generated_faces
            .iter()
            .find(|f| {
                corner.iter().all(|p| {
                    let p = matrix.transform_point(*p);
                    f.vertex_iter().any(|v| v.point().distance(p) < 1e-9)
                })
            })
            .expect("corner face");
        assert_eq!(face.vertex_iter().count(), corner.len());
        assert_eq!(matches!(face.surface(), Surface::Plane(_)), planar);
        if let Some(expected) = &expected_vertices {
            let actual: Vec<_> = result.solid.vertex_iter().map(|v| v.point()).collect();
            for p in expected {
                let p = matrix.transform_point(*p);
                assert!(
                    actual.iter().any(|q| p.distance(*q) < 1e-7),
                    "missing {p:?}"
                );
            }
            for p in &actual {
                assert!(
                    expected
                        .iter()
                        .any(|q| matrix.transform_point(*q).distance(*p) < 1e-7),
                    "unexpected {p:?}"
                );
            }
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn balanced_convex_corner_is_a_triangle() {
    check(Case {
        distances: [0.08; 3],
        angles: [45.0; 3],
        occt_volume: 0.990741333333,
        corner: vec![[0.0, 0.92, 0.92], [0.08, 1.0, 0.92], [0.08, 0.92, 1.0]],
    });
}

#[test]
fn dominant_xz_bevel_corner_is_a_planar_quad() {
    check(Case {
        distances: [0.3, 0.08, 0.08],
        angles: [45.0; 3],
        occt_volume: 0.949645333333,
        corner: vec![
            [0.0, 0.92, 0.7],
            [0.3, 0.92, 1.0],
            [0.3, 1.0, 0.92],
            [0.08, 1.0, 0.7],
        ],
    });
}

#[test]
fn dominant_xy_bevel_corner_is_a_planar_quad() {
    check(Case {
        distances: [0.08, 0.3, 0.08],
        angles: [45.0; 3],
        occt_volume: 0.949645333333,
        corner: vec![
            [0.3, 1.0, 0.92],
            [0.0, 0.7, 0.92],
            [0.08, 0.7, 1.0],
            [0.3, 0.92, 1.0],
        ],
    });
}

#[test]
fn dominant_yz_bevel_corner_is_a_planar_quad() {
    check(Case {
        distances: [0.08, 0.08, 0.3],
        angles: [45.0; 3],
        occt_volume: 0.949645333333,
        corner: vec![
            [0.08, 0.7, 1.0],
            [0.08, 1.0, 0.7],
            [0.0, 0.92, 0.7],
            [0.0, 0.7, 0.92],
        ],
    });
}

#[test]
fn unequal_angle_corner_is_a_bilinear_quad() {
    check(Case {
        distances: [0.08, 0.2, 0.1],
        angles: [40.0, 30.0, 30.0],
        occt_volume: 0.983363172648,
        corner: vec![
            [0.115470053837925, 1.0, 0.9],
            [0.0, 0.8, 0.92],
            [0.067127970494182, 0.8, 1.0],
            [0.115470053837925, 0.942264973081037, 1.0],
        ],
    });
}
