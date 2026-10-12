mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

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

#[test]
fn curved_rim_trims_both_ends_of_one_cap_edge_and_one_end_face() {
    let (radius, height) = (5_f64, 10.);
    for (round, reverse) in [(0.5_f64, false), (1.2, true)] {
        let matrix = if reverse {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        };
        let input = builder::transformed(&half_cylinder(radius, height), matrix);
        common::assert_solid(&input, PI * radius.powi(2) * height / 2., &[0], TOL);
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            matrix.transform_point(Point3::new(0., radius, height)),
        );
        let wire = wire![if reverse { edge.inverse() } else { edge }];
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_along_wire(&input, &wire, round, TOL).unwrap();
        let removed = PI / 2.
            * (2. * radius * round.powi(2)
                - 5. * round.powi(3) / 3.
                - PI / 2. * (radius - round) * round.powi(2));
        let expected = PI * radius.powi(2) * height / 2. - removed;
        assert!(result.solid.is_geometric_consistent());
        assert!(!result.generated_faces.is_empty());
        let inverse = matrix.invert().unwrap();
        for face in &result.generated_faces {
            let surface = face.oriented_surface();
            let (ur, vr) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (ur.unwrap(), vr.unwrap());
            for i in 0..=8 {
                for j in 0..=16 {
                    let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                    let p = inverse.transform_point(surface.subs(u, v));
                    let rho = p.x.hypot(p.y);
                    let outward = Vector3::new(
                        (rho - radius + round) * p.x / rho,
                        (rho - radius + round) * p.y / rho,
                        p.z - height + round,
                    );
                    assert!((outward.magnitude() - round).abs() < TOL);
                    let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(normal.dot(outward.normalize()) > 1. - 1e-5);
                }
            }
        }
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

fn parabolic_prism() -> Solid {
    let a = builder::vertex(Point3::new(0., -1., 0.5));
    let b = builder::vertex(Point3::new(0., 1., 0.5));
    let arc = Edge::new(
        &a,
        &b,
        BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![a.point(), Point3::new(0., 0., -0.5), b.point()],
        )
        .into(),
    );
    let base = builder::try_attach_plane(&[wire![arc, builder::line(&b, &a)]]).unwrap();
    builder::tsweep(&base, Vector3::new(0.2, 0., 1.))
}

#[test]
fn parabolic_cap_preserves_the_rolling_radius_beyond_its_contact_endpoints() {
    for (scale, reverse) in [(10_f64, false), (25., true)] {
        let matrix = (if reverse {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        }) * Matrix4::from_scale(scale);
        let input = builder::transformed(&parabolic_prism(), matrix);
        common::assert_solid(&input, 2. / 15. * scale.powi(3), &[0], TOL);
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            matrix.transform_point(Point3::new(0.2, 0., 1.)),
        );
        let wire = wire![if reverse { edge.inverse() } else { edge }];
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_along_wire(&input, &wire, scale, TOL).unwrap();
        assert!(result.solid.is_geometric_consistent());
        assert!(!result.generated_faces.is_empty());
        let inverse = matrix.invert().unwrap();
        for face in &result.generated_faces {
            let surface = face.oriented_surface();
            let (ur, vr) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (ur.unwrap(), vr.unwrap());
            for i in 0..=8 {
                for j in 0..=16 {
                    let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                    let p = inverse.transform_point(surface.subs(u, v));
                    let gamma = (p.x + 0.8).clamp(-1., 1.).acos();
                    let (mut low, mut high) = (-2_f64, 2_f64);
                    for _ in 0..60 {
                        let q = (low + high) / 2.;
                        let y = q * (1. - 1. / (26. + q * q).sqrt())
                            + q / (1. + q * q).sqrt() * gamma.sin();
                        if y < p.y {
                            low = q;
                        } else {
                            high = q;
                        }
                    }
                    let q = (low + high) / 2.;
                    let l = (26. + q * q).sqrt();
                    let m = (1. + q * q).sqrt();
                    let center = Point3::new(-0.8, q * (1. - 1. / l), q * q / 2. - 4. + 26. / l);
                    let normal = Vector3::new(gamma.cos(), q / m * gamma.sin(), -gamma.sin() / m);
                    assert!((p - center - normal).magnitude() * scale < TOL);
                    let actual_normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(actual_normal.dot(normal) > 1. - 1e-5);
                }
            }
        }
        // Independent rolling-envelope integral; derivation and convergence in TWO_EDGE_BLEND_CAPS.md.
        let expected = 0.132_185_649_983_887_7 * scale.powi(3);
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
