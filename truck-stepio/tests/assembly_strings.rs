use truck_assembly::assy::*;
use truck_modeling::*;
use truck_stepio::{common::PartAttrs, out::*};
#[test]
fn assembly_labels_escape_step_strings_without_changing_geometry() {
    let edge: truck_modeling::Edge = builder::line(
        &builder::vertex(Point3::origin()),
        &builder::vertex(Point3::new(1.0, 0.0, 0.0)),
    );
    let face: Face = builder::tsweep(&edge, Vector3::unit_y());
    let solid: Solid = builder::tsweep(&face, Vector3::unit_z());
    let solid = solid.compress();
    let mut assembly = Assembly::new();
    let attrs = PartAttrs {
        id: "part's id".into(),
        name: "Krone's bracket".into(),
        description: "a user's part at C:\\parts PIEZA_STEP_ESCAPE_".into(),
    };
    let nodes = assembly.create_nodes([
        NodeEntity {
            shape: None,
            attrs: attrs.clone(),
        },
        NodeEntity {
            shape: Some(StepModel::from(&solid)),
            attrs: attrs.clone(),
        },
    ]);
    assembly.create_edge(
        nodes[0],
        nodes[1],
        EdgeEntity {
            matrix: Matrix4::from_translation(Vector3::unit_x()),
            attrs,
        },
    );
    let text =
        StepDisplay::new(StepHeaderDescriptor::default(), StepDesign::new(assembly)).to_string();
    assert!(text.contains("'Krone''s bracket'"));
    let text = text.replace("DATA;", "/* a comment with '' and \\ */DATA;");
    let table = truck_stepio::r#in::Table::try_from_step(&text).unwrap();
    assert!(table
        .product
        .values()
        .all(|product| product.name == "Krone's bracket"));
    assert!(table
        .product
        .values()
        .all(|product| product.description == "a user's part at C:\\parts PIEZA_STEP_ESCAPE_"));
    assert!(table.errors.is_empty(), "{:?}", table.errors);
    assert_eq!(table.next_assembly_usage_occurrence.len(), 1);
    assert_eq!(table.manifold_solid_brep.len(), 1);
}
