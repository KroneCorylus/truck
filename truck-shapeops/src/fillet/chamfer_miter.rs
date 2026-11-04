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
    preferences: &[super::ChamferCornerPreference],
    data: super::planar::Neighborhood,
) -> Result<Shell, Diagnostic> {
    let partial =
        data.cylinders.iter().any(Option::is_some) && selected.len() != data.original_edges.len();
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
    let mut used_preferences = HashSet::default();
    let mut has_planar_corners = false;
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
            if partial {
                let corner = tangent_corner(&data, v, cylinder, selected, distances)
                    .ok_or_else(|| error(Code::UnsupportedGeometry))?;
                for (k, f, vertex) in corner.contacts {
                    set_contact(k, f, &vertex);
                }
                cones[corner.rim] = Some(corner.cone);
                continue;
            }
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
        if partial && chosen.len() == 3 {
            let preferred = preferences
                .iter()
                .find(|p| p.vertex == data.vertices[v].id())
                .map(|p| p.edge);
            let plan = planar_corner_plan(&data, v, distances, preferred).ok_or_else(|| {
                Diagnostic::new(
                    Code::UnsupportedGeometry,
                    "chamfer_solid_edges",
                    "certify_planar_corner",
                )
            })?;
            has_planar_corners = true;
            if plan.used_preference {
                used_preferences.insert(data.vertices[v].id());
            }
            // Remote conic hulls are excluded by tangent_neighborhood. Replacing
            // those arcs by chords therefore preserves winding at this footprint.
            let in_trim = |point: Point3, f: usize| {
                let normal = data.planes[f].unwrap().0;
                let original = shell[f]
                    .boundaries()
                    .iter()
                    .filter(|wire| {
                        let polygon: Vec<_> =
                            wire.iter().map(|edge| edge.front().point()).collect();
                        super::planar::inside(point, &polygon, normal, epsilon)
                    })
                    .count()
                    % 2
                    == 1;
                if original {
                    return true;
                }
                // A reflex wire corner exposes exactly the local triangle between
                // its two certified contacts. Test that triangle, never a face
                // halfspace or the convex hull of the entire source loop.
                let reflex = shell[f].boundaries().iter().any(|wire| {
                    wire.iter().enumerate().any(|(i, next)| {
                        if next.front() != &data.vertices[v] {
                            return false;
                        }
                        let previous = &wire[(i + wire.len() - 1) % wire.len()];
                        let incoming = previous.back().point() - previous.front().point();
                        let outgoing = next.back().point() - next.front().point();
                        incoming.cross(outgoing).dot(normal)
                            < -epsilon * incoming.magnitude() * outgoing.magnitude()
                    })
                });
                let points: Vec<_> = plan
                    .contacts
                    .iter()
                    .filter(|(_, face, _)| *face == f)
                    .map(|(_, _, p)| p.point())
                    .collect();
                reflex
                    && points.len() == 2
                    && points[0].distance(points[1]) > epsilon
                    && super::planar::inside(
                        point,
                        &[data.vertices[v].point(), points[0], points[1]],
                        normal,
                        epsilon,
                    )
            };
            for (i, (_, f, vertex)) in plan.contacts.iter().enumerate() {
                if !in_trim(vertex.point(), *f)
                    || plan.contacts[i + 1..]
                        .iter()
                        .filter(|(_, other, _)| other == f)
                        .any(|(_, _, other)| {
                            !in_trim(vertex.point() + 0.5 * (other.point() - vertex.point()), *f)
                        })
                {
                    return Err(Diagnostic::new(
                        Code::OutsideNeighbour,
                        "chamfer_solid_edges",
                        "planar_corner_trim",
                    )
                    .face(*f));
                }
            }
            for (k, f, vertex) in plan.contacts {
                set_contact(k, f, &vertex);
            }
            if plan.polygon.len() == 3 {
                let points: Vec<_> = plan.polygon.iter().map(Vertex::point).collect();
                let normal = polygon_area(&points).normalize();
                if normal.dot(plan.normal) <= 0. {
                    return Err(error(Code::OutsideNeighbour));
                }
                transitions.push((normal, plan.polygon));
            } else {
                patches.push(plan.polygon);
            }
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
            let convex = incident.iter().all(|&k| {
                let [f, g] = data.sides[k];
                let edge = &data.original_edges[k];
                data.planes[f]
                    .unwrap()
                    .0
                    .cross(data.planes[g].unwrap().0)
                    .dot(edge.back().point() - edge.front().point())
                    > 0.
            });
            if convex && root_a != root_b {
                let vertex = data.vertices[v].point();
                let (deep, shallow, root) =
                    if root_a.point().distance2(vertex) > root_b.point().distance2(vertex) {
                        (a, b, &root_a)
                    } else {
                        (b, a, &root_b)
                    };
                let face = *data.sides[shallow].iter().find(|&&f| f != common).unwrap();
                let joint = intersection([
                    data.planes[face].unwrap(),
                    bevels[a].unwrap(),
                    bevels[b].unwrap(),
                ])?;
                // The deeper bevel wraps to the unselected edge through the shallower
                // bevel's terminal face; their common boundary is tip--joint.
                for k in [a, b] {
                    for face in data.sides[k] {
                        set_contact(
                            k,
                            face,
                            if face == common {
                                &tip
                            } else if k == deep {
                                root
                            } else {
                                &joint
                            },
                        );
                    }
                }
                for face in [f, g] {
                    set_contact(other, face, root);
                }
                let end = usize::from(data.original_edges[deep].front() != &data.vertices[v]);
                corners[deep][end] = vec![joint];
                continue;
            }
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
                let preference = preferences
                    .iter()
                    .find(|p| p.vertex == data.vertices[v].id());
                let preferred_plan = if let Some(preference) = preference {
                    let invalid = || {
                        Diagnostic::new(
                            Code::InvalidParameter,
                            "chamfer_solid_edges",
                            "select_corner_preference",
                        )
                    };
                    let axis = data
                        .cylinders
                        .iter()
                        .flatten()
                        .next()
                        .ok_or_else(invalid)?
                        .1;
                    let axis_edge = chosen
                        .iter()
                        .copied()
                        .find(|&k| away(k).cross(axis).magnitude() <= TOLERANCE)
                        .ok_or_else(invalid)?;
                    let caps: Vec<_> = chosen.iter().copied().filter(|&k| k != axis_edge).collect();
                    let h = distances[&data.original_edges[axis_edge].id()][0];
                    let same =
                        |a: f64, b: f64| (a - b).abs() <= 64. * f64::EPSILON * a.abs().max(b.abs());
                    if caps.len() != 2
                        || chosen.iter().any(|&k| {
                            let d = distances[&data.original_edges[k].id()];
                            !same(d[0], d[1])
                                || !same(d[0], if k == axis_edge { h } else { 2. * h })
                        })
                        || chosen.iter().enumerate().any(|(i, &k)| {
                            chosen[i + 1..]
                                .iter()
                                .any(|&j| away(k).dot(away(j)).abs() > TOLERANCE)
                        })
                        || plans.len() != 2
                        || plans.iter().any(|plan| plan.5)
                    {
                        return Err(invalid());
                    }
                    let plan = plans
                        .iter()
                        .find(|plan| {
                            data.original_edges[plan.0].id() == preference.edge
                                && caps.contains(&plan.0)
                        })
                        .ok_or_else(invalid)?;
                    used_preferences.insert(preference.vertex);
                    Some(plan)
                } else {
                    None
                };
                let bounded_triangle =
                    if preferred_plan.is_none() && data.cylinders.iter().any(Option::is_some) {
                        let values: Vec<_> = chosen
                            .iter()
                            .flat_map(|&k| distances[&data.original_edges[k].id()])
                            .collect();
                        let equal = values.iter().all(|&v| {
                            (v - values[0]).abs() <= 64. * f64::EPSILON * v.max(values[0])
                        });
                        if equal {
                            None
                        } else {
                            // Build intercepts at the corner origin: subtracting world plane
                            // offsets can erase the difference between unequal setbacks.
                            let mut local_bevels = vec![None; bevels.len()];
                            for &k in &chosen {
                                let [i, j] = data.sides[k];
                                let mut d = distances[&data.original_edges[k].id()];
                                if i > j {
                                    d.reverse();
                                }
                                let normal = -data.planes[i].unwrap().0 / d[1]
                                    - data.planes[j].unwrap().0 / d[0];
                                local_bevels[k] = Some((normal, 1.));
                            }
                            if !unequal_triangle_relations(
                                &data,
                                &chosen,
                                &local_bevels,
                                Point3::origin(),
                            ) {
                                return Err(error(Code::UnsupportedGeometry));
                            }
                            let scale = values.iter().copied().fold(0., f64::max);
                            let local_crossing = |a, b| {
                                let (n, _) = data.planes[common(a, b).ok()?]?;
                                let (m, e) = local_bevels[a]?;
                                let (l, f) = local_bevels[b]?;
                                Matrix3::from_cols(n, m, l)
                                    .transpose()
                                    .invert()
                                    .map(|inverse| inverse * Vector3::new(0., e, f))
                            };
                            let exact = plans
                                .iter()
                                .find(|plan| {
                                    if !plan.5 {
                                        return false;
                                    }
                                    let (p, d, f) = (plan.0, plan.1, plan.2);
                                    let [Some(x), Some(y), Some(z)] = [
                                        local_crossing(d, f),
                                        local_crossing(d, p),
                                        local_crossing(f, p),
                                    ] else {
                                        return false;
                                    };
                                    let u = (y - x).dot(away(d)) * away(d);
                                    let v = (z - x).dot(away(f)) * away(f);
                                    (u - v).magnitude() <= 64. * f64::EPSILON * scale
                                        && u.magnitude() <= 64. * f64::EPSILON * scale
                                })
                                .ok_or_else(|| error(Code::UnsupportedGeometry))?;
                            Some(exact)
                        }
                    } else {
                        None
                    };
                let plan = preferred_plan
                    .or(bounded_triangle)
                    .or_else(|| plans.iter().find(|plan| plan.5).or(plans.first()));
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
    if used_preferences.len() != preferences.len() {
        return Err(Diagnostic::new(
            Code::InvalidParameter,
            "chamfer_solid_edges",
            "unused_corner_preference",
        ));
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
            if partial && !selected.contains(&edge.id()) {
                if !matches!(edge.curve(), Curve::Line(_)) {
                    return Err(error(Code::UnsupportedGeometry));
                }
                if data.sides[k].iter().any(|&f| data.cylinders[f].is_some())
                    && [a, b].iter().any(|v| {
                        let t = (v.point() - start).dot(direction);
                        t < -epsilon || t > length + epsilon
                    })
                {
                    return Err(error(Code::OutsideNeighbour));
                }
            }
            if !selected.contains(&edge.id()) {
                // Termination faces can grow into the material exposed at an inset corner.
                let can_extend = |vertex: &Vertex| {
                    let v = data.vertices.iter().position(|v| v == vertex).unwrap();
                    data.incident[v].iter().any(|&j| {
                        let [f, g] = data.sides[j];
                        let axis = data.original_edges[j].back().point()
                            - data.original_edges[j].front().point();
                        match (data.planes[f], data.planes[g]) {
                            (Some((n, _)), Some((m, _))) => {
                                n.cross(m).dot(axis.normalize()) < -epsilon
                            }
                            _ => false,
                        }
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
    if has_planar_corners {
        let endpoint_points: HashMap<_, Vec<_>> = data
            .vertices
            .iter()
            .enumerate()
            .map(|(v, vertex)| {
                let mut points = vec![vertex.point()];
                for &k in &data.incident[v] {
                    let end = usize::from(data.original_edges[k].front() != vertex);
                    points.extend(contacts[k].iter().map(|pair| pair[end].point()));
                    points.extend(corners[k][end].iter().map(Vertex::point));
                }
                (vertex.id(), points)
            })
            .collect();
        let footprints: Vec<Vec<_>> = data
            .original_edges
            .iter()
            .map(|edge| {
                endpoint_points[&edge.front().id()]
                    .iter()
                    .chain(&endpoint_points[&edge.back().id()])
                    .copied()
                    .collect()
            })
            .collect();
        let reaches: Vec<_> = data
            .original_edges
            .iter()
            .map(|edge| {
                [edge.front(), edge.back()]
                    .iter()
                    .flat_map(|v| {
                        endpoint_points[&v.id()]
                            .iter()
                            .map(move |p| p.distance(v.point()))
                    })
                    .fold(0.0_f64, f64::max)
            })
            .collect();
        let separating_axes: Vec<_> = data
            .planes
            .iter()
            .flatten()
            .map(|p| p.0)
            .chain(bevels.iter().flatten().map(|p| p.0))
            .collect();
        let separated_footprints = |k: usize, j: usize| {
            let origin = footprints[k][0];
            separating_axes.iter().any(|axis| {
                let bounds = |points: &[Point3]| {
                    points
                        .iter()
                        .map(|p| (*p - origin).dot(*axis))
                        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                            (lo.min(x), hi.max(x))
                        })
                };
                let (a, b) = bounds(&footprints[k]);
                let (c, d) = bounds(&footprints[j]);
                let budget = 64. * f64::EPSILON * (a.abs() + b.abs() + c.abs() + d.abs());
                a > d + epsilon + budget || c > b + epsilon + budget
            })
        };
        for (k, edge) in data
            .original_edges
            .iter()
            .enumerate()
            .filter(|(_, e)| selected.contains(&e.id()) && matches!(e.curve(), Curve::Line(_)))
        {
            for (j, remote) in data.original_edges.iter().enumerate() {
                if k == j
                    || !matches!(remote.curve(), Curve::Line(_))
                    || [edge.front(), edge.back()]
                        .iter()
                        .any(|v| *v == remote.front() || *v == remote.back())
                {
                    continue;
                }
                if !separated_footprints(k, j)
                    && !separated_segments(
                        edge.front().point(),
                        edge.back().point(),
                        remote.front().point(),
                        remote.back().point(),
                        reaches[k] + reaches[j],
                    )
                {
                    return Err(Diagnostic::new(
                        Code::OutsideNeighbour,
                        "chamfer_solid_edges",
                        "planar_corner_footprint",
                    ));
                }
            }
        }
    }
    if !partial && data.cylinders.iter().any(Option::is_some) {
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
        if partial && boundaries == face.boundaries() {
            result.push(face.clone());
            continue;
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

// The admitted unequal family has bevel intercepts (a,b), (a,b), (a,b)
// in some orthogonal inward frame X,y,z, with 0<a<b. No proximity rule
// substitutes for the coincident opposite-face endpoints checked by the caller.
fn unequal_triangle_relations(
    data: &super::planar::Neighborhood,
    chosen: &[usize],
    bevels: &[Option<(Vector3, f64)>],
    vertex: Point3,
) -> bool {
    let mut faces: Vec<_> = chosen.iter().flat_map(|&k| data.sides[k]).collect();
    faces.sort_unstable();
    faces.dedup();
    if faces.len() != 3 {
        return false;
    }
    let axes: Vec<_> = faces.iter().map(|&f| -data.planes[f].unwrap().0).collect();
    if (0..3).any(|i| (i + 1..3).any(|j| axes[i].dot(axes[j]).abs() > 64. * f64::EPSILON)) {
        return false;
    }
    let intercept = |i: usize, j: usize, axis: usize| {
        let k = chosen
            .iter()
            .copied()
            .find(|&k| data.sides[k].contains(&faces[i]) && data.sides[k].contains(&faces[j]))
            .unwrap();
        let (n, d) = bevels[k].unwrap();
        (d - n.dot(vertex.to_vec())) / n.dot(axes[axis])
    };
    for x in 0..3 {
        for y in 0..3 {
            if x == y {
                continue;
            }
            let z = 3 - x - y;
            let a = intercept(x, y, x);
            let b = intercept(x, y, y);
            let same = |u: f64, v: f64| (u - v).abs() <= 64. * f64::EPSILON * u.abs().max(v.abs());
            if a > 0.
                && b > a
                && a.is_finite()
                && b.is_finite()
                && same(intercept(x, z, x), a)
                && same(intercept(x, z, z), b)
                && same(intercept(y, z, y), a)
                && same(intercept(y, z, z), b)
            {
                return true;
            }
        }
    }
    false
}

struct TangentCorner {
    contacts: Vec<(usize, usize, Vertex)>,
    rim: usize,
    cone: Surface,
}

fn tangent_corner(
    data: &super::planar::Neighborhood,
    v: usize,
    cylinder: usize,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
) -> Option<TangentCorner> {
    let (origin, axis, radius) = data.cylinders[cylinder]?;
    let incident = &data.incident[v];
    if incident.len() != 3 {
        return None;
    }
    let faces: HashSet<_> = incident.iter().flat_map(|&k| data.sides[k]).collect();
    let cap = faces
        .iter()
        .copied()
        .find(|&f| data.planes[f].is_some_and(|(n, _)| n.cross(axis).magnitude() < 1e-10))?;
    let side = faces.iter().copied().find(|&f| f != cap && f != cylinder)?;
    let x = data.planes[cap]?.0;
    let vertex = data.vertices[v].point();
    let delta = vertex - origin;
    let radial = delta - axis * delta.dot(axis);
    let z = radial / radius;
    if (radial.magnitude() - radius).abs() > radius * 1e-10
        || (data.planes[side]?.0 - z).magnitude() > 1e-10
    {
        return None;
    }
    let between = |a, b| {
        incident
            .iter()
            .copied()
            .find(|&k| data.sides[k].contains(&a) && data.sides[k].contains(&b))
    };
    let rim = between(cap, cylinder)?;
    let straight = between(cap, side)?;
    let seam = between(side, cylinder)?;
    if !selected.contains(&data.original_edges[rim].id())
        || !selected.contains(&data.original_edges[straight].id())
        || selected.contains(&data.original_edges[seam].id())
        || !matches!(data.original_edges[rim].curve(), Curve::Conic(_))
        || !matches!(data.original_edges[straight].curve(), Curve::Line(_))
        || !matches!(data.original_edges[seam].curve(), Curve::Line(_))
    {
        return None;
    }
    let Curve::Conic(arc) = data.original_edges[rim].curve() else {
        return None;
    };
    let (lo, hi) = arc.range_tuple();
    let m = arc.transform();
    let center = Point3::from_vec(m.w.truncate());
    let expected_center = vertex - radius * z;
    if !lo.is_finite()
        || !hi.is_finite()
        || hi - lo <= 0.
        || hi - lo >= std::f64::consts::PI
        || center.distance(expected_center) > radius * 1e-10
        || (m.x.truncate().magnitude() - radius).abs() > radius * 1e-10
        || (m.y.truncate().magnitude() - radius).abs() > radius * 1e-10
        || m.x.truncate().dot(m.y.truncate()).abs() > radius * radius * 1e-10
        || m.x.truncate().dot(axis).abs() > radius * 1e-10
        || m.y.truncate().dot(axis).abs() > radius * 1e-10
    {
        return None;
    }
    let setback = |k: usize, f: usize| {
        distances
            .get(&data.original_edges[k].id())
            .map(|d| d[usize::from(f == *data.sides[k].iter().max().unwrap())])
    };
    let a = setback(rim, cylinder)?;
    let b = setback(rim, cap)?;
    if !a.is_finite()
        || !b.is_finite()
        || a <= 0.
        || b <= 0.
        || b >= radius
        || (a - setback(straight, side)?).abs() > 64. * f64::EPSILON * a
        || (b - setback(straight, cap)?).abs() > 64. * f64::EPSILON * b
        || vertex.to_vec().magnitude() * f64::EPSILON * 128. > a.min(b) * 1e-8
    {
        return None;
    }
    let t = Vertex::new(vertex - b * z);
    let u = Vertex::new(vertex - a * x);
    let cone = Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
        Curve::Line(Line(u.point(), t.point())),
        vertex - radius * z,
        axis,
    )));
    Some(TangentCorner {
        contacts: vec![
            (rim, cap, t.clone()),
            (straight, cap, t),
            (rim, cylinder, u.clone()),
            (straight, side, u.clone()),
            (seam, cylinder, u.clone()),
            (seam, side, u),
        ],
        rim,
        cone,
    })
}

struct PlanarCornerPlan {
    contacts: Vec<(usize, usize, Vertex)>,
    polygon: Vec<Vertex>,
    normal: Vector3,
    used_preference: bool,
}

// All intersections are solved about the original vertex. A plan owns its contact
// identities; the assembly's shared_line cache supplies opposite boundary uses.
fn planar_corner_plan(
    data: &super::planar::Neighborhood,
    v: usize,
    distances: &HashMap<EdgeID, [f64; 2]>,
    preferred: Option<EdgeID>,
) -> Option<PlanarCornerPlan> {
    let chosen = &data.incident[v];
    if chosen.len() != 3 {
        return None;
    }
    let origin = data.vertices[v].point();
    let mut normals = Vec::new();
    let mut offsets = Vec::new();
    let mut axes = Vec::new();
    let mut concave = Vec::new();
    for (i, &k) in chosen.iter().enumerate() {
        let edge = &data.original_edges[k];
        if !matches!(edge.curve(), Curve::Line(_)) {
            return None;
        }
        let [a, b] = data.sides[k];
        let n = data.planes[a]?.0;
        let m = data.planes[b]?.0;
        let mut d = *distances.get(&edge.id())?;
        if d.iter().any(|x| !x.is_finite() || *x <= TOLERANCE) {
            return None;
        }
        if a > b {
            d.reverse();
        }
        let delta = edge.back().point() - edge.front().point();
        if !delta.magnitude2().is_finite() || delta.magnitude() <= TOLERANCE {
            return None;
        }
        let axis = delta.normalize();
        axes.push(if edge.front() == &data.vertices[v] {
            axis
        } else {
            -axis
        });
        concave.push(n.cross(m).dot(axis) < 0.);
        let p = d[0] * n.cross(axis);
        let q = -d[1] * m.cross(axis);
        let cross = (q - p).cross(axis);
        if !cross.magnitude2().is_finite() || cross.magnitude() <= TOLERANCE {
            return None;
        }
        normals.push(cross.normalize());
        offsets.push(normals[i].dot(p));
    }
    let frame = Matrix3::from_cols(axes[0], axes[1], axes[2]);
    if frame.determinant().abs() <= TOLERANCE {
        return None;
    }
    let common = |i: usize, j: usize| {
        data.sides[chosen[i]]
            .iter()
            .copied()
            .find(|f| data.sides[chosen[j]].contains(f))
    };
    let mut crossing = HashMap::default();
    for (i, j) in [(0, 1), (1, 2), (0, 2)] {
        let f = common(i, j)?;
        let n = data.planes[f]?.0;
        let matrix = Matrix3::from_cols(n, normals[i], normals[j]).transpose();
        if matrix.determinant().abs() <= TOLERANCE {
            return None;
        }
        let point = matrix.invert()? * Vector3::new(0., offsets[i], offsets[j]);
        if !point.magnitude2().is_finite()
            || n.dot(point).abs() > TOLERANCE
            || (normals[i].dot(point) - offsets[i]).abs() > TOLERANCE
            || (normals[j].dot(point) - offsets[j]).abs() > TOLERANCE
        {
            return None;
        }
        let vertex = Vertex::new(origin + point);
        if (vertex.point() - origin - point).magnitude() > TOLERANCE {
            return None;
        }
        crossing.insert((i, j), (point, vertex));
    }
    let at = |i: usize, j: usize| &crossing[&(i.min(j), i.max(j))];
    let outward = normals.iter().copied().sum::<Vector3>();
    if outward.so_small() {
        return None;
    }
    let outward = outward.normalize();
    let mut contacts = Vec::new();
    for (i, j) in [(0, 1), (1, 2), (0, 2)] {
        let f = common(i, j)?;
        contacts.push((chosen[i], f, at(i, j).1.clone()));
        contacts.push((chosen[j], f, at(i, j).1.clone()));
    }
    let section = |p: usize| {
        let a = (p + 1) % 3;
        let b = (p + 2) % 3;
        let x = at(a, b).0;
        let alpha = (at(p, a).0 - x).dot(axes[a]);
        let beta = (at(p, b).0 - x).dot(axes[b]);
        (a, b, alpha, beta, x + alpha * axes[a], x + beta * axes[b])
    };
    let count = concave.iter().filter(|&&x| x).count();
    if preferred.is_some() && count != 0 && count != 3 {
        return None;
    }
    let support_outward = (0..3)
        .map(|i| common(i, (i + 1) % 3).and_then(|f| data.planes[f].map(|p| p.0)))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .sum::<Vector3>()
        .normalize();
    let signed_section = |p: usize| {
        let (a, b, _, _, d, c) = section(p);
        let mut polygon = vec![
            at(p, a).1.clone(),
            at(p, b).1.clone(),
            Vertex::new(origin + c),
            Vertex::new(origin + d),
        ];
        // The affine Jacobian has one strict projected sign at all four corners.
        // Together with the convex projected boundary this certifies an embedded graph.
        regular_planar_patch(&mut polygon, origin, support_outward).then_some(())?;
        let inverse = frame.invert()?;
        for point in [at(p, a).0, at(p, b).0, c, d] {
            let coordinates = inverse * point;
            if coordinates.x < -TOLERANCE
                || coordinates.y < -TOLERANCE
                || coordinates.z < -TOLERANCE
            {
                return None;
            }
        }
        let transverse = (c - at(p, a).0)
            .magnitude()
            .max((d - at(p, b).0).magnitude());
        let separation = (c - d).magnitude() - 0.3 * transverse;
        if separation.abs() <= TOLERANCE {
            return None;
        }
        Some(separation > 0.)
    };
    let mut signed = false;
    let pivot = if count == 1 {
        Some(concave.iter().position(|&x| x)?)
    } else if count == 3 {
        let candidates: Vec<_> = (0..3)
            .filter(|&p| {
                let (_, _, alpha, beta, _, _) = section(p);
                alpha >= -TOLERANCE && beta >= -TOLERANCE
            })
            .collect();
        if let Some(edge) = preferred {
            if candidates.len() != 1
                || data.original_edges[chosen[candidates[0]]].id() != edge
                || signed_section(candidates[0]) != Some(true)
            {
                return None;
            }
            signed = true;
            Some(candidates[0])
        } else {
            // A regular noncollapsed reentrant section is a separate material choice;
            // never silently substitute the historical collapsed-triangle family.
            if candidates.len() == 1 && signed_section(candidates[0]) == Some(true) {
                return None;
            }
            None
        }
    } else if count == 0 {
        let candidates: Vec<_> = (0..3)
            .filter(|&p| {
                let (_, _, alpha, beta, _, _) = section(p);
                alpha >= -TOLERANCE && beta >= -TOLERANCE
            })
            .collect();
        if candidates.is_empty() {
            let edge = preferred?;
            let p = chosen
                .iter()
                .position(|&k| data.original_edges[k].id() == edge)?;
            let section = signed_section(p)?;
            signed = true;
            if section {
                Some(p)
            } else {
                None
            }
        } else {
            let mut sections = Vec::new();
            for &p in &candidates {
                let (a, b, _, _, d, c) = section(p);
                let transverse = (c - at(p, a).0)
                    .magnitude()
                    .max((d - at(p, b).0).magnitude());
                // OCCT C3 applies this physical-length collapse policy only when all
                // concavities match. Evaluate every admissible pivot, not its first
                // traversal candidate. A threshold-uncertain decision is rejected.
                let separation = (c - d).magnitude() - 0.3 * transverse;
                if separation.abs() <= TOLERANCE {
                    return None;
                }
                if separation > 0. {
                    sections.push(p);
                }
            }
            if let Some(edge) = preferred {
                // The bounded A5 ambiguity has one regular section competing with a
                // triangle. Only its section preference has an admitted contract.
                if candidates.len() != 2
                    || sections.len() != 1
                    || data.original_edges[chosen[sections[0]]].id() != edge
                    || axes
                        .iter()
                        .enumerate()
                        .any(|(i, a)| axes[i + 1..].iter().any(|b| a.dot(*b).abs() > TOLERANCE))
                {
                    return None;
                }
                let alternate = *candidates.iter().find(|&&p| p != sections[0])?;
                let (_, _, alpha, beta, _, _) = section(alternate);
                if alpha.abs() > TOLERANCE || beta.abs() > TOLERANCE {
                    return None;
                }
                Some(sections[0])
            } else if sections.is_empty() {
                None
            } else if candidates.len() == 1 {
                Some(sections[0])
            } else {
                return None;
            }
        }
    } else {
        return None;
    };
    let mut polygon = if let Some(p) = pivot {
        let (a, b, _, _, d, c) = section(p);
        let f = common(a, b)?;
        let d = Vertex::new(origin + d);
        let c = Vertex::new(origin + c);
        for (k, face, vertex) in &mut contacts {
            if *face == f {
                *vertex = if *k == chosen[a] {
                    d.clone()
                } else {
                    c.clone()
                };
            }
        }
        vec![at(p, a).1.clone(), at(p, b).1.clone(), c, d]
    } else {
        vec![at(0, 1).1.clone(), at(1, 2).1.clone(), at(0, 2).1.clone()]
    };
    let outward = if signed { support_outward } else { outward };
    if !regular_planar_patch(&mut polygon, origin, outward) {
        return None;
    }
    Some(PlanarCornerPlan {
        contacts,
        polygon,
        normal: outward,
        used_preference: preferred.is_some(),
    })
}

fn regular_planar_patch(polygon: &mut [Vertex], origin: Point3, outward: Vector3) -> bool {
    let points: Vec<_> = polygon.iter().map(|p| p.point() - origin).collect();
    let area = (1..points.len() - 1)
        .map(|i| (points[i] - points[0]).cross(points[i + 1] - points[0]))
        .sum::<Vector3>();
    if !area.magnitude2().is_finite() || area.magnitude() <= TOLERANCE * TOLERANCE {
        return false;
    }
    if area.dot(outward) < 0. {
        polygon.reverse();
    }
    for i in 0..polygon.len() {
        let a = polygon[i].point();
        let b = polygon[(i + 1) % polygon.len()].point();
        let c = polygon[(i + 2) % polygon.len()].point();
        let turn = (b - a).cross(c - b).dot(outward);
        if !turn.is_finite() || turn <= TOLERANCE * ((b - a).magnitude() + (c - b).magnitude()) {
            return false;
        }
    }
    true
}

fn separated_segments(a: Point3, b: Point3, c: Point3, d: Point3, clearance: f64) -> bool {
    let u = b - a;
    let v = d - c;
    let w = a - c;
    let uu = u.magnitude2();
    let vv = v.magnitude2();
    let uv = u.dot(v);
    if uu <= TOLERANCE * TOLERANCE || vv <= TOLERANCE * TOLERANCE {
        return false;
    }
    let axis = u / uu.sqrt();
    let p = (c - a) - axis * (c - a).dot(axis);
    let q = (d - a) - axis * (d - a).dot(axis);
    let delta = q - p;
    let parameter = if delta.magnitude2() > f64::MIN_POSITIVE {
        (-p.dot(delta) / delta.magnitude2()).clamp(0., 1.)
    } else {
        0.
    };
    let budget =
        64. * f64::EPSILON * ((c - a).magnitude() + (d - a).magnitude() + uu.sqrt() + vv.sqrt());
    let lower = (p + parameter * delta).magnitude();
    if lower.is_finite() && lower > clearance + TOLERANCE + budget {
        return true;
    }
    let point_segment = |p: Point3, q: Point3, axis: Vector3| {
        p.distance(q + ((p - q).dot(axis) / axis.magnitude2()).clamp(0., 1.) * axis)
    };
    let mut distance = point_segment(a, c, v)
        .min(point_segment(b, c, v))
        .min(point_segment(c, a, u))
        .min(point_segment(d, a, u));
    let determinant = uu * vv - uv * uv;
    if determinant <= 64. * f64::EPSILON * uu * vv {
        return false;
    }
    let s = (uv * v.dot(w) - vv * u.dot(w)) / determinant;
    let t = (uu * v.dot(w) - uv * u.dot(w)) / determinant;
    if (0. ..=1.).contains(&s) && (0. ..=1.).contains(&t) {
        distance = distance.min((w + s * u - t * v).magnitude());
    }
    let budget = budget * (uu * vv / determinant);
    budget.is_finite() && distance.is_finite() && distance > clearance + TOLERANCE + budget
}

fn selected_rim_neighbors(edges: &[Edge], selected: &HashSet<EdgeID>) -> Vec<Vec<usize>> {
    edges
        .iter()
        .filter(|rim| {
            #[cfg(test)]
            rim_topology_tests::QUERIES.with(|q| q.set((q.get().0 + 1, q.get().1)));
            selected.contains(&rim.id()) && matches!(rim.curve(), Curve::Conic(_))
        })
        .map(|rim| {
            edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| {
                    #[cfg(test)]
                    rim_topology_tests::QUERIES.with(|q| q.set((q.get().0, q.get().1 + 1)));
                    [rim.front(), rim.back()]
                        .iter()
                        .any(|v| *v == edge.front() || *v == edge.back())
                })
                .map(|(k, _)| k)
                .collect()
        })
        .collect()
}

fn tangent_neighborhood(
    data: &super::planar::Neighborhood,
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    preferences: &[super::ChamferCornerPreference],
) -> Option<()> {
    if data.cylinders.iter().filter(|c| c.is_some()).count() != 1
        || data.planes.iter().filter(|p| p.is_none()).count() != 1
        || data
            .original_edges
            .iter()
            .any(|e| !matches!(e.curve(), Curve::Line(_) | Curve::Conic(_)))
    {
        return None;
    }
    let cylinder = data.cylinders.iter().position(Option::is_some)?;
    let boundaries = shell[cylinder].boundaries();
    if boundaries.len() != 1 || boundaries[0].len() != 4 {
        return None;
    }
    // Every accepted tangent corner requires a selected conic rim on this cylinder.
    // Reject its absence before constructing unrelated planar corner footprints.
    if !boundaries[0]
        .iter()
        .any(|edge| selected.contains(&edge.id()) && matches!(edge.curve(), Curve::Conic(_)))
    {
        return None;
    }
    let mut ends = 0;
    let mut corner_extent = HashMap::default();
    for (v, incident) in data.incident.iter().enumerate() {
        let chosen = incident
            .iter()
            .filter(|&&k| selected.contains(&data.original_edges[k].id()))
            .count();
        if chosen == 0 {
            continue;
        }
        if incident.len() != 3 {
            return None;
        }
        if chosen == 3 {
            let preferred = preferences
                .iter()
                .find(|p| p.vertex == data.vertices[v].id())
                .map(|p| p.edge);
            let plan = planar_corner_plan(data, v, distances, preferred)?;
            let radius = plan
                .polygon
                .iter()
                .map(|p| p.point().distance(data.vertices[v].point()))
                .fold(0.0_f64, f64::max);
            corner_extent.insert(data.vertices[v].id(), radius);
        }
        if incident.iter().any(|&k| data.sides[k].contains(&cylinder)) {
            tangent_corner(data, v, cylinder, selected, distances)?;
            ends += 1;
        }
    }
    if ends != 2 && ends != 4 {
        return None;
    }
    // Conservative analytic boxes exclude remote trims from every selected strip.
    // Conics use their entire ellipse, so this can reject safe, tightly packed trims.
    let (origin, axis, _) = data.cylinders[cylinder]?;
    let delta = shell[cylinder].vertex_iter().next()?.point() - origin;
    let radial = (delta - axis * delta.dot(axis)).normalize();
    let tangent = axis.cross(radial);
    let vector = |v: Vector3| Vector3::new(v.dot(radial), v.dot(tangent), v.dot(axis));
    let point = |p: Point3| Point3::from_vec(vector(p - origin));
    let bounds = |edge: &Edge| -> Option<(Point3, Point3)> {
        match edge.curve() {
            Curve::Line(_) => {
                let a = point(edge.front().point());
                let b = point(edge.back().point());
                Some((
                    Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
                    Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
                ))
            }
            Curve::Conic(c) => {
                let m = c.transform();
                if m.x.w != 0. || m.y.w != 0. || m.z.w != 0. || m.w.w != 1. {
                    return None;
                }
                let center = point(Point3::from_vec(m.w.truncate()));
                let u = vector(m.x.truncate());
                let v = vector(m.y.truncate());
                let radius = Vector3::new(u.x.hypot(v.x), u.y.hypot(v.y), u.z.hypot(v.z));
                Some((center - radius, center + radius))
            }
            _ => None,
        }
    };
    for (id, radius) in &corner_extent {
        let center = point(data.vertices.iter().find(|v| v.id() == *id)?.point());
        for arc in data
            .original_edges
            .iter()
            .filter(|e| matches!(e.curve(), Curve::Conic(_)))
        {
            let (lo, hi) = bounds(arc)?;
            if (0..3).all(|i| {
                lo[i] <= center[i] + radius + TOLERANCE && hi[i] >= center[i] - radius - TOLERANCE
            }) {
                return None;
            }
        }
    }
    let rim_neighbors = selected_rim_neighbors(&data.original_edges, selected);
    for (k, edge) in data
        .original_edges
        .iter()
        .enumerate()
        .filter(|(_, e)| selected.contains(&e.id()))
    {
        let (lo, hi) = bounds(edge)?;
        let d = distances.get(&edge.id())?;
        let reach = [edge.front(), edge.back()]
            .iter()
            .filter_map(|v| corner_extent.get(&v.id()))
            .copied()
            .fold(d[0].max(d[1]), f64::max);
        // The complete bilinear footprint lies in its control-point hull. Twice
        // each endpoint reach also excludes two opposing changed footprints.
        let margin = 2. * reach;
        if !margin.is_finite() || margin <= 0. {
            return None;
        }
        for (j, remote) in data.original_edges.iter().enumerate() {
            if remote.id() == edge.id()
                || [edge.front(), edge.back()]
                    .iter()
                    .any(|v| *v == remote.front() || *v == remote.back())
            {
                continue;
            }
            let local_rim = rim_neighbors
                .iter()
                .any(|neighbors| neighbors.contains(&k) && neighbors.contains(&j));
            if local_rim {
                continue;
            }
            let (a, b) = bounds(remote)?;
            if (0..3).all(|i| a[i] <= hi[i] + margin && b[i] >= lo[i] - margin) {
                if !corner_extent.is_empty()
                    && matches!(edge.curve(), Curve::Line(_))
                    && matches!(remote.curve(), Curve::Line(_))
                {
                    // Exact reaches are available only after both endpoint plans
                    // and one/two-edge terminals have been assembled. Miter checks
                    // their complete hull capsules before constructing any faces.
                    continue;
                }
                return None;
            }
        }
    }
    Some(())
}

fn touches_tangent_junction(
    data: &super::planar::Neighborhood,
    selected: &HashSet<EdgeID>,
) -> bool {
    if data.cylinders.iter().filter(|c| c.is_some()).count() != 1
        || data.planes.iter().filter(|p| p.is_none()).count() != 1
    {
        return false;
    }
    let cylinder = data.cylinders.iter().position(Option::is_some).unwrap();
    let (origin, axis, radius) = data.cylinders[cylinder].unwrap();
    data.incident.iter().enumerate().any(|(v, incident)| {
        if !incident
            .iter()
            .any(|&k| selected.contains(&data.original_edges[k].id()))
            || !incident.iter().any(|&k| data.sides[k].contains(&cylinder))
        {
            return false;
        }
        let delta = data.vertices[v].point() - origin;
        let radial = (delta - axis * delta.dot(axis)) / radius;
        let faces: HashSet<_> = incident.iter().flat_map(|&k| data.sides[k]).collect();
        faces
            .iter()
            .any(|&f| data.planes[f].is_some_and(|(n, _)| n.cross(axis).magnitude() < 1e-10))
            && faces
                .iter()
                .any(|&f| data.planes[f].is_some_and(|(n, _)| (n - radial).magnitude() < 1e-10))
    })
}

fn curved_network(
    data: super::planar::Neighborhood,
    shell: &Shell,
    selected: &HashSet<EdgeID>,
) -> Option<super::planar::Neighborhood> {
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
    preferences: &[super::ChamferCornerPreference],
    tol: f64,
) -> Result<Shell, Diagnostic> {
    if !preferences.is_empty() {
        let data =
            super::planar::collect_neighborhood(shell, selected, "chamfer_solid_edges", true)?;
        if selected.len() != data.original_edges.len() && touches_tangent_junction(&data, selected)
        {
            tangent_neighborhood(&data, shell, selected, distances, preferences).ok_or_else(
                || {
                    Diagnostic::new(
                        Code::InvalidParameter,
                        "chamfer_solid_edges",
                        "a5_corner_preference_domain",
                    )
                },
            )?;
            return miter(shell, selected, distances, tol, preferences, data);
        }
        let data = curved_network(data, shell, selected).ok_or_else(|| {
            Diagnostic::new(
                Code::InvalidParameter,
                "chamfer_solid_edges",
                "corner_preference_network",
            )
        })?;
        for preference in preferences {
            let vertex = data
                .vertices
                .iter()
                .position(|v| v.id() == preference.vertex)
                .ok_or_else(|| {
                    Diagnostic::new(
                        Code::InvalidParameter,
                        "chamfer_solid_edges",
                        "validate_corner_preference",
                    )
                })?;
            if data.incident[vertex]
                .iter()
                .flat_map(|&k| data.sides[k])
                .any(|f| data.planes[f].is_none())
            {
                return Err(Diagnostic::new(
                    Code::InvalidParameter,
                    "chamfer_solid_edges",
                    "nonplanar_corner_preference",
                ));
            }
        }
        return miter(shell, selected, distances, tol, preferences, data);
    }
    if let Some(result) = super::chamfer_extrusion::construct(shell, selected, distances, tol) {
        return Ok(result);
    }
    if let Some(result) = quarter_cylinder_corner(shell, selected, distances, tol) {
        return result;
    }
    let mut data =
        super::planar::collect_neighborhood(shell, selected, "chamfer_solid_edges", false)?;
    // These curved paths require exactly one nonplanar face: a positive cylinder.
    // All other analytic support fields therefore remain the uncurved defaults.
    let cylinder = data.planes.iter().position(Option::is_none).and_then(|f| {
        if data.planes[f + 1..].iter().any(Option::is_none) {
            return None;
        }
        match shell[f].oriented_surface().elementary() {
            Some((
                Elementary::Cylinder {
                    origin,
                    axis,
                    radius,
                },
                true,
            )) => Some((f, (origin, axis, radius))),
            _ => None,
        }
    });
    if let Some((f, support)) = cylinder {
        data.cylinders[f] = Some(support);
        if tangent_neighborhood(&data, shell, selected, distances, &[]).is_some() {
            return miter(shell, selected, distances, tol, &[], data);
        }
        if touches_tangent_junction(&data, selected) {
            if data.incident.iter().enumerate().any(|(v, incident)| {
                incident.len() == 3
                    && incident
                        .iter()
                        .all(|&k| selected.contains(&data.original_edges[k].id()))
                    && incident
                        .iter()
                        .all(|&k| data.sides[k].iter().all(|&f| data.planes[f].is_some()))
                    && planar_corner_plan(&data, v, distances, None).is_none()
            }) {
                return Err(Diagnostic::new(
                    Code::UnsupportedGeometry,
                    "chamfer_solid_edges",
                    "certify_planar_corner",
                ));
            }
            return Err(Diagnostic::new(
                Code::InvalidParameter,
                "chamfer_solid_edges",
                "partial_tangent_domain",
            ));
        }
        data.cylinders[f] = None;
    }
    match super::planar::validate_neighborhood(shell, selected, "chamfer_solid_edges", false, &data)
    {
        Ok(()) => miter(shell, selected, distances, tol, &[], data),
        Err(original) => {
            if let Some((f, support)) = cylinder {
                data.cylinders[f] = Some(support);
            }
            if let Some(data) = curved_network(data, shell, selected) {
                if let Ok(result) = miter(shell, selected, distances, tol, &[], data) {
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
    fn a5_collapse_policy_rejects_uncertain_threshold() {
        let vertex = Vertex::new(Point3::origin());
        let edge = builder::tsweep(&vertex, Vector3::unit_x());
        let face = builder::tsweep(&edge, Vector3::unit_y());
        let solid = builder::tsweep(&face, Vector3::unit_z());
        let shell = &solid.boundaries()[0];
        let selected = shell.edge_iter().map(|e| e.id()).collect();
        let data = neighborhood(shell, &selected, "chamfer_solid_edges", false).unwrap();
        let v = data
            .vertices
            .iter()
            .position(|v| v.point() == Point3::origin())
            .unwrap();
        let before = serde_json::to_value(solid.compress()).unwrap();
        for ratio in [0.29_f64, 0.3 - 1e-9, 0.3, 0.3 + 1e-9, 0.31] {
            let (h, a) = (0.1_f64, 0.05_f64);
            // For the unique z pivot, |C-D|/max(|C-A|,|D-B|)
            // is sqrt(2)*(h-b)/sqrt(h*h+(h-b)*(h-b)+a*a).
            let b = h - ratio * h.hypot(a) / (2. - ratio * ratio).sqrt();
            let mut distances = HashMap::default();
            for &k in &data.incident[v] {
                let e = &data.original_edges[k];
                let vertical = (e.back().point() - e.front().point()).z.abs() > 0.5;
                let mut faces = data.sides[k];
                faces.sort();
                distances.insert(
                    e.id(),
                    faces.map(|f| {
                        if vertical {
                            h
                        } else if data.planes[f].unwrap().0.z.abs() > 0.9 {
                            b
                        } else {
                            a
                        }
                    }),
                );
            }
            let plan = planar_corner_plan(&data, v, &distances, None);
            if (ratio - 0.3).abs() < 1e-8 {
                assert!(plan.is_none(), "uncertain ratio {ratio}");
            } else {
                assert_eq!(plan.unwrap().polygon.len(), if ratio < 0.3 { 3 } else { 4 });
            }
            assert_eq!(before, serde_json::to_value(solid.compress()).unwrap());
        }
    }

    #[test]
    fn a5_patch_graph_rejects_folds_and_collapses() {
        let patch =
            |points: [[f64; 3]; 4]| points.map(|p| Vertex::new(Point3::new(p[0], p[1], p[2])));
        for mut polygon in [
            patch([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.5], [0., 1., 0.]]),
            patch([[0., 1., 0.], [1., 1., 0.5], [1., 0., 0.], [0., 0., 0.]]),
        ] {
            assert!(regular_planar_patch(
                &mut polygon,
                Point3::origin(),
                Vector3::unit_z()
            ));
        }
        for mut polygon in [
            patch([[0., 0., 0.], [1., 1., 0.], [1., 0., 0.], [0., 1., 0.]]),
            patch([[0., 0., 0.], [1., 0., 0.], [0.2, 0.2, 1.], [0., 1., 0.]]),
            patch([[0., 0., 0.], [1., 0., 0.], [1., 1e-12, 0.], [0., 1., 0.]]),
            patch([[0., 0., 0.], [1., 0., 0.], [1., 1., f64::NAN], [0., 1., 0.]]),
        ] {
            assert!(!regular_planar_patch(
                &mut polygon,
                Point3::origin(),
                Vector3::unit_z()
            ));
        }
    }

    #[test]
    fn a5_footprint_separation_handles_parallel_and_ill_conditioned_lines() {
        let p = |x, y, z| Point3::new(x, y, z);
        assert!(separated_segments(
            p(0., 0., 0.),
            p(1., 0., 0.),
            p(0., 1., 0.),
            p(1., 1., 0.),
            0.9
        ));
        assert!(!separated_segments(
            p(0., 0., 0.),
            p(1., 0., 0.),
            p(0., 1., 0.),
            p(1., 1., 0.),
            1.
        ));
        assert!(!separated_segments(
            p(-1e6, 0., 0.),
            p(1e6, 0., 0.),
            p(-1e6, -1., 0.),
            p(1e6, 1., 0.),
            0.1
        ));
        assert!(!separated_segments(
            p(0., 0., 0.),
            p(1., 0., 0.),
            p(0.5, -1., 0.),
            p(0.5, 1., 0.),
            0.1
        ));
        assert!(separated_segments(
            p(0., 0., 0.),
            p(1., 0., 0.),
            p(0.5, -1., 1.),
            p(0.5, 1., 1.),
            0.9
        ));
    }

    #[test]
    fn unequal_triangle_relations_do_not_widen_with_translation() {
        let v = Vertex::new(Point3::origin());
        let edge = builder::tsweep(&v, Vector3::unit_x());
        let face = builder::tsweep(&edge, Vector3::unit_y());
        let solid = builder::tsweep(&face, Vector3::unit_z());
        let shell = &solid.boundaries()[0];
        let selected = shell.edge_iter().map(|e| e.id()).collect();
        let data = neighborhood(shell, &selected, "chamfer_solid_edges", false).unwrap();
        let chosen = &data.incident[data
            .vertices
            .iter()
            .position(|v| v.point() == Point3::origin())
            .unwrap()];
        for shift in [0., 4., 1e12] {
            for b in [0.2, 0.199] {
                let vertex = Point3::new(shift, shift, shift);
                let mut bevels = vec![None; data.original_edges.len()];
                for &k in chosen {
                    let direction = data.original_edges[k].back().point()
                        - data.original_edges[k].front().point();
                    let n = if direction.x.abs() > 0.5 {
                        Vector3::new(0., 10., 1. / b)
                    } else if direction.y.abs() > 0.5 {
                        Vector3::new(10., 0., 5.)
                    } else {
                        Vector3::new(10., 5., 0.)
                    };
                    bevels[k] = Some((n, n.dot(vertex.to_vec()) + 1.));
                }
                assert_eq!(
                    unequal_triangle_relations(&data, chosen, &bevels, vertex),
                    b == 0.2,
                    "translation {shift}, axis intercept {b}"
                );
            }
        }
    }

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

#[cfg(test)]
mod rim_topology_tests {
    use super::*;
    thread_local! {
        pub(super) static QUERIES: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
    }

    #[test]
    fn selected_rim_neighbors_preserve_identity_and_local_exemption() {
        let vertices: Vec<_> = (0..6)
            .map(|i| Vertex::new(Point3::new(i as f64, 0., 0.)))
            .collect();
        let line =
            |a: &Vertex, b: &Vertex| Edge::new(a, b, Curve::Line(Line(a.point(), b.point())));
        let arc = |a, b| {
            Edge::new(
                &vertices[a],
                &vertices[b],
                Curve::Conic(Processor::new(TrimmedCurve::new(
                    UnitCircle::<Point3>::new(),
                    (0., 1.),
                ))),
            )
        };
        // Two disjoint edges can be local through a selected rim; equal points
        // with distinct vertex identities must not establish that adjacency.
        let coincident = Vertex::new(vertices[0].point());
        let original = [
            arc(0, 1),
            line(&vertices[0], &vertices[2]),
            line(&vertices[1], &vertices[3]),
            line(&coincident, &vertices[4]),
            arc(3, 4),
            line(&vertices[4], &vertices[5]),
        ];
        for reversed in [false, true] {
            let edges: Vec<_> = original
                .iter()
                .map(|e| if reversed { e.inverse() } else { e.clone() })
                .collect();
            for mask in 0..1 << edges.len() {
                let selected: HashSet<_> = edges
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, e)| e.id())
                    .collect();
                QUERIES.with(|q| q.set((0, 0)));
                let neighbors = selected_rim_neighbors(&edges, &selected);
                let expected_queries = (edges.len(), neighbors.len() * edges.len());
                QUERIES.with(|q| assert_eq!(q.get(), expected_queries));
                for (i, edge) in edges.iter().enumerate() {
                    for (j, remote) in edges.iter().enumerate() {
                        let reference = edges.iter().any(|rim| {
                            matches!(rim.curve(), Curve::Conic(_))
                                && selected.contains(&rim.id())
                                && [rim.front(), rim.back()]
                                    .iter()
                                    .any(|v| *v == edge.front() || *v == edge.back())
                                && [rim.front(), rim.back()]
                                    .iter()
                                    .any(|v| *v == remote.front() || *v == remote.back())
                        });
                        assert_eq!(
                            neighbors.iter().any(|n| n.contains(&i) && n.contains(&j)),
                            reference,
                            "reversed={reversed}, mask={mask}, pair=({i},{j})"
                        );
                    }
                }
                QUERIES.with(|q| assert_eq!(q.get(), expected_queries));
                if mask == 1 {
                    assert_eq!(neighbors, vec![vec![0, 1, 2]]);
                }
            }
        }
    }
}
