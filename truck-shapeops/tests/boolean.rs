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
#[ignore = "intersection-curve edges fold back near the cylinder seam, so the mesh overlaps"]
fn punched_cube_mesh_closed() { assert_mesh_closed(&punched_cube_solid(), TOL); }

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
#[ignore = "tangent contact along a curve is not supported yet"]
fn union_cylinder_on_plane() {
    let cube = unit_cube();
    let roller = cylinder(Point3::new(0.0, 0.5, 1.25), Vector3::unit_x(), 0.25, 1.0);
    let union = truck_shapeops::or(&cube, &roller, TOL).unwrap();
    assert_solid(&union, 1.0 + cylinder_volume(0.25, 1.0), &[0, 0], TOL);
}

#[test]
fn union_sphere_on_plane() {
    let cube = unit_cube();
    let ball = sphere(Point3::new(0.5, 0.5, 1.25), 0.25);
    let union = truck_shapeops::or(&cube, &ball, TOL).unwrap();
    assert_solid(&union, 1.0 + sphere_volume(0.25), &[0, 0], TOL);
}

#[test]
#[ignore = "contact along an edge is not supported yet"]
fn union_boxes_sharing_edge() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, 2.0, 1.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0, 0], TOL);
}

#[test]
fn union_boxes_sharing_corner() {
    let cube0 = unit_cube();
    let cube1 = cuboid(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
    let union = truck_shapeops::or(&cube0, &cube1, TOL).unwrap();
    assert_solid(&union, 2.0, &[0, 0], TOL);
}
