//! Operation-only allocation profile; set G2_INPUT and enable the allocations feature.
use std::hint::black_box;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_edges, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() { divan::main(); }

fn load(path: &str) -> (Solid, Vec<EdgeID>) {
    let text = std::fs::read_to_string(path).unwrap();
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
        Point3::new(200., 0., 1000.),
        Point3::new(200., 0., 1500.),
        Point3::new(100., 1000., 1000.),
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
    (input, selected)
}

#[divan::bench(sample_count = 3, sample_size = 1)]
fn public_g2(bencher: divan::Bencher) {
    let (input, selected) =
        load(&std::env::var("G2_INPUT").expect("set G2_INPUT to the captured STEP"));
    let before = serde_json::to_string(&input.compress()).unwrap();
    let run =
        || try_fillet_solid_edges(black_box(&input), black_box(&selected), 10., 0.001).unwrap();
    assert!(run().solid.is_geometric_consistent());
    bencher.bench(run);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
