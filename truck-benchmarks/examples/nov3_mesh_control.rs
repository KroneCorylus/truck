#[path = "../benches/fixtures.rs"]
mod fixtures;
use std::{hint::black_box, time::Instant};
use truck_meshalgo::prelude::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = &args[1];
    let count: usize = args[2].parse().unwrap();
    let mut times = Vec::new();
    let signature;
    if case.starts_with("plate") || case.starts_with("cylinder") {
        let (solid, tol) = if let Some(n) = case.strip_prefix("plate") {
            let n = n.parse().unwrap();
            let solid = fixtures::perforated_plate(n);
            fixtures::check_volume(&solid, fixtures::plate_volume(n));
            (solid, fixtures::TOL)
        } else {
            (
                fixtures::cylinder(),
                case.strip_prefix("cylinder").unwrap().parse().unwrap(),
            )
        };
        let mut last = None;
        for i in 0..count + 3 {
            let start = Instant::now();
            let mesh = black_box(&solid).triangulation(black_box(tol));
            let ns = start.elapsed().as_nanos();
            if i >= 3 {
                times.push(ns);
            }
            last = Some(mesh);
        }
        let mesh = last.unwrap();
        assert!(mesh.face_iter().all(|f| f.surface().is_some()));
        let poly = mesh.to_polygon();
        signature = serde_json::to_value(&poly).unwrap();
    } else {
        let n = 4096;
        let values: Vec<_> = (0..n)
            .map(|i| {
                let x = match case.as_str() {
                    "dense" => (i / 32) as f64 * 4.0 + (i % 32) as f64 * 0.002,
                    "separated" => i as f64 * 4.0,
                    "transitive" => i as f64 * 0.75,
                    _ => panic!("unknown case"),
                };
                Vector3::new(x, 0.0, 0.0)
            })
            .collect();
        let mesh = PolygonMesh::new(
            StandardAttributes {
                positions: values.iter().map(|v| Point3::new(v.x, v.y, v.z)).collect(),
                normals: values.clone(),
                uv_coords: values.iter().map(|v| Vector2::new(v.x, v.y)).collect(),
            },
            Faces::from_iter((0..n / 3).map(|i| {
                std::array::from_fn::<_, 3, _>(|j| StandardVertex {
                    pos: 3 * i + j,
                    uv: Some(3 * i + j),
                    nor: Some(3 * i + j),
                })
            })),
        );
        let mut last = None;
        for i in 0..count + 3 {
            let mut output = mesh.clone();
            let start = Instant::now();
            black_box(&mut output).put_together_same_attrs(black_box(1.0));
            let ns = start.elapsed().as_nanos();
            if i >= 3 {
                times.push(ns);
            }
            last = Some(output);
        }
        signature = serde_json::to_value(last.unwrap()).unwrap();
    }
    println!(
        "{}",
        serde_json::json!({"case":case,"ns":times,"output":signature})
    );
}
