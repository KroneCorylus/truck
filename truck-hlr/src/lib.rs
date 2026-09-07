//! Hidden-line projection of solids built with `truck-modeling` onto a plane. The result is
//! the set of 2D curves of `truck-drafting` a drawing view is made of, each flagged visible or
//! hidden, so a front, top or isometric view can be drawn without further conversion.
//!
//! # Current Status
//!
//! The edges of the solid are projected and classified. Silhouettes of curved faces, the
//! outline of a cylinder or a sphere that is not an edge of the B-rep, are not extracted yet.
//! Projection is parallel only.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

mod crossing;
mod occlusion;
mod projection;

use itertools::Itertools;
use std::collections::HashSet;
use truck_drafting::Curve as Curve2;
use truck_meshalgo::prelude::*;
use truck_modeling::*;

/// Whether a piece of a projected curve is seen from the viewer or covered by the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Nothing of the body lies between the piece and the viewer.
    Visible,
    /// The body lies between the piece and the viewer.
    Hidden,
}

/// A parallel projection of a solid onto a plane, as produced by [`project`].
#[derive(Clone, Debug)]
pub struct ProjectedView {
    /// 2D curves in the coordinates of the projection plane, each visible or hidden.
    pub curves: Vec<(Curve2, Visibility)>,
}

/// Projects the edges of `solid` onto `plane` in parallel along `direction` and classifies
/// them as visible or hidden.
///
/// `direction` is the view direction, from the eye into the scene; a point of the body is
/// hidden when the ray from it against `direction` meets the body. The 2D coordinates are
/// those of the plane, `u` along `plane.u_axis()` and `v` along `plane.v_axis()`, so the axes
/// need not be unit or orthogonal. A right-handed view has the viewer on the side the normal
/// `u × v` points to, that is `direction · normal < 0`.
///
/// Every edge projects exactly except an `IntersectionCurve`: a line stays a line, a conic
/// becomes a `CircleArc` under the projected transform, an ellipse or a segment traversed back
/// and forth when its plane contains `direction`, and B-spline and NURBS curves keep their knots
/// with their control points projected. An `IntersectionCurve` is sampled at the chord error
/// `tol` and interpolated by a cubic B-spline over the same parameters. Edges whose projection
/// fits in a circle of radius `tol`, such as those parallel to `direction`, are dropped.
///
/// Each projected curve is split where it crosses another, the crossings found from the
/// polylines at `tol` and refined by Newton's method, and every piece is classified from the
/// point of the edge at the middle parameter of the piece. Crossings within `tol` of an end
/// of a piece do not split it. Pieces are not merged, so an edge crossed twice gives three
/// curves even when all three have the same visibility. Two edges that project onto the same
/// curve, such as the front and back edges of a box seen along a face normal, are both kept,
/// one visible and one hidden.
///
/// The classification runs against the tessellation of `solid` at `tol`, so an occluder
/// thinner than `tol` along the view direction may be missed. Two calls with the same inputs
/// give the same output.
///
/// # Panics
///
/// `tol` must be at least `TOLERANCE`, and `direction` must not lie in the plane.
pub fn project(solid: &Solid, plane: &Plane, direction: Vector3, tol: f64) -> ProjectedView {
    let projection = projection::Projection::new(plane, direction);
    let mut seen = HashSet::new();
    let mut edges: Vec<(Curve, Curve2)> = Vec::new();
    for edge in solid.edge_iter() {
        if !seen.insert(edge.id()) {
            continue;
        }
        let curve = edge.curve();
        if let Some(projected) = projection.curve(&curve, tol) {
            edges.push((curve, projected));
        }
    }

    let projected: Vec<&Curve2> = edges.iter().map(|(_, curve)| curve).collect();
    let splits = crossing::split_parameters(&projected, tol);
    let mesh = solid.triangulation(tol).to_polygon();
    let toward_viewer = -direction;

    let mut curves = Vec::new();
    for ((curve, projected), splits) in edges.iter().zip(splits) {
        let (t0, t1) = projected.range_tuple();
        let bounds = std::iter::once(t0).chain(splits).chain(std::iter::once(t1));
        for (a, b) in bounds.tuple_windows() {
            let point = curve.subs((a + b) / 2.0);
            let visibility = match occlusion::blocked(&mesh, point, toward_viewer) {
                true => Visibility::Hidden,
                false => Visibility::Visible,
            };
            curves.push((crossing::piece(projected, a, b), visibility));
        }
    }
    ProjectedView { curves }
}
