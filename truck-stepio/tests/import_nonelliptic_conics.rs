use truck_modeling::{Curve, Surface};
use truck_stepio::r#in::step_geometry::{self as step, *};

fn transforms() -> [Matrix4; 2] {
    let mut shear = Matrix4::identity();
    shear.y.x = 0.7;
    [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_nonuniform_scale(5., 2., 3.)
            * shear,
    ]
}

fn verify(source: &Curve3D, hyperbolic: bool) {
    let curve = Curve::try_from(source).expect("finite conic must convert to modeling geometry");
    let (a, b) = source.range_tuple();
    let (u, v) = curve.range_tuple();
    assert!(curve.subs(u).distance(source.subs(a)) < 1e-9);
    assert!(curve.subs(v).distance(source.subs(b)) < 1e-9);
    for i in 0..=128 {
        let t = a + (b - a) * i as f64 / 128.;
        let p = source.subs(t);
        let s = curve.search_nearest_parameter(p, None, 100).unwrap();
        assert!(curve.subs(s).distance(p) < 1e-8);
        assert!(curve.der(s).normalize().dot(source.der(t).normalize()) > 1. - 1e-10);
        if !hyperbolic {
            assert!(curve.subs((t - a) / (b - a)).distance(p) < 1e-9);
            assert!((curve.der((t - a) / (b - a)) / (b - a)).distance(source.der(t)) < 1e-9);
            assert!(
                (curve.der2((t - a) / (b - a)) / (b - a).powi(2)).distance(source.der2(t)) < 1e-9
            );
        }
        let s = u + (v - u) * (i as f64 + 0.31) / 129.;
        let p = curve.subs(s);
        let t = source.search_parameter(p, None, 100).unwrap();
        assert!(source.subs(t).distance(p) < 1e-8);
    }
    if let Curve::NurbsCurve(nurbs) = &curve {
        for weight in nurbs.control_points().iter().map(|p| p.w) {
            assert!(weight.is_finite() && weight > 0.);
        }
        for &knot in nurbs.knot_vec().iter() {
            if knot > u && knot < v {
                let left = curve.der(knot - 1e-8).normalize();
                let right = curve.der(knot + 1e-8).normalize();
                assert!(left.dot(right) > 1. - 1e-10);
            }
        }
    }
    let axis = Vector3::new(0.3, -0.7, 4.);
    let swept = step::Surface::SweptCurve(SweptCurve::ExtrudedCurve(ExtrudedCurve::by_extrusion(
        source.clone(),
        axis,
    )));
    let mapped = Surface::try_from(&swept).unwrap();
    for i in 0..=16 {
        let p = swept.subs(a + (b - a) * i as f64 / 16., 0.37);
        let (s, t) = mapped.search_nearest_parameter(p, None, 100).unwrap();
        assert!(mapped.subs(s, t).distance(p) < 1e-8);
        let source_t = source.search_parameter(p - axis * 0.37, None, 100).unwrap();
        assert!(mapped.normal(s, t).dot(swept.normal(source_t, 0.37)) > 1. - 1e-10);
    }
}

#[test]
fn finite_parabolas_preserve_polynomial_geometry_and_orientation() {
    for range in [(-3., 2.), (1.2, 4.7), (-0.02, 0.01)] {
        for transform in transforms() {
            for reversed in [false, true] {
                let mut curve = Processor::with_transform(
                    TrimmedCurve::new(UnitParabola::<Point3>::new(), range),
                    transform,
                );
                if reversed {
                    curve.invert();
                }
                verify(&Curve3D::Conic(Conic3D::Parabola(curve)), false);
            }
        }
    }
}

#[test]
fn finite_hyperbolas_preserve_exact_geometry_and_orientation() {
    for range in [(-3., 2.), (1.2, 4.7), (-0.02, 0.01)] {
        for transform in transforms() {
            for reversed in [false, true] {
                let mut curve = Processor::with_transform(
                    TrimmedCurve::new(UnitHyperbola::<Point3>::new(), range),
                    transform,
                );
                if reversed {
                    curve.invert();
                }
                verify(&Curve3D::Conic(Conic3D::Hyperbola(curve)), true);
            }
        }
    }
}

#[test]
fn invalid_nonelliptic_conics_return_errors() {
    for range in [(1., 1.), (2., -1.), (f64::NAN, 1.), (0., f64::INFINITY)] {
        let parabola = Curve3D::Conic(Conic3D::Parabola(Processor::new(TrimmedCurve::new(
            UnitParabola::new(),
            range,
        ))));
        let hyperbola = Curve3D::Conic(Conic3D::Hyperbola(Processor::new(TrimmedCurve::new(
            UnitHyperbola::new(),
            range,
        ))));
        assert!(Curve::try_from(&parabola).is_err());
        assert!(Curve::try_from(&hyperbola).is_err());
    }
    let curve = Curve3D::Conic(Conic3D::Hyperbola(Processor::new(TrimmedCurve::new(
        UnitHyperbola::new(),
        (-1000., 1000.),
    ))));
    assert!(Curve::try_from(&curve).is_err());
    let curve = Curve3D::Conic(Conic3D::Parabola(Processor::with_transform(
        TrimmedCurve::new(UnitParabola::new(), (-1., 1.)),
        Matrix4::from_scale(f64::INFINITY),
    )));
    assert!(Curve::try_from(&curve).is_err());
}

fn verify_step_edges(hyperbolic: bool) {
    use ruststep::{ast::DataSection, tables::EntityTable};
    use std::str::FromStr;
    use truck_stepio::r#in::{EdgeCurveHolder, Table};
    for dim in [2, 3] {
        for reversed in [false, true] {
            for (a, b) in [(-3.0_f64, 2.0_f64), (0.2, 0.2000001)] {
                let coords = |t: f64| {
                    let (x, y) = if hyperbolic {
                        (t.cosh(), t.sinh())
                    } else {
                        (t * t, 2. * t)
                    };
                    let (x, y) = (2. - 2. * y, 3. + 2. * x);
                    if dim == 3 {
                        format!("{x:?},{y:?},5.")
                    } else {
                        format!("{x:?},{y:?}")
                    }
                };
                let (start, end) = if reversed { (b, a) } else { (a, b) };
                let placement = if dim == 3 {
                    "#4=AXIS2_PLACEMENT_3D('',#5,#6,#7);#5=CARTESIAN_POINT('',(2.,3.,5.));#6=DIRECTION('',(0.,0.,1.));#7=DIRECTION('',(0.,1.,0.));"
                } else {
                    "#4=AXIS2_PLACEMENT_2D('',#5,#7);#5=CARTESIAN_POINT('',(2.,3.));#7=DIRECTION('',(0.,1.));"
                };
                let kind = if hyperbolic {
                    "HYPERBOLA('',#4,2.,2.)"
                } else {
                    "PARABOLA('',#4,2.)"
                };
                let data = format!("DATA;#1=EDGE_CURVE('',#2,#3,#8,.{}.);#2=VERTEX_POINT('',#9);#3=VERTEX_POINT('',#10);{placement}#8={kind};#9=CARTESIAN_POINT('',({}));#10=CARTESIAN_POINT('',({}));ENDSEC;", if reversed { "F" } else { "T" },coords(start),coords(end));
                let table = Table::from_data_section(&DataSection::from_str(&data).unwrap());
                let edge = EntityTable::<EdgeCurveHolder>::get_owned(&table, 1).unwrap();
                if dim == 3 {
                    let curve = edge.parse_curve3d().unwrap();
                    assert_eq!(
                        matches!(curve, Curve3D::Conic(Conic3D::Hyperbola(_))),
                        hyperbolic
                    );
                    assert!(
                        curve
                            .front()
                            .distance(Point3::from(&edge.edge_start.vertex_geometry))
                            < 1e-9
                    );
                    assert!(
                        curve
                            .back()
                            .distance(Point3::from(&edge.edge_end.vertex_geometry))
                            < 1e-9
                    );
                    let (lo, hi) = curve.range_tuple();
                    assert!((lo - a).abs() < 1e-9 && (hi - b).abs() < 1e-9);
                    let mapped = Curve::try_from(&curve).unwrap();
                    assert!(mapped.front().distance(curve.front()) < 1e-9);
                    assert!(mapped.back().distance(curve.back()) < 1e-9);
                } else {
                    let curve = edge.parse_curve2d().unwrap();
                    assert_eq!(
                        matches!(curve, Curve2D::Conic(Conic2D::Hyperbola(_))),
                        hyperbolic
                    );
                    assert!(
                        curve
                            .front()
                            .distance(Point2::from(&edge.edge_start.vertex_geometry))
                            < 1e-9
                    );
                    assert!(
                        curve
                            .back()
                            .distance(Point2::from(&edge.edge_end.vertex_geometry))
                            < 1e-9
                    );
                    let (lo, hi) = curve.range_tuple();
                    assert!((lo - a).abs() < 1e-9 && (hi - b).abs() < 1e-9);
                }
            }
        }
    }
}

#[test]
fn step_parabola_edges_preserve_type_endpoints_and_sense() { verify_step_edges(false); }
#[test]
fn step_hyperbola_edges_preserve_type_endpoints_and_sense() { verify_step_edges(true); }
