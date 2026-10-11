mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;
const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/spherical-meridian/input.step")).unwrap();
    let source = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table.to_compressed_solid(source).unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(TOL / 100.);
    Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn imported_spherical_sector_tessellation_preserves_volume() {
    let radius = 150.;
    let height = radius * 3_f64.sqrt() / 2.;
    let expected = PI / 2. * (radius * radius * height - height.powi(3) / 3.);
    common::assert_solid(&input(), expected, &[0], TOL);
}
