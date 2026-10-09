use super::planar::{sample_wire, valid_boundaries};
use rustc_hash::FxHashMap as HashMap;
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

#[derive(Clone, Copy)]
pub(super) enum Blend {
    Fillet(f64),
    Chamfer([f64; 2]),
}

pub(super) fn blend_edges(
    shell: &Shell,
    selected: &[(EdgeID, Blend)],
    tol: f64,
) -> Result<Shell, Diagnostic> {
    let error = |code| Diagnostic::new(code, "blend", "intersect_straight_blends");
    let mut work = Vec::new();
    for &(id, kind) in selected {
        let faces: Vec<_> = shell
            .iter()
            .enumerate()
            .filter(|(_, f)| f.edge_iter().any(|e| e.id() == id))
            .map(|(i, _)| i)
            .collect();
        let [a, b] = *faces.as_slice() else {
            return Err(error(Code::UnsupportedTopology));
        };
        let (index, edge) = shell[a]
            .edge_iter()
            .enumerate()
            .find(|(_, e)| e.id() == id)
            .unwrap();
        let axis = (edge.back().point() - edge.front().point()).normalize();
        if !edge
            .curve()
            .parameter_division(edge.curve().range_tuple(), tol)
            .1
            .iter()
            .all(|p| (*p - edge.front().point()).cross(axis).magnitude() <= TOLERANCE)
        {
            return Err(error(Code::UnsupportedGeometry));
        }
        let curved = [a, b]
            .iter()
            .any(|&i| !matches!(shell[i].oriented_surface(), Surface::Plane(_)));
        work.push((
            !curved,
            a,
            b,
            index,
            edge.front().point() + (edge.back().point() - edge.front().point()) / 2.,
            axis,
            kind,
        ));
    }
    work.sort_by_key(|&(planar, a, b, index, _, _, _)| (planar, a, b, index));
    let mut result = shell.clone();
    for (_, a, b, _, point, axis, kind) in work {
        let candidates: Vec<_> = result[a]
            .edge_iter()
            .filter(|e| {
                result[b].edge_iter().any(|other| e.id() == other.id())
                    && [e.front().point(), e.back().point()]
                        .iter()
                        .all(|p| (*p - point).cross(axis).magnitude() <= TOLERANCE)
            })
            .collect();
        let edge = candidates
            .into_iter()
            .min_by(|a, b| {
                let distance = |e: &Edge| {
                    let p = e.front().point();
                    let d = e.back().point() - p;
                    let t = ((point - p).dot(d) / d.magnitude2()).clamp(0., 1.);
                    (p + t * d).distance2(point)
                };
                distance(a).total_cmp(&distance(b))
            })
            .ok_or_else(|| error(Code::UnsupportedTopology))?;
        result = blend(&result, edge.id(), kind, tol)?;
    }
    Ok(result)
}

pub(super) fn blend(
    shell: &Shell,
    id: EdgeID,
    blend: Blend,
    tol: f64,
) -> Result<Shell, Diagnostic> {
    let operation = match blend {
        Blend::Fillet(_) => "fillet_solid_edges",
        Blend::Chamfer(_) => "chamfer_solid_edge",
    };
    let error = |code| Diagnostic::new(code, operation, "intersect_end_faces");
    let sides: Vec<_> = shell
        .iter()
        .enumerate()
        .filter(|(_, f)| f.edge_iter().any(|e| e.id() == id))
        .map(|(f, _)| f)
        .collect();
    let [a, b] = *sides.as_slice() else {
        return Err(error(Code::UnsupportedTopology));
    };
    let edge = shell[a].edge_iter().find(|e| e.id() == id).unwrap();
    let (start, finish) = (edge.front(), edge.back());
    let delta = finish.point() - start.point();
    let length = delta.magnitude();
    if length <= TOLERANCE {
        return Err(error(Code::DegenerateCurve));
    }
    let axis = delta / length;
    let surfaces = [shell[a].oriented_surface(), shell[b].oriented_surface()];
    let ([n, m], [p, q], center) = section(&surfaces, start.point(), axis, blend)
        .ok_or_else(|| error(Code::UnsupportedGeometry))?;
    if !p.to_vec().magnitude2().is_finite()
        || !q.to_vec().magnitude2().is_finite()
        || p.distance(q) <= TOLERANCE
    {
        return Err(error(Code::OutsideNeighbour));
    }
    let mut contact: Vec<[Vertex; 2]> = Vec::new();
    let mut end_faces = Vec::new();
    let mut replacements = HashMap::default();
    let mut removed = rustc_hash::FxHashSet::default();
    let mut transitions = Vec::new();
    let mut range = (0_f64, length);
    for vertex in [start, finish] {
        let (end, last, walk) =
            end_walk(shell, [a, b], id, vertex).ok_or_else(|| error(Code::UnsupportedTopology))?;
        let mut points = Vec::new();
        let mut paths = Vec::new();
        for (side, origin, end) in [(a, p, end), (b, q, last)] {
            let line = Line(origin, origin + axis);
            let (point, path) = trim_contact(
                shell,
                side,
                id,
                vertex,
                end,
                &line,
                tol,
                &mut replacements,
                &mut removed,
            )
            .ok_or_else(|| error(Code::OutsideNeighbour))?;
            let t = (point.point() - origin).dot(axis);
            range.0 = range.0.min(t);
            range.1 = range.1.max(t);
            points.push(point);
            paths.push(path);
        }
        transitions.push((paths, walk));
        contact.push([points[0].clone(), points[1].clone()]);
        end_faces.push(end);
    }
    for (front, back) in contact[0].iter().zip(&contact[1]) {
        if (back.point() - front.point()).dot(axis) <= TOLERANCE {
            return Err(error(Code::OutsideNeighbour));
        }
    }
    let section_start = p + range.0 * axis;
    let section_end = q + range.0 * axis;
    let mut surface: Surface = if let Some(center) = center {
        let center = center + range.0 * axis;
        let middle = center
            + ((section_start - center) + (section_end - center)).normalize()
                * (section_start - center).magnitude();
        let section: Edge = builder::circle_arc(
            &Vertex::new(section_start),
            &Vertex::new(section_end),
            middle,
        );
        ExtrudedCurve::by_extrusion(section.oriented_curve(), axis * (range.1 - range.0)).into()
    } else {
        Plane::new(
            section_start,
            section_end,
            section_start + axis * (range.1 - range.0),
        )
        .into()
    };
    if surface.normal(0.5, 0.5).dot(n + m) < 0. {
        surface.invert();
    }
    let lines = [
        builder::line(&contact[0][0], &contact[1][0]),
        builder::line(&contact[0][1], &contact[1][1]),
    ];
    let mut cross = Vec::new();
    let mut connectors = HashMap::default();
    for end in 0..2 {
        let mut points = vec![contact[end][0].clone()];
        let (paths, walk) = &transitions[end];
        let initial = end_faces[end];
        let mut walls = Vec::new();
        let mut current = paths[0]
            .last()
            .map_or(initial, |step: &(Edge, Vertex, usize)| step.2);
        for (edge, vertex, next) in paths[0]
            .iter()
            .enumerate()
            .rev()
            .map(|(i, (e, v, _))| (e, v, if i == 0 { initial } else { paths[0][i - 1].2 }))
            .chain(walk.iter().map(|(e, v, f)| (e, v, *f)))
            .chain(paths[1].iter().map(|(e, v, f)| (e, v, *f)))
        {
            let direction = edge.back().point() - edge.front().point();
            let curve = edge.curve();
            if !curve
                .parameter_division(curve.range_tuple(), tol)
                .1
                .iter()
                .all(|p| {
                    (*p - edge.front().point())
                        .cross(direction.normalize())
                        .magnitude()
                        <= TOLERANCE
                })
            {
                return Err(error(Code::UnsupportedGeometry));
            }
            let line = Line(vertex.point(), vertex.point() + direction.normalize());
            let position = line_intersection(&surface, &line, vertex.point())
                .ok_or_else(|| error(Code::BlendConstructionFailed))?;
            let other = if edge.front() == vertex {
                edge.back()
            } else {
                edge.front()
            };
            let point = if position.near(&other.point()) {
                removed.insert(edge.id());
                other.clone()
            } else {
                let point = contact
                    .iter()
                    .flatten()
                    .find(|v| v.point().near(&position))
                    .cloned()
                    .or_else(|| shell.vertex_iter().find(|v| v.point().near(&position)))
                    .unwrap_or_else(|| Vertex::new(position));
                let piece = trim_edge(edge, vertex, &point, tol)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?;
                replacements.insert(edge.id(), piece);
                point
            };
            points.push(point);
            walls.push(current);
            current = next;
        }
        points.push(contact[end][1].clone());
        walls.push(current);
        let mut boundary = Wire::new();
        for (vertices, wall) in points.windows(2).zip(walls) {
            let wall = shell[wall].oriented_surface();
            let [v0, v1] = [&vertices[0], &vertices[1]];
            if v0 == v1 {
                continue;
            }
            let direction = v1.point() - v0.point();
            let exact = match (&wall, &surface) {
                (Surface::Plane(_), Surface::Plane(_)) => {
                    Some(Curve::Line(Line(v0.point(), v1.point())))
                }
                (Surface::Plane(plane), _) => super::projected_section::projected_section(
                    &surface,
                    *plane,
                    v0.point(),
                    v1.point(),
                ),
                (_, Surface::Plane(plane)) => super::projected_section::projected_section(
                    &wall,
                    *plane,
                    v0.point(),
                    v1.point(),
                ),
                _ => cylinder_miter(&wall, &surface, [v0.point(), v1.point()]).and_then(|plane| {
                    super::projected_section::projected_section(
                        &surface,
                        plane,
                        v0.point(),
                        v1.point(),
                    )
                }),
            };
            let edge = if let Some(curve) = exact {
                Edge::new(v0, v1, curve)
            } else {
                let parameters = |v: &Vertex| {
                    surface
                        .search_parameter(v.point(), None, 100)
                        .ok_or_else(|| error(Code::BlendConstructionFailed))
                };
                let [uv0, uv1] = [parameters(v0)?, parameters(v1)?];
                let tangent = |v: &Vertex, (u, w): (f64, f64)| {
                    let (a, b) = wall.search_parameter(v.point(), None, 100)?;
                    let normal = surface.normal(u, w).normalize();
                    let mut tangent = normal.cross(wall.normal(a, b));
                    if tangent.so_small() {
                        tangent = direction - normal * direction.dot(normal);
                    }
                    if tangent.dot(direction) < 0. {
                        tangent = -tangent;
                    }
                    (!tangent.so_small()).then_some(tangent)
                };
                let edge: Edge = super::create_pcurve_edge(
                    (
                        v0,
                        uv0,
                        tangent(v0, uv0).ok_or_else(|| error(Code::UnsupportedGeometry))?,
                    ),
                    (
                        v1,
                        uv1,
                        tangent(v1, uv1).ok_or_else(|| error(Code::UnsupportedGeometry))?,
                    ),
                    surface.clone(),
                )
                .ok_or_else(|| {
                    Diagnostic::new(Code::BlendConstructionFailed, operation, "leader")
                })?;
                let leader = edge.curve();
                // The cubic guides intersection searches; the two exact surfaces define the curve.
                edge.set_curve(Curve::IntersectionCurve(IntersectionCurve::new(
                    Box::new(wall),
                    Box::new(surface.clone()),
                    Box::new(leader),
                )));
                edge
            };
            connectors.insert((v0.id(), v1.id()), edge.clone());
            connectors.insert((v1.id(), v0.id()), edge.inverse());
            boundary.push_back(edge);
        }
        cross.push(boundary);
    }
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for boundary in face.boundaries() {
            let mut pieces = Vec::new();
            for old in boundary {
                if removed.contains(&old.id()) {
                    continue;
                }
                let piece = if old.id() == id {
                    if f == a {
                        lines[0].clone()
                    } else {
                        lines[1].inverse()
                    }
                } else if let Some(new) = replacements.get(&old.id()) {
                    if old.orientation() {
                        new.clone()
                    } else {
                        new.inverse()
                    }
                } else {
                    old.clone()
                };
                pieces.push(piece);
            }
            let mut wire = Wire::new();
            for i in 0..pieces.len() {
                wire.push_back(pieces[i].clone());
                let (a, b) = (pieces[i].back(), pieces[(i + 1) % pieces.len()].front());
                if a != b {
                    wire.push_back(
                        connectors
                            .get(&(a.id(), b.id()))
                            .ok_or_else(|| error(Code::InvalidOutputTopology))?
                            .clone(),
                    );
                }
            }
            boundaries.push(wire);
        }
        if boundaries == face.boundaries() {
            result.push(face.clone());
            continue;
        }
        if let Surface::Plane(plane) = face.oriented_surface() {
            if !valid_boundaries(face, &boundaries, plane.normal().normalize(), tol) {
                return Err(error(Code::OutsideNeighbour).face(f));
            }
        }
        result.push(
            Face::try_new(
                boundaries.clone(),
                extend_surface(face.oriented_surface(), &boundaries, tol),
            )
            .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    let mut boundary = Wire::new();
    boundary.push_back(lines[0].inverse());
    boundary.extend(cross[0].clone());
    boundary.push_back(lines[1].clone());
    boundary.extend(cross[1].inverse());
    result.push(
        Face::try_new(
            vec![boundary.clone()],
            extend_surface(surface, &[boundary], tol),
        )
        .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
    );
    if result.shell_condition() != ShellCondition::Closed {
        return Err(error(Code::InvalidOutputTopology));
    }
    Ok(result)
}

#[allow(clippy::type_complexity)]
pub(super) fn end_walk(
    shell: &Shell,
    [a, b]: [usize; 2],
    selected: EdgeID,
    vertex: &Vertex,
) -> Option<(usize, usize, Vec<(Edge, Vertex, usize)>)> {
    let mut previous = shell[a]
        .edge_iter()
        .find(|edge| edge.id() != selected && (edge.front() == vertex || edge.back() == vertex))?;
    let mut current = shell
        .iter()
        .enumerate()
        .find(|(i, face)| *i != a && face.edge_iter().any(|e| e.id() == previous.id()))?
        .0;
    let initial = current;
    let mut walk = Vec::new();
    for _ in 0..shell.len() {
        if current == a || current == b {
            return None;
        }
        let other = shell[current].edge_iter().find(|edge| {
            edge.id() != previous.id() && (edge.front() == vertex || edge.back() == vertex)
        })?;
        let next = shell
            .iter()
            .enumerate()
            .find(|(i, face)| *i != current && face.edge_iter().any(|e| e.id() == other.id()))?
            .0;
        if next == b {
            return Some((initial, current, walk));
        }
        walk.push((other.clone(), vertex.clone(), next));
        previous = other;
        current = next;
    }
    None
}

fn cylinder_miter(a: &Surface, b: &Surface, endpoints: [Point3; 2]) -> Option<Plane> {
    let Elementary::Cylinder {
        origin: p,
        axis: u,
        radius: r,
    } = a.elementary()?.0
    else {
        return None;
    };
    let Elementary::Cylinder {
        origin: q,
        axis: v,
        radius: s,
    } = b.elementary()?.0
    else {
        return None;
    };
    if (r - s).abs() > TOLERANCE {
        return None;
    }
    let dot = u.dot(v);
    let denominator = 1. - dot * dot;
    if denominator <= TOLERANCE {
        return None;
    }
    let delta = q - p;
    let origin = p + u * ((delta.dot(u) - dot * delta.dot(v)) / denominator);
    if (origin - q).cross(v).magnitude() > TOLERANCE {
        return None;
    }
    // Equal-radius cylinders meeting at intersecting axes share two planar ellipse branches.
    let normal = [u + v, u - v]
        .into_iter()
        .map(|n| n.normalize())
        .find(|n| {
            endpoints
                .iter()
                .all(|point| (*point - origin).dot(*n).abs() <= TOLERANCE)
        })?;
    let x = u.cross(normal).normalize();
    Some(Plane::new(origin, origin + x, origin + normal.cross(x)))
}

fn extend_surface(surface: Surface, boundaries: &[Wire], tol: f64) -> Surface {
    // Extend the parameter domain with the boundary, preserving the analytic cylinder.
    if !matches!(surface.elementary(), Some((Elementary::Cylinder { .. }, _))) {
        return surface;
    }
    if let Surface::RevolutedCurve(ref processor) = surface {
        let revolution = processor.entity();
        let Curve::Line(line) = revolution.entity_curve() else {
            return surface;
        };
        let Some(inverse) = processor.transform().invert() else {
            return surface;
        };
        let direction = line.1 - line.0;
        let (mut start, mut end) = (0_f64, 1_f64);
        for point in boundaries.iter().flat_map(|w| sample_wire(w, tol)) {
            let t =
                (inverse.transform_point(point) - line.0).dot(direction) / direction.magnitude2();
            start = start.min(t);
            end = end.max(t);
        }
        if start == 0. && end == 1. {
            return surface;
        }
        return Surface::RevolutedCurve(processor.map_ref(|r| {
            RevolutedCurve::by_revolution(
                Curve::Line(Line(line.0 + start * direction, line.0 + end * direction)),
                r.origin(),
                r.axis(),
            )
        }));
    }
    let Surface::Extruded(ref extrusion) = surface else {
        return surface;
    };
    let vector = extrusion.extruding_vector();
    let curve = extrusion.entity_curve();
    let origin = curve.subs(curve.range_tuple().0);
    let (mut start, mut end) = (0_f64, 1_f64);
    for point in boundaries.iter().flat_map(|w| sample_wire(w, tol)) {
        let t = (point - origin).dot(vector) / vector.magnitude2();
        start = start.min(t);
        end = end.max(t);
    }
    if start == 0. && end == 1. {
        return surface;
    }
    // Boundary samples bound the analytic curve only up to their tessellation tolerance.
    let margin = tol / vector.magnitude();
    start -= margin;
    end += margin;
    ExtrudedCurve::by_extrusion(
        curve.transformed(Matrix4::from_translation(vector * start)),
        vector * (end - start),
    )
    .into()
}

fn section(
    surfaces: &[Surface; 2],
    start: Point3,
    axis: Vector3,
    blend: Blend,
) -> Option<([Vector3; 2], [Point3; 2], Option<Point3>)> {
    let mut normals = [Vector3::zero(); 2];
    let mut cylinders = [None; 2];
    for i in 0..2 {
        let uv = surfaces[i].search_parameter(start, None, 100)?;
        normals[i] = surfaces[i].normal(uv.0, uv.1).normalize();
        match surfaces[i].elementary()?.0 {
            Elementary::Plane(_) => {}
            Elementary::Cylinder {
                origin,
                axis: cylinder_axis,
                radius,
            } => {
                if cylinder_axis.cross(axis).magnitude() > TOLERANCE {
                    return None;
                }
                let center = origin + cylinder_axis * (start - origin).dot(cylinder_axis);
                cylinders[i] = Some((center, radius));
            }
            _ => return None,
        }
    }
    let [n, m] = normals;
    let sense = n.cross(m).dot(axis).signum();
    if (1. + n.dot(m)).abs() <= TOLERANCE || n.cross(m).magnitude() <= TOLERANCE {
        return None;
    }
    if let Blend::Chamfer(distances) = blend {
        let mut contacts = [
            start + distances[0] * n.cross(axis),
            start - distances[1] * m.cross(axis),
        ];
        for i in 0..2 {
            if let Some((center, radius)) = cylinders[i] {
                contacts[i] = center + (contacts[i] - center).normalize() * radius;
            }
        }
        return Some((normals, contacts, None));
    }
    let Blend::Fillet(radius) = blend else {
        unreachable!()
    };
    let center = match cylinders {
        [None, None] => start - sense * radius * (n + m) / (1. + n.dot(m)),
        [Some(_), Some(_)] => return None,
        _ => {
            let i = usize::from(cylinders[0].is_none());
            let (origin, cylinder_radius) = cylinders[i]?;
            let normal = normals[1 - i];
            let radial = start - origin;
            let offset_radius =
                cylinder_radius - sense * radius * normals[i].dot(radial.normalize());
            let height = radial.dot(normal) - sense * radius;
            let lateral = radial - normal * radial.dot(normal);
            let square = offset_radius * offset_radius - height * height;
            if offset_radius <= TOLERANCE || square <= TOLERANCE || lateral.so_small() {
                return None;
            }
            origin + normal * height + lateral.normalize() * square.sqrt()
        }
    };
    let mut contacts = [center + sense * radius * n, center + sense * radius * m];
    for i in 0..2 {
        if let Some((origin, radius)) = cylinders[i] {
            contacts[i] = origin + (center - origin).normalize() * radius;
        }
    }
    Some((normals, contacts, Some(center)))
}

fn trim_edge(old: &Edge, vertex: &Vertex, point: &Vertex, tol: f64) -> Option<Edge> {
    let absolute = old.absolute_clone();
    let curve = absolute.curve();
    let front = absolute.front() == vertex;
    let (t0, t1) = curve.range_tuple();
    if let Some((a, b)) = curve
        .search_parameter(point.point(), if front { t0 } else { t1 }, 100)
        .and_then(|t| absolute.cut_with_parameter(point, t))
    {
        return Some(if front { b } else { a });
    }
    let other = if front {
        absolute.back()
    } else {
        absolute.front()
    };
    let direction = (vertex.point() - other.point()).normalize();
    let straight = curve
        .parameter_division((t0, t1), tol)
        .1
        .iter()
        .all(|p| (*p - other.point()).cross(direction).magnitude() <= TOLERANCE);
    if !straight
        || (point.point() - other.point()).dot(direction) <= TOLERANCE
        || (point.point() - other.point()).cross(direction).magnitude() > TOLERANCE
    {
        return None;
    }
    Some(if front {
        builder::line(point, other)
    } else {
        builder::line(other, point)
    })
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn trim_contact(
    shell: &Shell,
    side: usize,
    selected: EdgeID,
    vertex: &Vertex,
    end: usize,
    line: &Line<Point3>,
    tol: f64,
    replacements: &mut HashMap<EdgeID, Edge>,
    removed: &mut rustc_hash::FxHashSet<EdgeID>,
) -> Option<(Vertex, Vec<(Edge, Vertex, usize)>)> {
    let mut current = vertex.clone();
    let mut previous = selected;
    let mut end = end;
    let mut path = Vec::new();
    for _ in 0..shell[side].edge_iter().count() {
        let adjacent: Vec<_> = shell[side]
            .edge_iter()
            .filter(|e| e.id() != previous && (e.front() == &current || e.back() == &current))
            .collect();
        let [old] = adjacent.as_slice() else {
            return None;
        };
        let surface = shell[end].oriented_surface();
        if let Some(point) = line_intersection(&surface, line, current.point()) {
            let other = if old.front() == &current {
                old.back()
            } else {
                old.front()
            };
            if point.near(&other.point()) {
                removed.insert(old.id());
                return Some((other.clone(), path));
            }
            let point = Vertex::new(point);
            if let Some(piece) = trim_edge(old, &current, &point, tol) {
                replacements.insert(old.id(), piece);
                return Some((point, path));
            }
        }
        removed.insert(old.id());
        let next = if old.front() == &current {
            old.back()
        } else {
            old.front()
        };
        let boundary: Vec<_> = shell[side]
            .edge_iter()
            .filter(|e| e.id() != old.id() && (e.front() == next || e.back() == next))
            .collect();
        let [boundary] = boundary.as_slice() else {
            return None;
        };
        let next_face = shell
            .iter()
            .enumerate()
            .find(|(i, f)| *i != side && f.edge_iter().any(|e| e.id() == boundary.id()))?
            .0;
        let shared: Vec<_> = shell[end]
            .edge_iter()
            .filter(|e| {
                (e.front() == next || e.back() == next)
                    && shell[next_face]
                        .edge_iter()
                        .any(|other| other.id() == e.id())
            })
            .collect();
        let [shared] = shared.as_slice() else {
            return None;
        };
        path.push(((*shared).clone(), next.clone(), next_face));
        current = next.clone();
        previous = old.id();
        end = next_face;
    }
    None
}

fn line_intersection(surface: &Surface, line: &Line<Point3>, near: Point3) -> Option<Point3> {
    let direction = line.1 - line.0;
    match surface.elementary().map(|(s, _)| s) {
        Some(Elementary::Plane(plane)) => {
            let n = plane.normal();
            let denominator = n.dot(direction);
            if denominator.abs() <= TOLERANCE {
                return None;
            }
            Some(line.subs(n.dot(plane.origin() - line.0) / denominator))
        }
        Some(Elementary::Cylinder {
            origin,
            axis,
            radius,
        }) => {
            let radial = line.0 - origin;
            let radial = radial - axis * radial.dot(axis);
            let transverse = direction - axis * direction.dot(axis);
            let a = transverse.magnitude2();
            if a <= TOLERANCE * TOLERANCE {
                return None;
            }
            let b = radial.dot(transverse);
            let c = radial.magnitude2() - radius * radius;
            let discriminant = b * b - a * c;
            if discriminant < -TOLERANCE * a {
                return None;
            }
            let root = if discriminant.abs() <= TOLERANCE * TOLERANCE * a {
                0.
            } else {
                discriminant.max(0.).sqrt()
            };
            let points = [line.subs((-b - root) / a), line.subs((-b + root) / a)];
            Some(if points[0].distance2(near) <= points[1].distance2(near) {
                points[0]
            } else {
                points[1]
            })
        }
        _ => {
            let hint = surface.search_nearest_parameter(near, None, 100)?;
            let t = (near - line.0).dot(direction) / direction.magnitude2();
            let (_, t) = algo::surface::search_intersection_parameter(surface, hint, line, t, 100)?;
            t.is_finite().then(|| line.subs(t))
        }
    }
}
