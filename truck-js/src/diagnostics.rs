use crate::*;
use truck_base::diagnostics::Diagnostic;

pub(crate) fn js_error(diagnostic: Diagnostic) -> JsValue {
    let error = js_sys::Error::new(&diagnostic.message);
    match serde_json::to_string(&diagnostic) {
        Ok(json) => match js_sys::JSON::parse(&json) {
            Ok(metadata) => {
                js_sys::Object::assign(&error.into(), &metadata.unchecked_into()).into()
            }
            Err(error) => error,
        },
        Err(error) => js_sys::Error::new(&error.to_string()).into(),
    }
}

pub(crate) fn diagnostics_json(diagnostics: &[Diagnostic]) -> Result<JsValue, JsValue> {
    let json =
        serde_json::to_string(diagnostics).map_err(|e| js_sys::Error::new(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

/// A polygon mesh with explicit information about omitted faces.
#[wasm_bindgen]
pub struct MeshReport {
    pub(crate) mesh: PolygonMesh,
    pub(crate) diagnostics: Vec<Diagnostic>,
}
#[wasm_bindgen]
impl MeshReport {
    /// Returns the complete or partial mesh.
    pub fn mesh(&self) -> PolygonMesh { self.mesh.clone() }
    /// True when no faces were reported omitted; this does not certify mesh closure.
    pub fn is_complete(&self) -> bool { self.diagnostics.is_empty() }
    /// Structured omission diagnostics, available without capturing console output.
    pub fn diagnostics(&self) -> Result<JsValue, JsValue> { diagnostics_json(&self.diagnostics) }
}
impl std::fmt::Debug for MeshReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshReport")
            .field("diagnostics", &self.diagnostics)
            .finish_non_exhaustive()
    }
}
