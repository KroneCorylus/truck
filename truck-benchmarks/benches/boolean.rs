use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_modeling::*;

#[divan::bench(sample_count = 20, sample_size = 1)]
fn circle_remove(bencher: Bencher) {
    let plate = cuboid(Point3::origin(), Point3::new(60.0, 60.0, 10.0));
    let disc: Face =
        builder::try_attach_plane(vec![circle(Point3::new(30.0, 30.0, 5.0), 33.0)]).unwrap();
    let cutter: Solid = builder::tsweep(&disc, Vector3::unit_z() * 5.0);
    let run = || {
        truck_shapeops::try_subtract(black_box(&plate), black_box(&cutter), TOL).expect("pocket")
    };
    let segment =
        33.0_f64.powi(2) * (30.0_f64 / 33.0).acos() - 30.0 * (33.0_f64.powi(2) - 900.0).sqrt();
    check_volume(
        &run(),
        36000.0 - 5.0 * (std::f64::consts::PI * 33.0_f64.powi(2) - 4.0 * segment),
    );
    bencher.bench(run);
}

#[divan::bench(args = [1, 10, 30, 100], sample_count = 20, sample_size = 1, max_time = 10)]
fn subtract_batch(bencher: Bencher, holes: usize) {
    let (plate, cutters) = plate_and_cutters(holes);
    let cutters = Solid::new(
        cutters
            .iter()
            .flat_map(|s| s.boundaries().iter().cloned())
            .collect(),
    );
    let run = || {
        truck_shapeops::try_subtract(black_box(&plate), black_box(&cutters), TOL).expect("subtract")
    };
    let result = run();
    check_volume(&result, plate_volume(holes));
    assert_eq!(result.face_iter().count(), 6 + 2 * holes);
    bencher.bench(run);
}

#[divan::bench(args = [1, 10, 30, 100], sample_count = 20, sample_size = 1, max_time = 10)]
fn subtract_sequential(bencher: Bencher, holes: usize) {
    let (plate, cutters) = plate_and_cutters(holes);
    let run = || {
        cutters
            .iter()
            .fold(black_box(&plate).clone(), |solid, cutter| {
                truck_shapeops::try_subtract(&solid, black_box(cutter), TOL).expect("subtract")
            })
    };
    check_volume(&run(), plate_volume(holes));
    bencher.bench(run);
}

#[divan::bench(args = [0.1, 0.01, 0.001], sample_count = 20, sample_size = 1, max_time = 10)]
fn union_cylinders(bencher: Bencher, tol: f64) {
    let a = cylinder();
    let b = builder::translated(&a, Vector3::new(0.8, 0.0, 0.5));
    let run = || truck_shapeops::try_or(black_box(&a), black_box(&b), tol).expect("union");
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = [0.1, 0.01, 0.001], sample_count = 20, sample_size = 1, max_time = 10)]
fn intersect_cylinders(bencher: Bencher, tol: f64) {
    let a = cylinder();
    let b = builder::translated(&a, Vector3::new(0.8, 0.0, 0.5));
    let run = || truck_shapeops::try_and(black_box(&a), black_box(&b), tol).expect("intersection");
    check_solid(&run());
    bencher.bench(run);
}

#[divan::bench(args = [0.1, 0.01, 0.001], sample_count = 20, sample_size = 1, max_time = 10)]
fn intersect_nurbs_cylinders(bencher: Bencher, tol: f64) {
    let a = cylinder().mapped(
        |p| *p,
        |c| Curve::NurbsCurve(NurbsCurve::new(c.lift_up())),
        |s| match s {
            Surface::Plane(_) => s.clone(),
            Surface::Extruded(surface) => {
                let base = surface.entity_curve();
                let top = base.transformed(Matrix4::from_translation(surface.extruding_vector()));
                Surface::NurbsSurface(NurbsSurface::new(BSplineSurface::homotopy(
                    base.lift_up(),
                    top.lift_up(),
                )))
            }
            _ => unreachable!("cylinder fixture uses planes and extrusions"),
        },
    );
    let b = builder::translated(&a, Vector3::new(0.8, 0.0, 0.5));
    let run = || truck_shapeops::try_and(black_box(&a), black_box(&b), tol).expect("intersection");
    check_solid(&run());
    bencher.bench(run);
}
