mod common;

use common::{modeling::*, *};
use std::f64::consts::PI;
use truck_modeling::*;

const TOL: f64 = 0.01;

fn unit_cube() -> Solid { cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0)) }

fn subtract(solid0: &Solid, solid1: &Solid, tol: f64) -> Option<Solid> {
    let mut solid1 = solid1.clone();
    solid1.not();
    truck_shapeops::and(solid0, &solid1, tol)
}

fn punched_cube_solid() -> Solid {
    let hole = cylinder(Point3::new(0.5, 0.5, -0.5), Vector3::unit_z(), 0.25, 2.0);
    subtract(&unit_cube(), &hole, TOL).unwrap()
}

#[test]
fn cube_is_valid() { assert_solid(&unit_cube(), 1.0, &[0], TOL); }

#[test]
fn cylinder_is_valid() {
    let roller = cylinder(Point3::new(0.0, 0.5, 1.25), Vector3::unit_x(), 0.25, 1.0);
    assert_solid(&roller, cylinder_volume(0.25, 1.0), &[0], TOL);
}

#[test]
fn sphere_is_valid() {
    let ball = sphere(Point3::new(0.5, 0.5, 1.25), 0.25);
    assert_solid(&ball, sphere_volume(0.25), &[0], TOL);
}

#[test]
fn punched_cube() {
    let punched = punched_cube_solid();
    assert_topology(&punched, &[1]);
    assert_volume(&punched, 1.0 - PI / 16.0, TOL);
}

#[test]
fn punched_cube_mesh_closed() { assert_mesh_closed(&punched_cube_solid(), TOL); }

/// Samples every intersection-curve edge densely and checks the points never turn back. Fails
/// if the leader of the curve wiggles, which also makes the projection onto the intersection
/// fail and panic in `subs`.
fn assert_intersection_edges_smooth(solid: &Solid) {
    for edge in solid.edge_iter() {
        let curve = edge.curve();
        if !matches!(curve, truck_modeling::Curve::IntersectionCurve(_)) {
            continue;
        }
        let (t0, t1) = curve.range_tuple();
        let n = 200;
        let points: Vec<Point3> = (0..=n)
            .map(|k| curve.subs(t0 + (t1 - t0) * k as f64 / n as f64))
            .collect();
        for k in 1..n {
            let (d0, d1) = (points[k] - points[k - 1], points[k + 1] - points[k]);
            assert!(
                d0.dot(d1) >= 0.0,
                "intersection edge turns back at {:?}",
                points[k]
            );
        }
    }
}

#[test]
fn punched_cube_edges_are_smooth() { assert_intersection_edges_smooth(&punched_cube_solid()); }

/// The hole's seam is rotated so that it no longer lines up with the cube's symmetry planes,
/// and a coarse tolerance is used; this combination used to panic during tessellation.
#[test]
fn punched_cube_rotated_seam() {
    let hole: Solid = {
        use truck_modeling::*;
        let base = Point3::new(0.5, 0.5, -0.5);
        let radial = Vector3::new(f64::cos(0.3), f64::sin(0.3), 0.0);
        let vertex = builder::vertex(base + radial * 0.25);
        let circle = builder::rsweep(&vertex, base, Vector3::unit_z(), Rad(7.0), 2);
        let disk = builder::try_attach_plane(&[circle]).unwrap();
        builder::tsweep(&disk, Vector3::unit_z() * 2.0)
    };
    let tol = 0.05;
    let punched = subtract(&unit_cube(), &hole, tol).unwrap();
    assert_intersection_edges_smooth(&punched);
    assert_solid(&punched, 1.0 - PI / 16.0, &[1], tol);
}

#[test]
fn union_disjoint_cubes() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0, 0], TOL);
}

#[test]
#[ignore = "coincident faces are not supported yet"]
fn union_stacked_cubes() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 1.0, 2.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0], TOL);
}

#[test]
#[ignore = "coincident faces are not supported yet"]
fn union_offset_stacked_cubes() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(0.5, 0.5, 1.0), Point3::new(1.5, 1.5, 2.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0], TOL);
}

#[test]
#[ignore = "coincident faces are not supported yet"]
fn subtract_flush_pocket() {
    let cube = unit_cube();
    let pocket = cuboid(Point3::new(0.5, 0.25, 0.25), Point3::new(1.0, 0.75, 0.75));
    let result = subtract(&cube, &pocket, TOL).unwrap();
    assert_solid(&result, 1.0 - 0.125, &[0], TOL);
}

#[test]
fn union_cylinder_on_plane() {
    let cube = unit_cube();
    let roller = cylinder(Point3::new(0.0, 0.5, 1.25), Vector3::unit_x(), 0.25, 1.0);
    let union = truck_shapeops::or(&cube, &roller, TOL).unwrap();
    assert_solid(&union, 1.0 + cylinder_volume(0.25, 1.0), &[0, 0], TOL);
}

/// Subtracting a solid that only touches leaves the other solid unchanged.
#[test]
fn subtract_cylinder_on_plane() {
    let cube = unit_cube();
    let roller = cylinder(Point3::new(0.0, 0.5, 1.25), Vector3::unit_x(), 0.25, 1.0);
    let difference = subtract(&cube, &roller, TOL).unwrap();
    assert_solid(&difference, 1.0, &[0], TOL);
}

#[test]
fn union_sphere_on_plane() {
    let cube = unit_cube();
    let ball = sphere(Point3::new(0.5, 0.5, 1.25), 0.25);
    let union = truck_shapeops::or(&cube, &ball, TOL).unwrap();
    assert_solid(&union, 1.0 + sphere_volume(0.25), &[0, 0], TOL);
}

#[test]
fn union_boxes_sharing_edge() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, 2.0, 1.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0, 0], TOL);
}

#[test]
fn subtract_boxes_sharing_edge() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, 2.0, 1.0));
    let difference = subtract(&cube0, &cube1, TOL).unwrap();
    assert_solid(&difference, 1.0, &[0], TOL);
}

#[test]
fn union_boxes_sharing_corner() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0, 0], TOL);
}
