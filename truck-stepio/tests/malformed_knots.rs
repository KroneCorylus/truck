use std::str::FromStr;
use truck_geometry::prelude::*;
use truck_stepio::r#in::{
    ruststep::{ast::DataSection, tables::EntityTable},
    BSplineCurveWithKnotsHolder, BSplineSurfaceWithKnotsHolder, Table,
};

fn curve_with_multiplicities(
    knots: &str,
    multiplicities: &str,
) -> std::result::Result<BSplineCurve<Point3>, Box<dyn std::error::Error>> {
    let data = DataSection::from_str(&format!(
        "DATA;
        #1 = B_SPLINE_CURVE_WITH_KNOTS('',1,(#2,#3),.UNSPECIFIED.,.F.,.F.,{multiplicities},{knots},.UNSPECIFIED.);
        #2 = CARTESIAN_POINT('',(0.,0.,0.));
        #3 = CARTESIAN_POINT('',(2.,0.,0.));
        ENDSEC;"
    ))
    .unwrap();
    let table = Table::from_data_section(&data);
    let entity = EntityTable::<BSplineCurveWithKnotsHolder>::get_owned(&table, 1).unwrap();
    BSplineCurve::try_from(&entity)
}

fn surface_with_multiplicities(
    u_multi: &str,
    v_multi: &str,
    u_knots: &str,
    v_knots: &str,
) -> std::result::Result<BSplineSurface<Point3>, Box<dyn std::error::Error>> {
    let data = DataSection::from_str(&format!(
        "DATA;
        #1 = B_SPLINE_SURFACE_WITH_KNOTS('',1,1,((#2,#3),(#4,#5)),.UNSPECIFIED.,.F.,.F.,.F.,{u_multi},{v_multi},{u_knots},{v_knots},.UNSPECIFIED.);
        #2 = CARTESIAN_POINT('',(0.,0.,0.));
        #3 = CARTESIAN_POINT('',(0.,3.,0.));
        #4 = CARTESIAN_POINT('',(2.,0.,0.));
        #5 = CARTESIAN_POINT('',(2.,3.,0.));
        ENDSEC;"
    ))
    .unwrap();
    let table = Table::from_data_section(&data);
    let entity = EntityTable::<BSplineSurfaceWithKnotsHolder>::get_owned(&table, 1).unwrap();
    BSplineSurface::try_from(&entity)
}

#[test]
fn descending_curve_knots_return_error() {
    let error = curve("(1.,0.)").unwrap_err();
    assert!(matches!(
        error.downcast_ref::<truck_geometry::errors::Error>(),
        Some(truck_geometry::errors::Error::NotSortedVector)
    ));
}

#[test]
fn descending_surface_u_knots_return_error() {
    assert!(surface("(1.,0.)", "(0.,1.)").is_err());
}

#[test]
fn descending_surface_v_knots_return_error() {
    assert!(surface("(0.,1.)", "(1.,0.)").is_err());
}

#[test]
fn valid_knots_preserve_line_and_plane() {
    let curve = curve("(0.,1.)").unwrap();
    let surface = surface("(0.,1.)", "(0.,1.)").unwrap();
    for u in [0.0, 0.25, 0.5, 1.0] {
        assert_near!(curve.subs(u), Point3::new(2.0 * u, 0.0, 0.0));
        for v in [0.0, 0.25, 0.5, 1.0] {
            assert_near!(surface.subs(u, v), Point3::new(2.0 * u, 3.0 * v, 0.0));
        }
    }
}

fn curve(knots: &str) -> std::result::Result<BSplineCurve<Point3>, Box<dyn std::error::Error>> {
    curve_with_multiplicities(knots, "(2,2)")
}

fn surface(
    u: &str,
    v: &str,
) -> std::result::Result<BSplineSurface<Point3>, Box<dyn std::error::Error>> {
    surface_with_multiplicities("(2,2)", "(2,2)", u, v)
}

macro_rules! multiplicity_regressions {
    ($curve:ident, $u:ident, $v:ident, $multiplicities:literal) => {
        #[test]
        fn $curve() {
            assert!(curve_with_multiplicities("(0.,1.)", $multiplicities).is_err());
        }
        #[test]
        fn $u() {
            assert!(
                surface_with_multiplicities($multiplicities, "(2,2)", "(0.,1.)", "(0.,1.)")
                    .is_err()
            );
        }
        #[test]
        fn $v() {
            assert!(
                surface_with_multiplicities("(2,2)", $multiplicities, "(0.,1.)", "(0.,1.)")
                    .is_err()
            );
        }
    };
}

multiplicity_regressions!(
    short_curve_multiplicities,
    short_surface_u_multiplicities,
    short_surface_v_multiplicities,
    "(2)"
);
multiplicity_regressions!(
    extra_curve_multiplicities,
    extra_surface_u_multiplicities,
    extra_surface_v_multiplicities,
    "(2,2,1)"
);
multiplicity_regressions!(
    negative_curve_multiplicities,
    negative_surface_u_multiplicities,
    negative_surface_v_multiplicities,
    "(-1,2)"
);
multiplicity_regressions!(
    zero_curve_multiplicities,
    zero_surface_u_multiplicities,
    zero_surface_v_multiplicities,
    "(0,2)"
);
