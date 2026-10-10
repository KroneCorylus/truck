use super::{modeling::BlendResult, planar::contains_patch};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

// A concave round ending on transverse planes adds an exact swept circular segment.
// Union trims it across split caps and stepped supports, including a fully consumed gap face.
pub(super) fn fillet(
    solid: &Solid,
    selected: &[EdgeID],
    radius: f64,
    tol: f64,
) -> Result<BlendResult, Diagnostic> {
    let error = |code| Diagnostic::new(code, "fillet_solid_edges", "add_concave_rounds");
    let chosen: HashSet<_> = selected.iter().copied().collect();
    let mut seen = HashSet::default();
    let mut fillers = Vec::new();
    let mut direction: Option<Vector3> = None;
    let mut patches: Vec<(FaceID, [Point3; 4])> = Vec::new();
    let mut origins: HashMap<_, _> = solid.face_iter().map(|f| (f.id(), Some(f.id()))).collect();
    for face in solid.face_iter() {
        for edge in face.edge_iter() {
            if !chosen.contains(&edge.id()) || !seen.insert(edge.id()) {
                continue;
            }
            let other = solid
                .face_iter()
                .find(|f| f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id()))
                .ok_or_else(|| error(Code::UnsupportedTopology))?;
            let (Surface::Plane(a), Surface::Plane(b)) =
                (face.oriented_surface(), other.oriented_surface())
            else {
                return Err(error(Code::NonPlanarFace));
            };
            let delta = edge.back().point() - edge.front().point();
            let axis = delta.normalize();
            if direction.is_some_and(|first| first.cross(axis).magnitude() > TOLERANCE) {
                return Err(error(Code::UnsupportedGeometry));
            }
            direction = Some(axis);
            let [n, m] = [a.normal().normalize(), b.normal().normalize()];
            if !matches!(edge.curve(), Curve::Line(_)) || n.cross(m).dot(axis) >= -TOLERANCE {
                return Err(error(Code::UnsupportedGeometry));
            }
            let mut caps = Vec::new();
            for vertex in [edge.front(), edge.back()] {
                let cap = solid.face_iter().find(|f| {
                    f.vertex_iter().any(|v| v == *vertex)
                        && matches!(f.oriented_surface(), Surface::Plane(p) if p.normal().normalize().cross(axis).magnitude() <= TOLERANCE)
                }).ok_or_else(|| error(Code::UnsupportedGeometry))?;
                caps.push(cap);
            }
            let start = edge.front().point();
            let center = start + radius * (n + m) / (1. + n.dot(m));
            let p = center - radius * n;
            let q = center - radius * m;
            for (support, contact, normal) in [(face, p, n), (other, q, m)] {
                let patch = [start, start + delta, contact + delta, contact];
                if !contains_patch(support, &patch, normal, tol) {
                    return Err(error(Code::OutsideNeighbour));
                }
                for (_, previous) in patches.iter().filter(|(id, _)| *id == support.id()) {
                    let overlap = |direction: Vector3| {
                        let interval = |points: &[Point3; 4]| {
                            points
                                .iter()
                                .map(|p| p.to_vec().dot(direction))
                                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), t| {
                                    (low.min(t), high.max(t))
                                })
                        };
                        let (a, b) = interval(&patch);
                        let (c, d) = interval(previous);
                        b.min(d) - a.max(c) > TOLERANCE
                    };
                    if overlap(axis) && overlap(normal.cross(axis)) {
                        return Err(error(Code::OutsideNeighbour));
                    }
                }
                patches.push((support.id(), patch));
            }
            let [v, p, q] = [start, p, q].map(builder::vertex);
            let middle =
                center + ((p.point() - center) + (q.point() - center)).normalize() * radius;
            let wire: Wire = vec![
                builder::line(&v, &p),
                builder::circle_arc(&p, &q, middle),
                builder::line(&q, &v),
            ]
            .into();
            let section = builder::try_attach_plane(&[wire])
                .map_err(|_| error(Code::BlendConstructionFailed))?;
            let filler = builder::tsweep(&section, delta);
            for f in filler.face_iter() {
                let source = match f.oriented_surface() {
                    Surface::Plane(plane) => {
                        let normal = plane.normal().normalize();
                        if normal.cross(axis).magnitude() <= TOLERANCE {
                            Some(
                                caps[usize::from(
                                    (plane.origin() - start).dot(axis).abs()
                                        > delta.magnitude() / 2.,
                                )]
                                .id(),
                            )
                        } else if normal.cross(n).magnitude() <= TOLERANCE {
                            Some(face.id())
                        } else {
                            Some(other.id())
                        }
                    }
                    _ => None,
                };
                origins.insert(f.id(), source);
            }
            fillers.push(filler);
        }
    }
    let mut result = solid.clone();
    for filler in fillers {
        let mut history = HashMap::default();
        result = crate::transversal::union_with_history(&result, &filler, tol, &mut history)?;
        let next: Vec<_> = history
            .into_iter()
            .map(|(id, source)| (id, origins[&source]))
            .collect();
        origins.extend(next);
    }
    if result.boundaries().len() != solid.boundaries().len() || !result.is_geometric_consistent() {
        return Err(error(Code::InvalidOutputTopology));
    }
    let mut modified_faces = Vec::new();
    let mut generated_faces = Vec::new();
    for face in result.face_iter() {
        match origins[&face.id()] {
            Some(source) if source != face.id() => modified_faces.push((source, face.clone())),
            Some(_) => {}
            None => generated_faces.push(face.clone()),
        }
    }
    let order: HashMap<_, _> = solid
        .face_iter()
        .enumerate()
        .map(|(i, f)| (f.id(), i))
        .collect();
    modified_faces.sort_by_key(|(source, _)| order[source]);
    Ok(BlendResult {
        solid: result,
        modified_faces,
        generated_faces,
    })
}
