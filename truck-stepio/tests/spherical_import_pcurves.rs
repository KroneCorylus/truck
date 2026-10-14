use std::f64::consts::{FRAC_PI_2, PI, TAU};
use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

fn source(surface: SphericalSurface, line: Line<Point2>) -> Curve3D {
    Curve3D::PCurve(PCurve::new(
        Box::new(Curve2D::Line(line)),
        Box::new(Surface::ElementarySurface(ElementarySurface::Sphere(
            surface,
        ))),
    ))
}

#[test]
fn spherical_derivative_orders_follow_longitude_and_latitude() {
    let center = Point3::new(1., -2., 3.);
    let radius = 20.;
    let sphere = Sphere(truck_geometry::prelude::Sphere::new(center, radius));
    for (u, v) in [(0.3, -0.7), (2.3, 0.4), (TAU, FRAC_PI_2)] {
        for m in 0..=4 {
            for n in 0..=4 {
                let longitude = u + m as f64 * FRAC_PI_2;
                let latitude = v + n as f64 * FRAC_PI_2;
                let mut expected = radius
                    * Vector3::new(
                        longitude.cos() * latitude.cos(),
                        longitude.sin() * latitude.cos(),
                        if m == 0 { latitude.sin() } else { 0. },
                    );
                if (m, n) == (0, 0) {
                    expected += center.to_vec();
                }
                assert!(
                    sphere.der_mn(m, n, u, v).distance(expected) < 1e-10,
                    "derivative ({m}, {n}) at ({u}, {v})"
                );
            }
        }
    }
}

#[test]
fn spherical_pcurves_preserve_placement_orientation_and_derivatives() {
    let affine = Matrix4::from_cols(
        Vector4::new(1.5, 0.2, -0.3, 0.),
        Vector4::new(-0.5, 0.7, 0.9, 0.),
        Vector4::new(0.1, -0.2, 1.3, 0.),
        Vector4::new(13., -7., 23., 1.),
    );
    for transform in [
        Matrix4::identity(),
        affine,
        affine * Matrix4::from_nonuniform_scale(-1., 1., 1.),
    ] {
        for flipped in [false, true] {
            let mut surface = Processor::with_transform(
                Sphere(truck_geometry::prelude::Sphere::new(
                    Point3::new(1., -2., 3.),
                    20.,
                )),
                transform,
            );
            if flipped {
                surface.invert();
            }
            for mut line in [
                Line(Point2::new(PI, 0.), Point2::new(3. * PI / 4., PI / 4.)),
                Line(Point2::new(TAU - 0.3, -0.7), Point2::new(TAU + 0.8, 0.4)),
                Line(Point2::new(0.3, -FRAC_PI_2), Point2::new(2., FRAC_PI_2)),
                Line(Point2::new(0.3, 0.7), Point2::new(2., 0.7)),
                Line(Point2::new(0.3, -0.7), Point2::new(0.3, 0.7)),
            ] {
                if flipped {
                    line = Line(
                        Point2::new(line.0.y, line.0.x),
                        Point2::new(line.1.y, line.1.x),
                    );
                }
                for reversed in [false, true] {
                    let mut input = source(surface, line);
                    if reversed {
                        input.invert();
                    }
                    let mapped = Curve::try_from(&input).expect("exact spherical pcurve");
                    assert!(matches!(mapped, Curve::PCurve(_)));
                    assert_eq!(mapped.range_tuple(), input.range_tuple());
                    for i in 0..=128 {
                        let t = i as f64 / 128.;
                        assert!(mapped.subs(t).distance(input.subs(t)) < 1e-10);
                        assert!(mapped.der(t).distance(input.der(t)) < 1e-9);
                        assert!(mapped.der2(t).distance(input.der2(t)) < 1e-8);
                    }
                    let mut cut = mapped.clone();
                    let back = cut.cut(0.37);
                    for curve in [cut, back] {
                        let (a, b) = curve.range_tuple();
                        for i in 0..=8 {
                            let t = a + (b - a) * i as f64 / 8.;
                            assert!(curve.subs(t).distance(input.subs(t)) < 1e-10);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn spherical_pcurves_reject_invalid_geometry() {
    let sphere = Processor::new(Sphere(truck_geometry::prelude::Sphere::new(
        Point3::origin(),
        20.,
    )));
    for line in [
        Line(Point2::new(0.5, 0.7), Point2::new(0.5, 0.7)),
        Line(Point2::new(f64::NAN, 0.), Point2::new(1., 1.)),
        Line(Point2::new(0., 0.), Point2::new(1., f64::INFINITY)),
    ] {
        assert!(Curve::try_from(&source(sphere, line)).is_err());
    }
    let line = Line(Point2::new(0.1, 0.2), Point2::new(0.9, 1.2));
    for transform in [Matrix4::from_scale(0.), Matrix4::from_scale(f64::NAN)] {
        assert!(Curve::try_from(&source(sphere.transformed(transform), line)).is_err());
    }
    let collapsed = Processor::new(Sphere(truck_geometry::prelude::Sphere::new(
        Point3::origin(),
        0.,
    )));
    assert!(Curve::try_from(&source(collapsed, line)).is_err());
}
