//! Reads/writes STEP files from/to truck.
//!
//! # Current Status
//!
//! It is possible to output data modeled by truck-modeling, and to read STEP shapes into the
//! reader's own geometry or, through `try_mapped`, into the enums of truck-modeling.
//! Shapes created by set operations cannot be output yet.
//!
//! # Entity types in the test corpus that are not implemented
//!
//! Over the files in `resources/step/`, as `Table::unsupported` reports them. Units, product
//! categories and presentation are not read; geometry arrives in the units of the file.
//! `VERTEX_LOOP` bounds a degenerate face and is ignored on purpose. A test keeps this list
//! equal to the corpus.
//!
//! - `(CONVERSION_BASED_UNIT LENGTH_UNIT NAMED_UNIT)`
//! - `(GEOMETRIC_REPRESENTATION_CONTEXT GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT GLOBAL_UNIT_ASSIGNED_CONTEXT REPRESENTATION_CONTEXT)`
//! - `(GEOMETRIC_REPRESENTATION_CONTEXT PARAMETRIC_REPRESENTATION_CONTEXT REPRESENTATION_CONTEXT)`
//! - `(LENGTH_UNIT NAMED_UNIT SI_UNIT)`
//! - `(NAMED_UNIT PLANE_ANGLE_UNIT SI_UNIT)`
//! - `(NAMED_UNIT SI_UNIT SOLID_ANGLE_UNIT)`
//! - `APPLICATION_PROTOCOL_DEFINITION`
//! - `COLOUR_RGB`
//! - `CURVE_STYLE`
//! - `DESIGN_CONTEXT`
//! - `DIMENSIONAL_EXPONENTS`
//! - `DRAUGHTING_PRE_DEFINED_CURVE_FONT`
//! - `FILL_AREA_STYLE`
//! - `FILL_AREA_STYLE_COLOUR`
//! - `LENGTH_MEASURE_WITH_UNIT`
//! - `MECHANICAL_CONTEXT`
//! - `MECHANICAL_DESIGN_GEOMETRIC_PRESENTATION_REPRESENTATION`
//! - `PRESENTATION_STYLE_ASSIGNMENT`
//! - `PRODUCT_CATEGORY`
//! - `PRODUCT_CATEGORY_RELATIONSHIP`
//! - `PRODUCT_RELATED_PRODUCT_CATEGORY`
//! - `STYLED_ITEM`
//! - `SURFACE_SIDE_STYLE`
//! - `SURFACE_STYLE_FILL_AREA`
//! - `SURFACE_STYLE_USAGE`
//! - `UNCERTAINTY_MEASURE_WITH_UNIT`
//! - `VERTEX_LOOP`

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

/// STEP input module
/// # Example
/// ```
/// use truck_stepio::r#in::{*, step_geometry::*};
/// use ruststep::tables::EntityTable;
/// // read file
/// let step_string = include_str!(concat!(
///     env!("CARGO_MANIFEST_DIR"),
///     "/../resources/step/occt-cube.step",
/// ));
/// // parse step file
/// let exchange = ruststep::parser::parse(&step_string).unwrap();
/// // convert the parsing results to a Rust struct
/// let table = Table::from_data_section(&exchange.data[0]);
/// // get `CartesianPoint` registered in #102
/// let step_point = EntityTable::<CartesianPointHolder>::get_owned(&table, 102).unwrap();
/// // convert `CartesianPoint` in STEP to `Point3` in cgmath
/// let cgmath_point = Point3::from(&step_point);
/// // check parse result
/// assert_eq!(cgmath_point, Point3::new(0.0, 10.0, 0.0));
/// ```
#[cfg(feature = "in")]
pub mod r#in;
/// STEP output module
pub mod out;

/// common structures
pub mod common;
