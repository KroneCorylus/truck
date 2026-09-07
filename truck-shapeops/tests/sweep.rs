//! Sweeps along paths of lines and arcs built with `truck-modeling`, checked with the harness.

mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::{builder, errors::Error};

type MWire = truck_modeling::Wire;
type MFace = truck_modeling::Face;
type MSolid = truck_modeling::Solid;

const TOL: f64 = 0.001;

/// Disc of `radius` at `center` facing `normal`, as three arcs.
fn disc(center: Point3, normal: Vector3, radius: f64) -> MFace {
    let radial = normal.cross(Vector3::unit_z());
    let radial = match radial.so_small() {
        true => Vector3::unit_x(),
        false => radial.normalize(),
    };
    let vertex = builder::vertex(center + radial * radius);
    let circle: MWire = builder::rsweep(&vertex, center, normal, Rad(7.0), 3);
    builder::try_attach_plane(&[circle]).unwrap()
}

/// Circular arc from `v0` about `center`, turning about `axis` by `angle`.
fn arc(
    v0: &truck_modeling::Vertex,
    center: Point3,
    axis: Vector3,
    angle: f64,
) -> truck_modeling::Edge {
    let p0 = v0.point();
    let rotate = |a: f64| {
        Point3::from_vec(Matrix3::from_axis_angle(axis, Rad(a)) * (p0 - center) + center.to_vec())
    };
    let v1 = builder::vertex(rotate(angle));
    builder::circle_arc(v0, &v1, rotate(angle / 2.0))
}

fn volume(solid: &MSolid) -> f64 {
    use truck_meshalgo::prelude::*;
    from_modeling(solid)
        .triangulation(TOL)
        .to_polygon()
        .volume()
}

#[test]
fn disc_along_a_segment_matches_tsweep() {
    let (radius, length) = (0.5, 2.0);
    let profile = disc(Point3::origin(), Vector3::unit_x(), radius);
    let v0 = builder::vertex(Point3::origin());
    let v1 = builder::vertex(Point3::new(length, 0.0, 0.0));
    let path: MWire = vec![builder::line(&v0, &v1)].into();
    let swept = builder::sweep_along_wire(&profile, &path).unwrap();
    let extruded: MSolid = builder::tsweep(&profile, Vector3::unit_x() * length);
    assert_eq!(swept.boundaries()[0].len(), extruded.boundaries()[0].len());
    assert!((volume(&swept) - volume(&extruded)).abs() < 1e-9);
    assert_solid(
        &from_modeling(&swept),
        cylinder_volume(radius, length),
        &[0],
        TOL,
    );
}

#[test]
fn disc_along_an_arc_matches_rsweep() {
    let (radius, big, angle) = (0.5, 2.0, PI / 2.0);
    let start = Point3::new(big, 0.0, 0.0);
    let profile = disc(start, Vector3::unit_y(), radius);
    let path: MWire = vec![arc(
        &builder::vertex(start),
        Point3::origin(),
        Vector3::unit_z(),
        angle,
    )]
    .into();
    let swept = builder::sweep_along_wire(&profile, &path).unwrap();
    let revolved: MSolid =
        builder::rsweep(&profile, Point3::origin(), Vector3::unit_z(), Rad(angle), 1);
    assert_eq!(swept.boundaries()[0].len(), revolved.boundaries()[0].len());
    assert!((volume(&swept) - volume(&revolved)).abs() < 1e-9);
    let pappus = PI * radius * radius * big * angle;
    assert_solid(&from_modeling(&swept), pappus, &[0], TOL);
}

/// A line, a quarter turn to the left and another line: the volume is the section times the
/// length of the path, by Pappus, since the centroid of the disc rides on the path.
#[test]
fn disc_along_line_arc_line() {
    let radius = 0.4;
    let profile = disc(Point3::origin(), Vector3::unit_x(), radius);
    let v0 = builder::vertex(Point3::origin());
    let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let bend = arc(&v1, Point3::new(1.0, 1.0, 0.0), Vector3::unit_z(), PI / 2.0);
    let v3 = builder::vertex(Point3::new(2.0, 3.0, 0.0));
    let path: MWire = vec![
        builder::line(&v0, &v1),
        bend.clone(),
        builder::line(bend.back(), &v3),
    ]
    .into();
    let swept = builder::sweep_along_wire(&profile, &path).unwrap();
    assert_eq!(swept.boundaries()[0].len(), 3 * 3 + 2);
    let length = 1.0 + PI / 2.0 + 2.0;
    assert_solid(
        &from_modeling(&swept),
        PI * radius * radius * length,
        &[0],
        TOL,
    );
}

/// A disc around a full circle of four arcs: a torus, no caps, genus one.
#[test]
fn closed_planar_path_gives_a_torus() {
    let (radius, big) = (0.3, 2.0);
    let start = Point3::new(big, 0.0, 0.0);
    let profile = disc(start, Vector3::unit_y(), radius);
    let vertex = builder::vertex(start);
    let path: MWire = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_z(), Rad(7.0), 4);
    assert!(path.is_closed());
    let swept = builder::sweep_along_wire(&profile, &path).unwrap();
    assert_eq!(swept.boundaries()[0].len(), 4 * 3);
    let pappus = 2.0 * PI * big * PI * radius * radius;
    assert_solid(&from_modeling(&swept), pappus, &[1], TOL);
}

#[test]
fn corners_and_tight_arcs_are_rejected() {
    let profile = disc(Point3::origin(), Vector3::unit_x(), 0.6);
    let v0 = builder::vertex(Point3::origin());
    let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let v2 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
    let corner: MWire = vec![builder::line(&v0, &v1), builder::line(&v1, &v2)].into();
    assert_eq!(
        builder::sweep_along_wire(&profile, &corner).unwrap_err(),
        Error::PathNotSmooth(1)
    );
    let tight: MWire = vec![arc(
        &v0,
        Point3::new(0.0, 0.5, 0.0),
        Vector3::unit_z(),
        PI / 2.0,
    )]
    .into();
    assert_eq!(
        builder::sweep_along_wire(&profile, &tight).unwrap_err(),
        Error::PathTooTight(0)
    );
}
