mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

fn sector(case: &str) -> Solid {
    let text = match case {
        "E5" => include_str!("fixtures/toroidal-sectors/E5.step"),
        "E9" => include_str!("fixtures/toroidal-sectors/E9.step"),
        "J3" => include_str!("fixtures/toroidal-sectors/J3.step"),
        "J4" => include_str!("fixtures/toroidal-sectors/J4.step"),
        "J5" => include_str!("fixtures/toroidal-sectors/J5.step"),
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

fn volume(major: f64, minor: f64, meridian: f64, rotation: f64) -> f64 {
    let s = meridian.sin();
    rotation / 2.
        * (minor * major * major * s
            + major * minor * minor * (meridian + (2. * meridian).sin() / 2.)
            + minor.powi(3) * (s - s.powi(3) / 3.))
}

fn verify(case: &str, scale: f64, transformed: bool) {
    let (major, minor, angle, rotation) = match case {
        "E5" => (100., 100., PI / 4., PI / 2.),
        "E9" => (200., 50., 8. * PI / 9., PI / 2.),
        "J3" => (200., 50., 2. * PI, 1.5 * PI),
        "J4" => (200., 50., PI / 2., 2. * PI),
        "J5" => (200., 50., 5. * PI / 6., 2. * PI),
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
    let expected = volume(major * scale, minor * scale, angle, rotation);
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

#[test]
fn periodic_torus_import_preserves_transformed_open_ring_material() { verify("J3", 0.01, true); }
#[test]
fn periodic_torus_import_preserves_original_scale_open_ring_material() { verify("J3", 1., false); }
#[test]
fn periodic_torus_import_preserves_complete_major_revolution() { verify("J4", 0.01, false); }
#[test]
fn periodic_torus_import_preserves_reentrant_meridian_material() { verify("J5", 0.01, true); }

fn verify_meridian_fillet(scale: f64, transformed: bool, reversed: bool) {
    use truck_shapeops::fillet::try_fillet_solid_along_wire;
    let matrix = if transformed {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = matrix.invert().unwrap();
    let solid = builder::transformed(&sector("E9"), matrix);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let point = matrix.transform_point(Point3::new(200., 0., 50.));
    let edge = common::blend::edge_through(&solid.boundaries()[0], point);
    let edge = if reversed { edge.inverse() } else { edge };
    let result = try_fillet_solid_along_wire(&solid, &wire![edge], 10. * scale, 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    assert!(result.solid.is_geometric_consistent());
    assert_eq!(result.generated_faces.len(), 1);
    let surface = result.generated_faces[0].oriented_surface();
    let (u, v) = surface.try_range_tuple();
    let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
    for i in 0..=16 {
        for j in 0..=64 {
            let u = a + (b - a) * i as f64 / 16.;
            let v = c + (d - c) * j as f64 / 64.;
            let p = inverse.transform_point(surface.subs(u, v));
            let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
            let center = p - 10. * normal;
            assert!((center.y - 10.).abs() * scale < 0.002, "{center:?}");
            assert!(
                ((center.x.hypot(center.y) - 200.).hypot(center.z) - 40.).abs() * scale < 0.002,
                "{center:?}"
            );
        }
    }
    let cap_height = 50. * (8. * PI / 9.).sin();
    let extended_x = ((200. - (40_f64.powi(2) - cap_height.powi(2)).sqrt()).powi(2) - 100.).sqrt();
    assert!(result.solid.vertex_iter().any(|vertex| {
        inverse
            .transform_point(vertex.point())
            .distance(Point3::new(extended_x, 0., cap_height))
            * scale
            < 0.001
    }));
    let expected = 1_537_066.457_433_423_7 * scale.powi(3);
    common::assert_solid(&result.solid, expected, &[0], 0.001);
    common::blend::assert_step(&result.solid, expected, 0.001);
}

#[test]
fn torus_meridian_fillet_extends_its_reentrant_straight_end_boundary() {
    for reversed in [false, true] {
        for transformed in [false, true] {
            verify_meridian_fillet(0.01, transformed, reversed);
        }
    }
}

#[test]
fn original_scale_torus_meridian_fillet_preserves_radius_material_and_step() {
    verify_meridian_fillet(1., false, false);
}
