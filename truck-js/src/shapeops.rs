use crate::{wasm_bindgen, IntoWasm, Solid};
use truck_shapeops as shapeops;

const SHAPEOPS_TOLERANCE: f64 = 0.05;

/// and operator
#[wasm_bindgen]
pub fn and(solid0: &Solid, solid1: &Solid, tol: Option<f64>) -> Option<Solid> {
    let tol = tol.unwrap_or(SHAPEOPS_TOLERANCE);
    shapeops::and(solid0, solid1, tol).map(IntoWasm::into_wasm)
}

/// or operator
#[wasm_bindgen]
pub fn or(solid0: &Solid, solid1: &Solid, tol: Option<f64>) -> Option<Solid> {
    let tol = tol.unwrap_or(SHAPEOPS_TOLERANCE);
    shapeops::or(solid0, solid1, tol).map(IntoWasm::into_wasm)
}

/// not operator
#[wasm_bindgen]
pub fn not(solid: &Solid) -> Solid {
    let mut solid = solid.clone();
    solid.not();
    solid
}

/// Intersection; throws an Error with stable code, operation, stage and context on failure.
#[wasm_bindgen]
pub fn try_and(
    solid0: &Solid,
    solid1: &Solid,
    tol: Option<f64>,
) -> Result<Solid, wasm_bindgen::JsValue> {
    shapeops::try_and(solid0, solid1, tol.unwrap_or(SHAPEOPS_TOLERANCE))
        .map(IntoWasm::into_wasm)
        .map_err(crate::diagnostics::js_error)
}
/// Union; throws a structured Error on failure.
#[wasm_bindgen]
pub fn try_or(
    solid0: &Solid,
    solid1: &Solid,
    tol: Option<f64>,
) -> Result<Solid, wasm_bindgen::JsValue> {
    shapeops::try_or(solid0, solid1, tol.unwrap_or(SHAPEOPS_TOLERANCE))
        .map(IntoWasm::into_wasm)
        .map_err(crate::diagnostics::js_error)
}
/// Subtraction; empty and unchanged results are successful solids.
#[wasm_bindgen]
pub fn try_subtract(
    solid0: &Solid,
    solid1: &Solid,
    tol: Option<f64>,
) -> Result<Solid, wasm_bindgen::JsValue> {
    shapeops::try_subtract(solid0, solid1, tol.unwrap_or(SHAPEOPS_TOLERANCE))
        .map(IntoWasm::into_wasm)
        .map_err(crate::diagnostics::js_error)
}
