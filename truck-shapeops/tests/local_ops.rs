//! Local operations on solids built with `truck-modeling`, checked with the harness.

mod common;

use common::{
    blend::*,
    modeling::{cuboid, cylinder},
    *,
};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::{builder, FaceID};
use truck_shapeops::local::{delete_face, move_faces, replace_surfaces, LocalOpError};

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

/// Box `[0, a] × [0, b] × [0, c]` with the vertical edge at `x = a, y = 0` rounded with `rho`.
fn rounded_box(a: f64, b: f64, c: f64, rho: f64) -> MSolid {
    let s = rho / f64::sqrt(2.0);
    let v: Vec<_> = [(0.0, 0.0), (a - rho, 0.0), (a, rho), (a, b), (0.0, b)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
        .into();
    let wire: MWire = vec![
        builder::line(&v[0], &v[1]),
        builder::circle_arc(&v[1], &v[2], Point3::new(a - rho + s, rho - s, 0.0)),
        builder::line(&v[2], &v[3]),
        builder::line(&v[3], &v[4]),
        builder::line(&v[4], &v[0]),
    ]
    .into();
    let face = builder::try_attach_plane(&[wire]).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * c)
}

/// Removing the round face of a rounded box restores the sharp edge: the two planes meet in one
/// line and the end faces lose their arc for a vertex.
#[test]
fn deleting_a_round_face_restores_the_sharp_edge() {
    let (a, b, c, rho) = (2.0, 3.0, 1.0, 0.5);
    let rounded = rounded_box(a, b, c, rho);
    let round = rounded
        .face_iter()
        .find(|face| matches!(face.surface(), MSurface::Extruded(_)))
        .unwrap()
        .id();
    let sharp = delete_face(&rounded, round).unwrap();
    let harness = from_modeling(&sharp);
    assert_counts(&harness, 8, 12, 6);
    assert_solid(&harness, a * b * c, &[0], TOL);
    let corner = |z: f64| Point3::new(a, 0.0, z);
    let sharp_edges = sharp
        .edge_iter()
        .filter(|edge| {
            let (p, q) = (edge.front().point(), edge.back().point());
            matches!(edge.curve(), truck_modeling::Curve::Line(_))
                && ((p.near(&corner(0.0)) && q.near(&corner(c)))
                    || (p.near(&corner(c)) && q.near(&corner(0.0))))
        })
        .count();
    assert_eq!(sharp_edges, 2, "the sharp edge once per adjacent face");
    let top = face_id_at(&sharp, Point3::new(a / 2.0, b / 2.0, c));
    let top = sharp.face_iter().find(|f| f.id() == top).unwrap();
    assert_eq!(top.boundaries()[0].len(), 4);
    assert_solid(
        &from_modeling(&rounded),
        a * b * c - (1.0 - PI / 4.0) * rho * rho * c,
        &[0],
        TOL,
    );
}

#[test]
fn deleting_a_box_side_or_a_five_sided_face_is_rejected() {
    let (a, b, c) = (2.0, 3.0, 1.0);
    let solid = abc_box(a, b, c);
    let top = face_id_at(&solid, Point3::new(a / 2.0, b / 2.0, c));
    let err = delete_face(&solid, top).unwrap_err();
    assert!(
        matches!(err, LocalOpError::NoIntersection { face, .. } if face == top),
        "{err}"
    );
    let rounded = rounded_box(a, b, c, 0.5);
    let top = face_id_at(&rounded, Point3::new(a / 2.0, b / 2.0, c));
    let err = delete_face(&rounded, top).unwrap_err();
    assert!(
        matches!(err, LocalOpError::Unsupported { face } if face == top),
        "{err}"
    );
}

/// A cylinder's top replaced by a plane tilted about a point on the axis keeps its volume, and its
/// new top edges lie on the cylinder.
#[test]
fn cylinder_top_tilted() {
    let (r, h, slope) = (1.0, 2.0, 0.3);
    let solid = cylinder(Point3::origin(), Vector3::unit_z(), r, h);
    let top = face_id_at(&solid, Point3::new(0.0, 0.0, h));
    let tilted = plane(
        Point3::new(0.0, 0.0, h),
        Point3::new(1.0, 0.0, h + slope),
        Point3::new(0.0, 1.0, h),
    );
    let cut = replace_surfaces(&solid, &[(top, tilted)]).unwrap();
    let harness = from_modeling(&cut);
    assert_counts(&harness, 4, 6, 4);
    assert_solid(&harness, PI * r * r * h, &[0], TOL);
    for edge in cut.edge_iter() {
        let (p, q) = (edge.front().point(), edge.back().point());
        if p.z > h - 1.0 && q.z > h - 1.0 {
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            for i in 0..=10 {
                let p = curve.subs(t0 + (t1 - t0) * i as f64 / 10.0);
                assert!((p.x * p.x + p.y * p.y - r * r).abs() < 1e-6, "{p:?}");
                assert!((p.z - h - slope * p.x).abs() < 1e-6, "{p:?}");
            }
        }
    }
}

/// A cylinder's top raised keeps exact arcs.
#[test]
fn cylinder_top_raised() {
    let (r, h, d) = (1.0, 2.0, 0.75);
    let solid = cylinder(Point3::origin(), Vector3::unit_z(), r, h);
    let top = face_id_at(&solid, Point3::new(0.0, 0.0, h));
    let higher = plane(
        Point3::new(0.0, 0.0, h + d),
        Point3::new(1.0, 0.0, h + d),
        Point3::new(0.0, 1.0, h + d),
    );
    let taller = replace_surfaces(&solid, &[(top, higher)]).unwrap();
    let arcs = taller
        .edge_iter()
        .filter(|edge| {
            matches!(edge.curve(), truck_modeling::Curve::Conic(_)) && edge.front().point().z > h
        })
        .count();
    assert_eq!(arcs, 2 * 2, "the two top arcs, each seen from two faces");
    assert_solid(&from_modeling(&taller), PI * r * r * (h + d), &[0], TOL);
}

/// A face whose neighbours are spline surfaces cannot be replaced.
#[test]
fn face_beside_a_spline_is_rejected() {
    use truck_modeling::{builder, Rad};
    let square = polygon_at(&[(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)], 0.0);
    let center = Point3::new(0.0, 0.0, 2.0);
    let vertex = builder::vertex(center + Vector3::unit_x());
    let circle: MWire = builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(7.0), 4);
    let loft: MSolid = builder::try_loft(&builder::align_sections(&[square, circle])).unwrap();
    let cap = face_id_at(&loft, Point3::new(0.0, 0.0, 2.0));
    let higher = plane(
        Point3::new(0.0, 0.0, 3.0),
        Point3::new(1.0, 0.0, 3.0),
        Point3::new(0.0, 1.0, 3.0),
    );
    let err = replace_surfaces(&loft, &[(cap, higher)]).unwrap_err();
    assert!(
        matches!(err, LocalOpError::Unsupported { face } if face == cap),
        "{err}"
    );
}

fn polygon_at(points: &[(f64, f64)], z: f64) -> MWire {
    let v: Vec<_> = points
        .iter()
        .map(|&(x, y)| builder::vertex(Point3::new(x, y, z)))
        .collect();
    (0..v.len())
        .map(|i| builder::line(&v[i], &v[(i + 1) % v.len()]))
        .collect()
}
