use proptest::{prelude::*, property_test};
use std::f64::consts::PI;
use std::ops::RangeBounds;
use truck_geometry::prelude::*;

#[property_test]
fn surface(
    #[strategy = 1f64..=5f64] small_radius: f64,
    #[strategy = 0.1f64..=5f64] radius_diff: f64,
    #[strategy = (0f64..=2.0 * PI, 0f64..=2.0 * PI)] (u, v): (f64, f64),
    #[strategy = -0.4f64..=0.4] deform: f64,
) {
    const EPS: f64 = 1.0e-2;
    let large_radius = 2.0 * small_radius + radius_diff;
    let torus = Torus::new(Point3::origin(), large_radius, small_radius);

    let p = torus.subs(u, v);
    let r = large_radius * Point3::new(f64::cos(u), f64::sin(u), 0.0);
    prop_assert_near!(p.distance(r), small_radius);
    prop_assert_near!(p.z, small_radius * f64::sin(v));

    let uder0 = torus.uder(u, v);
    let uder1 = (torus.subs(u + EPS, v) - torus.subs(u - EPS, v)) / (2.0 * EPS);
    prop_assert!((uder0 - uder1).magnitude() < EPS, "{:?} {:?}", uder0, uder1);

    let vder0 = torus.vder(u, v);
    let vder1 = (torus.subs(u, v + EPS) - torus.subs(u, v - EPS)) / (2.0 * EPS);
    prop_assert!((vder0 - vder1).magnitude() < EPS, "{:?} {:?}", vder0, vder1);

    let uuder0 = torus.uuder(u, v);
    let uuder1 = (torus.uder(u + EPS, v) - torus.uder(u - EPS, v)) / (2.0 * EPS);
    prop_assert!(
        (uuder0 - uuder1).magnitude() < EPS,
        "{:?} {:?}",
        uuder0,
        uuder1
    );

    let uvder0 = torus.uvder(u, v);
    let uvder1 = (torus.vder(u + EPS, v) - torus.vder(u - EPS, v)) / (2.0 * EPS);
    prop_assert!(
        (uvder0 - uvder1).magnitude() < EPS,
        "{:?} {:?}",
        uvder0,
        uvder1
    );

    let vvder0 = torus.vvder(u, v);
    let vvder1 = (torus.vder(u, v + EPS) - torus.vder(u, v - EPS)) / (2.0 * EPS);
    prop_assert!(
        (vvder0 - vvder1).magnitude() < EPS,
        "{:?} {:?}",
        vvder0,
        vvder1
    );

    let n0 = torus.normal(u, v);
    let n1 = torus.uder(u, v).cross(torus.vder(u, v)).normalize();
    prop_assert_near!(n0, n1);

    let (u0, v0) = torus.search_parameter(p, None, 1).unwrap();
    let (urange, vrange) = torus.parameter_range();
    prop_assert!(urange.contains(&u0) && vrange.contains(&v0), "{u0}, {v0}");
    prop_assert_near!(torus.subs(u0, v0), p);

    let deform = deform * small_radius;
    let q = p + deform * n0;
    let (u0, v0) = torus.search_nearest_parameter(q, None, 1).unwrap();
    let (urange, vrange) = torus.parameter_range();
    prop_assert!(urange.contains(&u0) && vrange.contains(&v0), "{u0}, {v0}");
    prop_assert_near!(torus.subs(u0, v0), p);
}

#[property_test]
fn test_der_mn(
    #[strategy = (0f64..=PI, 0f64..=2.0 * PI)] (u, v): (f64, f64),
    #[strategy = (0usize..=4, 0usize..=4)] (m, n): (usize, usize),
    #[strategy = prop::array::uniform3(-100f64..=100.0)] center: [f64; 3],
    #[strategy = 6.0f64..=10.0] large_radius: f64,
    #[strategy = 0.1f64..=5.0] small_radius: f64,
    #[strategy = prop::bool::ANY] u_derivate: bool,
) {
    let torus = Torus::new(Point3::from(center), large_radius, small_radius);

    const EPS: f64 = 1.0e-4;
    let (der0, der1) = if u_derivate {
        let der0 = torus.der_mn(m + 1, n, u, v);
        let der1 = (torus.der_mn(m, n, u + EPS, v) - torus.der_mn(m, n, u - EPS, v)) / (2.0 * EPS);
        (der0, der1)
    } else {
        let der0 = torus.der_mn(m, n + 1, u, v);
        let der1 = (torus.der_mn(m, n, u, v + EPS) - torus.der_mn(m, n, u, v - EPS)) / (2.0 * EPS);
        (der0, der1)
    };
    prop_assert!((der0 - der1).magnitude() < 0.01 * der0.magnitude());
}

#[test]
fn torus_point_search_preserves_axis_and_near_seam_accuracy() {
    for scale in [0.01, 1., 100.] {
        let center = Point3::new(13., -7., 23.);
        let torus = Torus::new(center, 200. * scale, 50. * scale);
        for u0 in [0., PI / 2., PI, 3. * PI / 2., 2. * PI] {
            for v0 in [0., PI / 2., PI, 3. * PI / 2., 2. * PI] {
                for delta in [-1e-8, 0., 1e-8, -0.001264731342027] {
                    let (u, v) = (u0 + delta, v0 + delta);
                    let point = torus.subs(u, v);
                    let (a, b) = torus
                        .search_parameter(point, None, 100)
                        .expect("a regular torus point must project onto its support");
                    assert!(torus.subs(a, b).distance(point) <= 1e-10 * scale);
                    let displaced = point + torus.normal(u, v) * scale;
                    let (a, b) = torus
                        .search_nearest_parameter(displaced, None, 100)
                        .unwrap();
                    assert!(torus.subs(a, b).distance(point) <= 1e-10 * scale);
                    assert!(torus.search_parameter(displaced, None, 100).is_none());
                }
            }
        }
        let point = center + Vector3::new(249.99996001137114, 0., -0.06323655024303196) * scale;
        let (u, v) = torus.search_parameter(point, None, 100).unwrap();
        assert!(torus.subs(u, v).distance(point) <= 1e-10 * scale);
    }
}

#[test]
fn torus_normal_derivatives_match_independent_finite_differences() {
    let torus = Torus::new(Point3::new(13., -7., 23.), 200., 50.);
    let step = 1e-5;
    for u in [0.1, 0.7, 2.1, 4.3] {
        for v in [0.2, 0.9, 2.3, 5.4] {
            let uder = (torus.normal(u + step, v) - torus.normal(u - step, v)) / (2. * step);
            let vder = (torus.normal(u, v + step) - torus.normal(u, v - step)) / (2. * step);
            assert!(torus.normal_uder(u, v).distance(uder) < 1e-9);
            assert!(torus.normal_vder(u, v).distance(vder) < 1e-9);
            assert!(torus.normal(u, v).dot(torus.normal_uder(u, v)).abs() < 1e-12);
            assert!(torus.normal(u, v).dot(torus.normal_vder(u, v)).abs() < 1e-12);
        }
    }
}
