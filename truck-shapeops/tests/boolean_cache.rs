mod common;

use common::{modeling::*, *};
use truck_geometry::prelude::algo;
use truck_meshalgo::prelude::*;
use truck_modeling::*;

const TOL: f64 = 0.01;
const HOLES: usize = 6;
const RADIUS: f64 = 0.4;

#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Cut,
    Invertible,
    SearchParameterD1,
    SearchNearestParameterD1,
)]
struct Uncached(Curve);

impl ParameterDivision1D for Uncached {
    type Point = Point3;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        match &self.0 {
            Curve::IntersectionCurve(curve) => algo::curve::parameter_division(curve, range, tol),
            curve => curve.parameter_division(range, tol),
        }
    }
}
impl From<IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>> for Uncached {
    fn from(curve: IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>) -> Self {
        Self(curve.into())
    }
}

type GenericSolid<C> = truck_topology::Solid<Point3, C, Surface>;

fn subtract_all<C: truck_shapeops::ShapeOpsCurve<Surface>>(
    mut plate: GenericSolid<C>,
    holes: Vec<GenericSolid<C>>,
) -> GenericSolid<C> {
    assert_solid(&plate, 24.0, &[0], TOL);
    for (i, mut hole) in holes.into_iter().enumerate() {
        assert_solid(&hole, cylinder_volume(RADIUS, 3.0), &[0], TOL);
        hole.not();
        assert_solid(&hole, -cylinder_volume(RADIUS, 3.0), &[0], TOL);
        plate = truck_shapeops::and(&plate, &hole, TOL).unwrap();
        assert_solid(
            &plate,
            24.0 - (i + 1) as f64 * cylinder_volume(RADIUS, 1.0),
            &[i + 1],
            TOL,
        );
    }
    plate
}

/// The acceleration reuses exactly the same sampled points. Running without reuse must
/// produce the same topology and volume, including after repeated cuts of the growing face.
#[test]
fn cached_and_uncached_booleans_are_identical() {
    let plate = cuboid(Point3::origin(), Point3::new(6.0, 4.0, 1.0));
    let holes: Vec<_> = (0..HOLES)
        .map(|i| {
            cylinder(
                Point3::new(1.0 + 2.0 * (i % 3) as f64, 1.0 + 2.0 * (i / 3) as f64, -1.0),
                Vector3::unit_z(),
                RADIUS,
                3.0,
            )
        })
        .collect();
    let uncached_plate = plate.mapped(|p| *p, |c| Uncached(c.clone()), Clone::clone);
    let uncached_holes = holes
        .iter()
        .map(|hole| hole.mapped(|p| *p, |c| Uncached(c.clone()), Clone::clone))
        .collect();
    let cached = subtract_all(plate, holes);
    let uncached = subtract_all(uncached_plate, uncached_holes);
    assert_counts(&cached, 8 + 4 * HOLES, 12 + 6 * HOLES, 6 + 2 * HOLES);
    assert_counts(&uncached, 8 + 4 * HOLES, 12 + 6 * HOLES, 6 + 2 * HOLES);
    let volume0 = cached.triangulation(TOL).to_polygon().volume();
    let volume1 = uncached.triangulation(TOL).to_polygon().volume();
    assert!(
        (volume0 - volume1).abs() < 1.0e-12,
        "volumes {volume0} and {volume1}"
    );
    let mut points0: Vec<_> = cached.vertex_iter().map(|v| v.point()).collect();
    let mut points1: Vec<_> = uncached.vertex_iter().map(|v| v.point()).collect();
    let compare = |a: &Point3, b: &Point3| {
        a.x.total_cmp(&b.x)
            .then(a.y.total_cmp(&b.y))
            .then(a.z.total_cmp(&b.z))
    };
    points0.sort_by(compare);
    points1.sort_by(compare);
    assert_eq!(points0.len(), points1.len());
    for (a, b) in points0.into_iter().zip(points1) {
        assert!(a.distance(b) < 1.0e-12);
    }
}

#[test]
fn sampling_does_not_change_equality_or_serialization() {
    let a = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let b = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
    );
    let curve = IntersectionCurve::new(a, b, Line(Point3::origin(), Point3::new(1.0, 0.0, 0.0)));
    let json = serde_json::to_string(&curve).unwrap();
    curve.parameter_division((0.0, 1.0), 0.01);
    assert_eq!(serde_json::to_string(&curve).unwrap(), json);
    let restored: IntersectionCurve<Line<Point3>, Plane, Plane> =
        serde_json::from_str(&json).unwrap();
    assert_eq!(curve, restored);
    assert_eq!(
        curve.parameter_division((0.0, 1.0), 0.01),
        restored.parameter_division((0.0, 1.0), 0.01)
    );
}
