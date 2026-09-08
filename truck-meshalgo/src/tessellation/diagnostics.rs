//! Tessellation APIs that expose omitted faces.
use super::*;
use std::result::Result;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic, Report};

/// Identifies faces omitted from a tessellated shape, in input traversal order.
pub trait MeshingDiagnostics {
    /// `(shell_index, face_index)` for every face without a mesh.
    fn missing_faces(&self) -> Vec<(usize, usize)>;
}
impl MeshingDiagnostics for Shell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn missing_faces(&self) -> Vec<(usize, usize)> {
        self.iter()
            .enumerate()
            .filter(|(_, f)| f.surface().is_none())
            .map(|(i, _)| (0, i))
            .collect()
    }
}
impl MeshingDiagnostics for CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>> {
    fn missing_faces(&self) -> Vec<(usize, usize)> {
        self.faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.surface.is_none())
            .map(|(i, _)| (0, i))
            .collect()
    }
}
impl<P, C, S> MeshingDiagnostics for Solid<P, C, S>
where Shell<P, C, S>: MeshingDiagnostics
{
    fn missing_faces(&self) -> Vec<(usize, usize)> {
        self.boundaries()
            .iter()
            .enumerate()
            .flat_map(|(i, s)| s.missing_faces().into_iter().map(move |(_, j)| (i, j)))
            .collect()
    }
}
impl<P, C, S> MeshingDiagnostics for CompressedSolid<P, C, S>
where CompressedShell<P, C, S>: MeshingDiagnostics
{
    fn missing_faces(&self) -> Vec<(usize, usize)> {
        self.boundaries
            .iter()
            .enumerate()
            .flat_map(|(i, s)| s.missing_faces().into_iter().map(move |(_, j)| (i, j)))
            .collect()
    }
}
fn report<T: MeshingDiagnostics>(value: T, operation: &'static str) -> Report<T> {
    let diagnostics = value
        .missing_faces()
        .into_iter()
        .map(|(shell, face)| {
            Diagnostic::new(Code::TessellationFailed, operation, "tessellate")
                .shell(shell)
                .face(face)
        })
        .collect();
    Report { value, diagnostics }
}

/// Tessellates and reports every omitted face. Completeness does not certify mesh closure.
pub fn triangulation_with_diagnostics<T: MeshableShape>(
    shape: &T,
    tol: f64,
) -> Result<Report<T::MeshedShape>, Diagnostic>
where
    T::MeshedShape: MeshingDiagnostics,
{
    validate_tolerance(tol, "triangulation")?;
    Ok(report(shape.triangulation(tol), "triangulation"))
}

/// Robust tessellation with every omitted face reported.
pub fn robust_triangulation_with_diagnostics<T: RobustMeshableShape>(
    shape: &T,
    tol: f64,
) -> Result<Report<T::MeshedShape>, Diagnostic>
where
    T::MeshedShape: MeshingDiagnostics,
{
    validate_tolerance(tol, "robust_triangulation")?;
    Ok(report(
        shape.robust_triangulation(tol),
        "robust_triangulation",
    ))
}

/// Strict tessellation: returns the first omitted face as an error instead of a partial mesh.
/// Use [`triangulation_with_diagnostics`] to inspect all omissions.
pub fn try_triangulation<T: MeshableShape>(
    shape: &T,
    tol: f64,
) -> Result<T::MeshedShape, Diagnostic>
where
    T::MeshedShape: MeshingDiagnostics,
{
    let report = triangulation_with_diagnostics(shape, tol)?;
    match report.diagnostics.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(report.value),
    }
}
