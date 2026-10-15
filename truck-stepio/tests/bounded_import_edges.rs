use ruststep::{ast::DataSection, tables::EntityTable};
use truck_stepio::r#in::{step_geometry::*, EdgeCurve, EdgeCurveHolder, Table};

fn edge(entity: &str, p: Point3, q: Point3, reversed: bool) -> EdgeCurve {
    let (p, q, sense) = if reversed {
        (q, p, ".F.")
    } else {
        (p, q, ".T.")
    };
    let source = format!(
        "DATA;{entity}
        #2=CARTESIAN_POINT('',(1.,0.,0.));
        #3=CARTESIAN_POINT('',(1.,1.,0.));
        #4=CARTESIAN_POINT('',(0.,1.,0.));
        #5=CARTESIAN_POINT('',({},{},{}));
        #6=CARTESIAN_POINT('',({},{},{}));
        #7=VERTEX_POINT('',#5); #8=VERTEX_POINT('',#6);
        #9=EDGE_CURVE('',#7,#8,#1,{sense}); ENDSEC;",
        p.x, p.y, p.z, q.x, q.y, q.z
    );
    let data: DataSection = source.parse().unwrap();
    EntityTable::<EdgeCurveHolder>::get_owned(&Table::from_data_section(&data), 9).unwrap()
}

fn spline(rational: bool) -> (&'static str, NurbsCurve<Vector4>) {
    let weight = if rational { 0.5_f64.sqrt() } else { 1. };
    let entity = if rational {
        "#1=(BOUNDED_CURVE() B_SPLINE_CURVE(2,(#2,#3,#4),.UNSPECIFIED.,.F.,.F.)
        B_SPLINE_CURVE_WITH_KNOTS((3,3),(0.,1.),.UNSPECIFIED.)
        CURVE() GEOMETRIC_REPRESENTATION_ITEM()
        RATIONAL_B_SPLINE_CURVE((1.,0.7071067811865476,1.)) REPRESENTATION_ITEM(''));"
    } else {
        "#1=B_SPLINE_CURVE_WITH_KNOTS('',2,(#2,#3,#4),.UNSPECIFIED.,.F.,.F.,
        (3,3),(0.,1.),.UNSPECIFIED.);"
    };
    (
        entity,
        NurbsCurve::new(BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Vector4::new(1., 0., 0., 1.),
                Vector4::new(weight, weight, 0., weight),
                Vector4::new(0., 1., 0., 1.),
            ],
        )),
    )
}

#[test]
fn spline_edges_trim_to_vertices_in_both_dimensions_and_senses() {
    for rational in [false, true] {
        let (entity, original) = spline(rational);
        for (a, b) in [(0.2, 0.8), (0., 0.6), (0.4, 1.)] {
            for reversed in [false, true] {
                let edge = edge(entity, original.subs(a), original.subs(b), reversed);
                let c2 = edge.parse_curve2d().unwrap();
                let c3 = edge.parse_curve3d().unwrap();
                assert!(
                    (c2.range_tuple().0 - a).abs() < 1e-8 && (c2.range_tuple().1 - b).abs() < 1e-8
                );
                assert!(
                    (c3.range_tuple().0 - a).abs() < 1e-8 && (c3.range_tuple().1 - b).abs() < 1e-8
                );
                for i in 0..=32 {
                    let t = a + (b - a) * i as f64 / 32.;
                    let u = if reversed { a + b - t } else { t };
                    let expected = original.subs(u);
                    assert!(c3.subs(t).distance(expected) < 1e-8);
                    assert!(c2.subs(t).distance(Point2::new(expected.x, expected.y)) < 1e-8);
                    for n in 1..=2 {
                        let d =
                            original.der_n(n, u) * if reversed && n % 2 == 1 { -1. } else { 1. };
                        assert!(c3.der_n(n, t).distance(d) < 1e-7);
                        assert!(c2.der_n(n, t).distance(Vector2::new(d.x, d.y)) < 1e-7);
                    }
                }
            }
        }
    }
}

#[test]
fn bounded_edges_reject_unmatched_and_reversed_parameter_intervals() {
    let (entity, original) = spline(false);
    for (p, q) in [
        (Point3::new(5., 5., 0.), original.subs(0.8)),
        (original.subs(0.2), Point3::new(5., 5., 0.)),
        (original.subs(0.8), original.subs(0.2)),
    ] {
        let edge = edge(entity, p, q, false);
        assert!(edge.parse_curve2d().is_err());
        assert!(edge.parse_curve3d().is_err());
    }
}

#[test]
fn full_closed_spline_edges_keep_distinct_endpoint_parameters() {
    let entity="#1=B_SPLINE_CURVE_WITH_KNOTS('',3,(#2,#3,#4,#2),.UNSPECIFIED.,.T.,.F.,(4,4),(0.,1.),.UNSPECIFIED.);";
    for reversed in [false, true] {
        let edge = edge(
            entity,
            Point3::new(1., 0., 0.),
            Point3::new(1., 0., 0.),
            reversed,
        );
        let curve = edge.parse_curve3d().unwrap();
        assert_eq!(curve.range_tuple(), (0., 1.));
        assert!(curve.front().near(&curve.back()));
        assert!(curve.subs(0.5).distance(curve.front()) > 0.5);
        let curve = edge.parse_curve2d().unwrap();
        assert_eq!(curve.range_tuple(), (0., 1.));
        assert!(curve.front().near(&curve.back()));
        assert!(curve.subs(0.5).distance(curve.front()) > 0.5);
    }
}

#[test]
fn polyline_edges_trim_inside_one_or_multiple_segments() {
    for (p, q) in [
        (Point3::new(1., 0.2, 0.), Point3::new(1., 0.8, 0.)),
        (Point3::new(1., 0.2, 0.), Point3::new(0.3, 1., 0.)),
    ] {
        for reversed in [false, true] {
            let edge = edge("#1=POLYLINE('',(#2,#3,#4));", p, q, reversed);
            let (p, q) = if reversed { (q, p) } else { (p, q) };
            let curve = edge.parse_curve3d().unwrap();
            assert!(curve.front().near(&p) && curve.back().near(&q));
            let curve = edge.parse_curve2d().unwrap();
            assert!(
                curve.front().near(&Point2::new(p.x, p.y))
                    && curve.back().near(&Point2::new(q.x, q.y))
            );
        }
    }
}
