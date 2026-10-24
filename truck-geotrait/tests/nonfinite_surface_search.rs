use truck_base::cgmath64::*;
use truck_geotrait::*;

#[derive(Clone)]
struct FinitePlane;

impl ParametricSurface for FinitePlane {
    type Point = Point2;
    type Vector = Vector2;
    fn subs(&self, u: f64, v: f64) -> Point2 {
        assert!(u.is_finite() && v.is_finite());
        Point2::new(u * 1e-200, v)
    }
    fn uder(&self, _: f64, _: f64) -> Vector2 { Vector2::new(1e-200, 0.) }
    fn vder(&self, _: f64, _: f64) -> Vector2 { Vector2::unit_y() }
    fn uuder(&self, _: f64, _: f64) -> Vector2 { Vector2::zero() }
    fn uvder(&self, _: f64, _: f64) -> Vector2 { Vector2::zero() }
    fn vvder(&self, _: f64, _: f64) -> Vector2 { Vector2::zero() }
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector2 {
        match (m, n) {
            (0, 0) => self.subs(u, v).to_vec(),
            (1, 0) => self.uder(u, v),
            (0, 1) => self.vder(u, v),
            _ => Vector2::zero(),
        }
    }
}

#[test]
fn surface_search_rejects_nonfinite_seeds_before_evaluation() {
    for hint in [(f64::NAN, 0.), (0., f64::INFINITY), (f64::NEG_INFINITY, 0.)] {
        assert!(
            algo::surface::search_parameter(&FinitePlane, Point2::origin(), hint, 10).is_none()
        );
    }
}

#[test]
fn surface_search_stops_before_evaluating_an_overflowed_newton_step() {
    assert!(
        algo::surface::search_parameter(&FinitePlane, Point2::new(1e200, 1.), (0., 0.), 10)
            .is_none()
    );
    let parameter =
        algo::surface::search_parameter(&FinitePlane, Point2::new(0.3e-200, 0.4), (0., 0.), 10)
            .unwrap();
    assert!((parameter.0 - 0.3).abs() < 1e-12);
    assert!((parameter.1 - 0.4).abs() < 1e-12);
}
