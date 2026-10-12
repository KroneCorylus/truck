//! Measures closed-rim construction; fixture import and output checks are outside timing.
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_along_wire, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn main() {
    let fixture = std::env::args().nth(1).expect("spline, sphere or cylinder");
    let input = match fixture.as_str() {
        "spline" | "sphere" => {
            let step = if fixture == "sphere" {
                include_str!("../../truck-shapeops/tests/fixtures/spherical-pole/input.step")
            } else {
                include_str!("../../truck-shapeops/tests/fixtures/spline-pipe/input.step")
            };
            let table = Table::from_step(step).unwrap();
            let (mut solid, skipped) = table
                .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
                .unwrap();
            assert!(skipped.is_empty());
            solid.split_closed_edges_and_faces(0.00001);
            Solid::extract(
                solid
                    .try_mapped(
                        |p| Some(*p),
                        |c| Curve::try_from(c).ok(),
                        |s| Surface::try_from(s).ok(),
                    )
                    .unwrap(),
            )
            .unwrap()
        }
        "cylinder" => {
            let vertex = builder::vertex(Point3::new(0., 50., 0.));
            let rim = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_x(), Rad(7.), 2);
            let face = builder::try_attach_plane(&[rim]).unwrap();
            builder::tsweep(&face, Vector3::unit_x() * 100.)
        }
        _ => panic!("spline, sphere or cylinder"),
    };
    assert!(input.is_geometric_consistent());
    let wire = if fixture == "sphere" {
        wire![input
            .edge_iter()
            .find(|edge| {
                edge.curve()
                    .search_parameter(Point3::new(-150., 0., 0.), None, 100)
                    .is_some()
            })
            .unwrap()]
    } else {
        let cap = input
            .face_iter()
            .find(|face| {
                matches!(face.surface(), Surface::Plane(_))
                    && face
                        .oriented_surface()
                        .search_parameter(Point3::origin(), None, 100)
                        .is_some()
            })
            .unwrap();
        cap.boundaries()[0].clone()
    };
    let before = serde_json::to_string(&input.compress()).unwrap();
    let mut samples = Vec::new();
    let mut last = None;
    for i in 0..6 {
        let start = Instant::now();
        let result = try_fillet_solid_along_wire(&input, &wire, 10., 0.001);
        let elapsed = start.elapsed().as_secs_f64() * 1000.;
        if i != 0 {
            samples.push(elapsed);
            println!(
                "sample_ms={elapsed:.9} status={}",
                if result.is_ok() { "ok" } else { "failed" }
            );
        }
        last = Some(result);
    }
    samples.sort_by(f64::total_cmp);
    println!("median_ms={:.9}", samples[2]);
    match last.unwrap() {
        Ok(result) => {
            assert!(result.solid.is_geometric_consistent());
            let mut mesh = result.solid.triangulation(0.001).to_polygon();
            mesh.put_together_same_attrs(TOLERANCE)
                .remove_degenerate_faces()
                .remove_unused_attrs();
            assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
            println!("closed=true volume={}", mesh.volume());
        }
        Err(error) => println!("diagnostic={error:?}"),
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
