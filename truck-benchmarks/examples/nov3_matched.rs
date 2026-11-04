//! Ten retained Nov3 captures. Build through scripts/nov3_matched.py.
#[allow(dead_code, unused_imports)]
mod capture {
    include!(env!("NOV3_CAPTURE_HELPERS"));
    pub fn run() {
        use std::{hint::black_box, time::Instant};
        let args: Vec<_> = std::env::args().collect();
        let prefix = &args[1];
        let smoke = args[2] == "smoke";
        let samples: usize = if smoke { 2 } else { args[2].parse().unwrap() };
        assert!(samples > 0);
        let task: Value =
            serde_json::from_str(&std::fs::read_to_string(format!("{prefix}.input.json")).unwrap())
                .unwrap();
        let input = import(&format!("{prefix}.input.step")).unwrap();
        assert!(input.is_geometric_consistent());
        // Replay's physical matching and native contour/setback conversion are preparation.
        let fillets: Vec<_> = if task["operation"] == "fillet" {
            fillet_contour_selection(&input, &task)
                .unwrap()
                .into_iter()
                .map(|(e, r)| (e.id(), r))
                .collect()
        } else {
            vec![]
        };
        let chamfers: Vec<_> = if fillets.is_empty() {
            chamfer_distance_selection(&input, task["selection"].as_array().unwrap(), true)
                .unwrap()
                .into_iter()
                .map(|(e, d)| (e.id(), d))
                .collect()
        } else {
            vec![]
        };
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut reference = None;
        for i in 0..=samples {
            let start = (!smoke).then(Instant::now);
            let result = black_box(if fillets.is_empty() {
                try_chamfer_solid_edges_with_distances(
                    black_box(&input),
                    black_box(&chamfers),
                    &[],
                    TOL,
                )
            } else {
                try_fillet_solid_edges_with_radii(black_box(&input), black_box(&fillets), TOL)
            })
            .expect("public operation failed");
            let ms = start.map(|t| t.elapsed().as_secs_f64() * 1000.);
            assert!(result.solid.is_geometric_consistent());
            let serial = serde_json::to_string(&result.solid.compress()).unwrap();
            if let Some(ref expected) = reference {
                assert_eq!(&serial, expected);
            } else {
                reference = Some(serial);
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            if i == 0 {
                let prepared = prepare_for_step(
                    &result.solid.compress(),
                    if fillets.is_empty() {
                        CHAMFER_STEP_TOL
                    } else {
                        0.00005
                    },
                )
                .unwrap();
                let design = StepDesign::from_model(StepModel::from(&prepared));
                std::fs::write(
                    &args[3],
                    StepDisplay::new(Default::default(), design).to_string(),
                )
                .unwrap();
            }
            let mut row = json!({"engine":"truck","sample":i,"warmup":i==0,"smoke":smoke,"consistent":true,"repeatable":true,"input_unchanged":true});
            if let Some(ms) = ms {
                row["ms"] = json!(ms);
            }
            println!("{row}");
        }
    }
}
fn main() { capture::run(); }
