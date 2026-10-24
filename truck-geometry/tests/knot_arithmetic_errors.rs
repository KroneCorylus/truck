use truck_geometry::prelude::*;

#[test]
fn unrepresentable_knot_arithmetic_returns_errors_and_normalization_is_atomic() {
    use truck_geometry::errors::{Error, Result};
    fn arithmetic<T>(result: Result<T>) {
        assert!(matches!(result, Err(Error::NonFiniteKnotArithmetic)));
    }
    let tiny = f64::from_bits(1);
    let tiny_knots = KnotVec::from(vec![0., 0., tiny, tiny]);
    let points = vec![Point3::origin(), Point3::new(1., 0., 0.)];
    arithmetic(BSplineCurve::try_new(tiny_knots.clone(), points.clone()));
    arithmetic(BSplineSurface::try_new(
        (tiny_knots.clone(), KnotVec::bezier_knot(1)),
        vec![points.clone(), points.clone()],
    ));
    arithmetic(tiny_knots.try_bspline_basis_functions(1, 0, 0.));
    let mut recoverable = tiny_knots;
    recoverable.try_normalize().unwrap();
    assert_eq!(recoverable.as_slice(), &[0., 0., 1., 1.]);
    arithmetic(BSplineCurve::try_new(
        KnotVec::from(vec![0., 0., tiny, 1., 1.]),
        vec![Point3::origin(); 3],
    ));
    for values in [
        vec![0., 0., f64::INFINITY, f64::INFINITY],
        vec![-f64::MAX, -f64::MAX, f64::MAX, f64::MAX],
    ] {
        let mut knots = KnotVec::from(values);
        let before = knots.clone();
        arithmetic(knots.try_normalize());
        assert_eq!(knots, before);
        arithmetic(BSplineCurve::try_new(knots.clone(), points.clone()));
        arithmetic(knots.try_bspline_basis_functions(1, 0, 0.));
    }
    let quadratic = KnotVec::from(vec![0., 0., 0., 1e-200, 1e-200, 1e-200]);
    arithmetic(BSplineCurve::try_new(
        quadratic.clone(),
        vec![
            Point3::origin(),
            Point3::new(1., 2., 0.),
            Point3::new(3., 0., 1.),
        ],
    ));
    assert!(quadratic.try_bspline_basis_functions(2, 0, 5e-201).is_ok());
    arithmetic(quadratic.try_bspline_basis_functions(2, 2, 5e-201));
    let narrow = KnotVec::from(vec![0., 0., 1e-200, 1e-200]);
    arithmetic(BSplineSurface::try_new(
        (narrow.clone(), narrow),
        vec![points.clone(), points],
    ));
    let mut collision = KnotVec::from(vec![-1., -1., 0., 1e-20, 1., 1.]);
    let before = collision.clone();
    assert!(matches!(
        collision.try_normalize(),
        Err(Error::CollapsedKnotSpan)
    ));
    assert_eq!(collision, before);
}

#[test]
fn a_tiny_interior_span_with_wide_degree_supports_remains_supported() {
    let knots = KnotVec::from(vec![-1., -1., -1., 0., 1e-200, 0.5, 1., 1., 1.]);
    let points = (0..6)
        .map(|i| Point3::new(i as f64, (i * i) as f64, (i % 2) as f64))
        .collect();
    let curve = BSplineCurve::try_new(knots, points).unwrap();
    for t in [-0.75, -0.1, 0., 5e-201, 0.25, 0.75, 1.] {
        let p = curve.subs(t);
        assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
        for d in [curve.der(t), curve.der2(t)] {
            assert!(d.x.is_finite() && d.y.is_finite() && d.z.is_finite());
        }
    }
}
