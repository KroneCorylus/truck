//! Measures a STEP revolution fixture before/after division changes, retaining mesh accuracy.
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

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
    assert!(solid.is_geometric_consistent());
    let run = || solid.triangulation(0.001).to_polygon();
    drop(run());
    let mut milliseconds = Vec::new();
    let mut output = None;
    for _ in 0..5 {
        let start = Instant::now();
        let mesh = run();
        milliseconds.push(start.elapsed().as_secs_f64() * 1000.);
        output = Some(mesh);
    }
    let mut mesh = output.unwrap();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    let mut sorted = milliseconds.clone();
    sorted.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::json!({
            "tolerance":0.001,"milliseconds":milliseconds,"median_ms":sorted[2],
            "vertices":mesh.positions().len(),"volume":mesh.volume(),
            "mesh_closed":mesh.shell_condition()==ShellCondition::Closed,
        })
    );
}
