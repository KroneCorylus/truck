mod divide_face;
mod faces_classification;
mod integrate;
pub(crate) mod intersection_curve;
mod loops_store;
mod polyline_construction;
pub(crate) use integrate::try_smooth_leader;
pub use integrate::{
    and, or, solid_components, subtract, subtract_with_effect, try_and, try_or,
    try_solid_components, try_subtract, try_subtract_with_effect, ShapeOpsCurve, ShapeOpsSurface,
    SubtractionResult,
};
