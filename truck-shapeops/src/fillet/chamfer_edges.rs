use super::convex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

pub(super) fn chamfer_edges(
    shell: &Shell,
    edges: &[EdgeID],
    distance: f64,
    tol: f64,
) -> Result<Shell, Diagnostic> {
    let error = |code| Diagnostic::new(code, "chamfer_solid_edges", "construct_blend");
    let selected: HashSet<_> = edges.iter().copied().collect();
    let convex::ConvexShell {
        original_edges,
        mut planes,
        incident,
        sides,
        ..
    } = convex::validate(shell, &selected, tol, "chamfer_solid_edges")?;
    if incident.iter().any(|edges| edges.len() != 3) {
        return Err(error(Code::UnsupportedTopology));
    }
    let mut bevels = Vec::new();
    for (k, edge) in original_edges.iter().enumerate() {
        if !selected.contains(&edge.id()) {
            continue;
        }
        let [a, b] = sides[k];
        let (n, m) = (planes[a].0, planes[b].0);
        if n.cross(m).so_small() {
            return Err(error(Code::UnsupportedGeometry));
        }
        let normal = (n + m).normalize();
        let d =
            normal.dot(edge.front().point().to_vec()) - distance * ((1.0 - n.dot(m)) / 2.0).sqrt();
        planes.push((normal, d));
        bevels.push(k);
    }

    // The convex result is the intersection of the original and bevel half-spaces.
    // Enumerate plane triples so junctions share vertices independently of selection order.
    let epsilon = tol.min(TOLERANCE);
    let mut vertices: Vec<Vertex> = Vec::new();
    for a in 0..planes.len() {
        for b in a + 1..planes.len() {
            for c in b + 1..planes.len() {
                let [(n, d), (m, e), (l, f)] = [planes[a], planes[b], planes[c]];
                let matrix = Matrix3::from_cols(n, m, l).transpose();
                if matrix.determinant().abs() <= TOLERANCE {
                    continue;
                }
                let Some(inverse) = matrix.invert() else {
                    continue;
                };
                let p = Point3::from_vec(inverse * Vector3::new(d, e, f));
                if planes.iter().any(|&(n, d)| n.dot(p.to_vec()) > d + epsilon) {
                    continue;
                }
                if !vertices.iter().any(|v| v.point().distance(p) <= epsilon) {
                    vertices.push(Vertex::new(p));
                }
            }
        }
    }
    let mut result = Shell::new();
    let mut shared_edges: HashMap<(usize, usize), Edge> = HashMap::default();
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::default();
    for (f, &(normal, d)) in planes.iter().enumerate() {
        let mut polygon: Vec<_> = vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| (normal.dot(v.point().to_vec()) - d).abs() <= epsilon)
            .map(|(i, _)| i)
            .collect();
        if polygon.len() < 3 {
            return Err(error(Code::OutsideNeighbour)
                .parameter("distance", distance)
                .face(f));
        }
        let center = polygon
            .iter()
            .map(|&i| vertices[i].point().to_vec())
            .sum::<Vector3>()
            / polygon.len() as f64;
        let x = (vertices[polygon[0]].point().to_vec() - center).normalize();
        let y = normal.cross(x);
        polygon.sort_by(|&a, &b| {
            let angle = |i: usize| {
                let v = vertices[i].point().to_vec() - center;
                v.dot(y).atan2(v.dot(x))
            };
            angle(a).total_cmp(&angle(b))
        });
        let mut wire = Wire::new();
        for i in 0..polygon.len() {
            let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
            if vertices[a].point().distance(vertices[b].point()) <= epsilon {
                return Err(error(Code::OutsideNeighbour).parameter("distance", distance));
            }
            let key = (a.min(b), a.max(b));
            edge_faces.entry(key).or_default().push(f);
            let edge = shared_edges
                .entry(key)
                .or_insert_with(|| builder::line(&vertices[a], &vertices[b]));
            wire.push_back(if edge.front() == &vertices[a] {
                edge.clone()
            } else {
                edge.inverse()
            });
        }
        let surface = if f < shell.len() {
            shell[f].oriented_surface()
        } else {
            let p = Point3::from_vec(center);
            Surface::Plane(Plane::new(p, p + x, p + y))
        };
        result.push(Face::try_new(vec![wire], surface).map_err(|e| {
            error(Code::InvalidOutputTopology)
                .face(f)
                .with_coded_source(e)
        })?);
    }
    let adjacent = |a: usize, b: usize| {
        edge_faces
            .values()
            .any(|faces| faces.contains(&a) && faces.contains(&b))
    };
    for (k, edge) in original_edges.iter().enumerate() {
        let [a, b] = sides[k];
        if !selected.contains(&edge.id()) && !adjacent(a, b) {
            return Err(error(Code::OutsideNeighbour).parameter("distance", distance));
        }
    }
    for (i, &k) in bevels.iter().enumerate() {
        if sides[k].iter().any(|&f| !adjacent(f, shell.len() + i)) {
            return Err(error(Code::OutsideNeighbour).parameter("distance", distance));
        }
    }
    if result.shell_condition() != ShellCondition::Closed {
        return Err(error(Code::InvalidOutputTopology));
    }
    Ok(result)
}
