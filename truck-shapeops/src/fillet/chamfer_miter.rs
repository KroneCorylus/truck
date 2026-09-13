use super::planar::{neighborhood, polygon_area, valid_boundaries, valid_polygon};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

pub(super) fn chamfer(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Result<Shell, Diagnostic> {
    let distance = distances
        .values()
        .flatten()
        .copied()
        .fold(0.0_f64, f64::max);
    let error = |code| {
        Diagnostic::new(code, "chamfer_solid_edges", "construct_miter")
            .parameter("distance", distance)
    };
    let data = neighborhood(shell, selected, "chamfer_solid_edges")?;
    let epsilon = tol.min(TOLERANCE);
    let mut bevels = vec![None; data.original_edges.len()];
    // Each original edge has one endpoint pair per adjacent face. Bevels separate the pairs.
    let mut contacts: Vec<_> = data
        .original_edges
        .iter()
        .map(|edge| std::array::from_fn::<_, 2, _>(|_| [edge.front().clone(), edge.back().clone()]))
        .collect();
    for (k, edge) in data.original_edges.iter().enumerate() {
        if selected.contains(&edge.id()) {
            let [a, b] = data.sides[k];
            let (n, m) = (data.planes[a].unwrap().0, data.planes[b].unwrap().0);
            let mut d = distances[&edge.id()];
            if a > b {
                d.reverse();
            }
            let axis = (edge.back().point() - edge.front().point()).normalize();
            // Boundary orientation points into each face for both convex and concave edges.
            let p = edge.front().point() + d[0] * n.cross(axis);
            let q = edge.front().point() - d[1] * m.cross(axis);
            let chord = q - p;
            if !chord.magnitude2().is_finite() || chord.so_small() {
                return Err(error(Code::OutsideNeighbour));
            }
            let normal = chord.cross(axis).normalize();
            bevels[k] = Some((normal, normal.dot(p.to_vec())));
        }
    }
    let intersection = |[(n, d), (m, e), (l, f)]: [(Vector3, f64); 3]| {
        let matrix = Matrix3::from_cols(n, m, l).transpose();
        if matrix.determinant().abs() <= TOLERANCE {
            return Err(error(Code::UnsupportedGeometry));
        }
        let inverse = matrix
            .invert()
            .ok_or_else(|| error(Code::UnsupportedGeometry))?;
        Ok(Vertex::new(Point3::from_vec(
            inverse * Vector3::new(d, e, f),
        )))
    };
    let mut corners: Vec<[Option<Vertex>; 2]> = vec![[None, None]; data.original_edges.len()];
    let mut transitions = Vec::new();
    for (v, incident) in data.incident.iter().enumerate() {
        let chosen: Vec<_> = incident
            .iter()
            .copied()
            .filter(|&k| bevels[k].is_some())
            .collect();
        if chosen.is_empty() {
            continue;
        }
        if incident.len() != 3 {
            return Err(error(Code::UnsupportedTopology));
        }
        let mut set_contact = |k: usize, f: usize, point: &Vertex| {
            let side = usize::from(data.sides[k][0] != f);
            let end = usize::from(data.original_edges[k].front() != &data.vertices[v]);
            contacts[k][side][end] = point.clone();
        };
        if chosen.len() == 1 {
            let k = chosen[0];
            for &other in incident.iter().filter(|&&j| j != k) {
                let [a, b] = data.sides[other];
                let point = intersection([
                    data.planes[a].unwrap(),
                    data.planes[b].unwrap(),
                    bevels[k].unwrap(),
                ])?;
                for f in [a, b] {
                    set_contact(other, f, &point);
                    if data.sides[k].contains(&f) {
                        set_contact(k, f, &point);
                    }
                }
            }
        } else if chosen.len() == 2 {
            let [a, b] = [chosen[0], chosen[1]];
            let common = *data.sides[a]
                .iter()
                .find(|f| data.sides[b].contains(f))
                .ok_or_else(|| error(Code::UnsupportedTopology))?;
            let other = *incident.iter().find(|k| !chosen.contains(k)).unwrap();
            let [f, g] = data.sides[other];
            let tip = intersection([
                data.planes[common].unwrap(),
                bevels[a].unwrap(),
                bevels[b].unwrap(),
            ])?;
            let root_a = intersection([
                data.planes[f].unwrap(),
                data.planes[g].unwrap(),
                bevels[a].unwrap(),
            ])?;
            let (normal, offset) = bevels[b].unwrap();
            let root_b = if (normal.dot(root_a.point().to_vec()) - offset).abs() <= epsilon {
                root_a.clone()
            } else {
                intersection([
                    data.planes[f].unwrap(),
                    data.planes[g].unwrap(),
                    bevels[b].unwrap(),
                ])?
            };
            for (k, root) in [(a, &root_a), (b, &root_b)] {
                for face in data.sides[k] {
                    set_contact(k, face, if face == common { &tip } else { root });
                    if face != common {
                        set_contact(other, face, root);
                    }
                }
            }
            if root_a != root_b {
                let mut polygon = vec![tip, root_a, root_b];
                let mut normal =
                    polygon_area(&polygon.iter().map(Vertex::point).collect::<Vec<_>>());
                if normal.magnitude2() <= epsilon.powi(4) || !normal.magnitude2().is_finite() {
                    return Err(error(Code::OutsideNeighbour));
                }
                if normal.dot(bevels[a].unwrap().0 + bevels[b].unwrap().0) < 0.0 {
                    normal = -normal;
                    polygon.reverse();
                }
                transitions.push((normal.normalize(), polygon));
            }
        } else {
            let concave: Vec<_> = chosen
                .iter()
                .copied()
                .filter(|&k| {
                    let [a, b] = data.sides[k];
                    let axis = data.original_edges[k].back().point()
                        - data.original_edges[k].front().point();
                    data.planes[a]
                        .unwrap()
                        .0
                        .cross(data.planes[b].unwrap().0)
                        .dot(axis)
                        < 0.0
                })
                .collect();
            if concave.len() == 1 {
                let c = concave[0];
                let convex: Vec<_> = chosen.iter().copied().filter(|&k| k != c).collect();
                let [a, b] = [convex[0], convex[1]];
                let common_face = |a: usize, b: usize| {
                    data.sides[a]
                        .iter()
                        .copied()
                        .find(|f| data.sides[b].contains(f))
                        .ok_or_else(|| error(Code::UnsupportedTopology))
                };
                let common = common_face(a, b)?;
                let setback = |k: usize| {
                    let [f, g] = data.sides[k];
                    distances[&data.original_edges[k].id()][usize::from(common == f.max(g))]
                };
                // A constant setback continues across the new edge above the inside bevel.
                if (setback(a) - setback(b)).abs() <= epsilon {
                    let [f, g] = [common_face(a, c)?, common_face(b, c)?];
                    let root_a = intersection([
                        data.planes[f].unwrap(),
                        bevels[a].unwrap(),
                        bevels[c].unwrap(),
                    ])?;
                    let root_b = intersection([
                        data.planes[g].unwrap(),
                        bevels[b].unwrap(),
                        bevels[c].unwrap(),
                    ])?;
                    let top = data.planes[common].unwrap();
                    let inside = bevels[c].unwrap();
                    let axis = top.0.cross(inside.0).normalize();
                    let origin = intersection([
                        top,
                        inside,
                        (axis, axis.dot(data.vertices[v].point().to_vec())),
                    ])?
                    .point();
                    let inward = (top.0 * top.0.dot(inside.0) - inside.0).normalize();
                    let contact = origin + setback(a) * inward;
                    let cross = (root_b.point() - root_a.point()).cross(contact - root_a.point());
                    if !cross.magnitude2().is_finite() || cross.so_small() {
                        return Err(error(Code::OutsideNeighbour));
                    }
                    let mut normal = cross.normalize();
                    if normal.dot(top.0 + inside.0) < 0.0 {
                        normal = -normal;
                    }
                    let plane = (normal, normal.dot(contact.to_vec()));
                    let top_a = intersection([top, bevels[a].unwrap(), plane])?;
                    let top_b = intersection([top, bevels[b].unwrap(), plane])?;
                    for (k, face, point) in [
                        (a, common, &top_a),
                        (b, common, &top_b),
                        (a, f, &root_a),
                        (c, f, &root_a),
                        (b, g, &root_b),
                        (c, g, &root_b),
                    ] {
                        set_contact(k, face, point);
                    }
                    let mut polygon = vec![root_a, root_b, top_b, top_a];
                    if polygon_area(&polygon.iter().map(Vertex::point).collect::<Vec<_>>())
                        .dot(normal)
                        < 0.0
                    {
                        polygon.reverse();
                    }
                    transitions.push((normal, polygon));
                    continue;
                }
            }
            let corner = intersection([0, 1, 2].map(|i| bevels[chosen[i]].unwrap()))?;
            for &k in &chosen {
                let end = usize::from(data.original_edges[k].front() != &data.vertices[v]);
                corners[k][end] = Some(corner.clone());
                for &other in chosen.iter().filter(|&&j| j != k) {
                    let common = *data.sides[k]
                        .iter()
                        .find(|f| data.sides[other].contains(f))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    if k < other {
                        let point = intersection([
                            data.planes[common].unwrap(),
                            bevels[k].unwrap(),
                            bevels[other].unwrap(),
                        ])?;
                        set_contact(k, common, &point);
                        set_contact(other, common, &point);
                    }
                }
            }
        }
    }
    for (k, edge) in data.original_edges.iter().enumerate() {
        if contacts[k]
            .iter()
            .all(|pair| pair == &[edge.front().clone(), edge.back().clone()])
        {
            continue;
        }
        let start = edge.front().point();
        let axis = edge.back().point() - start;
        let length = axis.magnitude();
        let direction = axis / length;
        for [a, b] in &contacts[k] {
            if (b.point() - a.point()).dot(direction) <= epsilon {
                return Err(error(Code::OutsideNeighbour));
            }
            if bevels[k].is_none() {
                // Termination faces can grow into the material exposed at an inset corner.
                let can_extend = |vertex: &Vertex| {
                    let v = data.vertices.iter().position(|v| v == vertex).unwrap();
                    data.incident[v].iter().any(|&j| {
                        let [f, g] = data.sides[j];
                        let axis = data.original_edges[j].back().point()
                            - data.original_edges[j].front().point();
                        data.planes[f]
                            .unwrap()
                            .0
                            .cross(data.planes[g].unwrap().0)
                            .dot(axis.normalize())
                            < -epsilon
                    })
                };
                for vertex in [a, b] {
                    let t = (vertex.point() - start).dot(direction);
                    if (t < -epsilon && !can_extend(edge.front()))
                        || (t > length + epsilon && !can_extend(edge.back()))
                    {
                        return Err(error(Code::OutsideNeighbour));
                    }
                }
            }
        }
    }
    let mut shared = HashMap::default();
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for boundary in face.boundaries() {
            let mut wire = Wire::new();
            for edge in boundary {
                let k = data.edge_index[&edge.id()];
                let side = usize::from(data.sides[k][0] != f);
                let mut pair = contacts[k][side].clone();
                if !edge.orientation() {
                    pair.reverse();
                }
                let mut points = vec![pair[0].clone(), pair[1].clone()];
                if bevels[k].is_none() && contacts[k][0] != contacts[k][1] {
                    // Different corner contacts leave a longer segment on one face; split
                    // it at the other contact so both faces reuse the common remainder.
                    let start = pair[0].point();
                    let axis = pair[1].point() - start;
                    for point in &contacts[k][1 - side] {
                        let length = axis.magnitude();
                        let t = (point.point() - start).dot(axis) / length;
                        if t > epsilon && t < length - epsilon {
                            points.push(point.clone());
                        }
                    }
                    points.sort_by(|a, b| {
                        (a.point() - start)
                            .dot(axis)
                            .total_cmp(&(b.point() - start).dot(axis))
                    });
                }
                let segments: Vec<_> = points
                    .windows(2)
                    .map(|pair| {
                        if &pair[0] == edge.front() && &pair[1] == edge.back() {
                            Ok(edge.clone())
                        } else {
                            shared_line(&pair[0], &pair[1], &mut shared)
                        }
                    })
                    .collect::<Result<_, _>>()?;
                let first = segments.first().unwrap();
                if let Some(previous) = wire.back() {
                    if previous.back() != first.front() {
                        let connector = shared_line(previous.back(), first.front(), &mut shared)?;
                        wire.push_back(connector);
                    }
                }
                wire.extend(segments);
            }
            if wire.back_vertex() != wire.front_vertex() {
                let connector = shared_line(
                    wire.back_vertex().unwrap(),
                    wire.front_vertex().unwrap(),
                    &mut shared,
                )?;
                wire.push_back(connector);
            }
            boundaries.push(wire);
        }
        if let Some((normal, _)) = data.planes[f] {
            if !valid_boundaries(face, &boundaries, normal, tol) {
                return Err(error(Code::OutsideNeighbour).face(f));
            }
            result.push(
                Face::try_new(boundaries, face.oriented_surface())
                    .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
            );
        } else {
            if boundaries != face.boundaries() {
                return Err(error(Code::NonPlanarFace).face(f));
            }
            result.push(face.clone());
        }
    }
    let bevel_polygons = bevels.iter().enumerate().filter_map(|(k, bevel)| {
        let (normal, _) = (*bevel)?;
        let [[a, b], [c, d]] = contacts[k].clone();
        let mut polygon = vec![b, a];
        polygon.extend(corners[k][0].clone());
        polygon.extend([c, d]);
        polygon.extend(corners[k][1].clone());
        Some((normal, polygon))
    });
    for (normal, polygon) in bevel_polygons.chain(transitions) {
        let points: Vec<_> = polygon.iter().map(Vertex::point).collect();
        if !valid_polygon(&points, normal, epsilon) {
            return Err(error(Code::OutsideNeighbour));
        }
        let x = (points[1] - points[0]).normalize();
        let surface = Plane::new(points[0], points[0] + x, points[0] + normal.cross(x));
        result.push(
            Face::try_new(
                vec![(0..polygon.len())
                    .map(|i| {
                        shared_line(&polygon[i], &polygon[(i + 1) % polygon.len()], &mut shared)
                    })
                    .collect::<Result<Wire, _>>()?],
                surface.into(),
            )
            .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    if result.shell_condition() != ShellCondition::Closed {
        return Err(error(Code::InvalidOutputTopology));
    }
    Ok(result)
}

fn shared_line(
    a: &Vertex,
    b: &Vertex,
    shared: &mut HashMap<(VertexID, VertexID), Edge>,
) -> Result<Edge, Diagnostic> {
    if a.point().distance(b.point()) <= TOLERANCE {
        return Err(Diagnostic::new(
            Code::OutsideNeighbour,
            "chamfer_solid_edges",
            "construct_boundary",
        ));
    }
    let edge = shared
        .entry((a.id(), b.id()))
        .or_insert_with(|| builder::line(a, b))
        .clone();
    shared.insert((b.id(), a.id()), edge.inverse());
    Ok(edge)
}
