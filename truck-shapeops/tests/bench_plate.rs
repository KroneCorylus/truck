//! Wall time of set operations on a plate with cylindrical holes, per stage.
//!
//! Ignored by default. Run with
//! `cargo test --release -p truck-shapeops --test bench_plate -- --ignored --nocapture`.

mod common;

use common::{modeling::*, *};
use std::time::Instant;
use truck_modeling::*;
use truck_shapeops::profile;

const TOL: f64 = 0.01;
const PITCH: f64 = 4.0;
const RADIUS: f64 = 1.0;
const THICKNESS: f64 = 2.0;

/// Columns and rows of the grid holding `n` holes.
fn grid(n: usize) -> (usize, usize) {
    let cols = (n as f64).sqrt().ceil() as usize;
    (cols, n.div_ceil(cols))
}

/// The plate over the grid and one cutter per hole, through the plate.
fn plate_and_holes(n: usize) -> (Solid, Vec<Solid>) {
    let (cols, rows) = grid(n);
    let plate = cuboid(
        Point3::origin(),
        Point3::new(cols as f64 * PITCH, rows as f64 * PITCH, THICKNESS),
    );
    let holes = (0..n)
        .map(|k| {
            let (i, j) = ((k % cols) as f64, (k / cols) as f64);
            let base = Point3::new((i + 0.5) * PITCH, (j + 0.5) * PITCH, -1.0);
            cylinder(base, Vector3::unit_z(), RADIUS, THICKNESS + 2.0)
        })
        .collect();
    (plate, holes)
}

fn subtract(solid0: &Solid, solid1: &Solid) -> Solid {
    let mut solid1 = solid1.clone();
    solid1.not();
    truck_shapeops::and(solid0, &solid1, TOL).unwrap()
}

/// All cutters as one solid with one shell per hole, subtracted in one call.
fn one_boolean(plate: &Solid, holes: &[Solid]) -> Solid {
    let shells = holes
        .iter()
        .flat_map(|hole| hole.boundaries().iter().cloned())
        .collect();
    subtract(plate, &Solid::new(shells))
}

/// One subtraction per hole, each on the result of the previous.
fn one_by_one(plate: &Solid, holes: &[Solid]) -> Solid {
    holes
        .iter()
        .fold(plate.clone(), |plate, hole| subtract(&plate, hole))
}

fn timed(n: usize, label: &str, run: impl FnOnce() -> Solid) -> Solid {
    profile::take();
    let start = Instant::now();
    let result = run();
    let total = start.elapsed();
    let t = profile::take();
    let ms = |d: std::time::Duration| d.as_secs_f64() * 1e3;
    println!(
        "{n:>4} {label:<12} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9} {:>9}",
        ms(total),
        ms(t.triangulation),
        ms(t.pairing),
        ms(t.interference),
        ms(t.division),
        ms(t.classification),
        ms(t.fitting),
        t.pairs_scanned,
        t.pairs_overlapping,
    );
    result
}

#[test]
#[ignore]
fn plate_with_holes() {
    println!(
        "{:>4} {:<12} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "n",
        "mode",
        "total ms",
        "triang.",
        "pairing",
        "interf.",
        "division",
        "classif.",
        "fitting",
        "scanned",
        "overlap"
    );
    for n in [10, 30, 100] {
        let (plate, holes) = plate_and_holes(n);
        let (cols, rows) = grid(n);
        let expected = (cols * rows) as f64 * PITCH * PITCH * THICKNESS
            - n as f64 * cylinder_volume(RADIUS, THICKNESS);
        let results = [
            timed(n, "one boolean", || one_boolean(&plate, &holes)),
            timed(n, "one by one", || one_by_one(&plate, &holes)),
        ];
        for result in &results {
            assert_counts(result, 8 + 4 * n, 12 + 6 * n, 6 + 2 * n);
            assert_solid(result, expected, &[n], TOL);
        }
    }
}
