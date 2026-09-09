use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_meshalgo::prelude::*;

#[divan::bench(args = [0.1, 0.01, 0.001])]
fn cylinder_tolerance(bencher: Bencher, tol: f64) {
    let solid = cylinder();
    let run = || black_box(&solid).triangulation(black_box(tol));
    assert!(run().face_iter().all(|f| f.surface().is_some()));
    bencher.bench(run);
}

#[divan::bench(args = [1, 10, 30, 100])]
fn plate(bencher: Bencher, holes: usize) {
    let solid = perforated_plate(holes);
    check_volume(&solid, plate_volume(holes));
    bencher.bench(|| black_box(&solid).triangulation(TOL));
}

#[divan::bench(args = HOLES)]
fn mesh_to_polygon(bencher: Bencher, holes: usize) {
    let mesh = perforated_plate(holes).triangulation(TOL);
    assert!(mesh.face_iter().all(|f| f.surface().is_some()));
    assert!(!mesh.to_polygon().positions().is_empty());
    bencher.bench(|| black_box(&mesh).to_polygon());
}
