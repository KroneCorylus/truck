use std::f64::consts::{PI, TAU};
use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

fn source(surface: ElementarySurface, line: Line<Point2>) -> Curve3D {
    Curve3D::PCurve(PCurve::new(
        Box::new(Curve2D::Line(line)),
        Box::new(Surface::ElementarySurface(surface)),
    ))
}

#[test]
fn diagonal_elementary_pcurves_preserve_coordinates_and_derivatives() {
    let affine = Matrix4::from_cols(
        Vector4::new(1.5, 0.2, -0.3, 0.),
        Vector4::new(-0.5, 0.7, 0.9, 0.),
        Vector4::new(0.1, -0.2, 1.3, 0.),
        Vector4::new(13., -7., 23., 1.),
    );
    let origin = Point3::new(1., -2., 3.);
    let cylinder = Processor::new(RevolutedCurve::by_revolution(
        Line(
            origin + Vector3::new(10., 0., 0.),
            origin + Vector3::new(10., 0., 20.),
        ),
        origin,
        Vector3::unit_z(),
    ));
    let cone = Processor::new(RevolutedCurve::by_revolution(
        Line(
            origin + Vector3::new(10., 0., 0.),
            origin + Vector3::new(4., 0., 20.),
        ),
        origin,
        Vector3::unit_z(),
    ));
    for transform in [
        Matrix4::identity(),
        affine,
        affine * Matrix4::from_nonuniform_scale(-1., 1., 1.),
    ] {
        for elementary in [
            ElementarySurface::CylindricalSurface(cylinder),
            ElementarySurface::ConicalSurface(cone),
            ElementarySurface::ToroidalSurface(Processor::new(Torus::new(origin, 20., 5.))),
        ] {
            for flipped in [false, true] {
                let mut surface = elementary.transformed(transform);
                if flipped {
                    surface.invert();
                }
                for mut line in [
                    Line(Point2::new(0.1, -0.3), Point2::new(0.9, 2.1)),
                    Line(Point2::new(-0.7, TAU - 0.3), Point2::new(1.1, TAU + 0.8)),
                    Line(Point2::new(TAU, -PI), Point2::new(0., PI)),
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
                        let mapped = Curve::try_from(&input).unwrap();
                        assert!(matches!(mapped, Curve::PCurve(_)));
                        assert_eq!(mapped.range_tuple(), input.range_tuple());
                        for i in 0..=128 {
                            let t = i as f64 / 128.;
                            assert!(
                                mapped.subs(t).distance(input.subs(t)) < 1e-10,
                                "point at {t}"
                            );
                            assert!(
                                mapped.der(t).distance(input.der(t)) < 1e-9,
                                "tangent at {t}"
                            );
                            assert!(
                                mapped.der2(t).distance(input.der2(t)) < 1e-8,
                                "curvature at {t}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn diagonal_elementary_pcurves_reject_nonfinite_and_collapsed_geometry() {
    let cylinder = Processor::new(RevolutedCurve::by_revolution(
        Line(Point3::new(10., 0., 0.), Point3::new(10., 0., 20.)),
        Point3::origin(),
        Vector3::unit_z(),
    ));
    for elementary in [
        ElementarySurface::CylindricalSurface(cylinder),
        ElementarySurface::ConicalSurface(cylinder),
        ElementarySurface::ToroidalSurface(Processor::new(Torus::new(Point3::origin(), 20., 5.))),
    ] {
        for line in [
            Line(Point2::new(0.5, 0.7), Point2::new(0.5, 0.7)),
            Line(Point2::new(f64::NAN, 0.), Point2::new(1., 1.)),
            Line(Point2::new(0., 0.), Point2::new(1., f64::INFINITY)),
            Line(
                Point2::new(-f64::MAX, -f64::MAX),
                Point2::new(f64::MAX, f64::MAX),
            ),
        ] {
            assert!(Curve::try_from(&source(elementary, line)).is_err());
        }
        let line = Line(Point2::new(0.1, 0.2), Point2::new(0.9, 1.2));
        for transform in [Matrix4::from_scale(0.), Matrix4::from_scale(f64::NAN)] {
            assert!(Curve::try_from(&source(elementary.transformed(transform), line)).is_err());
        }
    }
    let cone = ElementarySurface::ConicalSurface(Processor::new(RevolutedCurve::by_revolution(
        Line(Point3::new(10., 0., 0.), Point3::new(0., 0., 20.)),
        Point3::origin(),
        Vector3::unit_z(),
    )));
    assert!(Curve::try_from(&source(
        cone,
        Line(Point2::new(1., 0.3), Point2::new(1., 2.)),
    ))
    .is_err());
}
