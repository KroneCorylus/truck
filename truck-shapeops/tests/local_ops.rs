//! Local operations on solids built with `truck-modeling`, checked with the harness.

mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::{builder, FaceID};
use truck_shapeops::local::{move_faces, LocalOpError};

type MWire = truck_modeling::Wire;
type MSolid = truck_modeling::Solid;
type MSurface = truck_modeling::Surface;

const TOL: f64 = 0.001;

/// Plate `[0, size]²` of `thickness` with a round hole of `radius` at `center`.
fn plate_with_hole(size: f64, thickness: f64, center: Point3, radius: f64) -> MSolid {
    let v: Vec<_> = [(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
        .into();
    let outer: MWire = (0..4)
        .map(|i| builder::line(&v[i], &v[(i + 1) % 4]))
        .collect();
    let vertex = builder::vertex(center + Vector3::unit_x() * radius);
    let hole: MWire = builder::rsweep(&vertex, center, -Vector3::unit_z(), Rad(7.0), 3);
    let face = builder::try_attach_plane(&[outer, hole]).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * thickness)
}

fn hole_faces(plate: &MSolid) -> Vec<FaceID> {
    plate
        .face_iter()
        .filter(|face| matches!(face.surface(), MSurface::Extruded(_)))
        .map(|face| face.id())
        .collect()
}

fn has_face_through(solid: &MSolid, point: Point3) -> bool {
    solid.face_iter().any(|face| {
        let surface = face.surface();
        surface
            .search_parameter(point, None, 10)
            .is_some_and(|(u, v)| surface.subs(u, v).near(&point))
    })
}

#[test]
fn hole_moved_within_the_plate() {
    let (size, thickness, radius) = (10.0, 1.0, 1.0);
    let center = Point3::new(3.0, 3.0, 0.0);
    let plate = plate_with_hole(size, thickness, center, radius);
    let volume = size * size * thickness - PI * radius * radius * thickness;
    let faces = hole_faces(&plate);
    assert_eq!(faces.len(), 3);

    let shift = Vector3::new(5.0, 0.0, 0.0);
    let moved = move_faces(&plate, &faces, shift).unwrap();
    assert_eq!(moved.face_iter().count(), plate.face_iter().count());
    assert_solid(&from_modeling(&moved), volume, &[1], TOL);
    let on_hole = Point3::new(radius, 0.0, thickness / 2.0);
    assert!(has_face_through(&moved, center + shift + on_hole.to_vec()));
    assert!(!has_face_through(&moved, center + on_hole.to_vec()));
    // the input is untouched
    assert!(has_face_through(&plate, center + on_hole.to_vec()));
    assert_solid(&from_modeling(&plate), volume, &[1], TOL);
}

#[test]
fn hole_moved_out_of_the_plate_is_rejected() {
    let (size, thickness, radius) = (10.0, 1.0, 1.0);
    let center = Point3::new(3.0, 3.0, 0.0);
    let plate = plate_with_hole(size, thickness, center, radius);
    let faces = hole_faces(&plate);
    let planes: Vec<_> = plate
        .face_iter()
        .filter(|face| matches!(face.surface(), MSurface::Plane(_)))
        .map(|face| face.id())
        .collect();
    let err = move_faces(&plate, &faces, Vector3::new(8.0, 0.0, 0.0)).unwrap_err();
    let LocalOpError::OutsideNeighbour { face, neighbour } = err else {
        panic!("{err}");
    };
    assert!(faces.contains(&face) && planes.contains(&neighbour));
    // a hole lifted out of the plate's planes is not a move this operation can do
    let err = move_faces(&plate, &faces, Vector3::new(0.0, 0.0, 0.5)).unwrap_err();
    assert!(matches!(err, LocalOpError::Unsupported { face } if faces.contains(&face)));
    let volume = size * size * thickness - PI * radius * radius * thickness;
    assert_solid(&from_modeling(&plate), volume, &[1], TOL);
}
