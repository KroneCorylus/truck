mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/spherical-pole/input.step")).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
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

fn volumes(radius: f64, round: f64) -> (f64, f64) {
    let height = radius * 3_f64.sqrt() / 2.;
    let original =
        PI / 4. * (radius * radius * height - height.powi(3) / 3. + 2. * radius.powi(3) / 3.);
    let center = (radius * radius - 2. * radius * round).sqrt();
    let area = |rho: f64| {
        (height * (rho * rho - height * height).sqrt()
            + rho * rho * (height / rho).asin()
            + PI * rho * rho / 2.)
            / 2.
    };
    let limit = (-round / (radius - round)).acos();
    let n = 4096;
    let step = limit / n as f64;
    let removed = (0..=n)
        .map(|i| {
            let angle = i as f64 * step;
            let y = round * (1. - angle.cos());
            let sphere = (radius * radius - y * y).sqrt();
            let fillet = center + round * angle.sin();
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * (area(sphere) - area(fillet)) * round * angle.sin()
        })
        .sum::<f64>()
        * step
        / 3.;
    (original, original - removed)
}

fn check_blend(scale: f64, reverse: bool) {
    let matrix = (if reverse {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    }) * Matrix4::from_scale(scale);
    let inverse = matrix.invert().unwrap();
    let input = builder::transformed(&input(), matrix);
    let point = matrix.transform_point(Point3::new(-150., 0., 0.));
    let edge = input
        .edge_iter()
        .find(|edge| edge.curve().search_parameter(point, None, 100).is_some())
        .unwrap();
    let edge = if reverse { edge.inverse() } else { edge };
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_fillet_solid_along_wire(&input, &wire![edge], 10. * scale, TOL).unwrap();
    assert!(result.solid.is_geometric_consistent());
    let center = (150_f64.powi(2) - 2. * 150. * 10.).sqrt();
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for i in 0..=8 {
            for j in 0..=16 {
                let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                let p = inverse.transform_point(surface.subs(u, v));
                let rho = p.x.hypot(p.z);
                assert!(((rho - center).hypot(p.y - 10.) - 10.).abs() * scale < TOL);
                let expected = Vector3::new(
                    p.x / rho * (rho - center),
                    p.y - 10.,
                    p.z / rho * (rho - center),
                )
                .normalize();
                let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                assert!(normal.dot(expected) > 1. - 1e-5);
            }
        }
    }
    let (original, rounded) = volumes(150. * scale, 10. * scale);
    common::assert_solid(&input, original, &[0], TOL);
    common::assert_solid(&result.solid, rounded, &[0], TOL);
    common::blend::assert_step(&result.solid, rounded, TOL);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn spherical_meridian_fillet_reaches_the_polar_end_face() { check_blend(0.02, false); }

#[test]
fn polar_contacts_survive_scale_rotation_and_reversed_selection() { check_blend(0.04, true); }

#[test]
fn original_scale_spherical_pole_fillet_preserves_step_accuracy() { check_blend(1., false); }
