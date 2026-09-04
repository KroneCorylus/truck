mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_shapeops::fillet::fillet_along_wire;

const TOL: f64 = 0.001;

/// Area of the section removed by a right-angle fillet and the distance of its centroid from
/// either wall, for use with Pappus's theorem.
fn fillet_section(radius: f64) -> (f64, f64) {
    let area = radius * radius * (1.0 - PI / 4.0);
    let centroid = radius * (10.0 - 3.0 * PI) / (12.0 - 3.0 * PI);
    (area, centroid)
}

/// The chain of edges of `shell` through `points`, oriented head to tail.
fn chain(shell: &Shell, points: &[Point3]) -> Wire {
    let edges: Vec<Edge> = points.iter().map(|&p| edge_through(shell, p)).collect();
    let mut wire = Wire::new();
    for (i, edge) in edges.iter().enumerate() {
        let forward = match wire.back_vertex() {
            Some(v) => edge.front() == v,
            None => edges
                .get(i + 1)
                .is_none_or(|next| edge.back() == next.front() || edge.back() == next.back()),
        };
        wire.push_back(if forward {
            edge.clone()
        } else {
            edge.inverse()
        });
    }
    assert!(wire.is_continuous());
    wire
}

/// Checks that faces sharing a contact edge or an interior cross edge have the same normal along
/// it. Edges with an intersection curve are the trimmed ends of an open chain and are skipped.
fn assert_tangent_along_blends(shell: &Shell, blends: std::ops::Range<usize>) {
    let normal_at = |face: &Face, p: Point3| {
        let surface = face.oriented_surface();
        let (u, v) = surface
            .search_parameter(p, None, 100)
            .expect("point not on face");
        surface.normal(u, v)
    };
    for k in blends.clone() {
        for edge in shell[k].edge_iter() {
            if matches!(edge.curve(), Curve::Intersection(_)) {
                continue;
            }
            let other = shell
                .iter()
                .enumerate()
                .find(|(i, face)| *i != k && face.edge_iter().any(|e| e.id() == edge.id()))
                .map(|(_, face)| face)
                .expect("edge not shared");
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            for i in 1..8 {
                let p = curve.subs(t0 + (t1 - t0) * i as f64 / 8.0);
                let (n0, n1) = (normal_at(&shell[k], p), normal_at(other, p));
                assert!(
                    (n0 - n1).magnitude() < 0.01,
                    "normals differ at {p:?}: {n0:?} vs {n1:?}"
                );
            }
        }
    }
}

/// Fillets the whole top rim of a cylinder: a closed chain of two arcs passing through the seam
/// vertices of the two side faces.
#[test]
fn fillet_cylinder_rim() {
    let (r, h, radius) = (1.0, 1.0, 0.2);
    let solid = from_modeling(&modeling::cylinder(
        Point3::origin(),
        Vector3::unit_z(),
        r,
        h,
    ));
    let shell = solid.into_boundaries().pop().unwrap();
    let wire = chain(&shell, &[Point3::new(r, 0.0, h), Point3::new(-r, 0.0, h)]);
    assert!(wire.is_closed());
    let faces = shell.len();
    let shell = fillet_along_wire(&shell, &wire, radius, TOL).unwrap();
    assert_eq!(shell.len(), faces + 2);
    assert_tangent_along_blends(&shell, faces..faces + 2);

    let (area, centroid) = fillet_section(radius);
    let expected = cylinder_volume(r, h) - 2.0 * PI * (r - centroid) * area;
    assert_solid(&Solid::new(vec![shell]), expected, &[0], TOL);
}

/// Prism of height `h` over the rectangle `[0, w] × [0, d]` with the two corners at `x = w`
/// rounded with radius `rho`. The side at `x = 0` is split in two edges.
fn rounded_prism(w: f64, d: f64, rho: f64, h: f64) -> Solid {
    use truck_modeling::{builder, Point3, Vector3, Wire};
    let s = rho / f64::sqrt(2.0);
    let v: Vec<_> = [
        (0.0, 0.0),
        (w - rho, 0.0),
        (w, rho),
        (w, d - rho),
        (w - rho, d),
        (0.0, d),
        (0.0, d / 2.0),
    ]
    .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
    .into();
    let wire: Wire = vec![
        builder::line(&v[0], &v[1]),
        builder::circle_arc(&v[1], &v[2], Point3::new(w - rho + s, rho - s, 0.0)),
        builder::line(&v[2], &v[3]),
        builder::circle_arc(&v[3], &v[4], Point3::new(w - rho + s, d - rho + s, 0.0)),
        builder::line(&v[4], &v[5]),
        builder::line(&v[5], &v[6]),
        builder::line(&v[6], &v[0]),
    ]
    .into();
    let face = builder::try_attach_plane(&[wire]).unwrap();
    from_modeling(&builder::tsweep(&face, Vector3::unit_z() * h))
}

/// Fillets an open chain of five edges (line, arc, line, arc, line) around the rounded side of a
/// prism's top face. The chain ends at right-angle corners with separate end faces.
#[test]
fn fillet_open_chain_around_rounded_corners() {
    let (w, d, rho, h, radius) = (3.0, 2.0, 0.5, 1.0, 0.2);
    let s = rho / f64::sqrt(2.0);
    let solid = rounded_prism(w, d, rho, h);
    let shell = solid.into_boundaries().pop().unwrap();
    let wire = chain(
        &shell,
        &[
            Point3::new((w - rho) / 2.0, 0.0, h),
            Point3::new(w - rho + s, rho - s, h),
            Point3::new(w, d / 2.0, h),
            Point3::new(w - rho + s, d - rho + s, h),
            Point3::new((w - rho) / 2.0, d, h),
        ],
    );
    assert!(!wire.is_closed());
    let faces = shell.len();
    let shell = fillet_along_wire(&shell, &wire, radius, TOL).unwrap();
    assert_eq!(shell.len(), faces + 5);
    assert_tangent_along_blends(&shell, faces..faces + 5);

    let base_area = w * d - 2.0 * rho * rho + PI * rho * rho / 2.0;
    let (area, centroid) = fillet_section(radius);
    let chain_length = 2.0 * (w - rho) + (d - 2.0 * rho) + PI * (rho - centroid);
    let expected = base_area * h - area * chain_length;
    assert_solid(&Solid::new(vec![shell]), expected, &[0], TOL);
}

/// A chain with a corner is rejected.
#[test]
fn fillet_along_wire_rejects_corner() {
    let solid = from_modeling(&modeling::cuboid(
        Point3::origin(),
        Point3::new(1.0, 1.0, 1.0),
    ));
    let shell = solid.into_boundaries().pop().unwrap();
    let wire = chain(
        &shell,
        &[Point3::new(0.5, 0.0, 1.0), Point3::new(1.0, 0.5, 1.0)],
    );
    assert!(fillet_along_wire(&shell, &wire, 0.2, TOL).is_none());
}
