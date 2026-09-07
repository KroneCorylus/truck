//! Crate for operation shapes. Provides boolean operations to Solid, and shape healing for importing shapes from other CAD systems.
//!
//! # Current Status
//!
//! ## Boolean Operation
//!
//! Boolean operations are supported for shapes where faces intersect transversally or lie on the same surface.
//! Faces that only touch, along a curve, along a common edge or at a point, are not cut; the result then keeps
//! one shell per solid. Coincident faces are cut along each other's boundaries; their overlap is kept once when
//! the outward normals agree and dropped when they oppose. Pieces of boundary that two coincident faces share
//! must be discretized identically, which holds for planar faces and for shared curves.
//! Tangent crossings sampled by the interference mesh are split into branches with shared
//! singular vertices, with curvature distinguishing crossings from contact. Equal-radius
//! perpendicular cylinders (Steinmetz intersection and union), a sphere subtracted from an
//! equal-radius blind bore, and a rod subtracted through an equal-width slot are supported.
//! Singularities missed by the mesh and higher-order contacts remain outside this support.
//!
//! The kernel works in absolute units: two points closer than the `TOLERANCE` of `truck-base`,
//! `1e-6`, are the same point, and the `tol` of `and` and `or` is a chord error in the same
//! units. Choose the units of a model so that its features are large against `TOLERANCE`, and
//! `tol` below its smallest feature.
//!
//! Furthermore, performance optimization using BSP (Binary Space Partitioning) or similar methods remains a future task.
//!
//! ## Fillet and Chamfer
//!
//! Fillets and chamfers can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
//! Fillets with a constant or varying radius can also be applied to a tangent-continuous chain of edges with `fillet_along_wire`,
//! whose ends may be at vertices with any number of faces. `fillet_edges` blends equal-radius straight-edge
//! chains on closed convex planar shells, with exact spherical patches at three-edge corners.
//!
//! ## Local Operations
//!
//! `local::move_faces` moves a group of faces, rigidly with their edges when their boundary stays on the neighbouring
//! surfaces, such as a hole through a plate, and by re-intersection otherwise. `local::offset_faces` offsets faces
//! along their normals.
//! `local::intersect_surfaces` gives the curves where two surfaces meet over given parameter rectangles, exact for
//! a plane against a plane, a cylinder or a cone normal to it, and lifted from the tessellations otherwise.
//! `local::replace_surfaces` gives faces new surfaces and re-intersects them with their neighbours: planes, cylinders and cones.
//! `local::delete_face` removes a four-edged face between planes, such as the fillet or chamfer of one edge, and
//! restores the sharp edge.
//! `local::draft` tilts planar and cylindrical faces about a neutral plane, cylinders becoming cones.
//! `local::shell` hollows a convex-edged solid to walls of a thickness, with chosen faces opened.
//! `local::thicken` makes a planar shell into a slab of a thickness.

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

mod healing;
#[doc(hidden)]
pub mod profile;
pub use healing::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
mod transversal;
pub use transversal::{and, or, ShapeOpsCurve, ShapeOpsSurface};
mod alternative;
pub mod local;

/// Attaching fillets and chamfers
///
/// # Current Status
/// Fillets and chamfers can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
/// Fillets with a constant or varying radius can also be applied to a tangent-continuous chain of edges with `fillet_along_wire`,
/// whose ends may be at vertices with any number of faces. `fillet_edges` adds equal-radius spherical
/// corners where three selected edges of a closed convex planar shell meet.
pub mod fillet;
