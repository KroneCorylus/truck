//! Measures imported post-root fillets within and across the rounded roof support.
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
    let wire = solid
        .face_iter()
        .find_map(|face| {
            face.boundaries().into_iter().find(|wire| {
                wire.iter().all(|edge| {
                    let curve = edge.curve();
                    let (a, b) = curve.range_tuple();
                    [a, (a + b) / 2., b].iter().all(|&t| {
                        let p = curve.subs(t);
                        (p.z - 5.).abs() < 1e-6 && ((p.x - 2.5).hypot(p.y - 2.5) - 1.).abs() < 1e-6
                    })
                })
            })
        })
        .unwrap();
    let median = |mut values: Vec<f64>| {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    let mut results = Vec::new();
    for radius in [0.2, 1.] {
        let run = || {
            let start = Instant::now();
            let result = try_fillet_solid_along_wire(&solid, &wire, radius, 0.001);
            let construction = start.elapsed().as_secs_f64() * 1000.;
            let result = result.ok();
            let consistent = result
                .as_ref()
                .is_none_or(|r| r.solid.is_geometric_consistent());
            assert!(consistent);
            let preparation = result.as_ref().map(|r| {
                let compressed = r.solid.compress();
                let start = Instant::now();
                let step = prepare_for_step(&compressed, 0.00005);
                (start.elapsed().as_secs_f64() * 1000., step.is_some())
            });
            (construction, result.is_some(), preparation)
        };
        let _ = run();
        let samples: Vec<_> = (0..5).map(|_| run()).collect();
        let preparation: Vec<_> = samples.iter().filter_map(|s| s.2.map(|p| p.0)).collect();
        results.push(serde_json::json!({
            "radius": radius,
            "construction_ms": samples.iter().map(|s| s.0).collect::<Vec<_>>(),
            "construction_median_ms": median(samples.iter().map(|s| s.0).collect()),
            "constructed": samples.iter().map(|s| s.1).collect::<Vec<_>>(),
            "step_median_ms": (!preparation.is_empty()).then(|| median(preparation)),
            "step_prepared": samples.iter().map(|s| s.2.map(|p| p.1)).collect::<Vec<_>>(),
        }));
    }
    println!(
        "{}",
        serde_json::json!({"operation_tolerance":0.001,"step_tolerance":0.00005,"results":results})
    );
}
