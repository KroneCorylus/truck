use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use truck_geometry::prelude::*;

#[derive(Clone, Debug)]
struct CountedLine {
    line: Line<Point3>,
    calls: Arc<AtomicUsize>,
}

impl ParametricCurve for CountedLine {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Point3 {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.line.subs(t)
    }
    fn der_n(&self, n: usize, t: f64) -> Vector3 {
        if n == 0 {
            self.subs(t).to_vec()
        } else {
            self.line.der_n(n, t)
        }
    }
    fn der(&self, t: f64) -> Vector3 { self.line.der(t) }
    fn der2(&self, t: f64) -> Vector3 { self.line.der2(t) }
    fn parameter_range(&self) -> ParameterRange { self.line.parameter_range() }
}
impl BoundedCurve for CountedLine {}
impl Invertible for CountedLine {
    fn invert(&mut self) { self.line.invert(); }
}
impl Cut for CountedLine {
    fn cut(&mut self, t: f64) -> Self {
        Self {
            line: self.line.cut(t),
            calls: self.calls.clone(),
        }
    }
}
impl Transformed<Matrix4> for CountedLine {
    fn transform_by(&mut self, matrix: Matrix4) { self.line.transform_by(matrix); }
}

fn planes() -> [Plane; 2] {
    [
        Plane::new(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ),
        Plane::new(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        ),
    ]
}

#[test]
fn unchanged_clones_reuse_division_and_mutations_detach() {
    let calls = Arc::new(AtomicUsize::new(0));
    let [a, b] = planes();
    let curve = IntersectionCurve::new(
        a,
        b,
        CountedLine {
            line: Line(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
            calls: calls.clone(),
        },
    );
    let original = curve.parameter_division((0.0, 1.0), 0.01);
    let n = calls.load(Ordering::Relaxed);
    assert!(n > 0);
    let mut clone = curve.clone();
    assert_eq!(clone.parameter_division((0.0, 1.0), 0.01), original);
    assert_eq!(
        calls.load(Ordering::Relaxed),
        n,
        "unchanged clone re-sampled its curve"
    );
    clone.leader_mut().line.1.x = 2.0;
    let changed = clone.parameter_division((0.0, 1.0), 0.01);
    assert_ne!(changed, original);
    let n = calls.load(Ordering::Relaxed);
    assert_eq!(curve.parameter_division((0.0, 1.0), 0.01), original);
    assert_eq!(
        calls.load(Ordering::Relaxed),
        n,
        "changing a clone invalidated the original"
    );
    let check = |curve: &IntersectionCurve<CountedLine, Plane, Plane>, range, tol| {
        let n = calls.load(Ordering::Relaxed);
        let cached = curve.parameter_division(range, tol);
        assert!(
            calls.load(Ordering::Relaxed) > n,
            "mutation or changed key did not invalidate"
        );
        assert_eq!(cached, algo::curve::parameter_division(curve, range, tol));
    };
    check(&clone, (0.0, 0.5), 0.01);
    check(&clone, (0.0, 0.5), 0.02);
    *clone.surface0_mut() = a.transformed(Matrix4::from_translation(Vector3::unit_z()));
    check(&clone, (0.0, 0.5), 0.02);
    *clone.surface1_mut() = b.transformed(Matrix4::from_translation(Vector3::unit_y()));
    check(&clone, (0.0, 0.5), 0.02);
    clone.invert();
    check(&clone, (0.0, 0.5), 0.02);
    clone.transform_by(Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0)));
    check(&clone, (0.0, 0.5), 0.02);
    let tail = clone.cut(0.5);
    check(&clone, (0.0, 1.0), 0.02);
    check(&tail, (0.0, 1.0), 0.02);
}
