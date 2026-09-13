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
    let planes = [a, b].map(|f| match shell[f].oriented_surface() {
        Surface::Plane(p) => Some(p),
        _ => None,
    });
    let [Some(pa), Some(pb)] = planes else {
        return Err(error(Code::NonPlanarFace));
    };
    let edge = shell[a].edge_iter().find(|e| e.id() == id).unwrap();
    let (start, finish) = (edge.front(), edge.back());
    let delta = finish.point() - start.point();
    let length = delta.magnitude();
    if length <= TOLERANCE {
        return Err(error(Code::DegenerateCurve));
    }
    let axis = delta / length;
    let n = pa.normal().normalize();
    let m = pb.normal().normalize();
    let sense = n.cross(m).dot(axis).signum();
    if (1. + n.dot(m)).abs() <= TOLERANCE || n.cross(m).magnitude() <= TOLERANCE {
        return Err(error(Code::UnsupportedGeometry));
    }
    let (p, q, center) = match blend {
        Blend::Chamfer([a, b]) => (
            start.point() + a * n.cross(axis),
            start.point() - b * m.cross(axis),
            None,
        ),
        Blend::Fillet(r) => {
            let c = start.point() - sense * r * (n + m) / (1. + n.dot(m));
            (c + sense * r * n, c + sense * r * m, Some(c))
        }
    };
    if !p.to_vec().magnitude2().is_finite()
        || !q.to_vec().magnitude2().is_finite()
        || p.distance(q) <= TOLERANCE
    {
        return Err(error(Code::OutsideNeighbour));
    }
    let mut contact: Vec<[Vertex; 2]> = Vec::new();
    let mut end_faces = Vec::new();
    let mut replacements = HashMap::default();
    let mut range = (0_f64, length);
    for vertex in [start, finish] {
        let ends: Vec<_> = shell
            .iter()
            .enumerate()
            .filter(|(f, face)| {
                *f != a && *f != b && face.vertex_iter().any(|v| v.id() == vertex.id())
            })
            .map(|(f, _)| f)
            .collect();
        let [end] = *ends.as_slice() else {
            return Err(error(Code::UnsupportedTopology));
        };
        let surface = shell[end].oriented_surface();
        let hint = surface
            .search_parameter(vertex.point(), None, 100)
            .ok_or_else(|| error(Code::UnsupportedGeometry))?;
        let mut points = Vec::new();
        for (side, origin) in [(a, p), (b, q)] {
            let line = Line(origin, origin + axis);
            let (_, t) = algo::surface::search_intersection_parameter(
                &surface,
                hint,
                &line,
                (vertex.point() - start.point()).dot(axis),
                100,
            )
            .ok_or_else(|| error(Code::OutsideNeighbour))?;
            if !t.is_finite() {
                return Err(error(Code::OutsideNeighbour));
            }
            range.0 = range.0.min(t);
            range.1 = range.1.max(t);
            let point = Vertex::new(line.subs(t));
            let adjacent: Vec<_> = shell[side]
                .edge_iter()
                .filter(|e| e.id() != id && (e.front() == vertex || e.back() == vertex))
                .collect();
            let [old] = adjacent.as_slice() else {
                return Err(error(Code::UnsupportedTopology));
            };
            let curve = old.curve();
            let (t0, t1) = curve.range_tuple();
            let absolute = old.absolute_clone();
            let front = absolute.front() == vertex;
            let parameter = curve.search_parameter(point.point(), if front { t0 } else { t1 }, 100);
            let piece = parameter
                .and_then(|t| absolute.cut_with_parameter(&point, t))
                .map(|(a, b)| if front { b } else { a });
            let piece = if let Some(piece) = piece {
                piece
            } else {
                // A new contact can lie beyond the old endpoint on a cylindrical ruling.
                let other = if front {
                    absolute.back()
                } else {
                    absolute.front()
                };
                let direction = vertex.point() - other.point();
                let straight = matches!(curve, Curve::Line(_))
                    || matches!(surface.elementary(),Some((Elementary::Cylinder{axis,..},_)) if axis.cross(direction.normalize()).magnitude()<=TOLERANCE);
                if !straight
                    || (point.point() - other.point()).dot(direction.normalize()) <= TOLERANCE
                    || (point.point() - other.point())
                        .cross(direction.normalize())
                        .magnitude()
                        > TOLERANCE
                {
                    return Err(error(Code::OutsideNeighbour));
                }
                if front {
                    builder::line(&point, other)
                } else {
                    builder::line(other, &point)
                }
            };
            replacements.insert(old.id(), piece);
            points.push(point);
        }
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
        let wall = shell[end_faces[end]].oriented_surface();
        let [v0, v1] = &contact[end];
        let direction = v1.point() - v0.point();
        let parameters = |v: &Vertex| {
            surface
                .search_parameter(v.point(), None, 100)
                .ok_or_else(|| error(Code::BlendConstructionFailed))
        };
        let [uv0, uv1] = [parameters(v0)?, parameters(v1)?];
        let tangent = |v: &Vertex, (u, w): (f64, f64)| {
            let (a, b) = wall.search_parameter(v.point(), None, 100)?;
            let mut tangent = surface.normal(u, w).cross(wall.normal(a, b));
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
        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
        let leader = edge.curve();
        // The cubic guides intersection searches; the two exact surfaces define the curve.
        edge.set_curve(Curve::IntersectionCurve(IntersectionCurve::new(
            Box::new(wall),
            Box::new(surface.clone()),
            Box::new(leader),
        )));
        connectors.insert((v0.id(), v1.id()), edge.clone());
        connectors.insert((v1.id(), v0.id()), edge.inverse());
        cross.push(edge);
    }
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for boundary in face.boundaries() {
            let mut pieces = Vec::new();
            for old in boundary {
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
    result.push(
        Face::try_new(
            vec![wire![
                lines[0].inverse(),
                cross[0].clone(),
                lines[1].clone(),
                cross[1].inverse()
            ]],
            surface,
        )
        .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
    );
    if result.shell_condition() != ShellCondition::Closed {
        return Err(error(Code::InvalidOutputTopology));
    }
    Ok(result)
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
    ExtrudedCurve::by_extrusion(
        curve.transformed(Matrix4::from_translation(vector * start)),
        vector * (end - start),
    )
    .into()
}
