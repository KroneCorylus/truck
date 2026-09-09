use proptest::{prelude::*, property_test};
use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[property_test]
fn sphere_case(#[strategy = 0f64..=1.0] t: f64) {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
    let bsp = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
        ],
    );
    let curve = IntersectionCurve::new(sphere0, sphere1, bsp);
    let p = curve.subs(t);
    let v = curve.der(t);

    prop_assert_near!(p.to_vec().magnitude(), 1.0);
    prop_assert!(p.to_vec().dot(v).so_small());

    let t0 = match curve.search_parameter(p, None, 100) {
        Some(t0) => t0,
        None => {
            let reason = "search_parameter failed".into();
            return Err(TestCaseError::Fail(reason));
        }
    };
    prop_assert_near!(t, t0);
}

#[property_test]
fn cylinder_case(#[strategy = 0.0..=2.0 * PI] t: f64, #[strategy = 0usize..=4] n: usize) {
    let line0 = Line(Point3::new(1.0, 0.0, 2.0), Point3::new(-1.0, 0.0, 2.0));
    let cylinder0 = RevolutedCurve::by_revolution(line0, Point3::origin(), Vector3::unit_x());
    let line1 = Line(Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 0.0, -1.0));
    let cylinder1 = RevolutedCurve::by_revolution(line1, Point3::origin(), Vector3::unit_z());
    let z = (1.0 + f64::sqrt(3.0)) / 2.0;
    let lead_circle = Processor::with_transform(
        UnitCircle::<Point3>::new(),
        Matrix4::from_translation(z * Vector3::unit_z()),
    );
    let curve = IntersectionCurve::new(cylinder0, cylinder1, lead_circle);

    let p = curve.subs(t);
    prop_assert_near!(p.x * p.x + p.y * p.y, 1.0);
    prop_assert_near!(p.z * p.z + p.y * p.y, 4.0);

    let t0 = match curve.search_parameter(p, None, 100) {
        Some(t0) => t0,
        None => {
            let reason = "search_parameter failed".into();
            return Err(TestCaseError::Fail(reason));
        }
    };
    let diff = (t - t0).abs();
    prop_assert!(diff.near(&0.0) || diff.near(&(2.0 * PI)));

    const EPS: f64 = 1.0e-4;
    let v0 = curve.der_n(n + 1, t);
    let v1 = (curve.der_n(n, t + EPS) - curve.der_n(n, t - EPS)) / (2.0 * EPS);
    prop_assert!((v0 - v1).magnitude() < EPS * 10.0, "{v0:?} {v1:?}");

    let ders0 = (0..=n).map(|i| curve.der_n(i, t)).collect::<Vec<_>>();
    let ders1 = curve.ders(n, t);

    prop_assert_eq!(ders0.len(), ders1.len());
    let mut iter = ders0.into_iter().zip(&*ders1);
    iter.try_for_each(|(v0, v1)| {
        prop_assert_near!(v0, v1);
        Ok(())
    })?;
}

fn sphere_pair() -> IntersectionCurve<BSplineCurve<Point3>, Sphere, Sphere> {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
    let leader = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
        ],
    );
    IntersectionCurve::new(sphere0, sphere1, leader)
}

/// Walking the curve and seeding each step with the previous step's surface parameters is what
/// lets the boolean skip the presearch grid at every sample. The seeded solve finds the same
/// point as the unseeded one, but it follows the surface continuously instead of reseeding from
/// the surface's own range, so across a periodic seam it can name that point one full period
/// outside the range. The point is identical either way, which is why a caller checking only
/// the point cannot tell the two apart.
#[test]
fn hints_from_the_previous_sample_find_the_same_point_up_to_a_period() {
    let curve = sphere_pair();
    let mut wrapped = false;
    let mut hints: Option<(Point2, Point2)> = None;
    for i in 0..=20 {
        let t = f64::from(i) / 20.0;
        let (p, q0, q1) = curve.search_triple(t, 100).unwrap();
        if let Some((h0, h1)) = hints {
            let (hp, hq0, hq1) = curve
                .search_triple_with_hints(t, Some((h0.x, h0.y)), Some((h1.x, h1.y)), 100)
                .unwrap();
            assert_near!(hp, p);
            for d in [hq0.x - q0.x, hq0.y - q0.y, hq1.x - q1.x, hq1.y - q1.y] {
                assert!(
                    d.so_small() || d.abs().near(&(2.0 * PI)),
                    "parameter moved by {d}, which is neither nothing nor a period"
                );
                wrapped |= !d.so_small();
            }
        }
        hints = Some((q0, q1));
    }
    assert!(wrapped, "this leader is meant to cross the seam");
}

/// `newton::solve` stops when its step gets small, which on a near-singular Jacobian happens
/// while the residual is still large, so a bad seed can come back `Some` with parameters that
/// do not evaluate to the point it returned. Nothing here validates the triple, so callers that
/// supply hints have to check the result themselves rather than trust it.
#[test]
fn a_misleading_hint_can_return_a_triple_that_does_not_name_its_point() {
    let curve = sphere_pair();
    let (sphere0, sphere1) = (curve.surface0(), curve.surface1());
    let inconsistent = [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().any(|t| {
        [(0.0, 0.0), (3.0, -1.2), (-2.5, 0.9)].into_iter().any(|hint| {
            curve
                .search_triple_with_hints(t, Some(hint), Some(hint), 100)
                .is_some_and(|(p, q0, q1)| {
                    !sphere0.subs(q0.x, q0.y).near(&p) || !sphere1.subs(q1.x, q1.y).near(&p)
                })
        })
    });
    assert!(
        inconsistent,
        "a hint far from the sample no longer misleads the solver; if that is now guaranteed, \
         the checks in truck-shapeops that guard against it can be reconsidered"
    );
}
