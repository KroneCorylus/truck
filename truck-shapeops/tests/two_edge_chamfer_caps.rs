mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_along_wire;

const TOL: f64 = 0.001;

fn half_cylinder(radius: f64, height: f64) -> Solid {
    let a = builder::vertex(Point3::new(radius, 0., 0.));
    let b = builder::vertex(Point3::new(-radius, 0., 0.));
    let arc = Edge::new(
        &a,
        &b,
        Curve::Conic(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0., PI)),
            Matrix4::from_scale(radius),
        )),
    );
    let base = builder::try_attach_plane(&[wire![arc, builder::line(&b, &a)]]).unwrap();
    builder::tsweep(&base, Vector3::new(0., 0., height))
}

fn check(radial: f64, vertical: f64, scale: f64, reversed: bool) {
    let (radius, height) = (5_f64, 10.);
    let transform = if reversed {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let input = builder::transformed(&half_cylinder(radius, height), transform);
    let original = PI * radius.powi(2) * height / 2.;
    common::assert_solid(&input, original * scale.powi(3), &[0], TOL);
    let cap = input
        .face_iter()
        .find(|face| {
            inverse
                .transform_vector(face.oriented_surface().normal(0., 0.))
                .normalize()
                .z
                > 0.9
        })
        .unwrap();
    assert_eq!(cap.boundaries()[0].len(), 2);
    let edge = cap
        .edge_iter()
        .find(|edge| {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            inverse.transform_point(curve.subs((a + b) / 2.)).y > radius / 2.
        })
        .unwrap();
    let (wire, d0, d1) = if reversed {
        (wire![edge.inverse()], vertical * scale, radial * scale)
    } else {
        (wire![edge], radial * scale, vertical * scale)
    };
    let before = serde_json::to_string(&input.compress()).unwrap();
    let output = try_chamfer_solid_along_wire(&input, &wire, d0, d1, TOL).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert!(output.solid.is_geometric_consistent());
    assert_eq!(output.generated_faces.len(), 1);
    assert_eq!(output.solid.face_iter().count(), 5);
    // Integrate the removed half-annulus while its radial inset grows linearly over the height.
    let removed = PI / 2. * (radius * radial * vertical - radial.powi(2) * vertical / 3.);
    let expected = (original - removed) * scale.powi(3);
    common::assert_solid(&output.solid, expected, &[0], TOL);
    common::blend::assert_step(&output.solid, expected, TOL);
    let surface = output.generated_faces[0].oriented_surface();
    let (ur, vr) = surface.try_range_tuple();
    let ((a, b), (c, d)) = (ur.unwrap(), vr.unwrap());
    for i in 1..16 {
        for j in 0..=8 {
            let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 8.);
            let p = inverse.transform_point(surface.subs(u, v));
            let rho = p.x.hypot(p.y);
            let error = rho - radius + radial + radial / vertical * (p.z - height);
            assert!(error.abs() * scale < 1e-6);
            let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
            let expected = Vector3::new(p.x / rho, p.y / rho, radial / vertical).normalize();
            assert!(normal.dot(expected) > 1. - 1e-6);
        }
    }
}

#[test]
fn equal_chamfer_closes_both_ends_on_one_terminal_face() {
    for reversed in [false, true] {
        check(0.5, 0.5, 1., reversed);
    }
}

#[test]
fn unequal_chamfer_preserves_side_distances_on_shared_caps() {
    for (radial, vertical, scale) in [(0.7, 0.3, 1.), (0.2, 0.9, 0.1)] {
        for reversed in [false, true] {
            check(radial, vertical, scale, reversed);
        }
    }
}
