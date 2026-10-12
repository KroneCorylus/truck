//! Measures toroidal STEP import/preprocessing; file reading and output checks are outside timing.
use std::f64::consts::PI;
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

fn import(text: &str) -> std::result::Result<Solid, String> {
    let table = Table::from_step(text).ok_or("STEP parse failed")?;
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .map_err(|e| format!("{e:?}"))?;
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(0.00001);
    let solid = raw
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .ok_or("Unsupported imported geometry")?;
    Solid::extract(solid).map_err(|e| format!("{e:?}"))
}

fn main() {
    let case = std::env::args().nth(1).expect("E5, E9, J3, J4 or J5");
    let (text, major, minor, meridian, angle) = match case.as_str() {
        "E5" => (
            include_str!("../../truck-shapeops/tests/fixtures/toroidal-sectors/E5.step"),
            100.,
            100.,
            PI / 4.,
            PI / 2.,
        ),
        "E9" => (
            include_str!("../../truck-shapeops/tests/fixtures/toroidal-sectors/E9.step"),
            200.,
            50.,
            8. * PI / 9.,
            PI / 2.,
        ),
        "J3" => (
            include_str!("../../truck-shapeops/tests/fixtures/toroidal-sectors/J3.step"),
            200.,
            50.,
            2. * PI,
            1.5 * PI,
        ),
        "J4" => (
            include_str!("../../truck-shapeops/tests/fixtures/toroidal-sectors/J4.step"),
            200.,
            50.,
            PI / 2.,
            2. * PI,
        ),
        "J5" => (
            include_str!("../../truck-shapeops/tests/fixtures/toroidal-sectors/J5.step"),
            200.,
            50.,
            5. * PI / 6.,
            2. * PI,
        ),
        _ => panic!("E5, E9, J3, J4 or J5"),
    };
    let mut samples = Vec::new();
    let mut last = None;
    for i in 0..6 {
        let start = Instant::now();
        let result = import(text);
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
        Ok(solid) => {
            assert!(solid.is_geometric_consistent());
            let solid = builder::transformed(&solid, Matrix4::from_scale(0.01));
            let mut mesh = solid.triangulation(0.0001).to_polygon();
            mesh.put_together_same_attrs(TOLERANCE)
                .remove_degenerate_faces()
                .remove_unused_attrs();
            assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
            let sine = meridian.sin();
            let expected = angle / 2.
                * (minor * major * major * sine
                    + major * minor * minor * (meridian + (2. * meridian).sin() / 2.)
                    + minor.powi(3) * (sine - sine.powi(3) / 3.))
                * 1e-6;
            assert!((mesh.volume() - expected).abs() < expected * 0.002);
            println!(
                "closed=true scaled_volume={} expected={expected}",
                mesh.volume()
            );
        }
        Err(error) => println!("diagnostic={error}"),
    }
}
