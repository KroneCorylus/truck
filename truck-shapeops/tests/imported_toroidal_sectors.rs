mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

fn sector(case: &str) -> Solid {
    let text = match case {
        "E5" => include_str!("fixtures/toroidal-sectors/E5.step"),
        "E9" => include_str!("fixtures/toroidal-sectors/E9.step"),
        _ => unreachable!(),
    };
    let table = Table::from_step(text).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(0.00001);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn volume(major: f64, minor: f64, meridian: f64) -> f64 {
    let s = meridian.sin();
    PI / 4.
        * (minor * major * major * s
            + major * minor * minor * (meridian + (2. * meridian).sin() / 2.)
            + minor.powi(3) * (s - s.powi(3) / 3.))
}

fn verify(case: &str, scale: f64, transformed: bool) {
    let (major, minor, angle) = match case {
        "E5" => (100., 100., PI / 4.),
        "E9" => (200., 50., 8. * PI / 9.),
        _ => unreachable!(),
    };
    let transform = if transformed {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let solid = builder::transformed(&sector(case), transform);
    assert!(
        solid.is_geometric_consistent(),
        "toroidal boundaries lie on the complete revolved meridian"
    );
    let expected = volume(major * scale, minor * scale, angle);
    common::assert_solid(&solid, expected, &[0], 0.001);
    common::blend::assert_step(&solid, expected, 0.001);
}

#[test]
fn horn_torus_sector_keeps_its_curved_boundary_and_material() { verify("E5", 0.01, false); }
#[test]
fn transformed_ring_torus_sector_keeps_its_curved_boundary_and_material() {
    verify("E9", 0.01, true);
}
#[test]
fn original_scale_ring_torus_sector_is_consistent_closed_and_exportable() {
    verify("E9", 1., false);
}
