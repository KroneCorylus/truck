use truck_base::cgmath64::*;
use truck_geotrait::*;

macro_rules! finite_plane {
    ($name:ident, $point:ty, $vector:ty, $position:expr, $tangent:expr) => {
        #[derive(Clone)]
        struct $name;
        impl ParametricSurface for $name {
            type Point = $point;
            type Vector = $vector;
            fn subs(&self, u: f64, v: f64) -> Self::Point {
                assert!(
                    u.is_finite() && v.is_finite(),
                    "nonfinite surface evaluation"
                );
                ($position)(u, v)
            }
            fn uder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 0, u, v) }
            fn vder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 1, u, v) }
            fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(2, 0, u, v) }
            fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 1, u, v) }
            fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 2, u, v) }
            fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
                assert!(
                    u.is_finite() && v.is_finite(),
                    "nonfinite surface derivative"
                );
                match (m, n) {
                    (0, 0) => self.subs(u, v).to_vec(),
                    (1, 0) => $tangent,
                    (0, 1) => <$vector>::unit_y(),
                    _ => <$vector>::zero(),
                }
            }
        }
    };
}
finite_plane!(
    Plane2,
    Point2,
    Vector2,
    |u, v| Point2::new(u * 1e-200, v),
    Vector2::new(1e-200, 0.)
);
finite_plane!(
    Plane3,
    Point3,
    Vector3,
    |u, v| Point3::new(u * 1e-100, v, 0.),
    Vector3::new(1e-100, 0., 0.)
);
impl ParametricSurface3D for Plane3 {}

#[derive(Clone)]
struct FiniteLine {
    origin: Point3,
    tangent: Vector3,
}
impl ParametricCurve for FiniteLine {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Point3 {
        assert!(t.is_finite(), "nonfinite curve evaluation");
        self.origin + self.tangent * t
    }
    fn der(&self, t: f64) -> Vector3 { self.der_n(1, t) }
    fn der2(&self, t: f64) -> Vector3 { self.der_n(2, t) }
    fn der_n(&self, n: usize, t: f64) -> Vector3 {
        assert!(t.is_finite(), "nonfinite curve derivative");
        match n {
            0 => self.subs(t).to_vec(),
            1 => self.tangent,
            _ => Vector3::zero(),
        }
    }
}
fn thin_line() -> FiniteLine {
    FiniteLine {
        origin: Point3::origin(),
        tangent: Vector3::new(1e-100, 0., 0.),
    }
}

#[test]
fn nearest_surface_rejects_nonfinite_seeds() {
    for hint in [(f64::NAN, 0.), (0., f64::INFINITY), (f64::NEG_INFINITY, 0.)] {
        assert!(
            algo::surface::search_nearest_parameter(&Plane2, Point2::origin(), hint, 10).is_none()
        );
        assert!(
            algo::surface::search_nearest_parameter(&Plane3, Point3::origin(), hint, 10).is_none()
        );
    }
}
#[test]
fn nearest_surface_2d_rejects_overflowed_iterates() {
    assert!(
        algo::surface::search_nearest_parameter(&Plane2, Point2::new(1e200, 1.), (0., 0.), 10)
            .is_none()
    );
    let uv =
        algo::surface::search_nearest_parameter(&Plane2, Point2::new(0.3e-200, 0.4), (0., 0.), 10)
            .unwrap();
    assert!((uv.0 - 0.3).abs() < 1e-12 && (uv.1 - 0.4).abs() < 1e-12);
    assert_eq!(
        algo::surface::search_nearest_parameter(&Plane2, Point2::origin(), (0., 0.), 10),
        Some((0., 0.))
    );
}
#[test]
fn nearest_surface_3d_rejects_overflowed_iterates() {
    assert!(algo::surface::search_nearest_parameter(
        &Plane3,
        Point3::new(1e250, 1., 0.),
        (0., 0.),
        10
    )
    .is_none());
    let uv = algo::surface::search_nearest_parameter(
        &Plane3,
        Point3::new(0.3e-100, 0.4, 0.),
        (0., 0.),
        10,
    )
    .unwrap();
    assert!((uv.0 - 0.3).abs() < 1e-12 && (uv.1 - 0.4).abs() < 1e-12);
}
#[test]
fn curve_exact_rejects_nonfinite_seeds() {
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(algo::curve::search_parameter(&thin_line(), Point3::origin(), t, 10).is_none());
    }
}
#[test]
fn curve_nearest_rejects_nonfinite_seeds() {
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            algo::curve::search_nearest_parameter(&thin_line(), Point3::origin(), t, 10).is_none()
        );
    }
}
#[test]
fn curve_exact_rejects_overflowed_iterates() {
    assert!(
        algo::curve::search_parameter(&thin_line(), Point3::new(1e250, 0., 0.), 0., 10).is_none()
    );
    let t =
        algo::curve::search_parameter(&thin_line(), Point3::new(0.3e-100, 0., 0.), 0., 10).unwrap();
    assert!((t - 0.3).abs() < 1e-12);
}
#[test]
fn curve_nearest_rejects_overflowed_iterates() {
    assert!(algo::curve::search_nearest_parameter(
        &thin_line(),
        Point3::new(1e250, 0., 0.),
        0.,
        10
    )
    .is_none());
    let t =
        algo::curve::search_nearest_parameter(&thin_line(), Point3::new(0.3e-100, 0., 0.), 0., 10)
            .unwrap();
    assert!((t - 0.3).abs() < 1e-12);
}
#[test]
fn surface_curve_intersection_rejects_nonfinite_seeds() {
    let curve = FiniteLine {
        origin: Point3::new(0., 0., 1.),
        tangent: Vector3::unit_z(),
    };
    for (uv, t) in [
        ((f64::NAN, 0.), 0.),
        ((0., f64::INFINITY), 0.),
        ((0., 0.), f64::NEG_INFINITY),
    ] {
        assert!(algo::surface::search_intersection_parameter(&Plane3, uv, &curve, t, 10).is_none());
    }
}
#[test]
fn surface_curve_intersection_rejects_overflowed_iterates() {
    let curve = FiniteLine {
        origin: Point3::new(1e250, 0., 1.),
        tangent: Vector3::unit_z(),
    };
    assert!(
        algo::surface::search_intersection_parameter(&Plane3, (0., 0.), &curve, 0., 10).is_none()
    );
    let curve = FiniteLine {
        origin: Point3::new(0.3e-100, 0.4, 1.),
        tangent: Vector3::unit_z(),
    };
    let (uv, t) =
        algo::surface::search_intersection_parameter(&Plane3, (0., 0.), &curve, 0., 10).unwrap();
    assert!((uv.0 - 0.3).abs() < 1e-12 && (uv.1 - 0.4).abs() < 1e-12 && (t + 1.).abs() < 1e-12);
}
