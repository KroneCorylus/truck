//! Measures construction and STEP preparation for the captured spherical-meridian fillet.
use std::time::Instant;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::{out::prepare_for_step, r#in::Table};

fn main() {
    let path = std::env::args().nth(1).expect("STEP input path");
    let table = Table::from_step(&std::fs::read_to_string(path).unwrap()).unwrap();
    let source = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table.to_compressed_solid(source).unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(0.00001);
    let solid = Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap();
    let edge = solid
        .edge_iter()
        .find(|edge| {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            curve
                .subs((a + b) / 2.)
                .distance(Point3::new(-150., 0., 0.))
                < 1e-6
        })
        .unwrap();
    let wire = wire![edge];
    let run = || {
        let start = Instant::now();
        let result = try_fillet_solid_along_wire(&solid, &wire, 10., 0.001).unwrap();
        let construction = start.elapsed().as_secs_f64() * 1000.;
        let compressed = result.solid.compress();
        let start = Instant::now();
        let prepared = prepare_for_step(&compressed, 0.00005);
        let preparation = start.elapsed().as_secs_f64() * 1000.;
        assert!(result.solid.is_geometric_consistent());
        let knots: Vec<_> = result
            .generated_faces
            .iter()
            .filter_map(|face| {
                if let Surface::Fillet(surface) = face.surface() {
                    Some(surface.entity().vknot_vec().len())
                } else {
                    None
                }
            })
            .collect();
        (construction, preparation, prepared.is_some(), knots)
    };
    drop(run());
    let samples: Vec<_> = (0..5).map(|_| run()).collect();
    let median = |values: Vec<f64>| {
        let mut values = values;
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    println!(
        "{}",
        serde_json::json!({
            "radius":10., "operation_tolerance":0.001, "step_tolerance":0.00005,
            "construction_ms":samples.iter().map(|sample| sample.0).collect::<Vec<_>>(),
            "construction_median_ms":median(samples.iter().map(|sample| sample.0).collect()),
            "step_ms":samples.iter().map(|sample| sample.1).collect::<Vec<_>>(),
            "step_median_ms":median(samples.iter().map(|sample| sample.1).collect()),
            "step_succeeded":samples.iter().map(|sample| sample.2).collect::<Vec<_>>(),
            "longitudinal_knot_counts":samples[0].3,
        })
    );
}
