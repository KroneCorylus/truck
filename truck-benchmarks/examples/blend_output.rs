//! Measures tessellation or STEP preparation of an already constructed blend.
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_stepio::out::prepare_for_step;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let solid: Solid = serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let mode = &args[2];
    let tolerance: f64 = args[3].parse().unwrap();
    let scale: f64 = args.get(4).map_or(1., |s| s.parse().unwrap());
    let solid = if scale == 1. {
        solid
    } else {
        builder::transformed(
            &solid,
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale),
        )
    };
    assert!(solid.is_geometric_consistent());
    let compressed = solid.compress();
    let run = || {
        let start = Instant::now();
        match mode.as_str() {
            "mesh" => {
                let mut mesh = solid.triangulation(tolerance).to_polygon();
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                mesh.put_together_same_attrs(TOLERANCE)
                    .remove_degenerate_faces()
                    .remove_unused_attrs();
                (
                    elapsed,
                    serde_json::json!({
                        "closed": mesh.shell_condition() == ShellCondition::Closed,
                        "vertices": mesh.positions().len(),
                        "triangles": mesh.tri_faces().len(),
                        "volume": mesh.volume(),
                    }),
                )
            }
            "step" => {
                let prepared = prepare_for_step(&compressed, tolerance);
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                let control_points: usize = prepared.as_ref().map_or(0, |solid| {
                    solid
                        .boundaries
                        .iter()
                        .flat_map(|shell| &shell.edges)
                        .map(|edge| match &edge.curve {
                            Curve::BSplineCurve(curve) => curve.control_points().len(),
                            Curve::NurbsCurve(curve) => curve.control_points().len(),
                            _ => 0,
                        })
                        .sum()
                });
                (
                    elapsed,
                    serde_json::json!({
                        "prepared": prepared.is_some(), "spline_control_points": control_points,
                    }),
                )
            }
            _ => panic!("expected mesh or step"),
        }
    };
    drop(run());
    let samples: Vec<_> = (0..5).map(|_| run()).collect();
    let mut sorted: Vec<_> = samples.iter().map(|sample| sample.0).collect();
    sorted.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::json!({
            "mode": mode, "tolerance": tolerance, "scale": scale,
            "milliseconds": samples.iter().map(|sample| sample.0).collect::<Vec<_>>(),
            "median_ms": sorted[2],
            "checks": samples.iter().map(|sample| &sample.1).collect::<Vec<_>>(),
        })
    );
}
