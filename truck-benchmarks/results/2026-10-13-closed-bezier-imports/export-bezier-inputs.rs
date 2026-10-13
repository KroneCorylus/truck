use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::{out::*, r#in::Table};
fn main() {
    for case in ["bug30940_2", "bug30940_5"] {
        let prefix = format!("target/closed-bezier-replay/{case}/001");
        let table =
            Table::from_step(&std::fs::read_to_string(format!("{prefix}.input.step")).unwrap())
                .unwrap();
        let (mut raw, skipped) = table
            .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
            .unwrap();
        assert!(skipped.is_empty());
        raw.split_closed_edges_and_faces(1e-5);
        let solid = Solid::extract(
            raw.try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(solid.is_geometric_consistent());
        let mut mesh = solid.triangulation(0.001).to_polygon();
        mesh.put_together_same_attrs(TOLERANCE)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        println!(
            "{case} consistent=true mesh={:?} mesh_volume={}",
            mesh.shell_condition(),
            mesh.volume()
        );
        let prepared = prepare_for_step(&solid.compress(), 0.00005).unwrap();
        let step = StepDisplay::new(
            Default::default(),
            StepDesign::from_model(StepModel::from(&prepared)),
        )
        .to_string();
        std::fs::write(format!("{prefix}.imported.step"), step).unwrap();
    }
}
