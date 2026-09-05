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
//! Faces that cross each other through a tangent point are not supported.
//! Furthermore, performance optimization using BSP (Binary Space Partitioning) or similar methods remains a future task.
//!
//! ## Fillet and Chamfer
//!
//! Fillets and chamfers can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
//! Constant-radius fillets can also be applied to a tangent-continuous chain of edges with `fillet_along_wire`,
//! whose ends may be at vertices with any number of faces. Blending several fillets that meet at a vertex is unsupported.

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
pub use healing::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
mod transversal;
pub use transversal::{and, or, ShapeOpsCurve, ShapeOpsSurface};
mod alternative;

/// Attaching fillets and chamfers
///
/// # Current Status
/// Fillets and chamfers can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
/// Constant-radius fillets can also be applied to a tangent-continuous chain of edges with `fillet_along_wire`,
/// whose ends may be at vertices with any number of faces. Blending several fillets that meet at a vertex is unsupported.
pub mod fillet;
