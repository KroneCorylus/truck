use super::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy)]
enum Fault {
    None,
    Failure,
    WrongPoint,
    Nonfinite,
    OutsideRange,
}

#[derive(Clone)]
struct Counted {
    surface: Surface,
    calls: Rc<RefCell<Vec<(Point3, bool)>>>,
    fault: Fault,
}

impl ParametricSurface for Counted {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64, v: f64) -> Point3 { self.surface.subs(u, v) }
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.surface.uder(u, v) }
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.surface.vder(u, v) }
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.surface.uuder(u, v) }
    fn uvder(&self, u: f64, v: f64) -> Vector3 { self.surface.uvder(u, v) }
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.surface.vvder(u, v) }
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        self.surface.der_mn(m, n, u, v)
    }
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { self.surface.parameter_range() }
}
impl ParametricSurface3D for Counted {}

impl SearchParameter<D2> for Counted {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = hint.into();
        let seeded = matches!(hint, SPHint2D::Parameter(..));
        self.calls.borrow_mut().push((point, seeded));
        if seeded {
            match self.fault {
                Fault::None => {}
                Fault::Failure => return None,
                Fault::WrongPoint => return Some((0.5, 0.5)),
                Fault::Nonfinite => return Some((f64::NAN, f64::INFINITY)),
                Fault::OutsideRange => {
                    return self
                        .surface
                        .search_parameter(point, None, trials)
                        .map(|(u, v)| (u + 4. * TAU, v));
                }
            }
        }
        self.surface.search_parameter(point, hint, trials)
    }
}

fn fillet() -> Surface {
    let rolling = RbfSurface::new(
        Line(Point3::origin(), Point3::new(4., 0., 0.)),
        Surface::Plane(Plane::xy()),
        Surface::Plane(Plane::zx()),
        0.5,
    );
    let fillet =
        ApproxFilletSurface::approx_rolling_ball_fillet(&rolling, (0., 1.), 0.001).unwrap();
    Surface::Fillet(Processor::new(fillet.into()))
}

fn pcurve(surface: &Surface, a: Point2, b: Point2) -> Curve {
    Curve::PCurve(PCurve::new(
        BSplineCurve::from(Line(a, b)),
        Box::new(surface.clone()),
    ))
}

#[test]
fn sampled_inclusion_reuses_searches_without_dropping_samples() {
    let surface = Counted {
        surface: fillet(),
        calls: Rc::default(),
        fault: Fault::None,
    };
    for reversed in [false, true] {
        let mut curve = pcurve(&surface.surface, (0., 0.).into(), (0., 1.).into());
        if reversed {
            curve.invert();
        }
        surface.calls.borrow_mut().clear();
        assert!(sampled_include(&surface, &curve));
        let calls = surface.calls.borrow();
        let cold = calls.iter().filter(|(_, seeded)| !seeded).count();
        assert!(cold <= 3, "repeated full-surface searches: {cold}");
        let (a, b) = curve.range_tuple();
        for i in 0..=32 {
            let point = curve.subs(a + (b - a) * i as f64 / 32.);
            assert!(calls.iter().any(|(p, _)| p.near(&point)));
        }
    }
}

#[test]
fn bad_seeded_results_fall_back_and_off_surface_curves_stay_rejected() {
    let sphere = Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), 2.)));
    for base in [fillet(), sphere] {
        for fault in [
            Fault::Failure,
            Fault::WrongPoint,
            Fault::Nonfinite,
            Fault::OutsideRange,
        ] {
            let surface = Counted {
                surface: base.clone(),
                calls: Rc::default(),
                fault,
            };
            let curve = pcurve(&base, (0.1, 0.2).into(), (0.8, 0.7).into());
            assert!(sampled_include(&surface, &curve));
            let calls = surface.calls.borrow();
            assert!(calls.iter().any(|(_, seeded)| *seeded));
            assert!(calls.iter().filter(|(_, seeded)| !seeded).count() > 1);
            drop(calls);
            let off_surface =
                curve.transformed(Matrix4::from_translation(Vector3::new(0., 0., 10.)));
            assert!(!sampled_include(&surface, &off_surface));
            let leaves_surface = Curve::Line(Line(base.subs(0.1, 0.2), Point3::new(10., 10., 10.)));
            assert!(!sampled_include(&surface, &leaves_surface));
        }
    }
}

#[test]
fn inclusion_preserves_periodic_seams_poles_and_transformed_curves() {
    let sphere = Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), 2.)));
    let transform = Matrix4::from_translation(Vector3::new(30., -20., 10.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        * Matrix4::from_nonuniform_scale(0.5, 1.5, 2.);
    for surface in [fillet(), sphere] {
        let surface = surface.transformed(transform);
        let paths = if matches!(surface, Surface::Sphere(_)) {
            vec![
                ((TAU - 0.2, 0.5), (TAU + 0.2, 0.5)),
                ((0.2, 0.), (2., 0.)),
                ((0.2, 0.1), (0.8, 1.2)),
            ]
        } else {
            vec![((0., 0.), (0., 1.)), ((0.1, 0.1), (0.9, 0.9))]
        };
        for (a, b) in paths {
            let mut curve = pcurve(&surface, a.into(), b.into());
            for _ in 0..2 {
                let (a, b) = curve.range_tuple();
                assert!((0..=32).all(|i| surface
                    .search_parameter(curve.subs(a + (b - a) * i as f64 / 32.), None, 100)
                    .is_some()));
                assert!(surface.include(&curve));
                curve.invert();
            }
        }
    }
}
