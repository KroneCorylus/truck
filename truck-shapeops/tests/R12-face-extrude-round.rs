//! R12: extending a planar face with exact circular boundaries must join its source body.
mod common;
use std::f64::consts::PI;
use truck_modeling::*;

fn operands() -> [Solid; 2] {
    let compressed: [CompressedSolid; 2] =
        serde_json::from_str(include_str!("R12-face-extrude-round-operands.json")).unwrap();
    compressed.map(|s| Solid::extract(s).unwrap())
}

fn assert_result(solid: &Solid, volume: f64, genus: usize) {
    common::assert_solid(solid, volume, &[genus], 0.001);
    assert_eq!(
        truck_shapeops::solid_components(solid, 0.01).unwrap().len(),
        1
    );
}

fn reflect(solid: &Solid) -> Solid {
    let mut solid = builder::scaled(solid, Point3::origin(), Vector3::new(1.0, 1.0, -1.0));
    solid.not();
    solid
}

fn check_extension(input: [Solid; 2], volume: f64, genus: usize) {
    for reflected in [false, true] {
        let solids = if reflected {
            input.each_ref().map(reflect)
        } else {
            input.clone()
        };
        let before = serde_json::to_string(&solids.each_ref().map(Solid::compress)).unwrap();
        let curves: Vec<_> = solids
            .iter()
            .flat_map(|s| s.edge_iter())
            .map(|e| serde_json::to_string(&e.curve()).unwrap())
            .collect();
        for [a, b] in [[0, 1], [1, 0]] {
            let result = truck_shapeops::try_or(&solids[a], &solids[b], 0.01).unwrap();
            assert_result(&result, volume, genus);
            for edge in result.edge_iter() {
                assert!(
                    curves.contains(&serde_json::to_string(&edge.curve()).unwrap()),
                    "union must retain the exact input curves"
                );
            }
        }
        assert_eq!(
            before,
            serde_json::to_string(&solids.each_ref().map(Solid::compress)).unwrap()
        );
    }
}

#[test]
fn annular_face_extension() { check_extension(operands(), 210.0 * PI, 1); }

fn top_face(solid: &Solid) -> Face {
    solid.face_iter().find(|face| {
        matches!(face.surface(), Surface::Plane(p) if (p.origin().z - 5.0).abs() < 1.0e-8)
    }).unwrap().clone()
}

#[test]
fn disk_face_extension() {
    let disk =
        builder::try_attach_plane(vec![top_face(&operands()[0]).boundaries()[0].clone()]).unwrap();
    let base = builder::translated(&disk, -5.0 * Vector3::unit_z());
    check_extension(
        [
            builder::tsweep(&base, 5.0 * Vector3::unit_z()),
            builder::tsweep(&disk, 5.0 * Vector3::unit_z()),
        ],
        250.0 * PI,
        0,
    );
}

#[test]
fn planar_face_with_two_circular_holes() {
    let vertices = [(-8.0, -4.0), (8.0, -4.0), (8.0, 4.0), (-8.0, 4.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 5.0)));
    let outer: Wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let hole = top_face(&operands()[0]).boundaries()[1].clone();
    let face = builder::try_attach_plane(vec![
        outer,
        builder::translated(&hole, Vector3::new(-3.0, 0.0, 0.0)),
        builder::translated(&hole, Vector3::new(3.0, 0.0, 0.0)),
    ])
    .unwrap();
    let base = builder::translated(&face, -5.0 * Vector3::unit_z());
    check_extension(
        [
            builder::tsweep(&base, 5.0 * Vector3::unit_z()),
            builder::tsweep(&face, 5.0 * Vector3::unit_z()),
        ],
        1280.0 - 80.0 * PI,
        2,
    );
}

#[test]
fn subsequent_booleans() {
    let [base, tool] = operands();
    let extended = truck_shapeops::try_or(&base, &tool, 0.01).unwrap();
    let cut = truck_shapeops::try_subtract(&extended, &tool, 0.01).unwrap();
    assert_result(&cut, 105.0 * PI, 1);
    let overlap = truck_shapeops::try_and(&extended, &tool, 0.01).unwrap();
    assert_result(&overlap, 105.0 * PI, 1);
    let rejoined = truck_shapeops::try_or(&cut, &overlap, 0.01).unwrap();
    assert_result(&rejoined, 210.0 * PI, 1);
}

#[cfg(feature = "step-test")]
#[test]
fn step_round_trip() {
    use truck_stepio::{out::*, r#in::Table};
    let [base, tool] = operands();
    let extended = truck_shapeops::try_or(&base, &tool, 0.01).unwrap();
    let prepared = prepare_for_step(&extended.compress(), 0.0005).unwrap();
    let design = StepDesign::from_model(StepModel::from(&prepared));
    let text = StepDisplay::new(Default::default(), design).to_string();
    let table = Table::from_step(&text).unwrap();
    assert_eq!(table.manifold_solid_brep.len(), 1);
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (read, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    let read = read
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap();
    assert_result(&Solid::extract(read).unwrap(), 210.0 * PI, 1);
}
