use truck_geometry::errors::Error;
use truck_geometry::prelude::*;

#[test]
fn finite_extrapolation_overflow_is_ieee_infallibly_and_diagnosed_fallibly() {
    let knots = KnotVec::bezier_knot(3);
    let basis = knots.bspline_basis_functions(3, 0, 1e200);
    assert!(basis.as_slice().iter().any(|x| !x.is_finite()));
    assert!(knots.try_bspline_basis_functions(3, 0, 1e200).is_err());
}

#[test]
fn nonfinite_parameters_are_ieee_infallibly_and_rejected_fallibly() {
    let knots = KnotVec::bezier_knot(3);
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let basis = knots.bspline_basis_functions(3, 0, t);
        assert!(basis.as_slice().iter().any(|x| !x.is_finite()));
        assert!(knots.try_bspline_basis_functions(3, 0, t).is_err());
    }
}

#[test]
fn basis_paths_share_finite_values_and_preserve_structural_errors() {
    let knots = KnotVec::bezier_knot(3);
    for rank in 0..=4 {
        for t in [-1., 0., 0.3, 1., 2.] {
            assert_eq!(
                knots.bspline_basis_functions(3, rank, t).to_full_array(),
                knots
                    .try_bspline_basis_functions(3, rank, t)
                    .unwrap()
                    .to_full_array()
            );
        }
    }
    let zero = KnotVec::from(vec![0., 0., 0., 0.]);
    assert!(matches!(
        zero.try_bspline_basis_functions(1, 0, 0.),
        Err(Error::ZeroRange)
    ));
    assert!(std::panic::catch_unwind(|| zero.bspline_basis_functions(1, 0, 0.)).is_err());
    assert!(matches!(
        knots.try_bspline_basis_functions(8, 0, 0.),
        Err(Error::TooLargeDegree(..))
    ));
    assert!(std::panic::catch_unwind(|| knots.bspline_basis_functions(8, 0, 0.)).is_err());
}
