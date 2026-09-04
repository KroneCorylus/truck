mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

/// Chamfers the edge of `shell` through `edge_point`, between the faces through `face_point0`
/// and `face_point1`, with the side faces through `side_point0` and `side_point1`.
fn chamfer_edge(
    shell: &mut Shell,
    [face_point0, face_point1]: [Point3; 2],
    edge_point: Point3,
    [side_point0, side_point1]: [Point3; 2],
    (d0, d1): (f64, f64),
) {
    let face_idx0 = face_through(shell, face_point0);
    let face_idx1 = face_through(shell, face_point1);
    let edge = edge_through(shell, edge_point);
    // `side0` is the side at the front of the edge as oriented in `face0`.
    let front = shell[face_idx0]
        .edge_iter()
        .find(|e| e.id() == edge.id())
        .unwrap()
        .front()
        .id();
    let mut side_idx0 = face_through(shell, side_point0);
    let mut side_idx1 = face_through(shell, side_point1);
    if !shell[side_idx0].vertex_iter().any(|v| v.id() == front) {
        std::mem::swap(&mut side_idx0, &mut side_idx1);
    }
    let FilletWithSide {
        simple_fillet:
            SimpleFillet {
                fillet,
                face0,
                face1,
            },
        side0,
        side1,
    } = chamfer_with_side(
        &shell[face_idx0],
        &shell[face_idx1],
        edge.id(),
        Some(&shell[side_idx0]),
        Some(&shell[side_idx1]),
        d0,
        d1,
        TOL,
    )
    .unwrap();
    shell[face_idx0] = face0;
    shell[face_idx1] = face1;
    shell[side_idx0] = side0.unwrap();
    shell[side_idx1] = side1.unwrap();
    shell.push(fillet);
}

fn unit_cube() -> Shell {
    let cube = modeling::cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
    from_modeling(&cube).into_boundaries().pop().unwrap()
}

#[test]
fn chamfer_cube_symmetric() {
    let mut shell = unit_cube();
    let d = 0.2;
    chamfer_edge(
        &mut shell,
        [Point3::new(0.5, 0.0, 0.5), Point3::new(1.0, 0.5, 0.5)],
        Point3::new(1.0, 0.0, 0.5),
        [Point3::new(0.5, 0.5, 0.0), Point3::new(0.5, 0.5, 1.0)],
        (d, d),
    );
    let solid = Solid::new(vec![shell]);
    assert_solid(&solid, 1.0 - d * d / 2.0, &[0], TOL);
}

#[test]
fn chamfer_cube_asymmetric() {
    let mut shell = unit_cube();
    let (d0, d1) = (0.1, 0.3);
    chamfer_edge(
        &mut shell,
        [Point3::new(0.5, 0.0, 0.5), Point3::new(1.0, 0.5, 0.5)],
        Point3::new(1.0, 0.0, 0.5),
        [Point3::new(0.5, 0.5, 0.0), Point3::new(0.5, 0.5, 1.0)],
        (d0, d1),
    );
    let solid = Solid::new(vec![shell]);
    assert_solid(&solid, 1.0 - d0 * d1 / 2.0, &[0], TOL);
}

/// Half-cylinder prism: the flat face is the plane `x = 0`, the round face has radius `radius`
/// about the z-axis on the side `x >= 0`, and the height runs from `z = 0` to `height`.
fn half_cylinder(radius: f64, height: f64) -> Shell {
    use truck_modeling::*;
    let v0 = builder::vertex(Point3::new(0.0, -radius, 0.0));
    let v1 = builder::vertex(Point3::new(0.0, radius, 0.0));
    let arc = builder::circle_arc(&v0, &v1, Point3::new(radius, 0.0, 0.0));
    let chord = builder::line(&v1, &v0);
    let bottom = builder::try_attach_plane(&[wire![arc, chord]]).unwrap();
    let solid: Solid = builder::tsweep(&bottom, Vector3::unit_z() * height);
    from_modeling(&solid).into_boundaries().pop().unwrap()
}

#[test]
fn chamfer_edge_of_half_cylinder() {
    let (radius, height) = (0.5, 1.0);
    let (d0, d1) = (0.1, 0.15);
    let mut shell = half_cylinder(radius, height);
    let flat = Point3::new(0.0, 0.0, 0.5);
    let round = Point3::new(radius, 0.0, 0.5);
    chamfer_edge(
        &mut shell,
        [flat, round],
        Point3::new(0.0, radius, 0.5),
        [Point3::new(0.2, 0.0, 0.0), Point3::new(0.2, 0.0, height)],
        (d0, d1),
    );
    let solid = Solid::new(vec![shell]);

    // Cross-section removed: the triangle between the corner and the two contact points, plus the
    // circular segment between the chord from the corner to the round contact point and the arc.
    let phi = (d1 / radius).atan();
    let triangle = d0 * radius * phi.sin() / 2.0;
    let segment = radius * radius * (phi - phi.sin()) / 2.0;
    let expected = PI * radius * radius * height / 2.0 - (triangle + segment) * height;
    assert_solid(&solid, expected, &[0], TOL);
}
