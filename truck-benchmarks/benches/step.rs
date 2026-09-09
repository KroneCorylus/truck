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
