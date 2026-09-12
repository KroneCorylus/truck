use super::*;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};

type PolyShell = Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>;

pub(super) struct BoundaryNesting {
    parent: Vec<Option<usize>>,
    pub inverted: bool,
}

/// Groups a bounded solid into bodies, each with its exterior first and its cavities after it.
///
/// Shell order does not define material. Outward shells enclose material, inward shells
/// enclose voids, and their orientations must alternate through nesting. An island inside a
/// cavity is a separate body. The returned shells retain the input geometry and topology;
/// tessellation at `tol` is used only to determine containment. Body order is unspecified.
/// An empty solid returns an empty vector.
///
/// Returns `None` for invalid topology, failed tessellation, inconsistent nesting or
/// orientation, or an unbounded (inverted) solid. Shells must be embedded, with disjoint or
/// nested interiors. Pairs with disjoint bounding-box interiors are treated as non-nested,
/// including boundary contacts. Other detected boundary contacts or intersections between
/// shells are conservatively rejected.
/// As with booleans, `tol` must resolve the features and gaps between shells; this is not an
/// exact geometric validity checker for arbitrary self-intersecting input.
///
/// Export each returned body separately to STEP: its shell order meets the exporter's
/// exterior/void convention. Do not export an arbitrary multi-body `Solid` as one STEP solid.
pub fn solid_components<C: PolylineableCurve, S: MeshableSurface>(
    solid: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Vec<Solid<Point3, C, S>>> {
    try_solid_components(solid, tol).ok()
}

/// Groups bodies with diagnostic errors. Empty input successfully returns no bodies.
pub fn try_solid_components<C: PolylineableCurve, S: MeshableSurface>(
    solid: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<Vec<Solid<Point3, C, S>>, Diagnostic> {
    components(solid, tol).map_err(|e| e.operation("solid_components"))
}

fn components<C: PolylineableCurve, S: MeshableSurface>(
    solid: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<Vec<Solid<Point3, C, S>>, Diagnostic> {
    let (_, _, nesting) = triangulate_boundaries(solid, tol)?;
    if nesting.inverted {
        return Err(Diagnostic::new(
            Code::UnboundedResult,
            "solid_components",
            "classify_shells",
        ));
    }
    let mut bodies = Vec::new();
    for (i, shell) in solid.boundaries().iter().enumerate() {
        let mut depth = 0;
        let mut parent = nesting.parent[i];
        while let Some(j) = parent {
            depth += 1;
            parent = nesting.parent[j];
        }
        if depth % 2 == 0 {
            let mut boundaries = vec![shell.clone()];
            boundaries.extend(
                solid
                    .boundaries()
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| nesting.parent[*j] == Some(i))
                    .map(|(_, shell)| shell.clone()),
            );
            bodies.push(Solid::try_new(boundaries).map_err(|e| {
                Diagnostic::new(
                    Code::InvalidOutputTopology,
                    "solid_components",
                    "validate_output",
                )
                .shell(i)
                .with_coded_source(e)
            })?);
        }
    }
    Ok(bodies)
}

pub(super) fn triangulate_boundaries<C: PolylineableCurve, S: MeshableSurface>(
    solid: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<(PolyShell, Vec<PolygonMesh>, BoundaryNesting), Diagnostic> {
    validate_tolerance(tol, "boolean")?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, "boolean", "validate_input")
            .with_coded_source(e)
    })?;
    let mut poly_shell = PolyShell::default();
    let mut meshes = Vec::new();
    for (shell_index, shell) in solid.boundaries().iter().enumerate() {
        let mut poly = shell.triangulation(tol);
        if let Some(face) = poly.iter().position(|face| face.surface().is_none()) {
            return Err(
                Diagnostic::new(Code::TessellationFailed, "boolean", "tessellate")
                    .shell(shell_index)
                    .face(face),
            );
        }
        meshes.push(poly.to_polygon());
        poly_shell.append(&mut poly);
    }
    let nesting = boundary_nesting(&meshes)?;
    Ok((poly_shell, meshes, nesting))
}

fn boundary_nesting(meshes: &[PolygonMesh]) -> Result<BoundaryNesting, Diagnostic> {
    let error = |code| Diagnostic::new(code, "boolean", "classify_shells");
    let volumes: Vec<_> = meshes.iter().map(|mesh| mesh.volume()).collect();
    if let Some(i) = volumes.iter().position(|v| !v.is_finite() || *v == 0.0) {
        return Err(error(Code::DegenerateMesh).shell(i));
    }
    let boxes: Vec<_> = meshes.iter().map(|mesh| mesh.bounding_box()).collect();
    let mut parent = vec![None; meshes.len()];
    for i in 0..meshes.len() {
        for j in i + 1..meshes.len() {
            if (0..3).any(|axis| {
                boxes[i].max()[axis] <= boxes[j].min()[axis]
                    || boxes[j].max()[axis] <= boxes[i].min()[axis]
            }) {
                continue;
            }
            if meshes[i].collide_with(&meshes[j]).is_some() {
                return Err(error(Code::ShellsIntersect).shell(i).related(j));
            }
            let i_in_j = contains_shell(&meshes[j], &meshes[i])
                .ok_or_else(|| error(Code::ClassificationFailed).shell(i).related(j))?;
            let j_in_i = contains_shell(&meshes[i], &meshes[j])
                .ok_or_else(|| error(Code::ClassificationFailed).shell(j).related(i))?;
            if i_in_j && j_in_i {
                return Err(error(Code::InconsistentNesting).shell(i).related(j));
            }
            let (child, container) = match (i_in_j, j_in_i) {
                (true, false) => (i, j),
                (false, true) => (j, i),
                _ => continue,
            };
            if volumes[child].abs() >= volumes[container].abs() {
                return Err(error(Code::InconsistentNesting).shell(i).related(j));
            }
            if parent[child].is_none_or(|old: usize| volumes[container].abs() < volumes[old].abs())
            {
                parent[child] = Some(container);
            }
        }
    }
    let inverted = parent
        .iter()
        .position(Option::is_none)
        .is_some_and(|i| volumes[i] < 0.0);
    for (i, p) in parent.iter().enumerate() {
        let expected_negative = p.map_or(inverted, |j| volumes[j] > 0.0);
        if (volumes[i] < 0.0) != expected_negative {
            return Err(error(Code::InconsistentOrientation).shell(i));
        }
    }
    Ok(BoundaryNesting { parent, inverted })
}

fn contains_shell(container: &PolygonMesh, shell: &PolygonMesh) -> Option<bool> {
    let tri = shell.faces().triangle_iter().next()?;
    let p = Point3::from_vec(
        tri.iter()
            .map(|v| shell.positions()[v.pos].to_vec())
            .sum::<Vector3>()
            / 3.0,
    );
    let dir = hash::take_one_unit(p);
    let a = container.signed_crossing_faces(p, dir).abs();
    let b = container.signed_crossing_faces(p, -dir).abs();
    (a == b && a <= 1).then_some(a == 1)
}
