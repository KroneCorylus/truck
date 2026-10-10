use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_stepio::{out::*, r#in::Table};

#[divan::bench(args = [false, true], sample_count = 5, sample_size = 1)]
fn prepare_fillet(bencher: Bencher, nurbs: bool) {
    let disc = builder::try_attach_plane(&[circle(Point3::origin(), 5.)]).unwrap();
    let mut solid = builder::tsweep(&disc, Vector3::unit_z() * 8.);
    if nurbs {
        solid = solid.mapped(
            |point| *point,
            |curve| Curve::NurbsCurve(NurbsCurve::try_from(curve).unwrap()),
            |surface| match surface {
                Surface::Extruded(surface) => {
                    let bottom = NurbsCurve::try_from(surface.entity_curve()).unwrap();
                    let top =
                        bottom.transformed(Matrix4::from_translation(surface.extruding_vector()));
                    NurbsSurface::new(BSplineSurface::homotopy(
                        bottom.into_non_rationalized(),
                        top.into_non_rationalized(),
                    ))
                    .into()
                }
                other => other.clone(),
            },
        );
    }
    let rim = solid
        .face_iter()
        .find(|face| face.vertex_iter().all(|vertex| vertex.point().z.near(&8.)))
        .unwrap()
        .boundaries()[0]
        .clone();
    let result =
        truck_shapeops::fillet::try_fillet_solid_along_wire(&solid, &rim, 0.5, 0.001).unwrap();
    let compressed = result.solid.compress();
    let run = || prepare_for_step(black_box(&compressed), 0.00005).expect("fillet STEP fit");
    let area = 0.25 * (1. - std::f64::consts::PI / 4.);
    let centroid = 0.5 * (10. - 3. * std::f64::consts::PI) / (12. - 3. * std::f64::consts::PI);
    let volume = 200. * std::f64::consts::PI - 2. * std::f64::consts::PI * (5. - centroid) * area;
    check_volume(&result.solid, volume);
    let prepared = Solid::extract(run()).unwrap();
    let meshed = prepared.robust_triangulation(0.001);
    assert!(meshed.face_iter().all(|face| face.surface().is_some()));
    let mut mesh = meshed.to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(
        mesh.shell_condition(),
        truck_modeling::shell::ShellCondition::Closed
    );
    let area: f64 = mesh
        .faces()
        .triangle_iter()
        .map(|tri| {
            let [p, q, r] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos]);
            (q - p).cross(r - p).magnitude() / 2.
        })
        .sum();
    assert!((mesh.volume() - volume).abs() <= area * 0.0011);
    bencher.bench(run);
}

fn fixture(holes: usize) -> String {
    let solid = perforated_plate(holes).compress();
    let design = StepDesign::from_model(StepModel::from(&solid));
    StepDisplay::new(Default::default(), design).to_string()
}

#[divan::bench(args = HOLES)]
fn export(bencher: Bencher, holes: usize) {
    let solid = perforated_plate(holes);
    let run = || {
        let compressed = black_box(&solid).compress();
        let design = StepDesign::from_model(StepModel::from(&compressed));
        StepDisplay::new(Default::default(), design).to_string()
    };
    let table = Table::try_from_step(&run()).expect("parse exported STEP");
    let entity = table
        .manifold_solid_brep
        .values()
        .next()
        .expect("exported solid");
    let (converted, skipped) = table
        .to_compressed_solid(entity)
        .expect("convert exported solid");
    assert!(skipped.is_empty());
    assert_eq!(converted.boundaries[0].faces.len(), 6 + 2 * holes);
    bencher.bench(run);
}

#[divan::bench(args = HOLES)]
fn parse(bencher: Bencher, holes: usize) {
    let text = fixture(holes);
    let run = || Table::try_from_step(black_box(&text)).expect("parse STEP");
    assert_eq!(run().manifold_solid_brep.len(), 1);
    bencher.bench(run);
}

#[divan::bench(args = HOLES)]
fn convert(bencher: Bencher, holes: usize) {
    let table = Table::try_from_step(&fixture(holes)).unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let run = || {
        black_box(&table)
            .to_compressed_solid(black_box(entity))
            .expect("convert STEP")
    };
    let (solid, skipped) = run();
    assert!(skipped.is_empty());
    assert_eq!(solid.boundaries[0].faces.len(), 6 + 2 * holes);
    bencher.bench(run);
}

#[divan::bench(args = HOLES)]
fn import(bencher: Bencher, holes: usize) {
    let text = fixture(holes);
    let run = || {
        let table = Table::try_from_step(black_box(&text)).expect("parse STEP");
        let entity = table
            .manifold_solid_brep
            .values()
            .next()
            .expect("STEP solid");
        table.to_compressed_solid(entity).expect("convert STEP")
    };
    let (solid, skipped) = run();
    assert!(skipped.is_empty());
    assert_eq!(solid.boundaries[0].faces.len(), 6 + 2 * holes);
    bencher.bench(run);
}

fn comparison_path(name: &str) -> std::path::PathBuf {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/comparison");
    std::fs::create_dir_all(&directory).unwrap();
    directory.join(name)
}

#[divan::bench(args = HOLES)]
fn export_file(bencher: Bencher, holes: usize) {
    let solid = perforated_plate(holes);
    let path = comparison_path(&format!("truck-output-{holes}.step"));
    let run = || {
        let compressed = black_box(&solid).compress();
        let design = StepDesign::from_model(StepModel::from(&compressed));
        let text = StepDisplay::new(Default::default(), design).to_string();
        std::fs::write(&path, text).unwrap();
    };
    run();
    assert!(Table::try_from_step(&std::fs::read_to_string(&path).unwrap()).is_ok());
    bencher.bench(run);
}

#[divan::bench(args = HOLES)]
fn import_file(bencher: Bencher, holes: usize) {
    let path = comparison_path(&format!("shared-{holes}.step"));
    std::fs::write(&path, fixture(holes)).unwrap();
    let run = || {
        let text = std::fs::read_to_string(&path).unwrap();
        let table = Table::try_from_step(&text).expect("parse STEP");
        let entity = table.manifold_solid_brep.values().next().unwrap();
        table.to_compressed_solid(entity).expect("convert STEP")
    };
    let (solid, skipped) = run();
    assert!(skipped.is_empty());
    assert_eq!(solid.boundaries[0].faces.len(), 6 + 2 * holes);
    bencher.bench(run);
}
