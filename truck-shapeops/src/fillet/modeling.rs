//! Blend operations on modeling solids.

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

fn finish(solid: &Solid, index: usize, shell: Shell) -> Option<BlendResult> {
    let original = &solid.boundaries()[index];
    let modified_faces = original
        .iter()
        .zip(&shell)
        .filter(|(a, b)| a.id() != b.id())
        .map(|(a, b)| (a.id(), b.clone()))
        .collect();
    let generated_faces = shell.iter().skip(original.len()).cloned().collect();
    let mut boundaries = solid.boundaries().clone();
    boundaries[index] = shell;
    Some(BlendResult {
        solid: Solid::try_new(boundaries).ok()?,
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
    if !tol.is_finite() || tol <= 0.0 {
        return None;
    }
    let index = shell_index(solid, wire.front()?.id())?;
    for i in 0..=wire.len() * 32 {
        let t = i as f64 / 32.0;
        if radius.subs(t) <= 0.0 || (0..=2).any(|n| !radius.der_n(n, t).is_finite()) {
            return None;
        }
    }
    let shell = super::fillet_along_wire(&solid.boundaries()[index], wire, radius, tol)?;
    finish(solid, index, shell)
}

/// Equal-radius blends on selected edges of a modeling solid, including spherical corners.
///
/// All selected edges must belong to one closed convex planar boundary. See
/// [`super::fillet_edges`] for the supported junctions and geometric restrictions. The operation
/// must not intersect another boundary. Returns `None` on unsupported input without modifying
/// it. An empty selection returns the original solid and empty history.
pub fn fillet_solid_edges(
    solid: &Solid,
    edges: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Option<BlendResult> {
    if !radius.is_finite() || radius <= 0.0 || !tol.is_finite() || tol <= 0.0 {
        return None;
    }
    let Some(&edge) = edges.first() else {
        return Some(BlendResult {
            solid: solid.clone(),
            modified_faces: Vec::new(),
            generated_faces: Vec::new(),
        });
    };
    let index = shell_index(solid, edge)?;
    let shell = super::fillet_edges(&solid.boundaries()[index], edges, radius, tol)?;
    finish(solid, index, shell)
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
    let index = shell_index(solid, wire.front()?.id())?;
    let shell = super::chamfer_along_wire(&solid.boundaries()[index], wire, d0, d1, tol)?;
    finish(solid, index, shell)
}

/// Chamfers a single edge of a modeling solid, trimming both end faces.
///
/// The edge must have two adjacent faces and a distinct third face at each end. `d0` and `d1`
/// are positive distances on the adjacent faces, ordered by their occurrence in the shell.
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
    if [d0, d1, tol].iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    let index = shell_index(solid, edge)?;
    let shell = &solid.boundaries()[index];
    let adjacent: Vec<_> = shell
        .iter()
        .enumerate()
        .filter(|(_, f)| f.edge_iter().any(|e| e.id() == edge))
        .map(|(i, _)| i)
        .collect();
    let [a, b] = adjacent.as_slice() else {
        return None;
    };
    let oriented = shell[*a].edge_iter().find(|e| e.id() == edge)?;
    let mut ends = Vec::new();
    for vertex in [oriented.front(), oriented.back()] {
        let faces: Vec<_> = shell
            .iter()
            .enumerate()
            .filter(|(i, f)| *i != *a && *i != *b && f.vertex_iter().any(|v| v.id() == vertex.id()))
            .map(|(i, _)| i)
            .collect();
        if faces.len() != 1 {
            return None;
        }
        ends.push(faces[0]);
    }
    if ends[0] == ends[1] {
        return None;
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
    )?;
    let mut result = shell.clone();
    result[*a] = blend.simple_fillet.face0;
    result[*b] = blend.simple_fillet.face1;
    result[ends[0]] = blend.side0?;
    result[ends[1]] = blend.side1?;
    result.push(blend.simple_fillet.fillet);
    finish(solid, index, result)
}
