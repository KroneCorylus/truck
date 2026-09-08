use crate::*;
use std::result::Result;
use truck_meshalgo::tessellation::*;
use truck_stepio::r#in::step_geometry::*;
use truck_topology::compress::*;

/// step parse table
#[derive(Clone, Debug, Deref, DerefMut, From, Into)]
#[wasm_bindgen]
pub struct Table(truck_stepio::r#in::Table);

#[derive(Clone, Debug)]
enum SubShapeFromStep {
    Shell(CompressedShell<Point3, Curve3D, Surface>),
    #[allow(dead_code)]
    Solid(CompressedSolid<Point3, Curve3D, Surface>),
}

/// Shell and Solid parsed from step
#[derive(Clone, Debug, From, Into)]
#[wasm_bindgen]
pub struct ShapeFromStep(SubShapeFromStep);

#[wasm_bindgen]
impl ShapeFromStep {
    /// Robust meshing with explicit omitted-face diagnostics.
    pub fn to_polygon_with_diagnostics(&self, tol: f64) -> Result<MeshReport, JsValue> {
        use truck_meshalgo::tessellation::robust_triangulation_with_diagnostics;
        let (mesh, diagnostics) = match &self.0 {
            SubShapeFromStep::Shell(x) => {
                let report =
                    robust_triangulation_with_diagnostics(x, tol).map_err(diagnostics::js_error)?;
                (report.value.to_polygon().into(), report.diagnostics)
            }
            SubShapeFromStep::Solid(x) => {
                let report =
                    robust_triangulation_with_diagnostics(x, tol).map_err(diagnostics::js_error)?;
                (report.value.to_polygon().into(), report.diagnostics)
            }
        };
        Ok(MeshReport { mesh, diagnostics })
    }
    /// meshing shape from step
    pub fn to_polygon(&self, tol: f64) -> crate::PolygonMesh {
        use SubShapeFromStep::*;
        match &self.0 {
            Shell(x) => x.robust_triangulation(tol).to_polygon().into(),
            Solid(x) => x.robust_triangulation(tol).to_polygon().into(),
        }
    }
}

#[wasm_bindgen]
impl Table {
    /// read step file
    pub fn from_step(step_str: &str) -> Option<Table> {
        Some(Table(truck_stepio::r#in::Table::from_step(step_str)?))
    }
    /// Parses STEP or throws a structured error. Inspect `diagnostics()` for unreadable records.
    pub fn try_from_step(step_str: &str) -> Result<Table, JsValue> {
        truck_stepio::r#in::Table::from_step_with_diagnostics(step_str)
            .map(|report| Table(report.value))
            .map_err(diagnostics::js_error)
    }
    /// Structured unreadable-record diagnostics in file order.
    pub fn diagnostics(&self) -> Result<JsValue, JsValue> {
        diagnostics::diagnostics_json(&self.0.record_diagnostics())
    }
    /// Converts a shell and retains every detected omission in the result.
    pub fn get_shape_with_diagnostics(&self, idx: u64) -> Result<StepShapeReport, JsValue> {
        use truck_base::diagnostics::{Code, Diagnostic};
        let shell = self.shell.get(&idx).ok_or_else(|| {
            let mut error =
                Diagnostic::new(Code::StepMissingReference, "get_shape", "lookup_entity");
            error.context.entity_id = Some(idx);
            diagnostics::js_error(error)
        })?;
        let report = self
            .0
            .to_compressed_shell_with_diagnostics(shell)
            .map_err(diagnostics::js_error)?;
        Ok(StepShapeReport {
            shape: SubShapeFromStep::Shell(report.value).into(),
            diagnostics: report.diagnostics,
        })
    }
    /// get shell indices
    pub fn shell_indices(&self) -> Vec<u64> { self.0.shell.keys().copied().collect() }
    /// get shape from indices
    pub fn get_shape(&self, idx: u64) -> Option<ShapeFromStep> {
        let stepshell = self.shell.get(&idx)?;
        let (shell, skipped) = self
            .to_compressed_shell(stepshell)
            .map_err(|e| gloo::console::error!(format!("{e}")))
            .ok()?;
        for skipped in skipped {
            gloo::console::warn!(format!("{skipped}"));
        }
        Some(SubShapeFromStep::Shell(shell).into())
    }
}

/// Imported geometry with explicit skipped-entity diagnostics.
#[derive(Debug)]
#[wasm_bindgen]
pub struct StepShapeReport {
    shape: ShapeFromStep,
    diagnostics: Vec<truck_base::diagnostics::Diagnostic>,
}
#[wasm_bindgen]
impl StepShapeReport {
    /// Complete or partial converted geometry.
    pub fn shape(&self) -> ShapeFromStep { self.shape.clone() }
    /// True when conversion reported no omitted entities.
    pub fn is_complete(&self) -> bool { self.diagnostics.is_empty() }
    /// Structured skipped-entity and dependency diagnostics.
    pub fn diagnostics(&self) -> Result<JsValue, JsValue> {
        diagnostics::diagnostics_json(&self.diagnostics)
    }
}
