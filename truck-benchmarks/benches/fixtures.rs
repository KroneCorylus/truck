use std::f64::consts::{PI, TAU};
use truck_meshalgo::prelude::*;
use truck_modeling::*;

pub const TOL: f64 = 0.01;
pub const HOLES: &[usize] = &[1, 10, 30];

pub fn cuboid(min: Point3, max: Point3) -> Solid {
    primitive::cuboid(BoundingBox::from_iter([min, max]))
}

pub fn polygon(edges: usize, radius: f64, z: f64) -> Wire {
    let vertices: Vec<_> = (0..edges)
        .map(|i| {
            let angle = TAU * i as f64 / edges as f64;
            builder::vertex(Point3::new(radius * angle.cos(), radius * angle.sin(), z))
        })
        .collect();
    (0..edges)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % edges]))
        .collect()
}

pub fn circle(center: Point3, radius: f64) -> Wire {
    let vertex = builder::vertex(center + Vector3::unit_x() * radius);
    builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(TAU), 2)
}

pub fn cylinder() -> Solid {
    let disc: Face = builder::try_attach_plane(vec![circle(Point3::origin(), 1.0)]).unwrap();
    builder::tsweep(&disc, Vector3::unit_z() * 2.0)
}

fn dimensions(holes: usize) -> (usize, usize) {
    let cols = (holes as f64).sqrt().ceil() as usize;
    (cols, holes.div_ceil(cols))
}

fn centers(holes: usize, z: f64) -> impl Iterator<Item = Point3> {
    let (cols, _) = dimensions(holes);
    (0..holes).map(move |i| {
        Point3::new(
            (i % cols) as f64 * 4.0 + 2.0,
            (i / cols) as f64 * 4.0 + 2.0,
            z,
        )
    })
}

pub fn plate_and_cutters(holes: usize) -> (Solid, Vec<Solid>) {
    let (cols, rows) = dimensions(holes);
    let plate = cuboid(
        Point3::origin(),
        Point3::new(cols as f64 * 4.0, rows as f64 * 4.0, 2.0),
    );
    let cutters = centers(holes, -1.0)
        .map(|center| {
            let disc: Face = builder::try_attach_plane(vec![circle(center, 1.0)]).unwrap();
            builder::tsweep(&disc, Vector3::unit_z() * 4.0)
        })
        .collect();
    (plate, cutters)
}

pub fn perforated_plate(holes: usize) -> Solid {
    let (cols, rows) = dimensions(holes);
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let outer: Wire = primitive::rect(
        BoundingBox::from_iter([
            Point2::origin(),
            Point2::new(cols as f64 * 4.0, rows as f64 * 4.0),
        ]),
        plane,
    );
    let mut wires = vec![outer];
    wires.extend(centers(holes, 0.0).map(|center| circle(center, 1.0).inverse()));
    let face: Face = builder::try_attach_plane(wires).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * 2.0)
}

pub fn plate_volume(holes: usize) -> f64 {
    let (cols, rows) = dimensions(holes);
    (cols * rows) as f64 * 32.0 - holes as f64 * PI * 2.0
}

// Preflight checks run before the timed closure, so a fast empty/partial result cannot win.
pub fn check_solid(solid: &Solid) {
    assert!(!solid.boundaries().is_empty());
    Solid::try_new(solid.boundaries().clone()).expect("closed output topology");
    let meshed = solid.triangulation(TOL);
    assert!(
        meshed.face_iter().all(|face| face.surface().is_some()),
        "partial tessellation"
    );
    let volume = meshed.to_polygon().volume();
    assert!(
        volume.is_finite() && volume > 0.0,
        "non-positive output volume: {volume}"
    );
}

pub fn check_volume(solid: &Solid, expected: f64) {
    check_solid(solid);
    let actual = solid.triangulation(TOL).to_polygon().volume();
    assert!(
        (actual - expected).abs() < expected * 0.01,
        "volume {actual}, expected {expected}"
    );
}
