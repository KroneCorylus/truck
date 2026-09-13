//! Blend operations on modeling solids.

use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};
use truck_modeling::{EdgeID, Face, FaceID, ScalarFunctionD1, Shell, Solid, Wire};

/// A modeling solid after a blend, with face correspondence for downstream naming.
///
/// IDs identify topology in this process; they are not persistent names across recomputations.
/// Assign application names using the operation's selection and these correspondences. Vertices
/// and edges, including generated contact and corner edges, are accessible through each face's
/// topology iterators. No geometry is hidden in a separate blend representation.
#[derive(Clone, Debug)]
pub struct BlendResult {
    /// The resulting solid, ready for tessellation and subsequent boolean operations.
    pub solid: Solid,
    /// Input face ID and its replacement, in input face order. Unchanged faces are omitted.
    pub modified_faces: Vec<(FaceID, Face)>,
    /// Added blend and corner faces, in result order.
    pub generated_faces: Vec<Face>,
}

fn shell_index(solid: &Solid, edge: EdgeID) -> Option<usize> {
    solid
        .boundaries()
        .iter()
        .position(|s| s.edge_iter().any(|e| e.id() == edge))
}

fn finish(solid: &Solid, index: usize, shell: Shell) -> Result<BlendResult, Diagnostic> {
    let original = &solid.boundaries()[index];
    for (i, face) in shell.iter().enumerate() {
        if original.get(i).is_some_and(|old| old.id() == face.id()) {
            continue;
        }
        if !face.is_geometric_consistent() {
            return Err(Diagnostic::new(
                Code::BlendConstructionFailed,
                "blend",
                "validate_geometry",
            )
            .shell(index)
            .face(i));
        }
    }
    let modified_faces = original
        .iter()
        .zip(&shell)
        .filter(|(a, b)| a.id() != b.id())
        .map(|(a, b)| (a.id(), b.clone()))
        .collect();
    let generated_faces = shell.iter().skip(original.len()).cloned().collect();
    let mut boundaries = solid.boundaries().clone();
    boundaries[index] = shell;
    Ok(BlendResult {
        solid: Solid::try_new(boundaries).map_err(|e| {
            Diagnostic::new(Code::InvalidOutputTopology, "blend", "validate_output")
                .shell(index)
                .with_coded_source(e)
        })?,
        modified_faces,
        generated_faces,
    })
}

/// Fillets a tangent-continuous wire belonging to one boundary of a modeling solid.
///
/// The scope and radius parameterization are those of [`super::fillet_along_wire`]. In
/// particular, the radius covers `[0, wire.len()]`, with one unit per edge, and must supply
/// derivatives up to second order and extend beyond the endpoints. Radius positivity and finite
/// derivatives are checked at sampled stations. Intersecting distant faces or another boundary
/// of the solid is outside the supported scope. Returns `None` on unsupported selections,
/// invalid parameters, failed construction, or non-closed output. The input is never modified.
/// Rounding the rim where an existing convex cylindrical fillet meets a perpendicular plane
/// requires a smaller radius; collapsed contact curves (equal radii) and folded offsets
/// (larger radii) are unsupported.
pub fn fillet_solid_along_wire<R: ScalarFunctionD1>(
    solid: &Solid,
    wire: &Wire,
    radius: R,
    tol: f64,
) -> Option<BlendResult> {
    try_fillet_solid_along_wire(solid, wire, radius, tol).ok()
}

/// Diagnostic variant of [`fillet_solid_along_wire`], preserving the input on failure.
pub fn try_fillet_solid_along_wire<R: ScalarFunctionD1>(
    solid: &Solid,
    wire: &Wire,
    radius: R,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "fillet_solid_along_wire";
    validate_tolerance(tol, operation)?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;

    let selected: Vec<_> = wire.iter().map(|e| e.id()).collect();
    let index = validate_selection(solid, &selected, operation)?;
    if !wire.is_continuous() {
        return Err(Diagnostic::new(
            Code::UnsupportedTopology,
            operation,
            "validate_input",
        ));
    }
    let shell = super::try_fillet_along_wire(&solid.boundaries()[index], wire, radius, tol)
        .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell).map_err(|e| e.operation(operation))
}

/// Equal-radius blends on selected edges of a modeling solid, including spherical corners.
///
/// The convex planar construction is described by [`super::fillet_edges`]. Nonconvex
/// boundaries also support orthogonal planar neighborhoods with three edges per touched
/// vertex, retaining unrelated curves and holes. Two convex or two concave edges meet at
/// an elliptical miter. Three edges of the same sense have a spherical corner; two convex
/// edges meeting a concave edge have a tangent toroidal corner. Other mixed junctions are
/// unsupported. Contacts must fit the adjacent faces and avoid holes within twice `tol`.
/// A single straight edge between planar faces can also terminate on curved faces, with
/// exact surface-intersection ends and extension along straight cylindrical rulings.
/// The operation must not intersect distant faces or another boundary. Returns `None` on
/// unsupported input without modifying it. An empty selection returns the original solid
/// and empty history.
pub fn fillet_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Option<BlendResult> {
    try_fillet_solid_edges(solid, edges, radius, tol).ok()
}

/// Diagnostic variant of [`fillet_solid_edges`], preserving the input on failure.
pub fn try_fillet_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "fillet_solid_edges";
    validate_tolerance(tol, operation)?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    positive(radius, "radius", operation)?;

    let Some(_) = edges.first() else {
        return Ok(BlendResult {
            solid: solid.clone(),
            modified_faces: Vec::new(),
            generated_faces: Vec::new(),
        });
    };
    let index = validate_selection(solid, edges, operation)?;
    let shell = super::try_fillet_edges(&solid.boundaries()[index], edges, radius, tol)
        .or_else(|error| match error.code {
            Code::UnsupportedGeometry | Code::UnsupportedTopology | Code::NonPlanarFace => {
                super::planar_fillet::fillet(
                    &solid.boundaries()[index],
                    &edges.iter().copied().collect(),
                    radius,
                    tol,
                )
                .or_else(|local| {
                    if edges.len() == 1
                        && matches!(
                            local.code,
                            Code::NonPlanarFace
                                | Code::UnsupportedGeometry
                                | Code::UnsupportedTopology
                        )
                    {
                        super::planar_edge::blend(
                            &solid.boundaries()[index],
                            edges[0],
                            super::planar_edge::Blend::Fillet(radius),
                            tol,
                        )
                    } else {
                        Err(local)
                    }
                })
                .map_err(|local| match local.code {
                    Code::UnsupportedGeometry | Code::UnsupportedTopology | Code::NonPlanarFace => {
                        error
                    }
                    _ => local,
                })
            }
            _ => Err(error),
        })
        .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell).map_err(|e| e.operation(operation))
}

/// Equal-distance chamfers on straight sharp edges with planar adjacent and end faces.
///
/// Each touched vertex must have three incident edges. Distances are measured on both adjacent
/// faces, perpendicular to each selected edge. Bevel planes trim one another: two meet
/// along a miter, shortening the unselected edge. Three meet at a point, except where two
/// convex edges meet a concave edge: a planar transition joins the common face to the
/// inside bevel. Reversing selection order has no effect.
/// Nonconvex shells and one-, two- and three-edge junctions are supported. Two-edge bevels
/// use a triangular transition when their contacts on the unselected edge differ.
/// Unrelated curved faces, curved boundary segments and inner loops are preserved. Contact
/// boundaries that approach a hole within twice `tol` are rejected. Convex chamfers remove
/// material; concave chamfers add material between the support faces. Curved geometry at a
/// touched vertex is outside this API's scope. Use
/// [`try_chamfer_solid_edges_with_distances`] for unequal distances.
/// Sizes that remove an original face, an unselected edge, or a bevel contact are rejected.
/// The chamfers must not intersect distant faces or another boundary of the solid.
///
/// Original face replacements precede the generated bevel faces in the history. Geometry
/// added by this operation consists of exact planes and lines; boundary sampling is used
/// only for validation, never for modeling or replacing existing curves.
/// An empty selection returns the original solid and empty history. Failure leaves it unchanged.
pub fn chamfer_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    distance: f64,
    tol: f64,
) -> Option<BlendResult> {
    try_chamfer_solid_edges(solid, edges, distance, tol).ok()
}

/// Diagnostic variant of [`chamfer_solid_edges`]. Unsupported junctions return
/// [`Code::UnsupportedTopology`]; distances that do not fit return [`Code::OutsideNeighbour`].
pub fn try_chamfer_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    distance: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_edges";
    validate_tolerance(tol, operation)?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    positive(distance, "distance", operation)?;
    if edges.is_empty() {
        return Ok(BlendResult {
            solid: solid.clone(),
            modified_faces: Vec::new(),
            generated_faces: Vec::new(),
        });
    }
    let index = validate_selection(solid, edges, operation)?;
    let shell =
        super::chamfer_edges::chamfer_edges(&solid.boundaries()[index], edges, distance, tol)
            .map_err(|e| e.shell(index))?;
    finish(solid, index, shell).map_err(|e| e.operation(operation))
}

/// Chamfers selected straight edges with separate distances on their two adjacent faces.
///
/// Each tuple contains an edge ID and distances in shell face order. All selected edges
/// must lie in one shell and have planar support and end faces with three edges per touched
/// vertex. Unrelated curved faces and inner boundary loops are preserved. Two-edge bevels
/// use a triangular transition when their contacts on the unselected edge differ. Mixed three-edge corners use a
/// transition when the convex bevels have matching setbacks on their common face;
/// otherwise they use a three-plane miter. Other restrictions are those of
/// [`chamfer_solid_edges`]. The input remains unchanged on failure.
pub fn try_chamfer_solid_edges_with_distances(
    solid: &Solid,
    edges: &[(EdgeID, [f64; 2])],
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_edges";
    validate_tolerance(tol, operation)?;
    for (_, distances) in edges {
        positive(distances[0], "d0", operation)?;
        positive(distances[1], "d1", operation)?;
    }
    if edges.is_empty() {
        return try_chamfer_solid_edges(solid, &[], 1.0, tol);
    }
    let ids: Vec<_> = edges.iter().map(|&(id, _)| id).collect();
    if edges.iter().all(|(_, d)| *d == [edges[0].1[0]; 2]) {
        return try_chamfer_solid_edges(solid, &ids, edges[0].1[0], tol);
    }
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    let index = validate_selection(solid, &ids, operation)?;
    let distances = edges.iter().copied().collect();
    let selected = ids.into_iter().collect();
    let shell =
        super::chamfer_miter::chamfer(&solid.boundaries()[index], &selected, &distances, tol)
            .map_err(|e| e.shell(index))?;
    finish(solid, index, shell).map_err(|e| e.operation(operation))
}

/// Chamfers a closed tangent-continuous wire of a modeling solid.
///
/// Distances and supported geometry are described by [`super::chamfer_along_wire`]. Reversing
/// the wire exchanges the sides associated with `d0` and `d1`. Returns `None` on invalid input
/// or failed construction without modifying the input solid.
pub fn chamfer_solid_along_wire(
    solid: &Solid,
    wire: &Wire,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Option<BlendResult> {
    try_chamfer_solid_along_wire(solid, wire, d0, d1, tol).ok()
}

/// Diagnostic variant of [`chamfer_solid_along_wire`], preserving the input on failure.
pub fn try_chamfer_solid_along_wire(
    solid: &Solid,
    wire: &Wire,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_along_wire";
    validate_tolerance(tol, operation)?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    positive(d0, "d0", operation)?;
    positive(d1, "d1", operation)?;

    let selected: Vec<_> = wire.iter().map(|e| e.id()).collect();
    let index = validate_selection(solid, &selected, operation)?;
    if !wire.is_continuous() {
        return Err(Diagnostic::new(
            Code::UnsupportedTopology,
            operation,
            "validate_input",
        ));
    }
    let shell = super::try_chamfer_along_wire(&solid.boundaries()[index], wire, d0, d1, tol)
        .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell).map_err(|e| e.operation(operation))
}

/// Chamfers a single edge of a modeling solid, trimming both end faces.
///
/// The edge must have two adjacent faces and a distinct third face at each end. `d0` and `d1`
/// are positive distances on the adjacent faces, ordered by their occurrence in the shell.
/// Straight edges between planar faces support curved terminations, including extension
/// along cylindrical rulings; their end curves remain exact surface intersections.
/// Distances have the meaning described by [`super::simple_chamfer`]. The chamfer must fit within
/// those faces and must not intersect distant faces or another shell. Returns `None` for invalid
/// parameters, unsupported topology or failed construction, without modifying the input.
pub fn chamfer_solid_edge(
    solid: &Solid,
    edge: EdgeID,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Option<BlendResult> {
    try_chamfer_solid_edge(solid, edge, d0, d1, tol).ok()
}

/// Diagnostic variant of [`chamfer_solid_edge`], preserving the input on failure.
pub fn try_chamfer_solid_edge(
    solid: &Solid,
    edge: EdgeID,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_edge";
    validate_tolerance(tol, operation)?;
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    let failed = || Diagnostic::new(Code::BlendConstructionFailed, operation, "construct_blend");
    positive(d0, "d0", operation)?;
    positive(d1, "d1", operation)?;

    match try_chamfer_solid_edges_with_distances(solid, &[(edge, [d0, d1])], tol) {
        Ok(result) => return Ok(result),
        Err(error)
            if matches!(
                error.code,
                Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
            ) => {}
        Err(error) => return Err(error.operation(operation)),
    }

    let index = validate_selection(solid, &[edge], operation)?;
    let shell = &solid.boundaries()[index];
    match super::planar_edge::blend(
        shell,
        edge,
        super::planar_edge::Blend::Chamfer([d0, d1]),
        tol,
    ) {
        Ok(result) => return finish(solid, index, result),
        Err(error) if error.code == Code::OutsideNeighbour => return Err(error),
        Err(_) => {}
    }
    let adjacent: Vec<_> = shell
        .iter()
        .enumerate()
        .filter(|(_, f)| f.edge_iter().any(|e| e.id() == edge))
        .map(|(i, _)| i)
        .collect();
    let [a, b] = adjacent.as_slice() else {
        return Err(
            Diagnostic::new(Code::UnsupportedTopology, operation, "validate_input").shell(index),
        );
    };
    let oriented = shell[*a]
        .edge_iter()
        .find(|e| e.id() == edge)
        .ok_or_else(failed)?;
    let mut ends = Vec::new();
    for vertex in [oriented.front(), oriented.back()] {
        let faces: Vec<_> = shell
            .iter()
            .enumerate()
            .filter(|(i, f)| *i != *a && *i != *b && f.vertex_iter().any(|v| v.id() == vertex.id()))
            .map(|(i, _)| i)
            .collect();
        if faces.len() != 1 {
            return Err(
                Diagnostic::new(Code::UnsupportedTopology, operation, "validate_input")
                    .shell(index),
            );
        }
        ends.push(faces[0]);
    }
    if ends[0] == ends[1] {
        return Err(
            Diagnostic::new(Code::UnsupportedTopology, operation, "validate_input").shell(index),
        );
    }
    let blend = super::chamfer_with_side(
        &shell[*a],
        &shell[*b],
        edge,
        Some(&shell[ends[0]]),
        Some(&shell[ends[1]]),
        d0,
        d1,
        tol,
    )
    .ok_or_else(failed)?;
    let mut result = shell.clone();
    result[*a] = blend.simple_fillet.face0;
    result[*b] = blend.simple_fillet.face1;
    result[ends[0]] = blend.side0.ok_or_else(failed)?;
    result[ends[1]] = blend.side1.ok_or_else(failed)?;
    result.push(blend.simple_fillet.fillet);
    finish(solid, index, result).map_err(|e| e.operation(operation))
}

fn positive(
    value: f64,
    parameter: &'static str,
    operation: &'static str,
) -> Result<(), Diagnostic> {
    if !value.is_finite() || value <= 0.0 {
        Err(
            Diagnostic::new(Code::InvalidParameter, operation, "validate_input")
                .parameter(parameter, value),
        )
    } else {
        Ok(())
    }
}

fn validate_selection(
    solid: &Solid,
    edges: &[EdgeID],
    operation: &'static str,
) -> Result<usize, Diagnostic> {
    let mut seen = std::collections::HashSet::new();
    let mut shell = None;
    for (i, &edge) in edges.iter().enumerate() {
        if !seen.insert(edge) {
            return Err(
                Diagnostic::new(Code::DuplicateSelection, operation, "validate_input").selection(i),
            );
        }
        let index = shell_index(solid, edge).ok_or_else(|| {
            Diagnostic::new(Code::UnknownEdge, operation, "validate_input").selection(i)
        })?;
        if shell.is_some_and(|s| s != index) {
            return Err(
                Diagnostic::new(Code::MultipleShells, operation, "validate_input")
                    .selection(i)
                    .shell(index),
            );
        }
        shell = Some(index);
    }
    shell.ok_or_else(|| Diagnostic::new(Code::EmptySelection, operation, "validate_input"))
}

#[cfg(test)]
mod output_tests {
    use super::*;
    use truck_modeling::*;

    #[test]
    fn topological_closure_does_not_accept_off_surface_boundaries() {
        let input: Solid = primitive::cuboid(BoundingBox::from_iter([
            Point3::origin(),
            Point3::new(10., 10., 10.),
        ]));
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut shell = input.boundaries()[0].clone();
        let surface = shell[0]
            .oriented_surface()
            .transformed(Matrix4::from_translation(Vector3::new(1., 1., 1.)));
        shell[0] = Face::new(shell[0].boundaries(), surface);
        assert_eq!(shell.shell_condition(), ShellCondition::Closed);
        assert!(!shell.is_geometric_consistent());
        let error = finish(&input, 0, shell).unwrap_err();
        assert_eq!(error.code, Code::BlendConstructionFailed);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
