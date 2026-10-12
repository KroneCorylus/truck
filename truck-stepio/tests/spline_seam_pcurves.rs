use truck_modeling::Curve;
use truck_stepio::r#in::step_geometry::*;

#[test]
fn spline_isoparametric_trims_preserve_geometry_and_parameter_speed() {
    let matrix = Matrix4::from_translation(Vector3::new(13., -7., 23.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    let surface = BSplineSurface::new(
        (
            KnotVec::from(vec![2., 2., 2., 5., 5., 5.]),
            KnotVec::from(vec![-3., -3., 4., 4.]),
        ),
        vec![
            vec![Point3::new(1., 0., 0.), Point3::new(1., 0., 7.)],
            vec![Point3::new(1., 1., 0.), Point3::new(1., 1., 7.)],
            vec![Point3::new(0., 1., 0.), Point3::new(0., 1., 7.)],
        ],
    );
    let rational = NurbsSurface::try_from_bspline_and_weights(
        surface.clone(),
        vec![vec![1.; 2], vec![0.5_f64.sqrt(); 2], vec![1.; 2]],
    )
    .unwrap();
    let mut surfaces = [
        Surface::BSplineSurface(surface),
        Surface::NurbsSurface(rational),
    ];
    for surface in &mut surfaces {
        match surface {
            Surface::BSplineSurface(s) => {
                s.add_uknot(3.4);
                s.add_vknot(0.2);
            }
            Surface::NurbsSurface(s) => {
                s.add_uknot(3.4);
                s.add_vknot(0.2);
            }
            _ => unreachable!(),
        }
        surface.transform_by(matrix);
        for line in [
            Line(Point2::new(2.7, -2.), Point2::new(2.7, 3.2)),
            Line(Point2::new(2.3, 0.5), Point2::new(4.7, 0.5)),
            Line(Point2::new(2., -3.), Point2::new(5., -3.)),
            Line(Point2::new(5., -3.), Point2::new(5., 4.)),
        ] {
            for reverse in [false, true] {
                let mut source = Curve3D::PCurve(PCurve::new(
                    Box::new(Curve2D::Line(line)),
                    Box::new(surface.clone()),
                ));
                if reverse {
                    source.invert();
                }
                let mapped = Curve::try_from(&source).unwrap();
                assert_eq!(mapped.range_tuple(), (0., 1.));
                for i in 0..=64 {
                    let t = i as f64 / 64.;
                    assert!(mapped.subs(t).distance(source.subs(t)) < 1e-10);
                    assert!(mapped.der(t).distance(source.der(t)) < 1e-10);
                    assert!(mapped.der2(t).distance(source.der2(t)) < 1e-9);
                }
            }
        }
    }
}

#[test]
fn seam_search_roundoff_preserves_the_exact_section_but_real_offsets_are_rejected() {
    let surface = Surface::BSplineSurface(BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
        vec![
            vec![Point3::new(1., 0., 0.), Point3::new(1., 0., 7.)],
            vec![Point3::new(1., 1., 0.), Point3::new(1., 1., 7.)],
            vec![Point3::new(0., 1., 0.), Point3::new(0., 1., 7.)],
        ],
    ));
    for (delta, overshoot, success) in [
        (f64::EPSILON, f64::EPSILON, true),
        (0.001, 0., false),
        (0., 0.001, false),
    ] {
        let source = Curve3D::PCurve(PCurve::new(
            Box::new(Curve2D::Line(Line(
                Point2::new(0.5, -overshoot),
                Point2::new(0.5 + delta, 1. + overshoot),
            ))),
            Box::new(surface.clone()),
        ));
        let mapped = Curve::try_from(&source);
        if success {
            let curve = mapped.unwrap();
            for i in 0..=64 {
                let t = i as f64 / 64.;
                assert!(curve.subs(t).distance(Point3::new(0.75, 0.75, 7. * t)) < 1e-12);
                assert!(curve.der(t).distance(Vector3::new(0., 0., 7.)) < 1e-12);
            }
        } else {
            assert!(mapped.is_err());
        }
    }
}
