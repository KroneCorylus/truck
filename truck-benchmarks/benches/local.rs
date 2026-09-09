use super::fixtures::*;
use divan::{black_box, Bencher};
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::{fillet::*, local::*};

fn cube() -> Solid { cuboid(Point3::origin(), Point3::new(4.0, 4.0, 4.0)) }

fn edges(solid: &Solid, count: usize) -> Vec<EdgeID> {
    let mut seen = HashSet::new();
    solid
        .edge_iter()
        .map(|e| e.id())
        .filter(|id| seen.insert(*id))
        .take(count)
        .collect()
}

#[divan::bench(args = [1, 12], sample_count = 20, sample_size = 1, max_time = 2)]
fn fillet(bencher: Bencher, count: usize) {
    let solid = cube();
    let selected = edges(&solid, count);
    let run = || try_fillet_solid_edges(black_box(&solid), &selected, 0.2, TOL).expect("fillet");
    check_solid(&run().solid);
    bencher.bench(run);
}

#[divan::bench(args = [1, 12], sample_count = 20, sample_size = 1, max_time = 2)]
fn chamfer(bencher: Bencher, count: usize) {
    let solid = cube();
    let selected = edges(&solid, count);
    let run = || try_chamfer_solid_edges(black_box(&solid), &selected, 0.2, TOL).expect("chamfer");
    check_solid(&run().solid);
    bencher.bench(run);
}

#[divan::bench]
fn shell_box(bencher: Bencher) {
    let solid = cube();
    let top = solid
        .face_iter()
        .find(|f| f.oriented_surface().normal(0.0, 0.0).z > 0.5)
        .unwrap()
        .id();
    let run = || try_shell(black_box(&solid), &[top], 0.2).expect("shell");
    check_volume(&run(), 64.0 - 3.6 * 3.6 * 3.8);
    bencher.bench(run);
}

#[divan::bench(args = [1, 4])]
fn draft_box(bencher: Bencher, count: usize) {
    let solid = cube();
    let sides: Vec<_> = solid
        .face_iter()
        .filter(|f| f.oriented_surface().normal(0.0, 0.0).z.abs() < 0.5)
        .take(count)
        .map(|f| f.id())
        .collect();
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let run = || {
        try_draft(
            black_box(&solid),
            &sides,
            &plane,
            Vector3::unit_z(),
            Rad(0.05),
        )
        .expect("draft")
    };
    check_solid(&run());
    bencher.bench(run);
}
