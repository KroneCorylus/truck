//! Blend regressions use the public modeling representations directly.

use truck_geometry::prelude::*;
#[allow(unused_imports)]
pub use truck_modeling::{shell, wire, Curve, Edge, Face, Shell, Solid, Surface, Vertex, Wire};

/// Keeps the modeling solid and its topology IDs.
pub fn from_modeling(solid: &truck_modeling::Solid) -> Solid { solid.clone() }

/// Index of the face of `shell` whose surface passes through `point`.
pub fn face_through(shell: &Shell, point: Point3) -> usize {
    shell
        .face_iter()
        .position(|face| {
            let surface = face.surface();
            surface
                .search_parameter(point, None, 10)
                .is_some_and(|(u, v)| surface.subs(u, v).near(&point))
        })
        .expect("no face through the point")
}

/// The edge of `shell` that passes through `point`, within its parameter range.
pub fn edge_through(shell: &Shell, point: Point3) -> Edge {
    shell
        .edge_iter()
        .find(|edge| {
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            curve
                .search_nearest_parameter(point, None, 10)
                .is_some_and(|t| t0 <= t && t <= t1 && curve.subs(t).near(&point))
        })
        .expect("no edge through the point")
}
