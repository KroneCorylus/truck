mod common;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

#[test]
fn trimmed_native_spline_edges_preserve_closed_material_and_step() {
    let table = Table::from_step(include_str!("fixtures/bounded-step-edges/C1.step")).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
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
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        let solid = builder::transformed(&solid, transform);
        common::assert_solid(&solid, 0.002952545064816, &[0], 1e-5);
        common::blend::assert_step(&solid, 0.002952545064816, 1e-5);
    }
}
