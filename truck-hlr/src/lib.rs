//! Hidden-line projection of solids built with `truck-modeling` onto a plane. The result is
//! the set of 2D curves of `truck-drafting` a drawing view is made of, each flagged visible or
//! hidden, so a front, top or isometric view can be drawn without further conversion.
//!
//! # Current Status
//!
//! The edges of the solid and the silhouettes of its curved faces are projected, split at
//! their crossings and classified. Projection is parallel only.

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
mod silhouette;

use itertools::Itertools;
use std::collections::HashMap;
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

/// Projects the edges and silhouettes of `solid` onto `plane` in parallel along `direction`
/// and classifies them as visible or hidden.
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
/// The silhouette of a face is where its normal is perpendicular to `direction`. It is traced
/// over the face's tessellation at `tol`, which trims it to the face, with every crossing of a
/// triangle edge refined on the exact normal; the rulings of cylinders and cones are then exact
/// lines and the great circles of spheres exact arcs, anything else a cubic B-spline through
/// the crossings. A face whose normal is perpendicular to `direction` everywhere, such as a
/// cylinder wall seen along its axis, has no silhouette. A silhouette that coincides with an
/// edge, a ruling on a seam, is kept as well as the edge.
///
/// Each projected curve is split where it crosses another, the crossings found from the
/// polylines at `tol` and refined by Newton's method, and every piece is classified from the
/// point of the curve at the middle parameter of the piece. A meeting with parallel tangents is
/// not a crossing, so a silhouette touching the fold of an edge-on circle, or two edges that
/// project onto the same curve, do not split each other. Crossings within `tol` of an end of a
/// piece do not split it. Pieces are not merged, so an edge crossed twice gives three curves
/// even when all three have the same visibility. Two edges that project onto the same curve,
/// such as the front and back edges of a box seen along a face normal, are both kept, one
/// visible and one hidden.
///
/// The classification runs against the tessellation of `solid` at `tol`, so an occluder
/// thinner than `tol` along the view direction may be missed. A piece lying on a silhouette,
/// every silhouette piece and every edge piece on a face the ray grazes, is decided by how that
/// face bends along the ray: towards its outside, the ray is inside the body and the piece
/// hidden; away, the ray is outside and the test point is moved by `2 tol` out of the body,
/// past the tessellation sag, before testing the rest of the body; not at all, a plane or a
/// ruling along the ray, the test point is moved by `2 tol` into the body, past the chord sag of
/// the tessellation, and the far side of the body decides. So the far end circle of a cylinder
/// seen along its axis is hidden behind the near one, and both end circles of a cylinder seen
/// across its axis, edge-on, are visible. Two calls with the same inputs give the same output.
///
/// # Panics
///
/// `tol` must be at least `TOLERANCE`, and `direction` must not lie in the plane.
pub fn project(solid: &Solid, plane: &Plane, direction: Vector3, tol: f64) -> ProjectedView {
    let projection = projection::Projection::new(plane, direction);
    let faces: Vec<&Face> = solid.face_iter().collect();
    let supports: Vec<(Surface, bool)> = faces
        .iter()
        .map(|face| (face.surface(), face.orientation()))
        .collect();
    let meshed = solid.triangulation(tol);
    let mesh = meshed.to_polygon();

    let mut items: Vec<Item> = Vec::new();
    let mut seen: HashMap<EdgeID, Option<usize>> = HashMap::new();
    for (i, face) in faces.iter().enumerate() {
        for edge in face.boundaries().iter().flatten() {
            match seen.get(&edge.id()) {
                Some(Some(item)) => items[*item].faces.push(i),
                Some(None) => {}
                None => {
                    let curve = edge.curve();
                    let projected = projection.curve(&curve, tol);
                    seen.insert(edge.id(), projected.as_ref().map(|_| items.len()));
                    if let Some(projected) = projected {
                        items.push(Item::new(curve, projected, i, false));
                    }
                }
            }
        }
    }
    for (i, meshed_face) in meshed.face_iter().enumerate() {
        let Some(face_mesh) = meshed_face.surface() else {
            continue;
        };
        for curve in silhouette::silhouettes(&supports[i].0, &face_mesh, direction, tol) {
            if let Some(projected) = projection.curve(&curve, tol) {
                items.push(Item::new(curve, projected, i, true));
            }
        }
    }

    let projected: Vec<&Curve2> = items.iter().map(|item| &item.projected).collect();
    let splits = crossing::split_parameters(&projected, tol);
    let toward_viewer = -direction.normalize();

    let mut curves = Vec::new();
    for (item, splits) in items.iter().zip(splits) {
        let faces: Vec<occlusion::Support<'_>> = item
            .faces
            .iter()
            .map(|&i| occlusion::Support {
                surface: &supports[i].0,
                orientation: supports[i].1,
            })
            .collect();
        let (t0, t1) = item.projected.range_tuple();
        let bounds = std::iter::once(t0).chain(splits).chain(std::iter::once(t1));
        for (a, b) in bounds.tuple_windows() {
            let point = item.curve.subs((a + b) / 2.0);
            let hidden =
                occlusion::hidden(&mesh, point, toward_viewer, &faces, item.silhouette, tol);
            let visibility = match hidden {
                true => Visibility::Hidden,
                false => Visibility::Visible,
            };
            curves.push((crossing::piece(&item.projected, a, b), visibility));
        }
    }
    ProjectedView { curves }
}

/// A curve to draw: an edge or a silhouette, with the faces it lies on.
struct Item {
    curve: Curve,
    projected: Curve2,
    faces: Vec<usize>,
    silhouette: bool,
}

impl Item {
    fn new(curve: Curve, projected: Curve2, face: usize, silhouette: bool) -> Self {
        Self {
            curve,
            projected,
            faces: vec![face],
            silhouette,
        }
    }
}
