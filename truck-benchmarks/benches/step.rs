use super::fixtures::*;
use divan::{black_box, Bencher};
use truck_stepio::{out::*, r#in::Table};

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
