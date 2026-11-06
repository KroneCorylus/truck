//! Blend operations on modeling solids.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};
use truck_modeling::{
    EdgeID, Face, FaceID, Rad, ScalarFunctionD1, Shell, Solid, Surface, VertexID, Wire,
};

#[cfg(all(
    feature = "i2-reviewed-glibc",
    not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))
))]
compile_error!("i2-reviewed-glibc requires the audited x86_64 GNU/Linux deployment");
#[cfg(feature = "i2-reviewed-glibc")]
#[path = "i2_runtime.rs"]
mod i2_runtime;

/// Attempts only the bounded three-edge revolved junction family before contour splitting.
/// Returns None when ineligible; the caller retains its ordinary routing and diagnostics.
/// The input and calling thread's FP control state are never changed.
fn fillet_solid_revolved_junction(
    solid: &Solid,
    edges: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Option<BlendResult> {
    #[cfg(not(feature = "i2-reviewed-glibc"))]
    {
        let _ = (solid, edges, radius, tol);
        None
    }
    #[cfg(feature = "i2-reviewed-glibc")]
    {
        if solid.boundaries().len() != 1 || edges.len() != 3 || !i2_runtime::eligible() {
            return None;
        }
        validate_tolerance(tol, "fillet_solid_edges").ok()?;
        positive(radius, "radius", "fillet_solid_edges").ok()?;
        Solid::try_new(solid.boundaries().clone()).ok()?;
        validate_selection(solid, edges, "fillet_solid_edges").ok()?;
        let selected = edges.iter().copied().collect();
        let shell =
            super::planar_fillet::i2::build_for_solid(&solid.boundaries()[0], &selected, radius)?;
        let result = finish(solid, 0, shell, tol).ok()?;
        i2_runtime::eligible().then_some(result)
    }
}

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
/// An outward spherical support and two planes also admit a three-edge spherical corner,
/// joining a cylindrical round to two exact toroidal blends. The inward-offset sphere and
/// planes must intersect, and the contact curves must reach their remote planar boundaries
/// without consuming a support face. This includes corners at a sphere pole.
/// Outward cones also support three-edge corners with radial planes and perpendicular
/// caps. Exact offset-cone intersections locate the sphere; cone/radial-plane blends
/// follow hyperbolic rolling centers, and cone/cap blends are toroidal. The contacts
/// must remain inside their original neighbouring boundaries.
/// Outward toroidal supports with perpendicular caps and radial walls admit the same
/// three-edge convex corner construction. The offset torus locates the sphere and
/// fitted rolling contacts preserve the meridian branch, including nonconvex profiles.
/// A trivalent corner with one concave rim and two convex edges sharing a planar
/// wall can instead round the concave rim first, then its three-edge tangent chain.
/// Internal open-chain contacts continue across the common support when needed;
/// remote terminals and final trim checks still bound the construction.
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
/// Separate components may combine planar corner chains and tangent curved rims.
/// Tangent components use [`fillet_solid_along_wire`]; corner chains use this edge API.
/// They may share original support faces, but no component may
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
    try_fillet_solid_edges_impl(solid, edges, radius, tol, true)
}

fn try_fillet_solid_edges_impl(
    solid: &Solid,
    edges: &[EdgeID],
    radius: f64,
    tol: f64,
    allow_i2: bool,
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
    if allow_i2 {
        if let Some(result) = fillet_solid_revolved_junction(solid, edges, radius, tol) {
            return Ok(result);
        }
    }
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
                if let Some(result) = mixed_curved_corner(solid, index, edges, radius, tol) {
                    return Ok(result);
                }
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
/// Selected edges must belong to one shell. Two adjacent edges of an unsplit rectangular
/// box may meet in a two-cylinder crease miter, with positive distinct radii and clearance
/// from all opposite supports. Radii and their difference must exceed the construction
/// clearance (`max(8 * tol, 100 * TOLERANCE)`). Cavities and other unequal shared-vertex
/// selections are unsupported.
/// Contacts must fit the supporting faces. This operation
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
    if let Some(result) = (solid.boundaries().len() == 1)
        .then(|| super::planar_edge::unequal_box(shell, edges, tol))
        .flatten()
    {
        return finish(solid, index, result, tol).map_err(|e| e.operation(operation));
    }
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
        let result = if super::along_wire::is_tangent_continuous(&wire, wire.is_cyclic()) {
            try_fillet_solid_along_wire(&current, &wire, radius, tol)
        } else {
            let selected: Vec<_> = wire.iter().map(|e| e.id()).collect();
            try_fillet_solid_edges_impl(&current, &selected, radius, tol, false)
        }
        .ok()?;
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
/// Nonconvex shells and one-, two- and three-edge junctions are supported. Convex two-edge
/// bevels trim along their plane intersection; concave junctions retain their transition.
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
    let shell = super::chamfer_edges::chamfer_edges(
        &solid.boundaries()[index],
        solid.boundaries().len() == 1,
        edges,
        distance,
        tol,
    )
    .or_else(|error| {
        if error.stage == "certify_planar_corner" {
            return Err(error);
        }
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

/// Resolves a supported tied chamfer corner by selecting an incident edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChamferCornerPreference {
    /// Original corner vertex.
    pub vertex: VertexID,
    /// Selected incident edge whose bevel is the pivot; A6 requires a cap edge.
    pub edge: EdgeID,
}

/// Chamfers selected edges with separate distances on their two adjacent faces.
///
/// Preferences select an incident cap-edge pivot at tied orthogonal planar corners of
/// a fully selected convex one-cylinder network with equal cap setbacks twice the axis
/// setback. Bounded partial tangent networks also accept the regular section pivot
/// where two orthogonal convex planar candidates compete between that section and
/// an exactly collapsed triangle;
/// the triangle preference is unsupported. Supply one preference per tied vertex.
/// Missing preferences for those ambiguous partial corners reject.
/// Explicit signed-section preferences also support certified convex corners with no
/// monotone pivot, and a fully reentrant corner with one monotone section pivot.
/// These require an embedded projected chart, source-sector contacts and retained
/// trim/footprint certificates. The same-concavity physical-length collapse policy
/// applies to the nominated section, including a triangle result at a convex corner.
/// Existing unannotated reentrant triangle cases remain supported. The regular,
/// noncollapsed reentrant section family requires a preference and rejects without it.
/// Preferences are unordered and must be unique; calls carrying them never use fallback
/// constructions that discard the chosen pivot.
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
/// offset. Unrelated curved faces and inner boundary loops are preserved. Convex two-edge
/// bevels trim along their plane intersection; concave junctions retain their transition. Mixed three-edge corners use a
/// transition when the convex bevels have matching setbacks on their common face;
/// otherwise they use a three-plane miter. Other restrictions are those of
/// [`chamfer_solid_edges`]. The input remains unchanged on failure.
pub fn try_chamfer_solid_edges_with_distances(
    solid: &Solid,
    edges: &[(EdgeID, [f64; 2])],
    preferences: &[ChamferCornerPreference],
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let operation = "chamfer_solid_edges";
    validate_tolerance(tol, operation)?;
    for (_, distances) in edges {
        positive(distances[0], "d0", operation)?;
        positive(distances[1], "d1", operation)?;
    }
    if preferences.is_empty() && edges.is_empty() {
        return try_chamfer_solid_edges(solid, &[], 1.0, tol);
    }
    let mut vertices = HashSet::default();
    for preference in preferences {
        if !vertices.insert(preference.vertex)
            || !edges.iter().any(|(id, _)| *id == preference.edge)
            || !solid.edge_iter().any(|edge| {
                edge.id() == preference.edge
                    && [edge.front().id(), edge.back().id()].contains(&preference.vertex)
            })
        {
            return Err(Diagnostic::new(
                Code::InvalidParameter,
                operation,
                "validate_corner_preference",
            ));
        }
    }
    let ids: Vec<_> = edges.iter().map(|&(id, _)| id).collect();
    if preferences.is_empty() && edges.iter().all(|(_, d)| *d == [edges[0].1[0]; 2]) {
        return try_chamfer_solid_edges(solid, &ids, edges[0].1[0], tol);
    }
    Solid::try_new(solid.boundaries().clone()).map_err(|e| {
        Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
            .with_coded_source(e)
    })?;
    let index = validate_selection(solid, &ids, operation)?;
    let distances = edges.iter().copied().collect();
    let selected = ids.into_iter().collect();
    if !preferences.is_empty() {
        let shell = super::chamfer_miter::chamfer(
            &solid.boundaries()[index],
            solid.boundaries().len() == 1,
            &selected,
            &distances,
            preferences,
            tol,
        )
        .map_err(|e| e.shell(index))?;
        return finish(solid, index, shell, tol).map_err(|e| e.operation(operation));
    }
    let shell = super::chamfer_miter::chamfer(
        &solid.boundaries()[index],
        solid.boundaries().len() == 1,
        &selected,
        &distances,
        &[],
        tol,
    )
    .or_else(|error| {
        if error.stage == "certify_planar_corner" {
            return Err(error);
        }
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
/// For consumed corners, open chains must terminate on straight edges and retain the support faces.
/// At an open end, a straight terminal edge may extend toward the new contact, allowing
/// chamfers at concave radial steps. Interior and closed-chain edges remain bounded.
/// Open cylinder/plane rim contacts may continue to curved terminal boundaries, including
/// concave walls and equivalent NURBS guides, for all distance conventions. Consuming a short
/// terminal neighbor trims its surviving adjacent faces together; newly crossed exits require
/// a planar or certified ruled blend and straight single-span boundaries. Cylindrical terminal supports
/// retain their analytic geometry while their parameter domains grow to include the new boundary.
/// A single quadratic extrusion rim with a perpendicular planar cap may continue past a
/// prior planar bevel. This requires a convex source, polynomial boundaries, monotone planar
/// terminations, and certified clearance and injectivity; distance-angle continuation is excluded.
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
            (continued, consumed_chamfer_exit, None),
            mode,
        )
    };
    let shell = build(false)
        .or_else(|error| {
            if solid.boundaries().len() != 1
                || error.stage != "trim_faces"
                || matches!(
                    mode,
                    super::chamfer::DistanceMode::NormalSectionAngle { .. }
                )
            {
                return Err(error);
            }
            let Some(continuation) =
                quadratic_chamfer_continuation(&solid.boundaries()[index], wire, [d0, d1])
            else {
                return Err(error);
            };
            super::chamfer::try_chamfer_along_wire_with_trim(
                &solid.boundaries()[index],
                wire,
                (d0, d1),
                tol,
                trim,
                (false, consumed_chamfer_exit, Some(&continuation)),
                mode,
            )
        })
        .or_else(|error| {
            if error.code == Code::BlendConstructionFailed
                && error.stage == "trim_faces"
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
    let shell = shell
        .into_iter()
        .map(|face| {
            if solid.boundaries()[index]
                .iter()
                .any(|old| old.id() == face.id())
            {
                return face;
            }
            let surface = face.oriented_surface();
            if !matches!(
                surface.elementary(),
                Some((truck_modeling::Elementary::Cylinder { .. }, _))
            ) || face.is_geometric_consistent()
            {
                return face;
            }
            let boundaries = face.boundaries();
            let surface = super::planar_edge::extend_surface(surface, &boundaries, tol);
            Face::new(boundaries, surface)
        })
        .collect();
    finish(solid, index, shell, tol).map_err(|e| e.operation(operation))
}

// Straight single-span exits are certified against planar or cubic ruled blends.
// Outer None rejects unsupported/ambiguous geometry; Some(None) is a proven non-crossing.
pub(super) fn consumed_chamfer_exit(
    curve: &truck_modeling::Curve,
    blend: &Surface,
) -> Option<Option<truck_modeling::Point3>> {
    use truck_modeling::*;
    let line = NurbsCurve::try_from(curve).ok()?;
    if line.degree() != 1
        || line.control_points().len() != 2
        || !line.is_clamped()
        || line
            .control_points()
            .iter()
            .any(|p| !p.w.is_finite() || p.w <= 0.)
    {
        return None;
    }
    let (a, b) = curve.range_tuple();
    let (p, q) = (curve.subs(a), curve.subs(b));
    let Some((Elementary::Plane(plane), _)) = blend.elementary() else {
        return consumed_ruled_exit(curve, blend, p, q);
    };
    let normal = plane.normal();
    let origin = plane.subs(0., 0.);
    let (d0, d1) = ((p - origin).dot(normal), (q - origin).dot(normal));
    if !d0.is_finite() || !d1.is_finite() || (q - p).magnitude2() <= TOLERANCE2 {
        return None;
    }
    if d0.abs() <= TOLERANCE && d1.abs() <= TOLERANCE {
        return None;
    }
    if d0.abs() <= TOLERANCE {
        return Some(Some(p));
    }
    if d1.abs() <= TOLERANCE {
        return Some(Some(q));
    }
    if d0.signum() == d1.signum() {
        return Some(None);
    }
    Some(Some(p + (q - p) * (d0 / (d0 - d1))))
}

fn consumed_ruled_exit(
    curve: &truck_modeling::Curve,
    blend: &Surface,
    p: truck_modeling::Point3,
    q: truck_modeling::Point3,
) -> Option<Option<truck_modeling::Point3>> {
    use truck_modeling::*;
    let Surface::BSplineSurface(surface) = blend else {
        return None;
    };
    let degree = surface.udegree();
    if !(1..=3).contains(&degree)
        || surface.vdegree() != 1
        || !surface.uknot_vec().is_clamped(degree)
        || !surface.vknot_vec().is_clamped(1)
    {
        return None;
    }
    let rows = surface.control_points();
    if rows.iter().any(|row| row.len() != 2) {
        return None;
    }
    let sections: Vec<_> = (0..2)
        .map(|side| {
            BSplineCurve::new(
                surface.uknot_vec().clone(),
                rows.iter().map(|row| row[side]).collect(),
            )
            .bezier_decomposition()
        })
        .collect();
    let direction = (q - p).normalize();
    if !direction.magnitude2().is_finite() {
        return None;
    }
    let choose = |n: usize, k: usize| (0..k).fold(1., |v, i| v * (n - i) as f64 / (i + 1) as f64);
    let mut root_span = None;
    for (a, b) in sections[0].iter().zip(&sections[1]) {
        // A ruling can meet the line only when (A(u)-p) × (B(u)-p) · direction = 0.
        // Its Bernstein coefficients follow the product formula. Sign variations bound
        // roots on each Bezier span, certifying zero or one before Newton is attempted.
        let coefficients: Vec<f64> = (0..=2 * degree)
            .map(|k| {
                (k.saturating_sub(degree)..=k.min(degree))
                    .map(|i| {
                        let j = k - i;
                        (a.control_point(i) - p)
                            .cross(b.control_point(j) - p)
                            .dot(direction)
                            * choose(degree, i)
                            * choose(degree, j)
                            / choose(2 * degree, k)
                    })
                    .sum()
            })
            .collect();
        if coefficients.iter().any(|c| !c.is_finite())
            || coefficients[1..coefficients.len() - 1]
                .iter()
                .any(|c| c.abs() <= TOLERANCE2)
        {
            return None;
        }
        // Small nonzero endpoints still carry a sign: dropping it can merge distinct
        // roots on opposite sides of a knot into one apparent shared-knot root.
        let first_zero = coefficients[0] == 0.;
        let last_zero = coefficients[coefficients.len() - 1] == 0.;
        let signs =
            &coefficients[usize::from(first_zero)..coefficients.len() - usize::from(last_zero)];
        let variations = signs
            .windows(2)
            .filter(|c| c[0].signum() != c[1].signum())
            .count();
        if variations > 1 {
            return None;
        }
        let (start, end) = a.range_tuple();
        for span in [
            first_zero.then_some((start, start)),
            (variations == 1).then_some((start, end)),
            last_zero.then_some((end, end)),
        ]
        .into_iter()
        .flatten()
        {
            if let Some((previous_start, previous_end)) = root_span {
                // Adjacent Bezier spans share a knot root; count it only once.
                if previous_start != previous_end || span.0 != span.1 || span.0 != previous_start {
                    return None;
                }
            } else {
                root_span = Some(span);
            }
        }
    }
    let Some((root0, root1)) = root_span else {
        return Some(None);
    };
    let (a, b) = curve.range_tuple();
    let t = (a + b) / 2.;
    let mut uv = blend.search_nearest_parameter(curve.subs(t), None, 100)?;
    uv.0 = (root0 + root1) / 2.;
    let ((u, v), t) = algo::surface::search_intersection_parameter(blend, uv, curve, t, 100)?;
    let ((u0, u1), (v0, v1)) = surface.range_tuple();
    if !t.is_finite() || !u.is_finite() || !v.is_finite() {
        return None;
    }
    if u < root0 - TOLERANCE || u > root1 + TOLERANCE {
        return None;
    }
    if t < a - TOLERANCE || t > b + TOLERANCE || v < v0 - TOLERANCE || v > v1 + TOLERANCE {
        return Some(None);
    }
    let point = curve.subs(t.clamp(a, b));
    point
        .near(&blend.subs(u.clamp(u0, u1), v.clamp(v0, v1)))
        .then_some(Some(point))
}

fn quadratic_chamfer_continuation(
    shell: &Shell,
    wire: &Wire,
    distances: [f64; 2],
) -> Option<super::chamfer::ContactContinuation<truck_modeling::Curve, Surface>> {
    use truck_modeling::*;
    if wire.len() != 1
        || shell
            .iter()
            .any(|face| face.boundaries().len() != 1 || !face.is_geometric_consistent())
    {
        return None;
    }
    let distance = distances[0].max(distances[1]);
    let edge = &wire[0];
    let Curve::BSplineCurve(guide) = edge.oriented_curve() else {
        return None;
    };
    if guide.degree() != 2 || guide.control_points().len() != 3 || !guide.is_clamped() {
        return None;
    }
    let mut supports = [None, None];
    let mut cap = None;
    let mut wall = None;
    let mut wall_face = None;
    let mut cap_face = None;
    for (index, face) in shell.iter().enumerate() {
        let surface = face.oriented_surface();
        if let Some(found) = face.edge_iter().find(|e| e.id() == edge.id()) {
            let side = usize::from(found.front() != edge.front());
            if matches!(surface.elementary(), Some((Elementary::Plane(_), _))) {
                cap = Some(side);
                cap_face = Some(index);
            } else {
                wall = Some(side);
                wall_face = Some(index);
            }
            supports[side] = Some(surface);
        } else if !matches!(surface.elementary(), Some((Elementary::Plane(_), _))) {
            return None;
        }
    }
    let (cap, wall) = (cap?, wall?);
    let (cap_face, wall_face) = (cap_face?, wall_face?);
    let [Some(s0), Some(s1)] = supports else {
        return None;
    };
    let mut supports = [s0, s1];
    let Some((Elementary::Plane(plane), _)) = supports[cap].elementary() else {
        return None;
    };
    let normal = plane.normal();
    let Surface::BSplineSurface(wall_surface) = &supports[wall] else {
        return None;
    };
    let rows = wall_surface.control_points();
    if wall_surface.udegree() != 2
        || wall_surface.vdegree() != 1
        || !wall_surface.uknot_vec().is_clamped(2)
        || !wall_surface.vknot_vec().is_clamped(1)
        || rows.len() != 3
        || rows.iter().any(|r| r.len() != 2)
    {
        return None;
    }
    let extrusion = rows[0][1] - rows[0][0];
    let extrusion_roundoff = 512. * f64::EPSILON * extrusion.magnitude();
    if extrusion.cross(normal).magnitude() > extrusion_roundoff
        || rows
            .iter()
            .any(|r| (r[1] - r[0] - extrusion).magnitude() > extrusion_roundoff)
    {
        return None;
    }
    let (a, b) = guide.range_tuple();
    let p = guide.subs(a);
    let derivative = guide.der(a);
    let second = guide.der2(a);
    let linear = derivative - second * (derivative.dot(second) / second.magnitude2());
    let speed = linear.magnitude();
    if !continuation_curvature_admissible(distance, second.magnitude(), speed) {
        return None;
    }
    let axis = linear / speed;
    let inward = second.normalize();
    let curvature = second.magnitude() / speed.powi(2);
    let vertex_parameter = a - derivative.dot(second) / second.magnitude2();
    let origin =
        p + derivative * (vertex_parameter - a) + second * ((vertex_parameter - a).powi(2) / 2.);
    let scale = shell
        .vertex_iter()
        .map(|v| (v.point() - origin).magnitude())
        .fold(distance, f64::max);
    let roundoff = 512. * f64::EPSILON * scale.max(1.);
    if inward.dot(normal).abs() > 512. * f64::EPSILON
        || axis.dot(normal).abs() > 512. * f64::EPSILON
        || guide
            .control_points()
            .iter()
            .any(|q| (*q - p).dot(normal).abs() > roundoff)
    {
        return None;
    }
    let (wa, wb) = wall_surface.range_tuple().0;
    let middle = (wa + wb) / 2.;
    let point = wall_surface.subs(middle, 0.5);
    let x = (point - origin).dot(axis);
    if wall_surface
        .normal(middle, 0.5)
        .dot(inward - axis * (curvature * x))
        >= 0.
    {
        return None;
    }
    let mut lo = a;
    let mut hi = b;
    let mut boundaries = Vec::new();
    for edge in shell.edge_iter() {
        let curve = NurbsCurve::try_from(&edge.curve()).ok()?;
        if curve.degree() > 2
            || curve.control_points().len() != curve.degree() + 1
            || !curve.is_clamped()
        {
            return None;
        }
        let weight = curve.control_points()[0].w;
        if !weight.is_finite()
            || weight <= 0.
            || curve.control_points().iter().any(|p| p.w != weight)
        {
            return None;
        }
        let mut controls = Vec::new();
        for point in curve.control_points() {
            let point = Point3::new(point.x / point.w, point.y / point.w, point.z / point.w);
            let t = a + (point - p).dot(linear) / speed.powi(2);
            lo = lo.min(t - distance / speed);
            hi = hi.max(t + distance / speed);
            controls.push(point);
        }
        boundaries.push(controls);
    }
    // Bernstein products certify every boundary lies in the convex parabolic epigraph.
    // Together with the planar halfspaces and single closed face loops this identifies
    // the source as their convex intersection; holes and additional curved faces reject.
    let choose = |n: usize, k: usize| (0..k).fold(1., |v, i| v * (n - i) as f64 / (i + 1) as f64);
    let mut checked_curves = boundaries.clone();
    checked_curves.extend((0..2).map(|side| rows.iter().map(|row| row[side]).collect()));
    for (index, controls) in checked_curves.iter().enumerate() {
        let degree = controls.len() - 1;
        for k in 0..=2 * degree {
            let value = (k.saturating_sub(degree)..=k.min(degree))
                .map(|i| {
                    let j = k - i;
                    let pi = controls[i] - origin;
                    let pj = controls[j] - origin;
                    (0.5 * (pi.dot(inward) + pj.dot(inward))
                        - 0.5 * curvature * pi.dot(axis) * pj.dot(axis))
                        * choose(degree, i)
                        * choose(degree, j)
                        / choose(2 * degree, k)
                })
                .sum::<f64>();
            if !value.is_finite()
                || value < -roundoff
                || (index >= boundaries.len() && value.abs() > roundoff)
            {
                return None;
            }
        }
    }
    let at = |t: f64| p + derivative * (t - a) + second * ((t - a).powi(2) / 2.);
    let controls_for = |l: f64, r: f64| {
        [
            at(l),
            at(l) + (derivative + second * (l - a)) * ((r - l) / 2.),
            at(r),
        ]
    };
    let excluded = |l: f64, r: f64| {
        shell.iter().any(|face| {
            let Some((Elementary::Plane(plane), _)) = face.oriented_surface().elementary() else {
                return false;
            };
            let n = plane.normal();
            let o = plane.subs(0., 0.);
            let minimum = controls_for(l, r)
                .iter()
                .map(|p| (*p - o).dot(n))
                .fold(f64::INFINITY, f64::min);
            let offset = (-distances[wall] * n.dot(normal))
                .min(-distances[cap] * (n - normal * n.dot(normal)).magnitude());
            minimum + offset > roundoff
        })
    };
    // Discard only slabs whose entire ruling rectangle violates an original plane.
    for _ in 0..32 {
        let middle = (lo + a) / 2.;
        if !excluded(lo, middle) {
            break;
        }
        lo = middle;
    }
    for _ in 0..32 {
        let middle = (hi + b) / 2.;
        if !excluded(middle, hi) {
            break;
        }
        hi = middle;
    }
    let first = at(lo);
    let last = at(hi);
    let extended = BSplineCurve::new(
        KnotVec::from(vec![lo, lo, lo, hi, hi, hi]),
        vec![
            first,
            first + (derivative + second * (lo - a)) * ((hi - lo) / 2.),
            last,
        ],
    );
    // P(t,l) = G(t) - (1-l) dw*n + l*dc*N(t). The normal chart
    // determinant is q*(1-l*dc*k/q³)>0. Its x derivative is positive,
    // and at fixed x its inward derivative is positive: global injectivity.
    // |P_x-G_x|<=dc gives the finite enclosure above, including every
    // possible retained component, without extending a fitted contact curve.
    let dw = distances[wall];
    let dc = distances[cap];
    let inside_station = (a + b) / 2.;
    let station = at(inside_station);
    let station_x = (station - origin).dot(axis);
    let station_inward = (inward - axis * (curvature * station_x)).normalize();
    let section = [station - normal * dw, station + station_inward * dc];
    let mut terminal_signs = Vec::new();
    let mut terminal_directions = Vec::new();
    let mut terminal_planes = Vec::new();
    let mut remote_planes = Vec::new();
    for (index, face) in shell.iter().enumerate() {
        if index == wall_face {
            continue;
        }
        let Some((Elementary::Plane(plane), _)) = face.oriented_surface().elementary() else {
            return None;
        };
        let n = plane.normal();
        let o = plane.subs(0., 0.);
        if boundaries
            .iter()
            .flatten()
            .any(|p| (*p - o).dot(n) > roundoff)
        {
            return None;
        }
        if index == cap_face {
            continue;
        }
        let terminal = n.cross(normal).magnitude() > roundoff
            && face
                .edge_iter()
                .any(|e| shell[wall_face].edge_iter().any(|w| w.id() == e.id()));
        if terminal {
            let slopes = [lo, hi].map(|t| n.dot(derivative + second * (t - a)));
            if slopes[0] * slopes[1] <= 0.
                || slopes.iter().any(|v| v.abs() <= roundoff)
                || section.iter().any(|p| (*p - o).dot(n) >= -roundoff)
            {
                return None;
            }
            terminal_signs.push(slopes[0].is_sign_positive());
            terminal_directions.push(n * slopes[0].signum());
            terminal_planes.push((o, n));
        } else {
            // A hull bound excludes every remote face over the entire construction box.
            let maximum = extended
                .control_points()
                .iter()
                .map(|p| (*p - o).dot(n))
                .fold(f64::NEG_INFINITY, f64::max);
            let offset = (-dw * n.dot(normal)).max(dc * (n - normal * n.dot(normal)).magnitude());
            if maximum + offset >= -roundoff {
                return None;
            }
            remote_planes.push((o, n));
        }
    }
    // Each terminal inequality is strictly monotone at every section fraction.
    // A common interior ruling separates the two ends for all fractions, so their
    // intersection is one strip; the existing end walk must classify all its seams.
    if terminal_signs.len() != 3
        || terminal_signs.iter().all(|v| *v)
        || terminal_signs.iter().all(|v| !*v)
    {
        return None;
    }
    let mut bottom = extended.clone();
    bottom.transform_by(Matrix4::from_translation(-normal * extrusion.magnitude()));
    let mut surface = BSplineSurface::homotopy(bottom, extended.clone());
    if surface
        .normal((lo + hi) / 2., 0.5)
        .dot(supports[wall].normal(0.5, 0.5))
        < 0.
    {
        surface.invert();
    }
    supports[wall] = surface.into();
    Some(super::chamfer::ContactContinuation {
        guide: extended.into(),
        supports,
        chart: super::chamfer::QuadraticContactChart {
            origin,
            axes: [axis, inward],
            curvature,
            cap_side: cap,
            remote_planes,
            terminal_directions,
            terminal_planes,
            interior_parameter: inside_station,
            cap_normal: normal,
        },
    })
}

fn can_continue_chamfer_contacts(shell: &Shell, wire: &Wire) -> bool {
    use truck_modeling::*;
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
        Some((Elementary::Cylinder { axis, .. }, _)) => Some(*axis),
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
        Some((Elementary::Cylinder { axis: other, .. }, _)) => {
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

    match try_chamfer_solid_edges_with_distances(solid, &[(edge, [d0, d1])], &[], tol) {
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

fn mixed_curved_corner(
    solid: &Solid,
    index: usize,
    selected: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Option<BlendResult> {
    use truck_modeling::*;
    if selected.len() != 3 {
        return None;
    }
    let fit_tol = tol / 100.;
    let shell = &solid.boundaries()[index];
    let mut remaining: HashSet<_> = selected.iter().copied().collect();
    let edges: Vec<_> = shell
        .edge_iter()
        .filter(|e| remaining.remove(&e.id()))
        .collect();
    if edges.len() != 3 {
        return None;
    }
    let common = [edges[0].front(), edges[0].back()]
        .into_iter()
        .find(|v| edges.iter().all(|e| e.front() == *v || e.back() == *v))?;
    let mut incident = HashSet::default();
    for edge in shell
        .edge_iter()
        .filter(|e| e.front() == common || e.back() == common)
    {
        incident.insert(edge.id());
    }
    if incident.len() != 3 {
        return None;
    }
    let mut concave = None;
    for (i, edge) in edges.iter().enumerate() {
        let edge = edge.absolute_clone();
        let curve = edge.curve();
        let (a, b) = curve.range_tuple();
        let mut sign = 0.;
        for j in 0..=16 {
            let t = a + (b - a) * (j as f64 + 0.5) / 17.;
            let p = curve.subs(t);
            let mut normals = [None, None];
            for face in shell.iter() {
                if let Some(e) = face.edge_iter().find(|e| e.id() == edge.id()) {
                    let surface = face.oriented_surface();
                    let (u, v) = surface.search_parameter(p, None, 100)?;
                    let side = usize::from(!e.orientation());
                    if normals[side]
                        .replace(surface.normal(u, v).normalize())
                        .is_some()
                    {
                        return None;
                    }
                }
            }
            let d = normals[0]?.cross(normals[1]?).dot(curve.der(t).normalize());
            if !d.is_finite() || d.abs() < 1e-7 || (sign != 0. && sign * d < 0.) {
                return None;
            }
            sign = d.signum();
        }
        if sign < 0. && concave.replace(i).is_some() {
            return None;
        }
    }
    let concave = concave?;
    let convex: Vec<_> = edges
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != concave)
        .map(|(_, e)| e)
        .collect();
    let wall = shell.iter().position(|f| {
        matches!(f.surface(), Surface::Plane(_))
            && convex
                .iter()
                .all(|e| f.edge_iter().any(|q| q.id() == e.id()))
    })?;
    if shell[wall]
        .edge_iter()
        .any(|e| e.id() == edges[concave].id())
    {
        return None;
    }
    let ends: Vec<_> = convex
        .iter()
        .map(|e| {
            if e.front() == common {
                e.back().clone()
            } else {
                e.front().clone()
            }
        })
        .collect();
    let first = try_fillet_solid_along_wire(
        solid,
        &Wire::from(vec![edges[concave].clone()]),
        radius,
        fit_tol,
    )
    .ok()?;
    if first.generated_faces.len() != 1 {
        return None;
    }
    let middle: HashSet<_> = first.generated_faces[0]
        .edge_iter()
        .map(|e| e.id())
        .collect();
    let mut path = None;
    for boundary in first.solid.boundaries()[index][wall].boundaries() {
        for wire in [boundary.clone(), boundary.inverse()] {
            let Some(start) = wire.iter().position(|e| e.front() == &ends[0]) else {
                continue;
            };
            let mut candidate = Wire::new();
            for j in 0..wire.len() {
                let edge = wire[(start + j) % wire.len()].clone();
                let done = edge.back() == &ends[1];
                candidate.push_back(edge);
                if done {
                    break;
                }
            }
            if candidate.len() == 3
                && candidate.back_vertex() == Some(&ends[1])
                && middle.contains(&candidate[1].id())
                && super::along_wire::is_tangent_continuous(&candidate, false)
                && path.replace(candidate).is_some()
            {
                return None;
            }
        }
    }
    let second = try_fillet_solid_along_wire(&first.solid, &path?, radius, fit_tol).ok()?;
    finish(solid, index, second.solid.boundaries()[index].clone(), tol).ok()
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

#[cfg(test)]
mod consumed_terminal_tests {
    use super::*;
    use truck_modeling::*;

    #[test]
    fn consumed_terminal_rejects_two_crossings_on_one_curved_exit() {
        let plane = Surface::Plane(Plane::new(
            Point3::origin(),
            Point3::new(0., 0., 1.),
            Point3::new(1., 0., 0.),
        ));
        // y(t) = (t - 0.2)(t - 0.7), so this single edge crosses y = 0 twice.
        let curve = Curve::BSplineCurve(BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(0., 0.14, 0.),
                Point3::new(0.5, -0.31, 0.),
                Point3::new(1., 0.24, 0.),
            ],
        ));
        for t in [0.2, 0.7] {
            assert!(curve.subs(t).y.abs() < 1e-12);
        }
        let uv = plane
            .search_nearest_parameter(curve.subs(0.5), None, 100)
            .unwrap();
        let (_, root) =
            algo::surface::search_intersection_parameter(&plane, uv, &curve, 0.5, 100).unwrap();
        assert!((root - 0.7).abs() < TOLERANCE);
        let before = serde_json::to_string(&curve).unwrap();
        assert!(consumed_chamfer_exit(&curve, &plane).is_none());
        assert_eq!(before, serde_json::to_string(&curve).unwrap());
    }

    #[test]
    fn consumed_ruled_exits_require_a_unique_bounded_root() {
        let section = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(-1., 1., 1.),
                Point3::new(0., -1., 1.),
                Point3::new(1., 1., 1.),
            ],
        );
        let outer = section.transformed(Matrix4::from_scale(2.));
        for (a, b) in [(section.clone(), outer.clone()), (outer, section)] {
            let blend = Surface::BSplineSurface(BSplineSurface::homotopy(a, b));
            let before = serde_json::to_string(&blend).unwrap();
            let unique = Curve::Line(Line(Point3::new(0.5, 0., 1.5), Point3::new(0.5, 2., 1.5)));
            for curve in [unique.clone(), unique.inverse()] {
                let point = consumed_chamfer_exit(&curve, &blend).unwrap().unwrap();
                assert_near!(point, Point3::new(0.5, 1. / 6., 1.5));
            }
            for curve in [
                Curve::Line(Line(Point3::new(3., 0., 1.5), Point3::new(3., 2., 1.5))),
                Curve::Line(Line(Point3::new(0.5, 10., 1.5), Point3::new(0.5, 11., 1.5))),
            ] {
                assert_eq!(consumed_chamfer_exit(&curve, &blend), Some(None));
            }
            // At z=1.5 the section is y=x²/1.5; y=0.5 has two roots ±sqrt(0.75).
            let twice = Curve::Line(Line(Point3::new(-2., 0.5, 1.5), Point3::new(2., 0.5, 1.5)));
            assert!(consumed_chamfer_exit(&twice, &blend).is_none());
            assert_eq!(before, serde_json::to_string(&blend).unwrap());
        }
    }

    #[test]
    fn consumed_ruled_exits_count_shared_knot_and_endpoint_roots_once() {
        let mut section = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(-1., 1., 1.),
                Point3::new(0., -1., 1.),
                Point3::new(1., 1., 1.),
            ],
        );
        section.add_knot(0.5);
        let blend = Surface::BSplineSurface(BSplineSurface::homotopy(
            section.clone(),
            section.transformed(Matrix4::from_scale(2.)),
        ));
        for x in [-1.5, 0., 1.5] {
            let curve = Curve::Line(Line(Point3::new(x, -1., 1.5), Point3::new(x, 2., 1.5)));
            let point = consumed_chamfer_exit(&curve, &blend).unwrap().unwrap();
            assert_near!(point, Point3::new(x, x * x / 1.5, 1.5));
        }
        let twice = Curve::Line(Line(Point3::new(-2., 0.5, 1.5), Point3::new(2., 0.5, 1.5)));
        assert!(consumed_chamfer_exit(&twice, &blend).is_none());
    }

    #[test]
    fn consumed_ruled_exits_reject_distinct_roots_near_a_shared_knot() {
        let epsilon = 5e-13;
        let slope = 1e-8;
        // S(u,v) = (100u, (u-0.5)^2 + slope*|u-0.5| - epsilon, v).
        let section = BSplineCurve::new(
            KnotVec::from(vec![0., 0., 0., 0.5, 0.5, 1., 1., 1.]),
            vec![
                Point3::new(0., 0.25 + 0.5 * slope - epsilon, 0.),
                Point3::new(25., 0.25 * slope - epsilon, 0.),
                Point3::new(50., -epsilon, 0.),
                Point3::new(75., 0.25 * slope - epsilon, 0.),
                Point3::new(100., 0.25 + 0.5 * slope - epsilon, 0.),
            ],
        );
        let blend = Surface::BSplineSurface(BSplineSurface::homotopy(
            section.clone(),
            section.transformed(Matrix4::from_translation(Vector3::unit_z())),
        ));
        let offset = ((slope * slope + 4. * epsilon).sqrt() - slope) / 2.;
        assert!(offset < TOLERANCE);
        assert!(200. * offset > 100. * TOLERANCE);
        for u in [0.5 - offset, 0.5 + offset] {
            assert!(blend.subs(u, 0.5).y.abs() < 1e-16);
        }
        let curve = Curve::Line(Line(Point3::new(0., 0., 0.5), Point3::new(100., 0., 0.5)));
        let before = serde_json::to_string(&blend).unwrap();
        for curve in [curve.clone(), curve.inverse()] {
            let result = consumed_chamfer_exit(&curve, &blend);
            assert!(result.is_none(), "two distinct roots accepted: {result:?}");
        }
        assert_eq!(before, serde_json::to_string(&blend).unwrap());
    }

    #[test]
    fn consumed_ruled_exits_use_the_fitted_patch_not_an_ideal_cone() {
        let a = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(-1., 1., 0.),
                Point3::new(0., -1., 0.),
                Point3::new(1., 1., 0.),
            ],
        );
        let b = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(-1., 1., 1.),
                Point3::new(1., 0., 1.),
                Point3::new(3., 3., 1.),
            ],
        );
        let blend = Surface::BSplineSurface(BSplineSurface::homotopy(a, b));
        let curve = Curve::Line(Line(Point3::new(0.8, -1., 0.5), Point3::new(0.8, 2., 0.5)));
        let point = consumed_chamfer_exit(&curve, &blend).unwrap().unwrap();
        // At z=0.5 the patch is (1.5u+0.5, u²+0.5u+0.5, 0.5).
        assert_near!(point, Point3::new(0.8, 0.64, 0.5));
    }

    #[test]
    fn consumed_terminal_linear_exits_are_bounded_and_unique() {
        let plane = Surface::Plane(Plane::new(
            Point3::origin(),
            Point3::new(0., 0., 1.),
            Point3::new(1., 0., 0.),
        ));
        for (a, b, expected) in [(-1., 1., Some(Point3::new(0.5, 0., 0.))), (1., 2., None)] {
            let line = Curve::Line(Line(Point3::new(0., a, 0.), Point3::new(1., b, 0.)));
            for curve in [
                line.clone(),
                line.inverse(),
                Curve::NurbsCurve(NurbsCurve::try_from(&line).unwrap()),
            ] {
                assert_eq!(consumed_chamfer_exit(&curve, &plane).unwrap(), expected);
            }
        }
        let coplanar = Curve::Line(Line(Point3::origin(), Point3::new(1., 0., 0.)));
        assert!(consumed_chamfer_exit(&coplanar, &plane).is_none());
    }
}

fn continuation_curvature_admissible(distance: f64, second: f64, speed: f64) -> bool {
    if !speed.is_finite() || speed <= 0. || distance * second / speed.powi(2) >= 0.5 {
        return false;
    }
    true
}

#[cfg(test)]
mod continuation_curvature_tests {
    use super::continuation_curvature_admissible;

    #[test]
    fn continuation_curvature_certificate_boundary() {
        for speed in [0.01_f64, 1., 100.] {
            let second = 0.2 * speed.powi(2);
            assert!(continuation_curvature_admissible(2.49, second, speed));
            assert!(!continuation_curvature_admissible(2.51, second, speed));
        }
        assert!(!continuation_curvature_admissible(2.5, 0.2, 1.));
        assert!(!continuation_curvature_admissible(1., 0.2, 0.));
        assert!(!continuation_curvature_admissible(1., 0.2, f64::INFINITY));
    }
}
