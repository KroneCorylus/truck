mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_along_wire;

const TOL: f64 = 0.001;
const ANGLE: f64 = 7.0 * std::f64::consts::PI / 12.0;

fn body(stepped: bool) -> (Solid, Wire) {
    let start = builder::vertex(Point3::new(3.0, 0.0, 0.0));
    let arc = builder::rsweep(&start, Point3::origin(), Vector3::unit_z(), Rad(ANGLE), 2);
    let mut boundary = arc.clone();
    if stepped {
        let outer_start = builder::vertex(Point3::new(4.5 * ANGLE.cos(), 4.5 * ANGLE.sin(), 0.0));
        boundary.push_back(builder::line(arc.back_vertex().unwrap(), &outer_start));
        let outer = builder::rsweep(
            &outer_start,
            Point3::origin(),
            Vector3::unit_z(),
            Rad(std::f64::consts::TAU - ANGLE),
            3,
        );
        let closing = builder::line(outer.back_vertex().unwrap(), arc.front_vertex().unwrap());
        boundary.extend(outer);
        boundary.push_back(closing);
    } else {
        let center = builder::vertex(Point3::origin());
        boundary.extend([
            builder::line(arc.back_vertex().unwrap(), &center),
            builder::line(&center, arc.front_vertex().unwrap()),
        ]);
    }
    let face = builder::try_attach_plane(&[boundary]).unwrap();
    (builder::tsweep(&face, Vector3::new(0.0, 0.0, 10.0)), arc)
}

fn check(stepped: bool, radial: f64, axial: f64, reject: bool) {
    for transformed in [false, true] {
        let scale = if transformed { 0.2 } else { 1.0 };
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(2.0, -3.0, 4.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let (base, arc) = body(stepped);
        let mut input = builder::transformed(&base, matrix);
        if transformed {
            let mut faces: Vec<_> = input.boundaries()[0].iter().cloned().collect();
            faces.reverse();
            input = Solid::new(vec![faces.into()]);
        }
        let mut wire: Wire = arc
            .iter()
            .map(|original| {
                let curve = original.oriented_curve();
                let (a, b) = curve.range_tuple();
                let edge = common::blend::edge_through(
                    &input.boundaries()[0],
                    matrix.transform_point(curve.subs((a + b) / 2.0)),
                );
                if edge
                    .front()
                    .point()
                    .near(&matrix.transform_point(original.front().point()))
                {
                    edge
                } else {
                    edge.inverse()
                }
            })
            .collect();
        if transformed {
            wire.invert();
        }
        assert!(wire.is_continuous());
        let cap = input
            .face_iter()
            .find(|f| {
                f.oriented_surface()
                    .normal(0.0, 0.0)
                    .normalize()
                    .dot(matrix.transform_vector(-Vector3::unit_z()).normalize())
                    > 0.9
            })
            .unwrap();
        let cap_edge = cap.edge_iter().find(|e| e.id() == wire[0].id()).unwrap();
        let (d0, d1) = if cap_edge.orientation() == wire[0].orientation() {
            (radial, axial)
        } else {
            (axial, radial)
        };
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_chamfer_solid_along_wire(&input, &wire, d0 * scale, d1 * scale, TOL);
        if reject {
            assert!(result.is_err(), "oversized terminal cut must fail");
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            continue;
        }
        let result = result.unwrap();
        let initial = if stepped {
            std::f64::consts::PI * 4.5_f64.powi(2) * 10.0
                - ANGLE / 2.0 * (4.5_f64.powi(2) - 9.0) * 10.0
        } else {
            ANGLE / 2.0 * 9.0 * 10.0
        };
        let volume = (initial - ANGLE / 2.0 * axial * (3.0 * radial - radial * radial / 3.0))
            * scale.powi(3);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), wire.len());
        let mut contacts = [false; 2];
        for face in &result.generated_faces {
            for vertex in face.vertex_iter() {
                let p = inverse.transform_point(vertex.point());
                let radius = (p.x * p.x + p.y * p.y).sqrt();
                let cap = (radius - (3.0 - radial)).abs() < 1e-6 && p.z.abs() < 1e-6;
                let cylinder = (radius - 3.0).abs() < 1e-6 && (p.z - axial).abs() < 1e-6;
                assert!(cap || cylinder, "unexpected contact {p:?}");
                contacts[0] |= cap;
                contacts[1] |= cylinder;
            }
        }
        assert_eq!(contacts, [true; 2]);
        for original in input.face_iter() {
            let replacement = result
                .modified_faces
                .iter()
                .find(|(id, _)| *id == original.id())
                .map(|(_, f)| f.id())
                .unwrap_or(original.id());
            assert!(result.solid.face_iter().any(|f| f.id() == replacement));
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn equal_chamfer_extends_concave_terminal_lines() { check(true, 1.0, 1.0, false); }
#[test]
fn unequal_chamfer_extends_concave_terminal_lines() { check(true, 1.0, 0.5, false); }
#[test]
fn angle_chamfer_extends_concave_terminal_lines() { check(true, 1.0, 1.0 / 3.0_f64.sqrt(), false); }
#[test]
fn convex_terminal_edges_keep_bounded_trims() {
    for axial in [1.0, 0.5, 1.0 / 3.0_f64.sqrt()] {
        check(false, 1.0, axial, false);
    }
}
#[test]
fn oversized_terminal_cuts_preserve_the_input() {
    for stepped in [false, true] {
        for (radial, axial) in [(4.0, 1.0), (1.0, 11.0)] {
            check(stepped, radial, axial, true);
        }
    }
}
