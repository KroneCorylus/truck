//! Blend operations on modeling solids.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};
use truck_modeling::{EdgeID, Face, FaceID, Rad, ScalarFunctionD1, Shell, Solid, Surface, Wire};

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
    /// Input face ID and its surviving replacements, in input face order. Unchanged faces are omitted.
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

fn finish(solid: &Solid, index: usize, shell: Shell, tol: f64) -> Result<BlendResult, Diagnostic> {
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
    for (i, (old, face)) in original.iter().zip(&shell).enumerate() {
        if old.id() == face.id() {
            continue;
        }
        if let Surface::Plane(plane) = old.oriented_surface() {
            let boundaries = face.boundaries();
            if !super::planar::valid_boundaries(old, &boundaries, plane.normal(), tol) {
                return Err(
                    Diagnostic::new(Code::OutsideNeighbour, "blend", "validate_trims")
                        .shell(index)
                        .face(i),
                );
            }
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
/// The base construction and radius parameterization are those of [`super::fillet_along_wire`]. In
/// particular, the radius covers `[0, wire.len()]`, with one unit per edge, and must supply
/// derivatives up to second order and extend beyond the endpoints. Radius positivity and finite
/// derivatives are checked at sampled stations. Closed-wire contacts can continue across a
/// planar outer boundary onto one tangent adjacent support per selected edge and side. The
/// continued contacts must remain inside their supporting faces; their shared trim vertices
/// reconnect the original boundary loops. Intersecting distant faces or another boundary
/// of the solid is outside the supported scope. Returns `None` on unsupported selections,
/// invalid parameters, failed construction, or non-closed output. The input is never modified.
/// Exact straight boundaries at open ends may extend toward the selected endpoint to meet
/// a reentrant contact; curved end boundaries retain their existing trimmed domains.
/// A constant-radius rim on perpendicular planar and analytic cylindrical walls may collapse
/// a convex cylindrical contact to one vertex when both radii are equal. The corner becomes
/// spherical; larger convex cylindrical corners retain circular contacts and toroidal patches,
/// while straight rim sections remain cylindrical. The cap must retain a nonzero boundary,
/// and open ends require straight transverse boundaries. Analytic recognition includes
/// procedural cylinders with proven translated circular sections. Folded offsets (larger
/// radii) remain unsupported by this collapsed-contact construction.
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
    let shell = super::try_fillet_along_wire(&solid.boundaries()[index], wire, radius.clone(), tol)
        .or_else(|error| {
            if error.code != Code::BlendConstructionFailed {
                return Err(error);
            }
            super::collapsed_contacts::fillet(&solid.boundaries()[index], wire, radius.clone())
                .ok_or(error)
        })
        .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell, tol)
        .or_else(|error| {
            if error.code != Code::OutsideNeighbour {
                return Err(error);
            }
            let shell =
                super::continued_support::fillet(&solid.boundaries()[index], wire, radius, tol)
                    .ok_or(error)?;
            finish(solid, index, shell, tol)
        })
        .map_err(|e| e.operation(operation))
}

/// Equal-radius blends on selected edges of a modeling solid, including spherical corners.
///
/// The convex planar construction is described by [`super::fillet_edges`].
/// Convex vertices with four or more incident edges support a spherical corner when all
/// edges are selected and every inward-offset support plane meets at the same center.
///
/// Nonconvex boundaries also support planar neighborhoods with three edges per touched vertex,
/// retaining unrelated curves and holes. A single edge ends on the plane of the remaining
/// face, elliptically when oblique. Two convex or two concave edges meet at an elliptical
/// miter when their shared face is perpendicular to the others. Three edges of the same
/// sense have a spherical corner at any angles. One or two convex edges meeting a concave
/// edge have a tangent toroidal corner when the face the convex edges share is
/// perpendicular to the two walls, which may meet at any angle. Other mixed junctions are
/// unsupported. Contacts must fit the adjacent faces and avoid holes within twice `tol`.
/// Three selected edges meeting between an outer cylinder, a perpendicular cap and a
/// radial plane also support an exact spherical corner, joining two cylindrical rounds
/// and a toroidal rim. The radius must stay below half the outer cylinder radius and
/// the contacts must fit all three remote terminations.
/// Straight edges between planes or a plane and an axial cylindrical ruling also support
/// curved end faces. Connected channel rims meet at exact intersections of the blend
/// surfaces, including elliptical miters between equal-radius cylinders. Contacts may
/// extend across adjacent end-face patches while preserving the original support surfaces.
/// This intersection construction also handles drafted planar rim junctions and recognizes
/// planar B-spline and positive-weight NURBS supports without changing their representation.
/// A single selected edge may end at several planar faces, such as the miter between two
/// earlier chamfers; its cylindrical round is trimmed against each face. Other selected
/// edges can still meet at three-edge spherical corners in the same operation.
/// Parallel concave edges ending on transverse planar caps also support stepped rib joints
/// and split coplanar caps. Equal-radius rounds may touch and consume their shared planar
/// support; overlapping contact patches are rejected. Face history retains split replacements.
/// Multiple open or closed tangent contours can also be rounded using
/// [`fillet_solid_along_wire`]. They may share original support faces, but no component may
/// modify a fillet generated by an earlier component. Components follow input-shell order,
/// independent of selection order; failure in any component leaves the input unchanged.
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
                    if matches!(
                        local.code,
                        Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
                    ) {
                        super::planar_edge::blend_edges(
                            &solid.boundaries()[index],
                            &edges
                                .iter()
                                .map(|&id| (id, super::planar_edge::Blend::Fillet(radius)))
                                .collect::<Vec<_>>(),
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
        .map_err(|e| e.operation(operation).shell(index));
    match shell {
        Ok(shell) => finish(solid, index, shell, tol).map_err(|e| e.operation(operation)),
        Err(error) => super::additive::fillet(solid, edges, radius, tol).or_else(|_| {
            if matches!(
                error.code,
                Code::UnsupportedGeometry | Code::UnsupportedTopology | Code::NonPlanarFace
            ) {
                if let Some(result) = separate_contours(solid, index, edges, radius, tol) {
                    return Ok(result);
                }
            }
            Err(error)
        }),
    }
}

/// Fillets selected edges with one positive, finite constant radius per edge.
///
/// Equal radii use [`try_fillet_solid_edges`] and retain its supported scope.
/// Unequal radii support separate straight edges between planes or an axial cylindrical
/// support, with the end-face restrictions of the existing straight-edge construction.
/// Selected edges must belong to one shell and must not share vertices. Contacts must fit
/// the supporting faces; unequal-radius corner junctions are unsupported. This operation
/// does not detect intersections with distant faces or other shells.
/// Selection order does not affect construction order. Face history relates the original
/// solid to the final result. Empty selection returns the original solid and empty history;
/// any failure leaves the input unchanged. Invalid radii identify their selection index.
pub fn try_fillet_solid_edges_with_radii(
    solid: &Solid,
    edges: &[(EdgeID, f64)],
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "fillet_solid_edges";
    validate_tolerance(tol, operation)?;
    for (i, &(_, radius)) in edges.iter().enumerate() {
        positive(radius, "radius", operation).map_err(|e| e.selection(i))?;
    }
    let ids: Vec<_> = edges.iter().map(|&(id, _)| id).collect();
    let radius = edges.first().map_or(1.0, |&(_, r)| r);
    if edges.iter().all(|&(_, r)| r == radius) {
        return try_fillet_solid_edges(solid, &ids, radius, tol);
    }
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    let index = validate_selection(solid, &ids, operation)?;
    let shell = &solid.boundaries()[index];
    let mut vertices = HashSet::default();
    for (i, id) in ids.iter().enumerate() {
        let edge = shell.edge_iter().find(|e| e.id() == *id).unwrap();
        if [edge.front(), edge.back()]
            .iter()
            .any(|v| !vertices.insert(v.id()))
        {
            return Err(
                Diagnostic::new(Code::UnsupportedTopology, operation, "validate_input")
                    .selection(i)
                    .shell(index),
            );
        }
    }
    let shell = super::planar_edge::blend_edges(
        shell,
        &edges
            .iter()
            .map(|&(id, r)| (id, super::planar_edge::Blend::Fillet(r)))
            .collect::<Vec<_>>(),
        tol,
    )
    .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell, tol).map_err(|e| e.operation(operation))
}

fn separate_contours(
    solid: &Solid,
    index: usize,
    selected: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Option<BlendResult> {
    let shell = &solid.boundaries()[index];
    let mut selected: HashSet<_> = selected.iter().copied().collect();
    let mut edges: Vec<_> = shell
        .edge_iter()
        .filter(|e| selected.remove(&e.id()))
        .collect();
    let mut degree = HashMap::default();
    for edge in &edges {
        for vertex in [edge.front(), edge.back()] {
            *degree.entry(vertex.id()).or_insert(0) += 1;
        }
    }
    if degree.values().any(|&n| n > 2) {
        return None;
    }
    let mut wires = Vec::new();
    while !edges.is_empty() {
        let mut wire = Wire::from(vec![edges.remove(0)]);
        while !wire.is_cyclic() {
            let front = wire.front_vertex()?;
            let back = wire.back_vertex()?;
            let Some(i) = edges.iter().position(|e| {
                [e.front(), e.back()]
                    .iter()
                    .any(|v| *v == front || *v == back)
            }) else {
                break;
            };
            let edge = edges.remove(i);
            if edge.front() == back {
                wire.push_back(edge);
            } else if edge.back() == back {
                wire.push_back(edge.inverse());
            } else if edge.back() == front {
                wire.push_front(edge);
            } else {
                wire.push_front(edge.inverse());
            }
        }
        if !super::along_wire::is_tangent_continuous(&wire, wire.is_cyclic()) {
            return None;
        }
        wires.push(wire);
    }
    if wires.len() < 2 {
        return None;
    }
    let mut current = solid.clone();
    let mut generated = HashSet::default();
    for wire in wires {
        if !wire
            .iter()
            .all(|edge| current.edge_iter().any(|e| e.id() == edge.id()))
        {
            return None;
        }
        let result = try_fillet_solid_along_wire(&current, &wire, radius, tol).ok()?;
        if result
            .modified_faces
            .iter()
            .any(|(id, _)| generated.contains(id))
        {
            return None;
        }
        generated.extend(result.generated_faces.iter().map(|f| f.id()));
        current = result.solid;
    }
    finish(solid, index, current.boundaries()[index].clone(), tol).ok()
}

/// Equal-distance chamfers on sharp edges with planar or cylindrical support.
///
/// Distances are measured on both adjacent faces, perpendicular to each selected edge.
/// Bevel planes trim one another: two meet along a miter, shortening the unselected edge.
/// Three meet at a point, except where two
/// convex edges meet a concave edge: a planar transition joins the common face to the
/// inside bevel. Reversing selection order has no effect.
/// Nonconvex shells and one-, two- and three-edge junctions are supported. Two-edge bevels
/// use a triangular transition when their contacts on the unselected edge differ.
/// Planar intersections also handle some higher-valence junctions, including one through
/// four selected ridges at a square pyramid's apex.
/// Unrelated curved faces, curved boundary segments and inner loops are preserved. Contact
/// boundaries that approach a hole within twice `tol` are rejected. Convex chamfers remove
/// material; concave chamfers add material between the support faces. Curved geometry at a
/// touched vertex is supported for straight plane/cylinder lips and their planar rim edges.
/// These bevels meet at exact surface intersections. Use
/// [`try_chamfer_solid_edges_with_distances`] for unequal distances.
/// Separate tangent-continuous wires can also be chamfered together using the wire API's
/// supported geometry. They may share original support faces, but a later wire must not
/// modify an earlier bevel. Components follow input-shell order, independent of selection order.
/// Sizes that remove an original face, an unselected edge, or a bevel contact are rejected.
/// The chamfers must not intersect distant faces or another boundary of the solid.
///
/// Original face replacements precede the generated bevel faces in the history. Geometry
/// added by this operation consists of exact planes and intersection curves. Boundary sampling
/// is used only for validation, never for modeling or replacing existing curves.
/// An empty selection returns the original solid and empty history. Failure leaves it unchanged.
pub fn chamfer_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    distance: f64,
    tol: f64,
) -> Option<BlendResult> {
    try_chamfer_solid_edges(solid, edges, distance, tol).ok()
}

/// An isolated chamfer may consume short neighboring edges while retaining their support faces.
/// Planar chamfers may cross curved terminal edges within their existing parameter bounds.
///
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
            .or_else(|error| {
                let retry_outside = edges.len() == 1 && error.code == Code::OutsideNeighbour;
                if matches!(
                    error.code,
                    Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
                ) || retry_outside
                {
                    super::planar_edge::blend_edges(
                        &solid.boundaries()[index],
                        &edges
                            .iter()
                            .map(|&id| (id, super::planar_edge::Blend::Chamfer([distance; 2])))
                            .collect::<Vec<_>>(),
                        tol,
                    )
                    .map_err(|next| if retry_outside { error } else { next })
                } else {
                    Err(error)
                }
            })
            .or_else(|error| {
                if matches!(
                    error.code,
                    Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
                ) {
                    separate_chamfer_wires(
                        solid,
                        index,
                        &edges
                            .iter()
                            .map(|&id| (id, [distance; 2]))
                            .collect::<Vec<_>>(),
                        tol,
                    )
                    .ok_or(error)
                } else {
                    Err(error)
                }
            })
            .map_err(|e| e.shell(index))?;
    finish(solid, index, shell, tol).map_err(|e| e.operation(operation))
}

/// Chamfers selected edges with separate distances on their two adjacent faces.
///
/// Each tuple contains an edge ID and distances in shell face order. All selected edges
/// must lie in one shell. Planar neighborhoods have three edges per touched vertex;
/// straight plane/cylinder lips and connected planar rim edges are also supported.
/// Separate tangent-continuous wires require constant distances on each oriented side of
/// each wire. Different wires may use different distances and share original support faces;
/// their trims must not modify a bevel generated by another selected wire.
/// An isolated chamfer may consume short neighboring edges while retaining their support faces.
/// Planar chamfers may cross curved terminal edges within their existing parameter bounds.
/// On cylindrical supports, distances use the nearest point to the perpendicular tangent
/// offset. Unrelated curved faces and inner boundary loops are preserved. Two-edge bevels
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
            .or_else(|error| {
                let retry_outside = edges.len() == 1 && error.code == Code::OutsideNeighbour;
                if matches!(
                    error.code,
                    Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
                ) || retry_outside
                {
                    super::planar_edge::blend_edges(
                        &solid.boundaries()[index],
                        &edges
                            .iter()
                            .map(|&(id, d)| (id, super::planar_edge::Blend::Chamfer(d)))
                            .collect::<Vec<_>>(),
                        tol,
                    )
                    .map_err(|next| if retry_outside { error } else { next })
                } else {
                    Err(error)
                }
            })
            .or_else(|error| {
                if matches!(
                    error.code,
                    Code::NonPlanarFace | Code::UnsupportedGeometry | Code::UnsupportedTopology
                ) {
                    separate_chamfer_wires(solid, index, edges, tol).ok_or(error)
                } else {
                    Err(error)
                }
            })
            .map_err(|e| e.shell(index))?;
    finish(solid, index, shell, tol).map_err(|e| e.operation(operation))
}

fn separate_chamfer_wires(
    solid: &Solid,
    index: usize,
    selected: &[(EdgeID, [f64; 2])],
    tol: f64,
) -> Option<Shell> {
    let shell = &solid.boundaries()[index];
    let mut ids: HashSet<_> = selected.iter().map(|(id, _)| *id).collect();
    let mut edges: Vec<_> = shell.edge_iter().filter(|e| ids.remove(&e.id())).collect();
    let mut degree = HashMap::default();
    for edge in &edges {
        for vertex in [edge.front(), edge.back()] {
            *degree.entry(vertex.id()).or_insert(0) += 1;
        }
    }
    if degree.values().any(|&n| n > 2) {
        return None;
    }
    let mut wires = Vec::new();
    while !edges.is_empty() {
        let mut wire = Wire::from(vec![edges.remove(0)]);
        while !wire.is_cyclic() {
            let front = wire.front_vertex()?;
            let back = wire.back_vertex()?;
            let Some(i) = edges.iter().position(|e| {
                [e.front(), e.back()]
                    .iter()
                    .any(|v| *v == front || *v == back)
            }) else {
                break;
            };
            let edge = edges.remove(i);
            if edge.front() == back {
                wire.push_back(edge);
            } else if edge.back() == back {
                wire.push_back(edge.inverse());
            } else if edge.back() == front {
                wire.push_front(edge);
            } else {
                wire.push_front(edge.inverse());
            }
        }
        if !super::along_wire::is_tangent_continuous(&wire, wire.is_cyclic()) {
            return None;
        }
        let distances: Option<Vec<_>> = wire
            .iter()
            .map(|edge| {
                let original = shell.edge_iter().find(|e| e.id() == edge.id())?;
                let mut d = selected.iter().find(|(id, _)| *id == edge.id())?.1;
                if edge.orientation() != original.orientation() {
                    d.reverse();
                }
                Some(d)
            })
            .collect();
        let distances = distances?;
        let d = distances[0];
        if !distances
            .iter()
            .all(|p| (p[0] - d[0]).abs() <= 1e-12 * d[0] && (p[1] - d[1]).abs() <= 1e-12 * d[1])
        {
            return None;
        }
        wires.push((wire, d));
    }
    if wires.len() < 2 {
        return None;
    }
    let mut current = solid.clone();
    let mut generated = HashSet::default();
    for (wire, d) in wires {
        if !wire
            .iter()
            .all(|edge| current.edge_iter().any(|e| e.id() == edge.id()))
        {
            return None;
        }
        let next = try_chamfer_solid_along_wire(&current, &wire, d[0], d[1], tol)
            .or_else(|error| {
                if wire.len() != 1
                    || !matches!(
                        error.code,
                        Code::BlendConstructionFailed
                            | Code::UnsupportedGeometry
                            | Code::UnsupportedTopology
                            | Code::NonPlanarFace
                    )
                {
                    return Err(error);
                }
                let edge = &wire[0];
                let Some(original) = current.edge_iter().find(|e| e.id() == edge.id()) else {
                    return Err(error);
                };
                let d = if original.orientation() == edge.orientation() {
                    d
                } else {
                    [d[1], d[0]]
                };
                try_chamfer_solid_edge(&current, edge.id(), d[0], d[1], tol)
            })
            .ok()?;
        if next
            .modified_faces
            .iter()
            .any(|(id, _)| generated.contains(id))
        {
            return None;
        }
        generated.extend(next.generated_faces.iter().map(|f| f.id()));
        current = next.solid;
    }
    Some(current.boundaries()[index].clone())
}

/// Chamfers an open or closed tangent-continuous wire of a modeling solid.
///
/// Distances and supported geometry are described by [`super::chamfer_along_wire`]. Reversing
/// the wire exchanges the sides associated with `d0` and `d1`. Returns `None` on invalid input
/// or failed construction without modifying the input solid.
/// On a planar rim bounded by perpendicular planes and convex cylinders, an inset may
/// consume circular contacts. These corners end in exact conical tips; adjacent planar
/// chamfers share a ridge beyond each tip. Consumed arcs must be bounded by straight edges.
/// Open chains must terminate on straight edges and the cuts must retain the support faces.
/// At an open end, a straight terminal edge may extend toward the new contact, allowing
/// chamfers at concave radial steps. Interior and closed-chain edges remain bounded.
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
    chamfer_solid_wire(
        solid,
        wire,
        (d0, d1),
        tol,
        super::chamfer::DistanceMode::TangentProjection,
    )
}

/// Chamfers a tangent wire with straight-line distances in planes perpendicular to its edge.
///
/// Contacts stay on the local inward branch of each support. Unlike tangent-projected offsets,
/// contacts are measured `d0` or `d1` from the guide in its normal section. Distances are
/// ordered along and against the wire. Trimming, history and input-preservation guarantees
/// are the same as [`try_chamfer_solid_along_wire`].
pub fn try_chamfer_solid_along_wire_chord(
    solid: &Solid,
    wire: &Wire,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    chamfer_solid_wire(
        solid,
        wire,
        (d0, d1),
        tol,
        super::chamfer::DistanceMode::NormalSection,
    )
}

/// Chamfers a tangent wire with a reference chord and a bevel angle in each normal section.
///
/// `reference` identifies an adjacent face and therefore the oriented reference side of the
/// wire. The chord is measured from the guide to that side's new contact. The angle, strictly
/// between zero and pi/2, is measured from its local section tangent toward the other support.
/// Both contacts remain on their local inward branches. Tangent cylinder/plane walls under a
/// shared planar cap may continue their contacts and join cap endpoints within `tol`.
pub fn try_chamfer_solid_along_wire_angle(
    solid: &Solid,
    wire: &Wire,
    reference: FaceID,
    distance: f64,
    angle: Rad<f64>,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_along_wire_angle";
    validate_tolerance(tol, operation)?;
    positive(distance, "distance", operation)?;
    if !angle.0.is_finite() || angle.0 <= 0. || angle.0 >= std::f64::consts::FRAC_PI_2 {
        return Err(
            Diagnostic::new(Code::InvalidParameter, operation, "validate_input")
                .parameter("angle", angle.0),
        );
    }
    if wire.is_empty() {
        return Err(Diagnostic::new(
            Code::EmptySelection,
            operation,
            "validate_input",
        ));
    }
    let invalid_reference =
        || Diagnostic::new(Code::UnsupportedTopology, operation, "validate_reference");
    let face = solid
        .face_iter()
        .find(|f| f.id() == reference)
        .ok_or_else(invalid_reference)?;
    let mut reference_side = None;
    for edge in wire {
        if let Some(boundary) = face.edge_iter().find(|e| e.id() == edge.id()) {
            let side = boundary.front() != edge.front();
            if reference_side.replace(side).is_some_and(|old| old != side) {
                return Err(invalid_reference());
            }
        }
    }
    let reference_side = reference_side.ok_or_else(invalid_reference)?;
    chamfer_solid_wire(
        solid,
        wire,
        (distance, distance),
        tol,
        super::chamfer::DistanceMode::NormalSectionAngle {
            reference_side,
            radians: angle.0,
        },
    )
    .map_err(|e| e.operation(operation))
}

fn chamfer_solid_wire(
    solid: &Solid,
    wire: &Wire,
    (d0, d1): (f64, f64),
    tol: f64,
    mode: super::chamfer::DistanceMode,
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
    let trim = |curve: &mut truck_modeling::Curve, edge: &truck_modeling::Edge, start| {
        let open_end = !wire.is_cyclic()
            && [wire.front_vertex(), wire.back_vertex()]
                .into_iter()
                .flatten()
                .any(|vertex| vertex == edge.front() || vertex == edge.back());
        if open_end {
            super::along_wire::trim_extended_contact_edge::<_, Surface>(curve, edge, start)
        } else {
            super::along_wire::trim_contact_edge::<_, Surface>(curve, edge, start)
        }
    };
    let build = |continued| {
        super::chamfer::try_chamfer_along_wire_with_trim(
            &solid.boundaries()[index],
            wire,
            (d0, d1),
            tol,
            trim,
            continued,
            mode,
        )
    };
    let shell = build(false)
        .or_else(|error| {
            if error.code == Code::BlendConstructionFailed
                && error.stage == "trim_faces"
                && mode != super::chamfer::DistanceMode::NormalSection
                && can_continue_chamfer_contacts(&solid.boundaries()[index], wire)
            {
                build(true)
            } else {
                Err(error)
            }
        })
        .or_else(|error| {
            if mode == super::chamfer::DistanceMode::TangentProjection {
                super::chamfer_collapse::chamfer(&solid.boundaries()[index], wire, d0, d1, tol)
                    .ok_or(error)
            } else {
                Err(error)
            }
        })
        .map_err(|e| e.operation(operation).shell(index))?;
    finish(solid, index, shell, tol).map_err(|e| e.operation(operation))
}

fn can_continue_chamfer_contacts(shell: &Shell, wire: &Wire) -> bool {
    use truck_modeling::*;
    if !wire.iter().any(|e| matches!(e.curve(), Curve::Conic(_)))
        || wire
            .iter()
            .any(|e| !matches!(e.curve(), Curve::Line(_) | Curve::Conic(_)))
    {
        return false;
    }
    let Some(cap) = shell.iter().find(|f| {
        matches!(
            f.oriented_surface().elementary(),
            Some((Elementary::Plane(_), _))
        ) && wire.iter().all(|e| f.edge_iter().any(|q| q.id() == e.id()))
    }) else {
        return false;
    };
    let walls: Vec<_> = shell
        .iter()
        .filter(|f| {
            f.id() != cap.id() && f.edge_iter().any(|e| wire.iter().any(|q| q.id() == e.id()))
        })
        .map(|f| f.oriented_surface().elementary())
        .collect();
    let Some(axis) = walls.iter().find_map(|w| match w {
        Some((Elementary::Cylinder { axis, .. }, true)) => Some(*axis),
        _ => None,
    }) else {
        return false;
    };
    let Some((Elementary::Plane(plane), _)) = cap.oriented_surface().elementary() else {
        return false;
    };
    if plane.normal().dot(axis).abs() < 1e-8 {
        return false;
    }
    walls.iter().all(|w| match w {
        Some((Elementary::Plane(p), _)) => p.normal().dot(axis).abs() < 1e-8,
        Some((Elementary::Cylinder { axis: other, .. }, true)) => {
            axis.cross(*other).magnitude() < 1e-8
        }
        _ => false,
    })
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
        Ok(result) => return finish(solid, index, result, tol),
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
    finish(solid, index, result, tol).map_err(|e| e.operation(operation))
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
        let error = finish(&input, 0, shell, 0.001).unwrap_err();
        assert_eq!(error.code, Code::BlendConstructionFailed);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
