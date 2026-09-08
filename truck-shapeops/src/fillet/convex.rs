use super::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};

pub(super) struct ConvexShell<C> {
    pub vertex_index: HashMap<VertexID<Point3>, usize>,
    pub vertices: Vec<Vertex<Point3>>,
    pub edge_index: HashMap<EdgeID<C>, usize>,
    pub original_edges: Vec<Edge<Point3, C>>,
    pub planes: Vec<(Vector3, f64)>,
    pub incident: Vec<Vec<usize>>,
    pub sides: Vec<[usize; 2]>,
}

pub(super) fn validate<C, S>(
    shell: &Shell<Point3, C, S>,
    selected: &HashSet<EdgeID<C>>,
    tol: f64,
    operation: &'static str,
) -> Result<ConvexShell<C>, Diagnostic>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
{
    let error = |code| Diagnostic::new(code, operation, "validate_input");
    let failed = || error(Code::BlendConstructionFailed);
    let mut vertex_index = HashMap::default();
    let mut vertices = Vec::new();
    let mut edge_index = HashMap::default();
    let mut original_edges = Vec::new();
    let mut edge_faces: Vec<Vec<(usize, bool)>> = Vec::new();
    let mut planes = Vec::new();
    for (i, face) in shell.iter().enumerate() {
        let boundaries = face.boundaries();
        if boundaries.len() != 1 {
            return Err(error(Code::UnsupportedTopology).face(i));
        }
        let surface = face.oriented_surface();
        let point = boundaries[0].front_vertex().ok_or_else(failed)?.point();
        let (u, v) = surface
            .search_parameter(point, None, 100)
            .ok_or_else(failed)?;
        let normal = surface.normal(u, v).normalize();
        let d = normal.dot(point.to_vec());
        let planar = |u, v| {
            let p = surface.subs(u, v);
            (normal.dot(p.to_vec()) - d).abs() <= tol.min(TOLERANCE)
                && surface.normal(u, v).normalize().near(&normal)
        };
        let interval = |(a, b): ParameterRange| {
            use std::ops::Bound::*;
            (
                match a {
                    Included(t) | Excluded(t) => t,
                    Unbounded => -1.0,
                },
                match b {
                    Included(t) | Excluded(t) => t,
                    Unbounded => 1.0,
                },
            )
        };
        let (ur, vr) = surface.parameter_range();
        let ((u0, u1), (v0, v1)) = (interval(ur), interval(vr));
        for a in 0..3 {
            for b in 0..3 {
                if !planar(
                    u0 + (u1 - u0) * a as f64 / 2.0,
                    v0 + (v1 - v0) * b as f64 / 2.0,
                ) {
                    return Err(error(Code::NonPlanarFace).face(i));
                }
            }
        }
        planes.push((normal, d));
        for edge in &boundaries[0] {
            for vertex in [edge.front(), edge.back()] {
                vertex_index.entry(vertex.id()).or_insert_with(|| {
                    let index = vertices.len();
                    vertices.push(vertex.clone());
                    index
                });
            }
            let k = *edge_index.entry(edge.id()).or_insert_with(|| {
                let k = original_edges.len();
                original_edges.push(edge.absolute_clone());
                edge_faces.push(Vec::new());
                k
            });
            edge_faces[k].push((i, edge.orientation()));
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            let start = edge.absolute_front().point();
            let direction = edge.absolute_back().point() - start;
            if direction.so_small() {
                return Err(error(Code::DegenerateCurve).face(i));
            }
            for k in 0..=4 {
                let point = curve.subs(a + (b - a) * k as f64 / 4.0);
                if (point - start).cross(direction.normalize()).magnitude() > TOLERANCE {
                    return Err(error(Code::UnsupportedGeometry).face(i));
                }
                let (u, v) = surface
                    .search_parameter(point, None, 100)
                    .ok_or_else(failed)?;
                if !planar(u, v) {
                    return Err(error(Code::NonPlanarFace).face(i));
                }
            }
        }
    }
    if selected.iter().any(|id| !edge_index.contains_key(id)) {
        return Err(error(Code::UnknownEdge));
    }
    if vertices.iter().any(|v| {
        planes
            .iter()
            .any(|&(n, d)| n.dot(v.point().to_vec()) > d + TOLERANCE)
    }) {
        return Err(error(Code::UnsupportedGeometry));
    }
    let mut incident = vec![Vec::new(); vertices.len()];
    let mut sides = Vec::new();
    for (k, edge) in original_edges.iter().enumerate() {
        let [(a, forward), (b, backward)] = edge_faces[k][..] else {
            return Err(error(Code::UnsupportedTopology));
        };
        if forward == backward {
            return Err(error(Code::UnsupportedTopology));
        }
        let pair = if forward { [a, b] } else { [b, a] };
        if selected.contains(&edge.id()) {
            let axis = edge.absolute_back().point() - edge.absolute_front().point();
            if planes[pair[0]]
                .0
                .cross(planes[pair[1]].0)
                .dot(axis.normalize())
                <= TOLERANCE
            {
                return Err(error(Code::ConcaveEdge));
            }
        }
        sides.push(pair);
        for v in [edge.front(), edge.back()] {
            incident[vertex_index[&v.id()]].push(k);
        }
    }
    Ok(ConvexShell {
        vertex_index,
        vertices,
        edge_index,
        original_edges,
        planes,
        incident,
        sides,
    })
}
