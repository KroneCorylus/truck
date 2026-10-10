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

#[divan::bench(sample_count = 5, sample_size = 1, max_time = 300)]
fn shell_tube(bencher: Bencher) {
    let face = builder::try_attach_plane(&[
        circle(Point3::origin(), 10.),
        circle(Point3::origin(), 3.).inverse(),
    ])
    .unwrap();
    let solid = builder::tsweep(&face, Vector3::unit_z() * 10.);
    let top = solid
        .face_iter()
        .find(|face| face.vertex_iter().all(|vertex| vertex.point().z.near(&10.)))
        .unwrap()
        .id();
    let run = || try_shell(black_box(&solid), &[top], 0.5).expect("tube shell");
    check_volume(&run(), 169. * std::f64::consts::PI);
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

#[divan::bench(sample_count = 20, sample_size = 1, max_time = 10)]
fn rolling_fillet(bencher: Bencher) {
    let solid = cube();
    let wire: Wire = vec![solid.boundaries()[0][0].boundaries()[0][0].clone()].into();
    let run =
        || try_fillet_solid_along_wire(black_box(&solid), &wire, 0.2, TOL).expect("rolling fillet");
    check_solid(&run().solid);
    bencher.bench(run);
}
