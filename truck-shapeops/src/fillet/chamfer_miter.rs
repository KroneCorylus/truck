use super::planar::{neighborhood, polygon_area, valid_boundaries, valid_polygon};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

fn miter(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
    data: super::planar::Neighborhood,
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
    let epsilon = tol.min(TOLERANCE);
    let mut bevels = vec![None; data.original_edges.len()];
    let mut cones = vec![None; data.original_edges.len()];
    let mut shared = HashMap::default();
    // Each original edge has one endpoint pair per adjacent face. Bevels separate the pairs.
    let mut contacts: Vec<_> = data
        .original_edges
        .iter()
        .map(|edge| std::array::from_fn::<_, 2, _>(|_| [edge.front().clone(), edge.back().clone()]))
        .collect();
    for (k, edge) in data.original_edges.iter().enumerate() {
        if selected.contains(&edge.id()) {
            let [a, b] = data.sides[k];
            if data.planes[a].is_none() || data.planes[b].is_none() {
                if !matches!(edge.curve(), Curve::Line(_)) {
                    continue;
                }
                let (cylinder, planar) = if data.cylinders[a].is_some() {
                    (a, b)
                } else {
                    (b, a)
                };
                let (origin, axis, radius) = data.cylinders[cylinder].unwrap();
                let radial = edge.front().point() - origin;
                let z = (radial - axis * radial.dot(axis)).normalize();
                let y = -data.planes[planar].unwrap().0;
                if y.dot(z).abs() > TOLERANCE {
                    return Err(error(Code::UnsupportedGeometry));
                }
                let d = distances[&edge.id()];
                let flat = d[usize::from(planar == a.max(b))];
                let tangent = d[usize::from(cylinder == a.max(b))];
                let p = edge.front().point() - flat * z;
                let q = edge.front().point() - radius * z
                    + radius * (tangent * y + radius * z) / radius.hypot(tangent);
                let mut normal = (q - p).cross(axis).normalize();
                if normal.dot(edge.front().point() - p) < 0. {
                    normal = -normal;
                }
                bevels[k] = Some((normal, normal.dot(p.to_vec())));
                continue;
            }
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
    let mut corners: Vec<[Vec<Vertex>; 2]> =
        vec![[Vec::new(), Vec::new()]; data.original_edges.len()];
    let mut transitions = Vec::new();
    // Non-planar corner quads, filled bilinearly between their straight sides.
    let mut patches: Vec<Vec<Vertex>> = Vec::new();
    for (v, incident) in data.incident.iter().enumerate() {
        let chosen: Vec<_> = incident
            .iter()
            .copied()
            .filter(|&k| selected.contains(&data.original_edges[k].id()))
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
        let curved = incident
            .iter()
            .flat_map(|&k| data.sides[k])
            .find(|&f| data.cylinders[f].is_some());
        if let Some(cylinder) = curved {
            let corner = network_corner(&data, v, cylinder, distances, tol)?;
            for (k, f, vertex) in corner.contacts {
                set_contact(k, f, &vertex);
            }
            let edge = Edge::new(&corner.vertices[0], &corner.vertices[2], corner.section);
            shared.insert((edge.front().id(), edge.back().id()), edge.clone());
            shared.insert((edge.back().id(), edge.front().id()), edge.inverse());
            cones[corner.rim] = Some(corner.cone);
            let mut polygon = corner.vertices;
            if polygon_area(&polygon.iter().map(Vertex::point).collect::<Vec<_>>())
                .dot(corner.normal)
                < 0.
            {
                polygon.reverse();
            }
            transitions.push((corner.normal, polygon));
            continue;
        }
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
                let edge = &data.original_edges[other];
                let direction = edge.back().point() - edge.front().point();
                let away = if edge.front() == &data.vertices[v] {
                    direction
                } else {
                    -direction
                };
                let adjacent = if (polygon[1].point() - polygon[2].point()).dot(away) < 0. {
                    a
                } else {
                    b
                };
                let face = *data.sides[adjacent].iter().find(|&&f| f != common).unwrap();
                let boundary_direction = if data.sides[other][0] == face {
                    direction
                } else {
                    -direction
                };
                // Oppose the longer surviving face boundary along the unselected edge.
                if (polygon[2].point() - polygon[1].point()).dot(boundary_direction) > 0. {
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
            if concave.is_empty() {
                let vertex = data.vertices[v].point();
                let common = |k: usize, j: usize| {
                    data.sides[k]
                        .iter()
                        .copied()
                        .find(|f| data.sides[j].contains(f))
                        .ok_or_else(|| error(Code::UnsupportedTopology))
                };
                let mut crossings = HashMap::default();
                for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                    let [k, other] = [chosen[i], chosen[j]];
                    let point = intersection([
                        data.planes[common(k, other)?].unwrap(),
                        bevels[k].unwrap(),
                        bevels[other].unwrap(),
                    ])?;
                    crossings.insert((k.min(other), k.max(other)), point);
                }
                let crossing = |a: usize, b: usize| crossings[&(a.min(b), a.max(b))].clone();
                let away = |k: usize| {
                    let edge = &data.original_edges[k];
                    let direction = (edge.back().point() - edge.front().point()).normalize();
                    if edge.front() == &data.vertices[v] {
                        direction
                    } else {
                        -direction
                    }
                };
                let depth = |k: usize, j: usize| (crossing(k, j).point() - vertex).dot(away(k));
                // OCCT's three-corner rule: a pivot bevel is crossed by the other two no
                // closer to the vertex than they cross each other. They end at their sections
                // through those crossings, unless the sections nearly meet on the remaining face.
                let plans: Vec<_> = (0..3)
                    .filter_map(|i| {
                        let p = chosen[i];
                        let [d, f] = [chosen[(i + 1) % 3], chosen[(i + 2) % 3]];
                        if depth(d, p) < depth(d, f) - epsilon
                            || depth(f, p) < depth(f, d) - epsilon
                        {
                            return None;
                        }
                        let x = crossing(d, f).point();
                        let s_d = x + (depth(d, p) - depth(d, f)) * away(d);
                        let s_f = x + (depth(f, p) - depth(f, d)) * away(f);
                        let size = s_f
                            .distance(crossing(p, d).point())
                            .max(s_d.distance(crossing(p, f).point()));
                        Some((p, d, f, s_d, s_f, s_d.distance(s_f) < 0.3 * size))
                    })
                    .collect();
                let plan = plans.iter().find(|plan| plan.5).or(plans.first());
                let mut polygon = Vec::new();
                for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                    let [k, other] = [chosen[i], chosen[j]];
                    let face = common(k, other)?;
                    set_contact(k, face, &crossing(k, other));
                    set_contact(other, face, &crossing(k, other));
                }
                match plan {
                    Some(&(p, d, f, s_d, s_f, false)) => {
                        let face = common(d, f)?;
                        let [s_d, s_f] = [Vertex::new(s_d), Vertex::new(s_f)];
                        set_contact(d, face, &s_d);
                        set_contact(f, face, &s_f);
                        polygon.extend([crossing(p, d), crossing(p, f), s_f, s_d]);
                    }
                    _ => {
                        polygon.extend([0, 1, 2].map(|i| crossing(chosen[i], chosen[(i + 1) % 3])))
                    }
                }
                let points: Vec<_> = polygon.iter().map(Vertex::point).collect();
                let area = polygon_area(&points);
                if !area.magnitude2().is_finite() || area.so_small() {
                    return Err(error(Code::OutsideNeighbour));
                }
                let mut normal = area.normalize();
                let center =
                    points.iter().map(|p| p.to_vec()).sum::<Vector3>() / points.len() as f64;
                if normal.dot(vertex.to_vec() - center) < 0.0 {
                    normal = -normal;
                    polygon.reverse();
                }
                if points
                    .iter()
                    .all(|p| normal.dot(p.to_vec() - center).abs() <= epsilon)
                {
                    transitions.push((normal, polygon));
                } else {
                    patches.push(polygon);
                }
                continue;
            }
            let corner = intersection([0, 1, 2].map(|i| bevels[chosen[i]].unwrap()))?;
            for &k in &chosen {
                let end = usize::from(data.original_edges[k].front() != &data.vertices[v]);
                corners[k][end] = vec![corner.clone()];
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
            if !selected.contains(&edge.id()) {
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
    if data.cylinders.iter().any(Option::is_some) {
        for vertex in contacts.iter().flatten().flatten() {
            if data
                .planes
                .iter()
                .flatten()
                .any(|&(n, d)| n.dot(vertex.point().to_vec()) > d + epsilon)
            {
                return Err(error(Code::OutsideNeighbour));
            }
            for &(origin, axis, radius) in data.cylinders.iter().flatten() {
                let delta = vertex.point() - origin;
                if (delta - axis * delta.dot(axis)).magnitude() > radius + epsilon {
                    return Err(error(Code::OutsideNeighbour));
                }
            }
        }
    }
    for (k, cone) in cones.iter().enumerate() {
        if cone.is_none() {
            continue;
        }
        let cylinder = data.sides[k]
            .iter()
            .copied()
            .find(|&f| data.cylinders[f].is_some())
            .unwrap();
        let (origin, axis, _) = data.cylinders[cylinder].unwrap();
        for [a, b] in &contacts[k] {
            let center = origin + axis * (a.point() - origin).dot(axis);
            let u = a.point() - center;
            let w = b.point() - center;
            if (u.magnitude() - w.magnitude()).abs() > TOLERANCE {
                return Err(error(Code::UnsupportedGeometry));
            }
            let normal = u.cross(w).normalize();
            let angle = u.angle(w).0;
            if !angle.is_finite() || angle <= TOLERANCE || angle >= std::f64::consts::PI {
                return Err(error(Code::OutsideNeighbour));
            }
            let curve = Curve::Conic(Processor::with_transform(
                TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., angle)),
                Matrix4::from_cols(
                    u.extend(0.),
                    normal.cross(u).extend(0.),
                    normal.extend(0.),
                    center.to_homogeneous(),
                ),
            ));
            let edge = Edge::new(a, b, curve);
            shared.insert((a.id(), b.id()), edge.clone());
            shared.insert((b.id(), a.id()), edge.inverse());
        }
    }
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
                if !selected.contains(&edge.id()) && contacts[k][0] != contacts[k][1] {
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
            if data.cylinders[f].is_some() {
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
    for (k, cone) in cones.into_iter().enumerate() {
        let Some(mut surface) = cone else {
            continue;
        };
        let [[a, b], [c, d]] = contacts[k].clone();
        let polygon = [b, a, c, d];
        let wire = (0..4)
            .map(|i| shared_line(&polygon[i], &polygon[(i + 1) % 4], &mut shared))
            .collect::<Result<Wire, _>>()?;
        let points: Vec<_> = polygon.iter().map(Vertex::point).collect();
        let uv = surface
            .search_nearest_parameter(points[0], None, 100)
            .ok_or_else(|| error(Code::IntersectionFailed))?;
        if surface.normal(uv.0, uv.1).dot(polygon_area(&points)) < 0. {
            surface.invert();
        }
        result.push(
            Face::try_new(vec![wire], surface)
                .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    for polygon in patches {
        let [a, b, c, d] = [0, 1, 2, 3].map(|i| polygon[i].point());
        let surface = BSplineSurface::new(
            (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            vec![vec![a, d], vec![b, c]],
        );
        result.push(
            Face::try_new(
                vec![(0..4)
                    .map(|i| shared_line(&polygon[i], &polygon[(i + 1) % 4], &mut shared))
                    .collect::<Result<Wire, _>>()?],
                Surface::BSplineSurface(surface),
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

// Reconstructs a whole five-face quarter-cylinder with one fully selected corner
// and a nonelliptic corner section. The two axial setbacks must coincide, so the
// longitudinal bevel ends at a single cylinder section. This is not local corner
// assembly: the network path shares contact/section construction but preserves the
// original boundary loops instead of broadening these whole-shell topology guards.
fn quarter_cylinder_corner(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Option<Result<Shell, Diagnostic>> {
    if shell.len() != 5 || selected.len() != 3 {
        return None;
    }
    let data = neighborhood(shell, selected, "chamfer_solid_edges", true).ok()?;
    let curved: Vec<_> = data
        .cylinders
        .iter()
        .enumerate()
        .filter_map(|(f, c)| c.map(|c| (f, c)))
        .collect();
    let [(cylinder, (on_axis, axis, radius))] = curved[..] else {
        return None;
    };
    if data.original_edges.len() != 9
        || data.vertices.len() != 6
        || shell.iter().any(|f| f.boundaries().len() != 1)
        || data.planes.iter().filter(|p| p.is_some()).count() != 4
    {
        return None;
    }
    let v = data.incident.iter().position(|ks| {
        ks.len() == 3
            && ks
                .iter()
                .all(|&k| selected.contains(&data.original_edges[k].id()))
    })?;
    let chosen = &data.incident[v];
    let faces: HashSet<_> = chosen.iter().flat_map(|&k| data.sides[k]).collect();
    let cap = faces
        .iter()
        .copied()
        .find(|&f| data.planes[f].is_some_and(|(n, _)| n.cross(axis).so_small()))?;
    let radial = faces.iter().copied().find(|&f| f != cylinder && f != cap)?;
    let x = data.planes[cap]?.0;
    let y = -data.planes[radial]?.0;
    let vertex = data.vertices[v].point();
    let delta = vertex - on_axis;
    let z = (delta - axis * delta.dot(axis)).normalize();
    if x.dot(y).abs() > TOLERANCE || y.dot(z).abs() > TOLERANCE {
        return None;
    }
    let far = (0..shell.len())
        .find(|&f| f != cap && data.planes[f].is_some_and(|(n, _)| n.dot(x) < -1. + TOLERANCE))?;
    let floor = (0..shell.len()).find(|&f| f != cap && f != far && f != radial && f != cylinder)?;
    if data.planes[floor]?.0.dot(z) > -1. + TOLERANCE {
        return None;
    }
    let length = x.dot(vertex.to_vec()) + data.planes[far]?.1;
    let origin = vertex - radius * z - length * x;
    let point = |a: f64, b: f64, c: f64| origin + a * x + b * y + c * z;
    // Recognize the expected vertices and curve representations, and sample sector
    // containment. These checks do not prove global winding, incidence or trim equivalence.
    let expected = [
        point(0., 0., 0.),
        point(0., radius, 0.),
        point(0., 0., radius),
        point(length, 0., 0.),
        point(length, radius, 0.),
        point(length, 0., radius),
    ];
    if expected
        .iter()
        .any(|p| !data.vertices.iter().any(|v| v.point().near(p)))
    {
        return None;
    }
    for edge in &data.original_edges {
        let curve = edge.curve();
        if !matches!(curve, Curve::Line(_) | Curve::Conic(_)) {
            return None;
        }
        let (lo, hi) = curve.range_tuple();
        for i in 0..=16 {
            let delta = curve.subs(lo + (hi - lo) * i as f64 / 16.) - origin;
            let (a, b, c) = (delta.dot(x), delta.dot(y), delta.dot(z));
            if a < -TOLERANCE
                || a > length + TOLERANCE
                || b < -TOLERANCE
                || c < -TOLERANCE
                || b * b + c * c > radius * radius + TOLERANCE
            {
                return None;
            }
        }
    }
    let between = |a, b| {
        chosen
            .iter()
            .copied()
            .find(|&k| data.sides[k].contains(&a) && data.sides[k].contains(&b))
    };
    let rim = between(cap, cylinder)?;
    let lip = between(radial, cylinder)?;
    let straight = between(cap, radial)?;
    let setback = |k: usize, f: usize| {
        distances[&data.original_edges[k].id()]
            [usize::from(f == *data.sides[k].iter().max().unwrap())]
    };
    let (axial, inset) = (setback(rim, cylinder), setback(rim, cap));
    let (flat, tangent) = (setback(lip, radial), setback(lip, cylinder));
    let (other_axial, side) = (setback(straight, radial), setback(straight, cap));
    if (axial - other_axial).abs() > 64. * f64::EPSILON * axial.max(other_axial) {
        return None;
    }
    let error = |code| Diagnostic::new(code, "chamfer_solid_edges", "quarter_cylinder_corner");
    Some((|| {
        if axial >= length - TOLERANCE
            || inset >= radius - TOLERANCE
            || flat >= radius - TOLERANCE
            || side >= radius - inset - TOLERANCE
        {
            return Err(error(Code::OutsideNeighbour));
        }
        let [cy, cz, rz] = cylinder_corner_contacts(radius, inset, flat, tangent, side)
            .ok_or_else(|| error(Code::OutsideNeighbour))?;
        let coords = [
            (0., 0., 0.),
            (length - axial, 0., 0.),
            (length, side, 0.),
            (length, radius - inset, 0.),
            (length - axial, radius, 0.),
            (0., radius, 0.),
            (0., cy, cz),
            (0., 0., radius - flat),
            (length - axial, cy, cz),
            (length - axial, 0., radius - flat),
            (length, side, rz),
        ];
        let vertices: Vec<_> = coords
            .iter()
            .map(|&(a, b, c)| Vertex::new(point(a, b, c)))
            .collect();
        let mut shared = HashMap::default();
        let mut cache = |a: usize, b: usize, curve: Curve| {
            let edge = Edge::new(&vertices[a], &vertices[b], curve);
            shared.insert((vertices[a].id(), vertices[b].id()), edge.clone());
            shared.insert((vertices[b].id(), vertices[a].id()), edge.inverse());
        };
        for (a, b, r) in [(5, 6, radius), (4, 8, radius), (3, 10, radius - inset)] {
            let start = coords[a].2.atan2(coords[a].1);
            let end = coords[b].2.atan2(coords[b].1);
            cache(
                a,
                b,
                Curve::Conic(Processor::with_transform(
                    TrimmedCurve::new(UnitCircle::<Point3>::new(), (start, end)),
                    Matrix4::from_cols(
                        (y * r).extend(0.),
                        (z * r).extend(0.),
                        y.cross(z).extend(0.),
                        point(coords[a].0, 0., 0.).to_homogeneous(),
                    ),
                )),
            );
        }
        let generator = Curve::Line(Line(vertices[4].point(), vertices[3].point()));
        let cone = Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
            generator,
            origin,
            y.cross(z),
        )));
        let (p, q, r) = (
            vertices[8].point(),
            vertices[9].point(),
            vertices[10].point(),
        );
        let mut normal = (q - p).cross(r - p).normalize();
        if normal.dot(vertex - p) < 0. {
            normal = -normal;
        }
        let u = (q - p).normalize();
        let plane = Surface::Plane(Plane::new(p, p + u, p + normal.cross(u)));
        let reach = 2. * (length + radius);
        // Sampled containment rejects obvious excursions; it is not a global curve bound.
        let within_corner = |curve: &Curve| {
            let (a, b) = curve.range_tuple();
            (0..=16).all(|i| {
                let d = curve.subs(a + (b - a) * i as f64 / 16.) - origin;
                [
                    (d.dot(x), length - axial, length),
                    (d.dot(y), 0., radius),
                    (d.dot(z), 0., radius),
                ]
                .iter()
                .all(|&(v, lo, hi)| v >= lo - TOLERANCE && v <= hi + TOLERANCE)
            })
        };
        let section = cone_plane_section(&plane, &cone, p, r, reach, tol, within_corner)?;
        cache(8, 10, section);
        let mut result = Shell::new();
        let mut add =
            |indices: &[usize], mut surface: Surface, outward: Vector3| -> Result<(), Diagnostic> {
                let points: Vec<_> = indices.iter().map(|&i| vertices[i].point()).collect();
                let mut ids = indices.to_vec();
                if polygon_area(&points).dot(outward) < 0. {
                    ids.reverse();
                }
                let wire = (0..ids.len())
                    .map(|i| {
                        shared_line(
                            &vertices[ids[i]],
                            &vertices[ids[(i + 1) % ids.len()]],
                            &mut shared,
                        )
                    })
                    .collect::<Result<Wire, _>>()?;
                let center =
                    points.iter().map(|p| p.to_vec()).sum::<Vector3>() / points.len() as f64;
                let uv = surface
                    .search_nearest_parameter(Point3::from_vec(center), None, 100)
                    .ok_or_else(|| error(Code::IntersectionFailed))?;
                if surface.normal(uv.0, uv.1).dot(outward) < 0. {
                    surface.invert();
                }
                result.push(
                    Face::try_new(vec![wire], surface)
                        .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
                );
                Ok(())
            };
        // Original face order is retained for the modeling history.
        for (f, face) in shell.iter().enumerate() {
            let (ids, normal): (&[usize], Vector3) = if f == cylinder {
                (&[5, 4, 8, 6], y + z)
            } else if f == floor {
                (&[2, 3, 4, 5, 0, 1], -z)
            } else if f == far {
                (&[5, 6, 7, 0], -x)
            } else if f == cap {
                (&[3, 2, 10], x)
            } else {
                (&[0, 7, 9, 1], -y)
            };
            add(ids, face.oriented_surface(), normal)?;
        }
        add(&[8, 4, 3, 10], cone, x * inset + y * axial + z * axial)?;
        let flat_face = |ids: [usize; 3]| {
            Surface::Plane(Plane::new(
                vertices[ids[0]].point(),
                vertices[ids[1]].point(),
                vertices[ids[2]].point(),
            ))
        };
        add(&[7, 6, 8, 9], flat_face([7, 6, 8]), z - y)?;
        add(&[2, 1, 9, 10], flat_face([2, 1, 9]), x - y)?;
        add(&[9, 8, 10], plane, normal)?;
        if result.shell_condition() != ShellCondition::Closed {
            return Err(error(Code::InvalidOutputTopology));
        }
        Ok(result)
    })())
}

fn curved_network(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
) -> Option<super::planar::Neighborhood> {
    let data =
        super::planar::collect_neighborhood(shell, selected, "chamfer_solid_edges", true).ok()?;
    if selected.len() != data.original_edges.len()
        || data.incident.iter().any(|edges| edges.len() != 3)
        || data.cylinders.iter().filter(|c| c.is_some()).count() != 1
        || data.planes.iter().filter(|p| p.is_none()).count() != 1
        || shell.iter().any(|f| f.boundaries().len() != 1)
        || data
            .original_edges
            .iter()
            .any(|e| !matches!(e.curve(), Curve::Line(_) | Curve::Conic(_)))
    {
        return None;
    }
    for (k, edge) in data.original_edges.iter().enumerate() {
        if let [Some((n, _)), Some((m, _))] = data.sides[k].map(|f| data.planes[f]) {
            if n.cross(m).dot(edge.back().point() - edge.front().point()) <= TOLERANCE {
                return None;
            }
        }
    }
    for edge in &data.original_edges {
        let curve = edge.curve();
        let (lo, hi) = curve.range_tuple();
        if (0..=32).any(|i| {
            let point = curve.subs(lo + (hi - lo) * i as f64 / 32.);
            data.planes
                .iter()
                .flatten()
                .any(|&(n, d)| n.dot(point.to_vec()) > d + TOLERANCE)
        }) {
            return None;
        }
    }
    // Unequal planar corners need a separate native corner policy. The symmetric
    // three-bevel cap has the independently determined triangular section.
    for edges in &data.incident {
        if edges
            .iter()
            .flat_map(|&k| data.sides[k])
            .all(|f| data.planes[f].is_some())
        {
            let values: Vec<_> = edges
                .iter()
                .flat_map(|&k| distances[&data.original_edges[k].id()])
                .collect();
            if values
                .iter()
                .any(|&v| (v - values[0]).abs() > 64. * f64::EPSILON * v.max(values[0]))
            {
                return None;
            }
        }
    }
    Some(data)
}

struct NetworkCorner {
    contacts: Vec<(usize, usize, Vertex)>,
    vertices: Vec<Vertex>,
    normal: Vector3,
    section: Curve,
    cone: Surface,
    rim: usize,
}

fn network_corner(
    data: &super::planar::Neighborhood,
    v: usize,
    cylinder: usize,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Result<NetworkCorner, Diagnostic> {
    let error = |code| Diagnostic::new(code, "chamfer_solid_edges", "curved_network_corner");
    let (on_axis, axis, radius) = data.cylinders[cylinder].unwrap();
    let incident = &data.incident[v];
    let faces: HashSet<_> = incident.iter().flat_map(|&k| data.sides[k]).collect();
    let cap = faces
        .iter()
        .copied()
        .find(|&f| data.planes[f].is_some_and(|(n, _)| n.cross(axis).so_small()))
        .ok_or_else(|| error(Code::UnsupportedGeometry))?;
    let radial = faces
        .iter()
        .copied()
        .find(|&f| f != cylinder && f != cap)
        .ok_or_else(|| error(Code::UnsupportedGeometry))?;
    let x = data.planes[cap].unwrap().0;
    let y = -data.planes[radial]
        .ok_or_else(|| error(Code::UnsupportedGeometry))?
        .0;
    let vertex = data.vertices[v].point();
    let delta = vertex - on_axis;
    let z = (delta - axis * delta.dot(axis)).normalize();
    if x.dot(y).abs() > TOLERANCE || y.dot(z).abs() > TOLERANCE {
        return Err(error(Code::UnsupportedGeometry));
    }
    let between = |a, b| {
        incident
            .iter()
            .copied()
            .find(|&k| data.sides[k].contains(&a) && data.sides[k].contains(&b))
            .ok_or_else(|| error(Code::UnsupportedTopology))
    };
    let rim = between(cap, cylinder)?;
    let lip = between(radial, cylinder)?;
    let straight = between(cap, radial)?;
    if matches!(data.original_edges[rim].curve(), Curve::Line(_))
        || !matches!(data.original_edges[lip].curve(), Curve::Line(_))
        || !matches!(data.original_edges[straight].curve(), Curve::Line(_))
    {
        return Err(error(Code::UnsupportedGeometry));
    }
    let setback = |k: usize, f: usize| {
        distances[&data.original_edges[k].id()]
            [usize::from(f == *data.sides[k].iter().max().unwrap())]
    };
    let (axial, inset) = (setback(rim, cylinder), setback(rim, cap));
    let (flat, tangent) = (setback(lip, radial), setback(lip, cylinder));
    let (other_axial, side) = (setback(straight, radial), setback(straight, cap));
    if (axial - other_axial).abs() > 64. * f64::EPSILON * axial.max(other_axial) {
        return Err(error(Code::UnsupportedGeometry));
    }
    let [cy, cz, rz] = cylinder_corner_contacts(radius, inset, flat, tangent, side)
        .ok_or_else(|| error(Code::OutsideNeighbour))?;
    let origin = vertex - radius * z;
    let p = origin - axial * x + cy * y + cz * z;
    let q = vertex - axial * x - flat * z;
    let r = origin + side * y + rz * z;
    let vertices = vec![Vertex::new(p), Vertex::new(q), Vertex::new(r)];
    let mut normal = (q - p).cross(r - p).normalize();
    if normal.dot(vertex - p) < 0. {
        normal = -normal;
    }
    let u = (q - p).normalize();
    let plane = Surface::Plane(Plane::new(p, p + u, p + normal.cross(u)));
    let cone = Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
        Curve::Line(Line(vertex - axial * x, vertex - inset * z)),
        origin,
        y.cross(z),
    )));
    let within = |curve: &Curve| {
        let (a, b) = curve.range_tuple();
        (0..=16).all(|i| {
            let d = curve.subs(a + (b - a) * i as f64 / 16.) - origin;
            [
                (d.dot(x), -axial, 0.),
                (d.dot(y), 0., radius),
                (d.dot(z), 0., radius),
            ]
            .iter()
            .all(|&(v, lo, hi)| v >= lo - TOLERANCE && v <= hi + TOLERANCE)
        })
    };
    let section = cone_plane_section(&plane, &cone, p, r, 4. * (radius + axial), tol, within)?;
    let contacts = [
        (rim, cylinder, 0),
        (lip, cylinder, 0),
        (lip, radial, 1),
        (straight, radial, 1),
        (rim, cap, 2),
        (straight, cap, 2),
    ]
    .map(|(k, f, i)| (k, f, vertices[i].clone()))
    .to_vec();
    Ok(NetworkCorner {
        contacts,
        vertices,
        normal,
        section,
        cone,
        rim,
    })
}

fn cylinder_corner_contacts(
    radius: f64,
    inset: f64,
    flat: f64,
    tangent: f64,
    side: f64,
) -> Option<[f64; 3]> {
    if inset >= radius - TOLERANCE
        || flat >= radius - TOLERANCE
        || side >= radius - inset - TOLERANCE
    {
        return None;
    }
    // The lip uses tangent-projection setback, while the cap contact is a circle/line intersection.
    let cy = radius * tangent / radius.hypot(tangent);
    let cz = radius * radius / radius.hypot(tangent);
    let rz = ((radius - inset).powi(2) - side * side).sqrt();
    if cy >= side || cz <= radius - flat || rz >= radius - flat {
        return None;
    }
    Some([cy, cz, rz])
}

fn cone_plane_section(
    plane: &Surface,
    cone: &Surface,
    p: Point3,
    r: Point3,
    reach: f64,
    tol: f64,
    within_corner: impl Fn(&Curve) -> bool,
) -> Result<Curve, Diagnostic> {
    let error = |code| Diagnostic::new(code, "chamfer_solid_edges", "quarter_cylinder_corner");
    let curves = crate::local::try_intersect_surfaces(
        plane,
        ((-reach, reach), (-reach, reach)),
        cone,
        ((0., 1.), (0., std::f64::consts::TAU)),
        tol,
    )?;
    for mut curve in curves {
        // Elliptic native corners need a four-boundary patch, not this three-contact section.
        if matches!(curve, Curve::Conic(_)) {
            return Err(error(Code::UnsupportedGeometry));
        }
        let Some(a) = curve.search_parameter(p, None, 100) else {
            continue;
        };
        let Some(b) = curve.search_parameter(r, None, 100) else {
            continue;
        };
        if (a - b).abs() <= TOLERANCE {
            continue;
        }
        let (lo, hi) = curve.range_tuple();
        if a.min(b) < lo - TOLERANCE || a.max(b) > hi + TOLERANCE {
            continue;
        }
        if a.max(b) < hi {
            curve.cut(a.max(b));
        }
        if a.min(b) > lo {
            curve = curve.cut(a.min(b));
        }
        if a > b {
            curve.invert();
        }
        if within_corner(&curve) {
            return Ok(curve);
        }
    }
    Err(error(Code::IntersectionFailed))
}

pub(super) fn chamfer(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Result<Shell, Diagnostic> {
    if let Some(result) = quarter_cylinder_corner(shell, selected, distances, tol) {
        return result;
    }
    match neighborhood(shell, selected, "chamfer_solid_edges", false) {
        Ok(data) => miter(shell, selected, distances, tol, data),
        Err(original) => {
            if let Some(data) = curved_network(shell, selected, distances) {
                if let Ok(result) = miter(shell, selected, distances, tol, data) {
                    return Ok(result);
                }
            }
            Err(original)
        }
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;

    #[test]
    fn collected_curved_network_does_not_expand_fillet_admission() {
        let start = Vertex::new(Point3::new(0., 1., 0.));
        let arc = builder::rsweep(
            &start,
            Point3::origin(),
            Vector3::unit_x(),
            Rad(std::f64::consts::FRAC_PI_2),
            1,
        );
        let center = Vertex::new(Point3::origin());
        let mut wire = arc.clone();
        wire.extend([
            builder::line(arc.back_vertex().unwrap(), &center),
            builder::line(&center, &start),
        ]);
        let solid = builder::tsweep(
            &builder::try_attach_plane(&[wire]).unwrap(),
            Vector3::unit_x(),
        );
        let shell = &solid.boundaries()[0];
        let selected: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
        let collected = super::super::planar::collect_neighborhood(
            shell,
            &selected,
            "fillet_solid_edges",
            true,
        )
        .unwrap();
        assert_eq!(collected.original_edges.len(), 9);
        assert_eq!(collected.cylinders.iter().flatten().count(), 1);
        for curved in [false, true] {
            let error = neighborhood(shell, &selected, "fillet_solid_edges", curved)
                .err()
                .unwrap();
            assert_eq!(error.code, Code::NonPlanarFace);
            assert_eq!(error.stage, "validate_neighborhood");
        }
    }
}
