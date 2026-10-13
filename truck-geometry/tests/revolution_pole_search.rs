use std::f64::consts::FRAC_PI_3;
use truck_geometry::prelude::*;

#[test]
fn nearest_search_accepts_roundoff_just_inside_a_fixed_pole() {
    for radius in [0.2, 1., 150.] {
        let profile = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., FRAC_PI_3)),
            Matrix4::from_cols(
                Vector4::new(0., radius, 0., 0.),
                Vector4::new(0., 0., radius, 0.),
                Vector4::unit_x(),
                Vector4::unit_w(),
            ),
        );
        let profile: NurbsCurve<Vector4> = profile.to_same_geometry();
        let surface = RevolutedCurve::by_revolution(profile, Point3::origin(), Vector3::unit_y());
        let pole = surface.subs(0., 0.);
        let point = Point3::new(pole.x, f64::from_bits(pole.y.to_bits() - 1), pole.z);
        for hint in [None, Some((0., 0.73))] {
            let (u, v) = surface
                .search_nearest_parameter(point, hint, 100)
                .expect("one-ulp pole perturbation must have a nearest parameter");
            assert!(u.abs() < 1e-8);
            assert!(surface.subs(u, v).distance(pole) < 1e-8);
            assert!(surface.search_parameter(point, hint, 100).is_some());
        }
    }
}

#[test]
fn sphere_nearest_search_has_finite_parameters_at_both_poles() {
    let sphere = Sphere::new(Point3::new(2., -3., 5.), 4.);
    for z in [-1., 1.] {
        let point = sphere.center() + Vector3::unit_z() * (5. * z);
        for hint in [None, Some((0., 0.73))] {
            let (u, v) = sphere.search_nearest_parameter(point, hint, 100).unwrap();
            assert!(u.is_finite() && v.is_finite());
            assert_near!(
                sphere.subs(u, v),
                sphere.center() + Vector3::unit_z() * (4. * z)
            );
            if let Some((_, angle)) = hint {
                assert_eq!(v, angle);
            }
        }
    }
}

#[test]
fn surface_newton_preserves_an_already_converged_polar_seed() {
    let sphere = Sphere::new(Point3::origin(), 1.);
    let point = Point3::new(0., 0., f64::from_bits(1_f64.to_bits() - 1));
    let parameter = algo::surface::search_nearest_parameter(&sphere, point, (0., 0.73), 100)
        .expect("a converged seed does not require an invertible Jacobian");
    assert_near!(sphere.subs(parameter.0, parameter.1), point);
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(2., -3., 5.)) * Matrix4::from_angle_x(Rad(0.7)),
    ] {
        let transformed = Processor::with_transform(sphere, transform);
        for surface in [transformed, transformed.inverse()] {
            let point = transform.transform_point(point);
            let (u, v) = surface.search_nearest_parameter(point, None, 100).unwrap();
            assert_near!(surface.subs(u, v), point);
        }
    }
}

#[test]
fn ellipse_inverse_keeps_the_trimmed_sheet_near_a_rounded_pole() {
    use std::f64::consts::FRAC_PI_2;
    let rounded_pole = (FRAC_PI_2 * 1e12).round() / 1e12;
    for scale in [0.01, 1., 10.] {
        let curve = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., rounded_pole)),
            Matrix4::from_cols(
                Vector4::new(150. * scale, 0., 0., 0.),
                Vector4::new(0., 1.110223024625e-14 * scale, 100. * scale, 0.),
                Vector4::new(0., -100. * scale, 1.110223024625e-14 * scale, 0.),
                Vector4::unit_w(),
            ),
        );
        let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_z());
        for offset in [0.01, 0.005, 0.001, 0.0001] {
            let parameter = FRAC_PI_2 - offset;
            let point = surface.subs(parameter, 0.);
            for hint in [None, Some((FRAC_PI_2, 0.)), Some((rounded_pole, 0.))] {
                for (name, uv) in [
                    ("exact", surface.search_parameter(point, hint, 100)),
                    (
                        "nearest",
                        surface.search_nearest_parameter(point, hint, 100),
                    ),
                ] {
                    let uv = uv.unwrap_or_else(|| {
                        panic!("{name} scale={scale} offset={offset} hint={hint:?}")
                    });
                    assert!(
                        (uv.0 - parameter).abs() < 1e-8,
                        "{name} scale={scale} offset={offset} hint={hint:?} uv={uv:?}"
                    );
                    assert!(surface.subs(uv.0, uv.1).distance(point) < 1e-6);
                }
            }
        }
    }
}
