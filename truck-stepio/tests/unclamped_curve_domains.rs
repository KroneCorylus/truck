use ruststep::{ast::DataSection, tables::EntityTable};
use truck_stepio::r#in::{step_geometry::*, EdgeCurveHolder, SurfaceAnyHolder, Table};

#[test]
fn unclamped_spline_edges_use_the_active_domain_after_rationalization() {
    for rational in [false, true] {
        for reversed in [false, true] {
            let entity = if rational {
                "#1=(BOUNDED_CURVE() B_SPLINE_CURVE(2,(#2,#3,#4),.UNSPECIFIED.,.F.,.F.)
                B_SPLINE_CURVE_WITH_KNOTS((1,2,2,1),(-1.,0.,1.,2.),.UNSPECIFIED.)
                CURVE() GEOMETRIC_REPRESENTATION_ITEM()
                RATIONAL_B_SPLINE_CURVE((1.,0.7071067811865476,1.)) REPRESENTATION_ITEM(''));"
            } else {
                "#1=B_SPLINE_CURVE_WITH_KNOTS('',2,(#2,#3,#4),.UNSPECIFIED.,.F.,.F.,
                (1,2,2,1),(-1.,0.,1.,2.),.UNSPECIFIED.);"
            };
            let (front, back, sense) = if reversed {
                (6, 5, ".F.")
            } else {
                (5, 6, ".T.")
            };
            let source = format!(
                "DATA;{entity}
                #2=CARTESIAN_POINT('',(1.,0.,0.));
                #3=CARTESIAN_POINT('',(1.,1.,0.));
                #4=CARTESIAN_POINT('',(0.,1.,0.));
                #5=VERTEX_POINT('',#2); #6=VERTEX_POINT('',#4);
                #7=EDGE_CURVE('',#{front},#{back},#1,{sense}); ENDSEC;"
            );
            let data: DataSection = source.parse().unwrap();
            let table = Table::from_data_section(&data);
            let edge = EntityTable::<EdgeCurveHolder>::get_owned(&table, 7).unwrap();
            let curve2 = edge.parse_curve2d().unwrap();
            let curve3 = edge.parse_curve3d().unwrap();
            assert_eq!(curve2.range_tuple(), (0., 1.));
            assert_eq!(curve3.range_tuple(), (0., 1.));
            for i in 0..=64 {
                let t = i as f64 / 64.;
                let s = if reversed { 1. - t } else { t };
                let w = if rational { 0.5_f64.sqrt() } else { 1. };
                let (a, b, c) = ((1. - s).powi(2), 2. * w * s * (1. - s), s * s);
                let expected = Point3::new((a + b) / (a + b + c), (b + c) / (a + b + c), 0.);
                assert!(curve3.subs(t).distance(expected) < 1e-12);
                assert!(curve2.subs(t).distance(Point2::new(expected.x, expected.y)) < 1e-12);
                if rational {
                    assert!((expected.to_vec().magnitude() - 1.).abs() < 1e-12);
                }
            }
            let (_, points) = curve3.parameter_division(curve3.range_tuple(), 0.001);
            assert!(points.len() > 2 && points.len() < 1000);
            assert!(points
                .iter()
                .all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()));
        }
    }
}

#[test]
fn unclamped_spline_surfaces_use_both_active_domains_after_rationalization() {
    for rational in [false, true] {
        let entity = if rational {
            "#1=(BOUNDED_SURFACE() B_SPLINE_SURFACE(2,1,((#2,#3),(#4,#5),(#6,#7)),
            .UNSPECIFIED.,.F.,.F.,.F.)
            B_SPLINE_SURFACE_WITH_KNOTS((1,2,2,1),(1,1,1,1),(-1.,0.,1.,2.),(-1.,0.,1.,2.),.UNSPECIFIED.)
            GEOMETRIC_REPRESENTATION_ITEM()
            RATIONAL_B_SPLINE_SURFACE(((1.,1.),(0.7071067811865476,0.7071067811865476),(1.,1.)))
            REPRESENTATION_ITEM('') SURFACE());"
        } else {
            "#1=B_SPLINE_SURFACE_WITH_KNOTS('',2,1,((#2,#3),(#4,#5),(#6,#7)),
            .UNSPECIFIED.,.F.,.F.,.F.,(1,2,2,1),(1,1,1,1),(-1.,0.,1.,2.),(-1.,0.,1.,2.),.UNSPECIFIED.);"
        };
        let source = format!(
            "DATA;{entity}
            #2=CARTESIAN_POINT('',(1.,0.,0.)); #3=CARTESIAN_POINT('',(1.,0.,2.));
            #4=CARTESIAN_POINT('',(1.,1.,0.)); #5=CARTESIAN_POINT('',(1.,1.,2.));
            #6=CARTESIAN_POINT('',(0.,1.,0.)); #7=CARTESIAN_POINT('',(0.,1.,2.)); ENDSEC;"
        );
        let data: DataSection = source.parse().unwrap();
        let table = Table::from_data_section(&data);
        let entity = EntityTable::<SurfaceAnyHolder>::get_owned(&table, 1).unwrap();
        let surface = Surface::try_from(&entity).unwrap();
        assert_eq!(surface.try_range_tuple(), (Some((0., 1.)), Some((0., 1.))));
        for i in 0..=32 {
            for j in 0..=8 {
                let (u, v) = (i as f64 / 32., j as f64 / 8.);
                let w = if rational { 0.5_f64.sqrt() } else { 1. };
                let (a, b, c) = ((1. - u).powi(2), 2. * w * u * (1. - u), u * u);
                let expected = Point3::new((a + b) / (a + b + c), (b + c) / (a + b + c), 2. * v);
                let point = surface.subs(u, v);
                assert!(point.distance(expected) < 1e-12);
                if rational {
                    let normal = Vector3::new(point.x, point.y, 0.);
                    assert!(surface.normal(u, v).dot(normal) > 1. - 1e-12);
                }
            }
        }
    }
}
