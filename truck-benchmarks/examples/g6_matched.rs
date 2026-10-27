//! Construction-only G6 benchmark. Import, selection, QA and destruction are untimed.
use std::{hint::black_box, time::Instant};
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_edges, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let samples: usize = args[2].parse().unwrap();
    let text = std::fs::read_to_string(&args[1]).unwrap();
    let table = Table::from_step(&text).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(1e-5);
    let input = Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap();
    let selected: Vec<_> = [
        Point3::new(193.75, 76.25, 100.),
        Point3::new(0., 10., 50.),
        Point3::new(50., 120., 100.),
    ]
    .map(|point| {
        input.boundaries()[0]
            .edge_iter()
            .find(|e| {
                let c = e.curve();
                let (a, b) = c.range_tuple();
                c.search_nearest_parameter(point, None, 100)
                    .is_some_and(|t| t >= a && t <= b && c.subs(t).near(&point))
            })
            .unwrap()
            .id()
    })
    .into();
    for id in &selected {
        let e = input.boundaries()[0]
            .edge_iter()
            .find(|e| e.id() == *id)
            .unwrap();
        eprintln!("selected {:?} {:?}", e.front().point(), e.back().point());
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(input.is_geometric_consistent());
    let mut reference = None;
    for i in 0..=samples {
        let start = Instant::now();
        let result = black_box(
            try_fillet_solid_edges(black_box(&input), black_box(&selected), 10., 0.001).unwrap(),
        );
        let ms = start.elapsed().as_secs_f64() * 1000.;
        assert!(result.solid.is_geometric_consistent());
        if i == samples && args.len() > 3 {
            use truck_stepio::out::*;
            let prepared = prepare_for_step(&result.solid.compress(), 0.00005).unwrap();
            let design = StepDesign::from_model(StepModel::from(&prepared));
            std::fs::write(
                &args[3],
                StepDisplay::new(Default::default(), design).to_string(),
            )
            .unwrap();
        }
        let serialized = serde_json::to_string(&result.solid.compress()).unwrap();
        if let Some(ref expected) = reference {
            assert_eq!(&serialized, expected);
        } else {
            reference = Some(serialized);
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        println!(
            "{}",
            serde_json::json!({"engine":"truck","sample":i,"warmup":i==0,"ms":ms,"consistent":true,"faces":result.solid.face_iter().count(),"repeatable":true})
        );
    }
}
