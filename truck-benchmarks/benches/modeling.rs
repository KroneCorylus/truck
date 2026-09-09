use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_modeling::*;

#[divan::bench(args = [4, 32, 128])]
fn extrude(bencher: Bencher, edges: usize) {
    let face: Face = builder::try_attach_plane(vec![polygon(edges, 2.0, 0.0)]).unwrap();
    let run = || -> Solid { builder::tsweep(black_box(&face), Vector3::unit_z() * 5.0) };
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = [4, 32, 128])]
fn revolve(bencher: Bencher, edges: usize) {
    let wire = polygon(edges, 0.5, 0.0);
    let wire = builder::rotated(
        &wire,
        Point3::origin(),
        Vector3::unit_x(),
        Rad(-std::f64::consts::FRAC_PI_2),
    );
    let wire = builder::translated(&wire, Vector3::unit_x() * 2.0);
    let face: Face = builder::try_attach_plane(vec![wire]).unwrap();
    let run = || -> Solid {
        builder::rsweep(
            black_box(&face),
            Point3::origin(),
            Vector3::unit_z(),
            Rad(std::f64::consts::TAU),
            3,
        )
    };
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = [2, 8, 32])]
fn loft(bencher: Bencher, sections: usize) {
    let wires: Vec<_> = (0..sections)
        .map(|i| {
            let t = i as f64 / (sections - 1) as f64;
            polygon(8, 1.0 + 0.2 * (t * std::f64::consts::PI).sin(), t * 5.0)
        })
        .collect();
    let run = || -> Solid { builder::try_loft(black_box(&wires)).expect("loft") };
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = [1, 8, 32])]
fn sweep_path(bencher: Bencher, segments: usize) {
    let face: Face = builder::try_attach_plane(vec![circle(Point3::origin(), 0.2)]).unwrap();
    let vertices: Vec<_> = (0..=segments)
        .map(|i| builder::vertex(Point3::new(0.0, 0.0, i as f64)))
        .collect();
    let path: Wire = vertices
        .windows(2)
        .map(|v| builder::line(&v[0], &v[1]))
        .collect();
    let run = || builder::sweep_along_wire(black_box(&face), black_box(&path), TOL).expect("sweep");
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = HOLES)]
fn transform(bencher: Bencher, holes: usize) {
    let solid = perforated_plate(holes);
    let run = || builder::translated(black_box(&solid), black_box(Vector3::new(1.0, 2.0, 3.0)));
    check_volume(&run(), plate_volume(holes));
    bencher.bench(run);
}
