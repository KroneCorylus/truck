use super::planar::{neighborhood, valid_boundaries};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

#[derive(Clone, Copy)]
struct End {
    center: Point3,
    miter: Option<(Vector3, Point3)>,
}

pub(super) fn fillet(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    radius: f64,
    tol: f64,
) -> Result<Shell, Diagnostic> {
    let error = |code| {
        Diagnostic::new(code, "fillet_solid_edges", "construct_local_fillet")
            .parameter("radius", radius)
    };
    let data = neighborhood(shell, selected, "fillet_solid_edges")?;
    let mut contacts: Vec<[[Vertex; 2]; 2]> = data
        .original_edges
        .iter()
        .map(|e| std::array::from_fn(|_| [e.front().clone(), e.back().clone()]))
        .collect();
    let mut ends: Vec<[Option<End>; 2]> = vec![[None, None]; data.original_edges.len()];
    let sign = |k: usize| {
        let [a, b] = data.sides[k];
        let axis = data.original_edges[k].back().point() - data.original_edges[k].front().point();
        data.planes[a]
            .unwrap()
            .0
            .cross(data.planes[b].unwrap().0)
            .dot(axis)
            .signum()
    };
    let mut spheres = Vec::new();
    let mut tori = Vec::new();
    let mut shared_arcs = HashMap::default();
    for (v, incident) in data.incident.iter().enumerate() {
        let chosen: Vec<_> = incident
            .iter()
            .copied()
            .filter(|&k| selected.contains(&data.original_edges[k].id()))
            .collect();
        if chosen.is_empty() {
            continue;
        }
        let mut faces: Vec<_> = incident.iter().flat_map(|&k| data.sides[k]).collect();
        faces.sort_unstable();
        faces.dedup();
        if faces.len() != 3 {
            return Err(error(Code::UnsupportedGeometry));
        }
        let normal = |f: usize| data.planes[f].unwrap().0;
        let orthogonal = |a: usize, b: usize| normal(a).dot(normal(b)).abs() <= TOLERANCE;
        let vertex = &data.vertices[v];
        let end_index = |k: usize| usize::from(data.original_edges[k].front() != vertex);
        let mut contact = |k: usize, f: usize, point: &Vertex| {
            contacts[k][usize::from(data.sides[k][0] != f)][end_index(k)] = point.clone();
        };
        let common = |a: usize, b: usize| {
            data.sides[a]
                .into_iter()
                .find(|f| data.sides[b].contains(f))
                .ok_or_else(|| error(Code::UnsupportedTopology))
        };
        let concave: Vec<_> = chosen.iter().copied().filter(|&k| sign(k) < 0.).collect();
        if chosen.len() >= 2 && concave.len() == 1 {
            let c = concave[0];
            let a = *chosen.iter().find(|&&k| k != c).unwrap();
            let b = *incident.iter().find(|&&k| k != a && k != c).unwrap();
            if sign(b) < 0. {
                return Err(error(Code::UnsupportedGeometry));
            }
            let top = common(a, b)?;
            let f = common(a, c)?;
            let g = common(b, c)?;
            // The rounds of the top rim sweep a torus around the concave round, which needs
            // the top face perpendicular to both walls; the walls may meet at any angle.
            if !orthogonal(top, f) || !orthogonal(top, g) {
                return Err(error(Code::UnsupportedGeometry));
            }
            let [n, m, l] = [top, f, g].map(normal);
            let plane = |f: usize, offset: f64| (normal(f), data.planes[f].unwrap().1 + offset);
            let center = solve([plane(top, -radius), plane(f, radius), plane(g, radius)])
                .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let root_a = Vertex::new(center - radius * m);
            let root_b = Vertex::new(center - radius * l);
            let top_a = Vertex::new(root_a.point() + radius * (n - m));
            let pair = chosen.len() == 2;
            // Along the wall of an unselected edge, the direction away from the other wall.
            let along = (m.dot(l) * l - m).normalize();
            if pair && m.dot(l) >= 0.5 - TOLERANCE {
                // The torus would meet the wall before it leaves the other wall.
                return Err(error(Code::UnsupportedGeometry));
            }
            let top_b = Vertex::new(if pair {
                center + radius * (n - l + 3_f64.sqrt() * along)
            } else {
                root_b.point() + radius * (n - l)
            });
            for (k, face, p) in [
                (a, top, &top_a),
                (a, f, &root_a),
                (b, top, &top_b),
                (b, g, if pair { &top_b } else { &root_b }),
                (c, f, &root_a),
                (c, g, &root_b),
            ] {
                contact(k, face, p);
            }
            for (k, center) in [
                (a, root_a.point() - radius * m),
                (b, root_b.point() - radius * l),
                (c, center),
            ] {
                if chosen.contains(&k) {
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                }
            }
            let cut = if pair {
                // The same torus serves two- and three-edge corners. An unselected
                // third edge leaves a planar wall, so trim the torus exactly against it.
                let section = circle(&top_a, &root_a, root_a.point() - radius * m).curve();
                let mut surface: Surface =
                    Processor::new(RevolutedCurve::by_revolution(section, center, n)).into();
                if surface.normal(0.5, 0.5).dot(n) < 0. {
                    surface.invert();
                }
                let wall = shell[g].oriented_surface();
                let uv0 = wall
                    .search_parameter(root_b.point(), None, 100)
                    .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                let uv1 = wall
                    .search_parameter(top_b.point(), None, 100)
                    .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                let edge: Edge = super::create_pcurve_edge(
                    (&root_b, uv0, n + along),
                    (&top_b, uv1, along),
                    wall.clone(),
                )
                .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                edge.set_curve(Curve::IntersectionCurve(IntersectionCurve::new(
                    Box::new(wall),
                    Box::new(surface.clone()),
                    Box::new(edge.curve()),
                )));
                cache_arc(&mut shared_arcs, &edge);
                Some((edge, surface))
            } else {
                None
            };
            let arc = circle(&top_a, &top_b, center + radius * n);
            cache_arc(&mut shared_arcs, &arc);
            tori.push((v, a, center, n, arc, cut));
            continue;
        }
        if !concave.is_empty() && concave.len() != chosen.len() {
            return Err(error(Code::UnsupportedGeometry));
        }
        let s = sign(chosen[0]);
        let touched: HashSet<_> = chosen.iter().flat_map(|&k| data.sides[k]).collect();
        let center = solve([faces[0], faces[1], faces[2]].map(|f| {
            let (n, d) = data.planes[f].unwrap();
            (n, d - if touched.contains(&f) { s * radius } else { 0. })
        }))
        .ok_or_else(|| error(Code::UnsupportedGeometry))?;
        // A single round ends on the plane of the remaining face, an ellipse when that plane
        // is oblique to the edge.
        let termination = match chosen[..] {
            [k] => {
                let end = *faces.iter().find(|f| !touched.contains(f)).unwrap();
                let edge = &data.original_edges[k];
                let axis = (edge.back().point() - edge.front().point()).normalize();
                (normal(end).cross(axis).magnitude() > TOLERANCE).then_some((normal(end), axis))
            }
            _ => None,
        };
        let mut face_contacts: HashMap<_, _> = touched
            .iter()
            .map(|&f| {
                let mut point = center + s * radius * normal(f);
                if let Some((n, axis)) = termination {
                    point -= axis * (n.dot(point - vertex.point()) / n.dot(axis));
                }
                (f, Vertex::new(point))
            })
            .collect();
        let miter = if chosen.len() == 2 {
            let shared = common(chosen[0], chosen[1])?;
            if faces.iter().any(|&f| f != shared && !orthogonal(f, shared)) {
                return Err(error(Code::UnsupportedGeometry));
            }
            let tip = Vertex::new(vertex.point() - s * radius * data.planes[shared].unwrap().0);
            for &f in &faces {
                if f != shared {
                    face_contacts.insert(f, tip.clone());
                }
            }
            let away = |k: usize| {
                let e = &data.original_edges[k];
                (if e.front() == vertex {
                    e.back().point() - vertex.point()
                } else {
                    e.front().point() - vertex.point()
                })
                .normalize()
            };
            Some((
                (away(chosen[0]) - away(chosen[1])).normalize(),
                vertex.point(),
            ))
        } else {
            termination.map(|(n, _)| (n, vertex.point()))
        };
        for &k in incident {
            let [a, b] = data.sides[k];
            if chosen.contains(&k) {
                for f in [a, b] {
                    contact(k, f, &face_contacts[&f]);
                }
                ends[k][end_index(k)] = Some(End { center, miter });
            } else {
                let p = face_contacts
                    .get(&a)
                    .or_else(|| face_contacts.get(&b))
                    .ok_or_else(|| error(Code::UnsupportedTopology))?;
                for f in [a, b] {
                    contact(k, f, p);
                }
            }
        }
        if chosen.len() == 3 {
            spheres.push((v, center, s, faces));
        }
    }
    let mut trims = Vec::new();
    let mut arcs: Vec<Option<[Edge; 2]>> = vec![None; data.original_edges.len()];
    let mut generated = Vec::new();
    for (k, edge) in data.original_edges.iter().enumerate() {
        let axis = edge.back().point() - edge.front().point();
        let make_line = |a: &Vertex, b: &Vertex| {
            if a == edge.front() && b == edge.back() {
                return Ok(edge.clone());
            }
            if (b.point() - a.point()).dot(axis.normalize()) <= TOLERANCE {
                return Err(error(Code::OutsideNeighbour));
            }
            Ok(builder::line(a, b))
        };
        let [[a0, a1], [b0, b1]] = contacts[k].clone();
        let line_a = make_line(&a0, &a1)?;
        if !selected.contains(&edge.id()) {
            trims.push([line_a.clone(), line_a]);
            continue;
        }
        let line_b = make_line(&b0, &b1)?;
        let [a, b] = data.sides[k];
        let [n, m] = [a, b].map(|f| data.planes[f].unwrap().0);
        let s = sign(k);
        let arc = |end: End, p: &Vertex, q: &Vertex| -> Edge {
            let from = end.center + s * radius * n;
            let to = end.center + s * radius * m;
            let base = circle(&Vertex::new(from), &Vertex::new(to), end.center);
            let curve = if let Some((normal, origin)) = end.miter {
                let mut curve = NurbsCurve::new(base.curve().lift_up());
                let direction = axis.normalize();
                for p in curve.control_points_mut() {
                    let distance =
                        normal.dot(p.truncate() - origin.to_vec() * p.w) / normal.dot(direction);
                    *p -= (direction * distance).extend(0.);
                }
                Curve::NurbsCurve(curve)
            } else {
                base.curve()
            };
            Edge::new(p, q, curve)
        };
        let [e0, e1] = ends[k].map(|e| e.unwrap());
        let arc0 = reuse_arc(&mut shared_arcs, arc(e0, &a0, &b0));
        let arc1 = reuse_arc(&mut shared_arcs, arc(e1, &a1, &b1));
        let mut surface: Surface = if e0.miter.is_none() && e1.miter.is_none() {
            ExtrudedCurve::by_extrusion(arc0.curve(), e1.center - e0.center).into()
        } else {
            let c0 = arc0.oriented_curve().lift_up();
            let c1 = arc1.oriented_curve().lift_up();
            NurbsSurface::new(BSplineSurface::homotopy(c0, c1)).into()
        };
        if surface.normal(0.5, 0.5).dot(n + m) < 0. {
            surface.invert();
        }
        generated.push(
            Face::try_new(
                vec![wire![
                    line_a.inverse(),
                    arc0.clone(),
                    line_b.clone(),
                    arc1.inverse()
                ]],
                surface,
            )
            .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
        trims.push([line_a, line_b]);
        arcs[k] = Some([arc0, arc1]);
    }
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for boundary in face.boundaries() {
            let lines: Vec<_> = boundary
                .iter()
                .map(|e| {
                    let k = data.edge_index[&e.id()];
                    let line = trims[k][usize::from(data.sides[k][0] != f)].clone();
                    if e.orientation() {
                        line
                    } else {
                        line.inverse()
                    }
                })
                .collect();
            let mut wire = Wire::new();
            for i in 0..lines.len() {
                wire.push_back(lines[i].clone());
                let (a, b) = (lines[i].back(), lines[(i + 1) % lines.len()].front());
                if a != b {
                    wire.push_back(
                        shared_arcs
                            .get(&(a.id(), b.id()))
                            .ok_or_else(|| error(Code::BlendConstructionFailed))?
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
        let normal = data.planes[f].ok_or_else(|| error(Code::NonPlanarFace))?.0;
        if !valid_boundaries(face, &boundaries, normal, tol) {
            return Err(error(Code::OutsideNeighbour).face(f));
        }
        result.push(
            Face::try_new(boundaries, face.oriented_surface())
                .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    result.extend(generated);
    let corner_arcs = |v: usize| -> Vec<Edge> {
        data.incident[v]
            .iter()
            .filter(|&&k| arcs[k].is_some())
            .map(|&k| {
                let end = usize::from(data.original_edges[k].front() != &data.vertices[v]);
                let arc = &arcs[k].as_ref().unwrap()[end];
                if end == 0 {
                    arc.inverse()
                } else {
                    arc.clone()
                }
            })
            .collect()
    };
    for (v, a, center, normal, top_arc, cut) in tori {
        let end = usize::from(data.original_edges[a].front() != &data.vertices[v]);
        let section = arcs[a].as_ref().unwrap()[end].oriented_curve();
        let mut surface: Surface =
            Processor::new(RevolutedCurve::by_revolution(section, center, normal)).into();
        if surface.normal(0.5, 0.5).dot(normal) < 0. {
            surface.invert();
        }
        let mut boundary = corner_arcs(v);
        boundary.push(top_arc);
        if let Some((edge, exact_surface)) = cut {
            boundary.push(edge);
            surface = exact_surface;
        }
        result.push(
            Face::try_new(vec![order_wire(boundary)?], surface)
                .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    for (v, center, s, faces) in spheres {
        let normals: Vec<_> = faces
            .iter()
            .map(|&f| s * data.planes[f].unwrap().0)
            .collect();
        let y = (data.vertices[v].point() - center).normalize();
        let z = normals[0].cross(y).normalize();
        let x = y.cross(z);
        let transform = Matrix4::from_cols(
            x.extend(0.),
            y.extend(0.),
            z.extend(0.),
            center.to_vec().extend(1.),
        );
        let mut surface = Surface::Sphere(
            Processor::new(Sphere::new(Point3::origin(), radius)).transformed(transform),
        );
        if s < 0. {
            surface.invert();
        }
        result.push(
            Face::try_new(vec![order_wire(corner_arcs(v))?], surface)
                .map_err(|e| error(Code::InvalidOutputTopology).with_coded_source(e))?,
        );
    }
    if result.shell_condition() != ShellCondition::Closed {
        return Err(error(Code::InvalidOutputTopology));
    }
    Ok(result)
}
/// The common point of three planes `n · p = d`.
fn solve(planes: [(Vector3, f64); 3]) -> Option<Point3> {
    let [(a, d), (b, e), (c, f)] = planes;
    let inverse = Matrix3::from_cols(a, b, c).transpose().invert()?;
    Some(Point3::from_vec(inverse * Vector3::new(d, e, f)))
}
fn circle(a: &Vertex, b: &Vertex, center: Point3) -> Edge {
    let u = a.point() - center;
    let v = b.point() - center;
    builder::circle_arc(a, b, center + (u + v).normalize() * u.magnitude())
}
fn cache_arc(cache: &mut HashMap<(VertexID, VertexID), Edge>, edge: &Edge) {
    cache.insert((edge.front().id(), edge.back().id()), edge.clone());
    cache.insert((edge.back().id(), edge.front().id()), edge.inverse());
}
fn reuse_arc(cache: &mut HashMap<(VertexID, VertexID), Edge>, edge: Edge) -> Edge {
    if let Some(found) = cache.get(&(edge.front().id(), edge.back().id())) {
        return found.clone();
    }
    cache_arc(cache, &edge);
    edge
}
fn order_wire(mut edges: Vec<Edge>) -> Result<Wire, Diagnostic> {
    let error = || {
        Diagnostic::new(
            Code::InvalidOutputTopology,
            "fillet_solid_edges",
            "join_corner",
        )
    };
    let mut wire = Wire::from(vec![edges.remove(0)]);
    while !edges.is_empty() {
        let last = wire.back_vertex().unwrap().clone();
        let i = edges
            .iter()
            .position(|e| e.front() == &last || e.back() == &last)
            .ok_or_else(error)?;
        let edge = edges.remove(i);
        wire.push_back(if edge.front() == &last {
            edge
        } else {
            edge.inverse()
        });
    }
    if !wire.is_closed() {
        return Err(error());
    }
    Ok(wire)
}
