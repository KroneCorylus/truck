use std::f64::consts::TAU;
use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

#[test]
fn spline_parameter_traces_preserve_extrusion_geometry_and_speed() {
    let polynomial = BSplineCurve::new(
        KnotVec::from(vec![2., 2., 2., 5., 5., 5.]),
        vec![
            Point3::new(-2., 0., 0.),
            Point3::new(0., 4., 1.),
            Point3::new(3., 1., -1.),
        ],
    );
    let rational = NurbsCurve::new(BSplineCurve::new(
        polynomial.knot_vec().clone(),
        polynomial
            .control_points()
            .iter()
            .zip([1., 0.7, 1.3])
            .map(|(p, w)| p.to_homogeneous() * w)
            .collect(),
    ));
    let profiles = [
        Curve3D::Line(Line(Point3::new(1., 2., 3.), Point3::new(4., -1., 2.))),
        Curve3D::Conic(Conic3D::Ellipse(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0., TAU)),
            Matrix4::from_nonuniform_scale(100., 50., 1.),
        ))),
        Curve3D::BSplineCurve(polynomial),
        Curve3D::NurbsCurve(rational),
    ];
    let affine = Matrix4::from_cols(
        Vector4::new(1.5, 0.2, -0.3, 0.),
        Vector4::new(-0.5, 0.7, 0.9, 0.),
        Vector4::new(0.1, -0.2, 1.3, 0.),
        Vector4::new(13., -7., 23., 1.),
    );
    for profile in profiles {
        for transform in [Matrix4::identity(), affine] {
            for reversed_profile in [false, true] {
                let mut profile = profile.clone();
                if reversed_profile {
                    profile.invert();
                }
                let (a, b) = profile.range_tuple();
                let mid = (a + b) / 2.;
                let near_mid = f64::from_bits(mid.to_bits() - 1);
                let surface = Surface::SweptCurve(SweptCurve::ExtrudedCurve(
                    ExtrudedCurve::by_extrusion(profile, Vector3::new(20., 0., 100.)),
                ))
                .transformed(transform);
                for trace in [
                    BSplineCurve::new(
                        KnotVec::from(vec![2., 2., 2., 2., 5., 5., 5., 5.]),
                        vec![
                            Point2::new(a + (b - a) * 0.1, -3.),
                            Point2::new(a + (b - a) * 0.3, 6.),
                            Point2::new(a + (b - a) * 0.85, -4.),
                            Point2::new(a + (b - a) * 0.9, 5.),
                        ],
                    ),
                    BSplineCurve::new(
                        KnotVec::from(vec![2., 2., 2., 5., 5., 5.]),
                        vec![
                            Point2::new(mid, -3.),
                            Point2::new(near_mid, 4.),
                            Point2::new(mid, 2.),
                        ],
                    ),
                ] {
                    for reversed_trace in [false, true] {
                        let mut input = Curve3D::PCurve(PCurve::new(
                            Box::new(Curve2D::BSplineCurve(trace.clone())),
                            Box::new(surface.clone()),
                        ));
                        if reversed_trace {
                            input.invert();
                        }
                        let mapped = Curve::try_from(&input).expect("exact extrusion pcurve");
                        assert_eq!(mapped.range_tuple(), input.range_tuple());
                        for i in 0..=128 {
                            let t = 2. + 3. * i as f64 / 128.;
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
fn spline_extrusion_traces_reject_invalid_geometry_and_changed_parameters() {
    let line = Curve3D::Line(Line(Point3::origin(), Point3::new(1., 0., 0.)));
    let source = |profile: Curve3D, vector: Vector3, points: Vec<Point2>| {
        let surface = Surface::SweptCurve(SweptCurve::ExtrudedCurve(ExtrudedCurve::by_extrusion(
            profile, vector,
        )));
        Curve3D::PCurve(PCurve::new(
            Box::new(Curve2D::BSplineCurve(BSplineCurve::new(
                KnotVec::bezier_knot(2),
                points,
            ))),
            Box::new(surface),
        ))
    };
    for points in [
        vec![Point2::new(0.5, 0.7); 3],
        vec![
            Point2::new(0., 0.),
            Point2::new(f64::NAN, 0.5),
            Point2::new(1., 1.),
        ],
        vec![
            Point2::new(0., 0.),
            Point2::new(0.5, f64::INFINITY),
            Point2::new(1., 1.),
        ],
    ] {
        assert!(Curve::try_from(&source(line.clone(), Vector3::unit_z(), points)).is_err());
    }
    let points = vec![
        Point2::new(0.5, 0.),
        Point2::new(0.5, 0.5),
        Point2::new(0.5, 1.),
    ];
    assert!(Curve::try_from(&source(line.clone(), Vector3::zero(), points.clone())).is_err());
    assert!(Curve::try_from(&source(
        line,
        Vector3::new(0., 0., f64::NAN),
        points.clone()
    ))
    .is_err());
    for profile in [
        Curve3D::Conic(Conic3D::Parabola(Processor::new(TrimmedCurve::new(
            UnitParabola::new(),
            (-3., 2.),
        )))),
        Curve3D::Conic(Conic3D::Hyperbola(Processor::new(TrimmedCurve::new(
            UnitHyperbola::new(),
            (-3., 2.),
        )))),
        Curve3D::Polyline(PolylineCurve(vec![
            Point3::origin(),
            Point3::new(1., 2., 0.),
            Point3::new(3., 1., 0.),
        ])),
    ] {
        assert!(Curve::try_from(&source(profile, Vector3::unit_z(), points.clone())).is_err());
    }
}
