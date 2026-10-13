mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_along_wire, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/polar-excursions/J2.step")).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(1e-5);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .expect("isolated polar excursion leaves a simple cap boundary")
}

fn height() -> f64 { 50. / 2_f64.sqrt() }
fn input_volume() -> f64 {
    PI * (2500. * height() - height().powi(3) / 3. + 2. * 50_f64.powi(3) / 3.)
}
fn center() -> (f64, f64) {
    let z = height() - 10.;
    ((1600. - z * z).sqrt(), z)
}
fn fillet_volume() -> f64 {
    let (c, z) = center();
    let contact = 1.25 * z;
    let primitive = |w: f64| {
        (c * c + 100.) * w - w.powi(3) / 3.
            + c * (w * (100. - w * w).max(0.).sqrt() + 100. * (w / 10.).asin())
    };
    PI * (2500. * contact - contact.powi(3) / 3. + 2. * 50_f64.powi(3) / 3. + primitive(10.)
        - primitive(contact - z))
}
fn transform(scale: f64, moved: bool) -> Matrix4 {
    (if moved {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    }) * Matrix4::from_scale(scale)
}

#[test]
fn polar_excursion_import_preserves_spherical_material_and_step() {
    for (scale, moved) in [(0.01, false), (0.01, true), (1., false)] {
        let solid = builder::transformed(&input(), transform(scale, moved));
        assert!(solid.is_geometric_consistent());
        common::assert_counts(&solid, 2, 2, 2);
        common::assert_solid(&solid, input_volume() * scale.powi(3), &[0], 0.001);
        common::blend::assert_step(&solid, input_volume() * scale.powi(3), 0.001);
    }
}

#[test]
fn spherical_cap_rim_preserves_radius_contacts_material_and_step() {
    for (scale, reversed) in [(0.01, false), (0.01, true), (1., false)] {
        let matrix = transform(scale, reversed);
        let inverse = matrix.invert().unwrap();
        let solid = builder::transformed(&input(), matrix);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let mut wire = solid
            .face_iter()
            .find(|face| matches!(face.surface(), Surface::Plane(_)))
            .unwrap()
            .boundaries()[0]
            .clone();
        if reversed {
            wire.invert();
        }
        let result = try_fillet_solid_along_wire(&solid, &wire, 10. * scale, 1e-7).unwrap();
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), 2);
        let (c, z) = center();
        let (mut cap_contacts, mut sphere_contacts) = (0, 0);
        for face in &result.generated_faces {
            let surface = face.oriented_surface();
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (d, e)) = (u.unwrap(), v.unwrap());
            for i in 0..=16 {
                for j in 0..=16 {
                    let (u, v) = (a + (b - a) * i as f64 / 16., d + (e - d) * j as f64 / 16.);
                    let p = inverse.transform_point(surface.subs(u, v));
                    let rho = p.x.hypot(p.y);
                    let radial =
                        Vector3::new((rho - c) * p.x / rho, (rho - c) * p.y / rho, p.z - z);
                    assert!((radial.magnitude() - 10.).abs() * scale < 1e-6);
                    let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(normal.dot(radial.normalize()) > 1. - 1e-8);
                    if (p.z - height()).abs() * scale < 1e-7 {
                        assert!((rho - c).abs() * scale < 1e-6);
                        assert!(normal.z > 1. - 1e-8);
                        cap_contacts += 1;
                    }
                    if (p.to_vec().magnitude() - 50.).abs() * scale < 1e-7 {
                        assert!((p.z - 1.25 * z).abs() * scale < 1e-6);
                        assert!(normal.dot(p.to_vec().normalize()) > 1. - 1e-8);
                        sphere_contacts += 1;
                    }
                }
            }
        }
        assert!(cap_contacts >= 34);
        assert!(sphere_contacts >= 34);
        common::assert_solid(&result.solid, fillet_volume() * scale.powi(3), &[0], 0.001);
        common::blend::assert_step(&result.solid, fillet_volume() * scale.powi(3), 0.001);
    }
}
