use std::f64::consts::{PI, TAU};
use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

#[test]
fn toroidal_isoparametric_trims_preserve_exact_images_and_tangents() {
    let transform = Matrix4::from_cols(
        Vector4::new(1.5, 0.2, -0.3, 0.),
        Vector4::new(-0.5, 0.7, 0.9, 0.),
        Vector4::new(0.1, -0.2, 1.3, 0.),
        Vector4::new(13., -7., 23., 1.),
    );
    for transform in [Matrix4::identity(), transform] {
        for flipped in [false, true] {
            let mut surface = Processor::with_transform(
                Torus::new(Point3::new(1., -2., 3.), 200., 50.),
                transform,
            );
            if flipped {
                surface.invert();
            }
            for mut line in [
                Line(Point2::new(0.7, -0.3), Point2::new(0.7, 2.1)),
                Line(Point2::new(-0.3, 0.7), Point2::new(2.1, 0.7)),
                Line(Point2::new(PI, 0.), Point2::new(PI, TAU)),
                Line(Point2::new(0., PI), Point2::new(TAU, PI)),
            ] {
                if flipped {
                    line.0 = Point2::new(line.0.y, line.0.x);
                    line.1 = Point2::new(line.1.y, line.1.x);
                }
                for reversed in [false, true] {
                    let mut source = Curve3D::PCurve(PCurve::new(
                        Box::new(Curve2D::Line(line)),
                        Box::new(Surface::ElementarySurface(
                            ElementarySurface::ToroidalSurface(surface),
                        )),
                    ));
                    if reversed {
                        source.invert();
                    }
                    let curve = Curve::try_from(&source).unwrap();
                    assert!(matches!(curve, Curve::Conic(_)));
                    let (a, b) = curve.range_tuple();
                    for i in 0..=64 {
                        let t = i as f64 / 64.;
                        let s = a + (b - a) * t;
                        assert!(curve.subs(s).distance(source.subs(t)) < 1e-10);
                        assert!((curve.der(s) * (b - a)).distance(source.der(t)) < 1e-9);
                        assert!((curve.der2(s) * (b - a).powi(2)).distance(source.der2(t)) < 1e-8);
                    }
                }
            }
        }
    }
}

#[test]
fn toroidal_seam_roundoff_is_bounded_and_real_diagonals_are_rejected() {
    let surface = Surface::ElementarySurface(ElementarySurface::ToroidalSurface(Processor::new(
        Torus::new(Point3::origin(), 200., 50.),
    )));
    for (delta, success) in [(4. * f64::EPSILON, true), (0.001, false)] {
        let source = Curve3D::PCurve(PCurve::new(
            Box::new(Curve2D::Line(Line(
                Point2::new(0.7, 0.),
                Point2::new(0.7 + delta, PI),
            ))),
            Box::new(surface.clone()),
        ));
        let curve = Curve::try_from(&source);
        if success {
            let curve = curve.unwrap();
            let (a, b) = curve.range_tuple();
            for i in 0..=64 {
                let t = i as f64 / 64.;
                assert!(curve.subs(a + (b - a) * t).distance(source.subs(t)) < 1e-10);
            }
        } else {
            assert!(curve.is_err());
        }
    }
    for (torus, transform, line) in [
        (
            Torus::new(Point3::origin(), 200., 50.),
            Matrix4::from_scale(1e12),
            Line(
                Point2::new(0.7, 0.),
                Point2::new(0.7 + 4. * f64::EPSILON, PI),
            ),
        ),
        (
            Torus::new(Point3::origin(), 1., 1.),
            Matrix4::identity(),
            Line(Point2::new(0., PI), Point2::new(PI, PI)),
        ),
        (
            Torus::new(Point3::origin(), 200., 50.),
            Matrix4::identity(),
            Line(Point2::new(0., f64::NAN), Point2::new(PI, 0.)),
        ),
    ] {
        let source = Curve3D::PCurve(PCurve::new(
            Box::new(Curve2D::Line(line)),
            Box::new(Surface::ElementarySurface(
                ElementarySurface::ToroidalSurface(Processor::with_transform(torus, transform)),
            )),
        ));
        assert!(Curve::try_from(&source).is_err());
    }
}
