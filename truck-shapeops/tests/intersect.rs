//! The curves where two surfaces meet, checked on the curves alone.

use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::*;
use truck_shapeops::local::{intersect_surfaces, parameter_domain};

const TOL: f64 = 0.001;

fn unit_square_face() -> Face {
    let v: Vec<_> = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
        .into();
    let wire: Wire = (0..4)
        .map(|i| builder::line(&v[i], &v[(i + 1) % 4]))
        .collect();
    builder::try_attach_plane(&[wire]).unwrap()
}

/// Full cylinder of `radius` about the line through `origin` along `axis`, as one extruded
/// circle.
fn cylinder(origin: Point3, axis: Vector3, radius: f64) -> Surface {
    let x = if axis.x.abs() < 0.9 {
        axis.cross(Vector3::unit_x()).normalize()
    } else {
        axis.cross(Vector3::unit_y()).normalize()
    };
    let y = axis.cross(x);
    let matrix = Matrix4::from_cols(
        (x * radius).extend(0.0),
        (y * radius).extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    );
    let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, 2.0 * PI));
    let curve = Curve::Conic(Processor::with_transform(circle, matrix));
    Surface::Extruded(ExtrudedCurve::by_extrusion(curve, axis))
}

fn distance_to(surface: &Surface, p: Point3) -> f64 {
    let (u, v) = surface.search_nearest_parameter(p, None, 100).unwrap();
    surface.subs(u, v).distance(p)
}

#[test]
fn plane_against_plane_is_an_exact_line() {
    let square = unit_square_face();
    let domain = parameter_domain(&square, 0.0).unwrap();
    let wall = Surface::Plane(Plane::new(
        Point3::new(0.5, 0.0, 0.0),
        Point3::new(0.5, 1.0, 0.0),
        Point3::new(0.5, 0.0, 1.0),
    ));
    let curves = intersect_surfaces(
        &square.surface(),
        domain,
        &wall,
        ((-1.0, 2.0), (-1.0, 2.0)),
        TOL,
    )
    .unwrap();
    let [Curve::Line(Line(p, q))] = curves.as_slice() else {
        panic!("{curves:?}");
    };
    let (lo, hi) = (Point3::new(0.5, 0.0, 0.0), Point3::new(0.5, 1.0, 0.0));
    assert!(
        (p.near(&lo) && q.near(&hi)) || (p.near(&hi) && q.near(&lo)),
        "{p:?} {q:?}"
    );
}

#[test]
fn plane_against_cylinder_is_an_exact_circle() {
    let cylinder = cylinder(Point3::origin(), Vector3::unit_z(), 1.0);
    let plane = Surface::Plane(Plane::new(
        Point3::new(0.0, 0.0, 0.5),
        Point3::new(1.0, 0.0, 0.5),
        Point3::new(0.0, 1.0, 0.5),
    ));
    let curves = intersect_surfaces(
        &plane,
        ((-2.0, 2.0), (-2.0, 2.0)),
        &cylinder,
        ((0.0, 2.0 * PI), (-1.0, 1.0)),
        TOL,
    )
    .unwrap();
    let [curve @ Curve::Conic(_)] = curves.as_slice() else {
        panic!("{curves:?}");
    };
    let (t0, t1) = curve.range_tuple();
    assert!((t1 - t0 - 2.0 * PI).abs() < 1e-12);
    for i in 0..20 {
        let p = curve.subs(t0 + (t1 - t0) * i as f64 / 20.0);
        assert!((p.x * p.x + p.y * p.y - 1.0).abs() < 1e-12 && (p.z - 0.5).abs() < 1e-12);
    }
}

#[test]
fn two_cylinders_meet_along_a_curve_on_both() {
    let big = cylinder(Point3::origin(), Vector3::unit_z(), 1.0);
    let small = cylinder(Point3::new(0.0, 0.0, 0.0), Vector3::unit_x(), 0.5);
    let curves = intersect_surfaces(
        &big,
        ((0.0, 2.0 * PI), (-1.0, 1.0)),
        &small,
        ((0.0, 2.0 * PI), (-2.0, 2.0)),
        TOL,
    )
    .unwrap();
    assert!(!curves.is_empty());
    for curve in &curves {
        assert!(matches!(curve, Curve::IntersectionCurve(_)), "{curve:?}");
        let (t0, t1) = curve.range_tuple();
        for i in 0..=20 {
            let p = curve.subs(t0 + (t1 - t0) * i as f64 / 20.0);
            assert!(distance_to(&big, p) < TOLERANCE && distance_to(&small, p) < TOLERANCE);
        }
    }
}
