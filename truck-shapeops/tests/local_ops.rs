//! Local operations on solids built with `truck-modeling`, checked with the harness.

mod common;

use common::{blend::*, modeling::cuboid, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::{builder, FaceID};
use truck_shapeops::local::{move_faces, replace_surfaces, LocalOpError};

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

/// The face of `solid` whose surface passes through `point`.
fn face_id_at(solid: &MSolid, point: Point3) -> FaceID {
    solid
        .face_iter()
        .find(|face| {
            let surface = face.surface();
            surface
                .search_parameter(point, None, 10)
                .is_some_and(|(u, v)| surface.subs(u, v).near(&point))
        })
        .expect("no face through the point")
        .id()
}

fn plane(o: Point3, p: Point3, q: Point3) -> MSurface { MSurface::Plane(Plane::new(o, p, q)) }

fn abc_box(a: f64, b: f64, c: f64) -> MSolid { cuboid(Point3::origin(), Point3::new(a, b, c)) }

#[test]
fn box_top_raised() {
    let (a, b, c, d) = (2.0, 3.0, 1.0, 0.5);
    let solid = abc_box(a, b, c);
    let top = face_id_at(&solid, Point3::new(a / 2.0, b / 2.0, c));
    let higher = plane(
        Point3::new(0.0, 0.0, c + d),
        Point3::new(1.0, 0.0, c + d),
        Point3::new(0.0, 1.0, c + d),
    );
    let taller = replace_surfaces(&solid, &[(top, higher)]).unwrap();
    let harness = from_modeling(&taller);
    assert_counts(&harness, 8, 12, 6);
    assert_solid(&harness, a * b * (c + d), &[0], TOL);
    assert_solid(&from_modeling(&solid), a * b * c, &[0], TOL);
}

#[test]
fn box_top_tilted() {
    let (a, b, c, slope) = (2.0, 3.0, 1.0, 0.25);
    let solid = abc_box(a, b, c);
    let top = face_id_at(&solid, Point3::new(a / 2.0, b / 2.0, c));
    let tilted = plane(
        Point3::new(0.0, 0.0, c),
        Point3::new(a, 0.0, c + a * slope),
        Point3::new(0.0, 1.0, c),
    );
    let wedge = replace_surfaces(&solid, &[(top, tilted)]).unwrap();
    let harness = from_modeling(&wedge);
    assert_counts(&harness, 8, 12, 6);
    assert_solid(&harness, a * b * c + b * a * a * slope / 2.0, &[0], TOL);
}

#[test]
fn box_top_that_misses_a_side_is_rejected() {
    let (a, b, c) = (2.0, 3.0, 1.0);
    let solid = abc_box(a, b, c);
    let top = face_id_at(&solid, Point3::new(a / 2.0, b / 2.0, c));
    let sides = [
        face_id_at(&solid, Point3::new(0.0, b / 2.0, c / 2.0)),
        face_id_at(&solid, Point3::new(a, b / 2.0, c / 2.0)),
    ];
    let upright = plane(
        Point3::new(a / 2.0, 0.0, 0.0),
        Point3::new(a / 2.0, 1.0, 0.0),
        Point3::new(a / 2.0, 0.0, 1.0),
    );
    let err = replace_surfaces(&solid, &[(top, upright)]).unwrap_err();
    let LocalOpError::NoIntersection { face, neighbour } = err else {
        panic!("{err}");
    };
    assert!(face == top && sides.contains(&neighbour));
    assert_solid(&from_modeling(&solid), a * b * c, &[0], TOL);
}

/// The top raised and a side pushed out at once: the edge between them lies on both new planes.
#[test]
fn two_adjacent_faces_replaced_together() {
    let (a, b, c, d, e) = (2.0, 3.0, 1.0, 0.5, 0.75);
    let solid = abc_box(a, b, c);
    let top = face_id_at(&solid, Point3::new(a / 2.0, b / 2.0, c));
    let side = face_id_at(&solid, Point3::new(a, b / 2.0, c / 2.0));
    let higher = plane(
        Point3::new(0.0, 0.0, c + d),
        Point3::new(1.0, 0.0, c + d),
        Point3::new(0.0, 1.0, c + d),
    );
    let further = plane(
        Point3::new(a + e, 0.0, 0.0),
        Point3::new(a + e, 1.0, 0.0),
        Point3::new(a + e, 0.0, 1.0),
    );
    let bigger = replace_surfaces(&solid, &[(top, higher), (side, further)]).unwrap();
    let harness = from_modeling(&bigger);
    assert_counts(&harness, 8, 12, 6);
    assert_solid(&harness, (a + e) * b * (c + d), &[0], TOL);
    let corner_edges = bigger
        .edge_iter()
        .filter(|edge| {
            let (p, q) = (edge.front().point(), edge.back().point());
            (p.x - (a + e)).abs() < 1e-9
                && (q.x - (a + e)).abs() < 1e-9
                && (p.z - (c + d)).abs() < 1e-9
                && (q.z - (c + d)).abs() < 1e-9
        })
        .count();
    assert!(corner_edges >= 1);
}
