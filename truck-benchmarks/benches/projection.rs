use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_modeling::*;

#[divan::bench(args = HOLES, sample_count = 20, sample_size = 1, max_time = 2)]
fn hidden_lines(bencher: Bencher, holes: usize) {
    let solid = perforated_plate(holes);
    let direction = -Vector3::new(1.0, 1.0, 1.0).normalize();
    let u = Vector3::unit_x().cross(direction).normalize();
    let v = u.cross(direction);
    let plane = Plane::new(Point3::origin(), Point3::from_vec(u), Point3::from_vec(v));
    let run = || truck_hlr::project(black_box(&solid), &plane, direction, TOL);
    let view = run();
    assert!(view
        .curves
        .iter()
        .any(|(_, v)| *v == truck_hlr::Visibility::Visible));
    assert!(view
        .curves
        .iter()
        .any(|(_, v)| *v == truck_hlr::Visibility::Hidden));
    bencher.bench(run);
}
