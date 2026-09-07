//! Lofts through sections built with `truck-modeling`, checked with the harness.

mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_modeling::{builder, errors::Error};

type MWire = truck_modeling::Wire;
type MSolid = truck_modeling::Solid;

const TOL: f64 = 0.001;

/// Circle of `radius` about the `z` axis at height `z`, counterclockwise from `+z`, as `arcs`
/// arcs from the angle `start`.
fn circle(radius: f64, z: f64, arcs: usize, start: f64) -> MWire {
    let center = Point3::new(0.0, 0.0, z);
    let vertex = builder::vertex(center + Vector3::new(start.cos(), start.sin(), 0.0) * radius);
    builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(7.0), arcs)
}

/// Closed polygon through `points`, in order.
fn polygon(points: &[Point3]) -> MWire {
    let vertices: Vec<_> = points.iter().map(|&p| builder::vertex(p)).collect();
    (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect()
}

/// Square with corners on the axes at distance `half`, counterclockwise from `+z`.
fn diamond(half: f64, z: f64) -> MWire {
    polygon(&[
        Point3::new(half, 0.0, z),
        Point3::new(0.0, half, z),
        Point3::new(-half, 0.0, z),
        Point3::new(0.0, -half, z),
    ])
}

/// Every section curve is an iso-curve of the face lofted through it.
fn assert_sections_are_iso_curves(solid: &MSolid, sections: &[MWire]) {
    let shell = &solid.boundaries()[0];
    let n = sections[0].len();
    for i in 0..n {
        let surface = shell[i].surface();
        for (j, section) in sections.iter().enumerate() {
            let curve = section[i].oriented_curve();
            let (t0, t1) = curve.range_tuple();
            let guess = j as f64 / (sections.len() - 1) as f64;
            let mut vs = Vec::new();
            for k in 0..5 {
                let s = (k as f64 + 0.5) / 5.0;
                let p = curve.subs(t0 + (t1 - t0) * s);
                let (u, v) = surface
                    .search_parameter(p, Some((s, guess)), 100)
                    .expect("section point not on the loft");
                assert!(
                    surface.subs(u, v).distance(p) < 1e-9,
                    "{p:?} is not on face {i}"
                );
                vs.push(v);
            }
            assert!(
                vs.iter().all(|v| (v - vs[0]).abs() < 1e-9),
                "section {j} is not an iso-curve of face {i}: {vs:?}"
            );
        }
    }
}

#[test]
fn square_to_circle() {
    let square = polygon(&[
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
        Point3::new(-1.0, -1.0, 0.0),
    ]);
    let sections = builder::align_sections(&[square, circle(1.0, 2.0, 4, 0.0)]);
    let solid: MSolid = builder::try_loft(&sections).unwrap();
    assert_eq!(solid.boundaries()[0].len(), 6);
    assert_sections_are_iso_curves(&solid, &sections);
    let solid = from_modeling(&solid);
    assert_topology(&solid, &[0]);
    assert_mesh_closed(&solid, TOL);
}

/// Three circles of radii 1, 2, 1 at heights 0, 1, 2. The chord-length parameters are 0, 1/2, 1
/// by symmetry, so the loft is the quadratic through the three circles: a surface of revolution
/// with radius `1 + 4 v (1 - v)` at height `2 v`, of volume `86 π / 15`.
#[test]
fn three_circles_make_a_surface_of_revolution() {
    let sections = [
        circle(1.0, 0.0, 4, 0.0),
        circle(2.0, 1.0, 4, 0.0),
        circle(1.0, 2.0, 4, 0.0),
    ];
    let solid: MSolid = builder::try_loft(&sections).unwrap();
    assert_sections_are_iso_curves(&solid, &sections);
    // the radius peaks at the middle section, so the normal is radial there
    for face in solid.boundaries()[0].iter().take(4) {
        let surface = face.oriented_surface();
        for k in 0..20 {
            let u = (k as f64 + 0.5) / 20.0;
            let p = surface.subs(u, 0.5);
            let radial = Vector3::new(p.x, p.y, 0.0).normalize();
            let angle = surface.normal(u, 0.5).angle(radial).0;
            assert!(angle < 1e-6, "normal is {angle} rad off radial at {p:?}");
            assert!((p.to_vec().truncate().magnitude() - 2.0).abs() < 1e-9);
        }
    }
    assert_solid(&from_modeling(&solid), 86.0 * PI / 15.0, &[0], TOL);
}

#[test]
fn mismatched_and_repeated_sections_are_rejected() {
    let square = diamond(1.0, 0.0);
    let triangle = polygon(&[
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(-1.0, 0.0, 1.0),
    ]);
    let mismatch =
        builder::try_loft_shell::<_, truck_modeling::Surface>(&[square.clone(), triangle]);
    assert_eq!(mismatch.unwrap_err(), Error::LoftSectionsMismatch(0, 1));
    let closed = builder::try_loft_shell::<_, truck_modeling::Surface>(&[
        square.clone(),
        circle(1.0, 1.0, 4, 0.0),
        square.clone(),
    ]);
    assert_eq!(closed.unwrap_err(), Error::LoftSectionsCoincide(0, 2));
    let one = builder::try_loft_shell::<_, truck_modeling::Surface>(&[square]);
    assert_eq!(one.unwrap_err(), Error::TooFewLoftSections);
}

/// A circle started at the far side and wound the other way is rotated and reversed to match
/// the square, and the loft through the aligned pair is a closed solid.
#[test]
fn align_sections_rotates_and_reverses() {
    let square = diamond(1.0, 0.0);
    let circle = circle(1.0, 1.0, 4, PI).inverse();
    let aligned = builder::align_sections(&[square, circle]);
    let points: Vec<Point3> = aligned[1].iter().map(|edge| edge.front().point()).collect();
    assert!(points[0].near(&Point3::new(1.0, 0.0, 1.0)), "{points:?}");
    assert!(points[1].near(&Point3::new(0.0, 1.0, 1.0)), "{points:?}");
    let solid: MSolid = builder::try_loft(&aligned).unwrap();
    let solid = from_modeling(&solid);
    assert_topology(&solid, &[0]);
    assert_mesh_closed(&solid, TOL);
}
