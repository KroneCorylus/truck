//! Fully chamfered convex corners follow OCCT's three-corner rule. The volumes quoted below
//! are OCCT 7.9.3 `BRepFilletAPI_MakeChamfer` results for the same requests.

mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances};

const TOL: f64 = 0.001;

/// Box `[0, 10]³` with the three edges at `(10, 10, 10)` chamfered. `s[i]` holds the setbacks
/// on the faces `x = 10`, `y = 10`, `z = 10` of the edges along y, z and x; an edge's entry for
/// the face it does not touch is unused.
struct Case {
    setbacks: [[f64; 3]; 3],
    equal: bool,
    /// Corner face vertices, derived from the rule by hand.
    corner: Vec<[f64; 3]>,
    occt_volume: f64,
}

/// Material between the box, the three bevel planes and the plane of a planar corner face.
fn halfspace_volume(case: &Case) -> f64 {
    let bevel = |p: Point3, q: Point3, axis: Vector3| {
        let n = (q - p).cross(axis).normalize();
        let n = if n.dot(Vector3::new(1.0, 1.0, 1.0)) < 0.0 {
            -n
        } else {
            n
        };
        (n, n.dot(p.to_vec()))
    };
    let [y_edge, z_edge, x_edge] = case.setbacks;
    let mut planes = vec![
        (Vector3::unit_x(), 10.0),
        (Vector3::unit_y(), 10.0),
        (Vector3::unit_z(), 10.0),
        (-Vector3::unit_x(), 0.0),
        (-Vector3::unit_y(), 0.0),
        (-Vector3::unit_z(), 0.0),
        bevel(
            Point3::new(10.0, 0.0, 10.0 - y_edge[0]),
            Point3::new(10.0 - y_edge[2], 0.0, 10.0),
            Vector3::unit_y(),
        ),
        bevel(
            Point3::new(10.0, 10.0 - z_edge[0], 0.0),
            Point3::new(10.0 - z_edge[1], 10.0, 0.0),
            Vector3::unit_z(),
        ),
        bevel(
            Point3::new(0.0, 10.0, 10.0 - x_edge[1]),
            Point3::new(0.0, 10.0 - x_edge[2], 10.0),
            Vector3::unit_x(),
        ),
    ];
    let c: Vec<_> = case.corner.iter().map(|&p| Vector3::from(p)).collect();
    let n = (c[1] - c[0]).cross(c[2] - c[0]).normalize();
    let n = if n.dot(Vector3::new(1.0, 1.0, 1.0)) < 0.0 {
        -n
    } else {
        n
    };
    for p in &c {
        assert!(
            (n.dot(*p - c[0])).abs() < 1e-12,
            "corner face is not planar"
        );
    }
    planes.push((n, n.dot(c[0])));
    common::polytope::volume(&planes)
}

fn check(case: Case, planar: bool) {
    let expected = if planar {
        let volume = halfspace_volume(&case);
        assert!((volume - case.occt_volume).abs() < 1e-8, "{volume} vs OCCT");
        volume
    } else {
        case.occt_volume
    };
    let mids = [
        Point3::new(10.0, 5.0, 10.0),
        Point3::new(10.0, 10.0, 5.0),
        Point3::new(5.0, 10.0, 10.0),
    ];
    for transformed in [false, true] {
        let scale = if transformed { 0.3 } else { 1.0 };
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(2.0, -3.0, 4.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let input = builder::transformed(
            &common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0)),
            matrix,
        );
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected: Vec<_> = mids
            .iter()
            .zip(case.setbacks)
            .map(|(&mid, setbacks)| {
                let id = common::blend::edge_through(
                    &input.boundaries()[0],
                    matrix.transform_point(mid),
                )
                .id();
                let distances: Vec<_> = input
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == id))
                    .map(|f| {
                        let normal = inverse
                            .transform_vector(f.oriented_surface().normal(0.0, 0.0))
                            .normalize();
                        setbacks[(0..3).find(|&i| normal[i] > 0.9).unwrap()] * scale
                    })
                    .collect();
                (id, <[f64; 2]>::try_from(distances).unwrap())
            })
            .collect();
        for _ in 0..2 {
            let result = if case.equal {
                let ids: Vec<_> = selected.iter().map(|(id, _)| *id).collect();
                try_chamfer_solid_edges(&input, &ids, case.setbacks[0][0] * scale, TOL * scale)
            } else {
                try_chamfer_solid_edges_with_distances(&input, &selected, TOL * scale)
            }
            .unwrap();
            let volume = expected * scale.powi(3);
            common::assert_solid(&result.solid, volume, &[0], TOL * scale);
            common::blend::assert_step(&result.solid, volume, TOL * scale);
            assert_eq!(result.generated_faces.len(), 4);
            let corner = result
                .generated_faces
                .iter()
                .find(|f| {
                    case.corner.iter().all(|&p| {
                        let p = matrix.transform_point(Point3::from(p));
                        f.vertex_iter().any(|v| v.point().near(&p))
                    })
                })
                .expect("corner face");
            assert_eq!(
                corner.vertex_iter().count(),
                if planar { case.corner.len() } else { 4 }
            );
            assert_eq!(matches!(corner.surface(), Surface::Plane(_)), planar);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            selected.reverse();
        }
    }
}

#[test]
fn equal_distance_corner_is_a_triangle() {
    check(
        Case {
            setbacks: [[1.0; 3]; 3],
            equal: true,
            corner: vec![[10.0, 9.0, 9.0], [9.0, 10.0, 9.0], [9.0, 9.0, 10.0]],
            occt_volume: 985.666666667,
        },
        true,
    );
}

#[test]
fn unequal_setbacks_meeting_at_the_crossings_form_a_triangle() {
    check(
        Case {
            setbacks: [[1.0, 0.0, 0.5], [1.0, 0.5, 0.0], [0.0, 1.0, 0.5]],
            equal: false,
            corner: vec![[10.0, 9.0, 9.0], [9.5, 10.0, 9.0], [9.5, 9.5, 10.0]],
            occt_volume: 992.75,
        },
        true,
    );
    check(
        Case {
            setbacks: [[1.0, 0.0, 0.75], [0.5, 0.75, 0.0], [0.0, 1.0, 0.5]],
            equal: false,
            corner: vec![[10.0, 9.5, 9.0], [9.25, 10.0, 9.0], [9.25, 9.5, 10.0]],
            occt_volume: 992.125,
        },
        true,
    );
}

// The large bevel along y is the pivot. The other two end at their sections through
// its crossings, so the corner face reaches the face y = 10 along a segment.
#[test]
fn dominant_bevel_corner_is_a_planar_quad() {
    check(
        Case {
            setbacks: [[3.0, 0.0, 3.0], [0.8, 0.8, 0.0], [0.0, 0.8, 0.8]],
            equal: false,
            corner: vec![
                [10.0, 9.2, 7.0],
                [9.2, 10.0, 7.0],
                [7.0, 10.0, 9.2],
                [7.0, 9.2, 10.0],
            ],
            occt_volume: 949.645333333,
        },
        true,
    );
}

// Two pivots are valid and give mirror-image quads with the same volume. Both contain the
// three bevel crossings; the fourth vertex is a section point on y = 10 or z = 10. The
// quad is not planar and OCCT fills it with a bilinear patch.
#[test]
fn twisted_corner_quad_is_a_bilinear_patch() {
    check(
        Case {
            setbacks: [[1.0, 0.0, 1.0], [1.0, 1.0, 0.0], [0.0, 0.5, 0.5]],
            equal: false,
            corner: vec![[10.0, 9.0, 9.0], [9.0, 10.0, 9.5], [9.0, 9.5, 10.0]],
            occt_volume: 989.145833333,
        },
        false,
    );
}
