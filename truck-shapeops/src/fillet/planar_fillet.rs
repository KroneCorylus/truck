use super::planar::{neighborhood, valid_boundaries};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_geometry::prelude::ContactCircle;
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
    let data = neighborhood(shell, selected, "fillet_solid_edges", selected.len() > 1)?;
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
    let mut complex_ends = HashMap::default();
    let mut cone_terminals = HashMap::default();
    let mut curved_terminals = HashSet::default();
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
        let normal = |f: usize| data.planes[f].unwrap().0;
        let orthogonal = |a: usize, b: usize| normal(a).dot(normal(b)).abs() <= TOLERANCE;
        let vertex = &data.vertices[v];
        let end_index = |k: usize| usize::from(data.original_edges[k].front() != vertex);
        let mut contact = |k: usize, f: usize, point: &Vertex| {
            contacts[k][usize::from(data.sides[k][0] != f)][end_index(k)] = point.clone();
        };
        let cone_face = faces.iter().copied().find(|&f| {
            data.cone_supports[f].is_some() && chosen.iter().any(|&k| data.sides[k].contains(&f))
        });
        if let Some(f) = cone_face {
            if faces.len() != 3 || !(chosen.len() == 1 || chosen.len() == 3) {
                return Err(error(Code::UnsupportedTopology));
            }
            let cone = data.cone_supports[f].unwrap();
            let (apex, axis, angle) = cone;
            let planes: Vec<_> = faces.iter().copied().filter(|&j| j != f).collect();
            let [a, b] = [planes[0], planes[1]];
            let pa = data.planes[a].ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let pb = data.planes[b].ok_or_else(|| error(Code::UnsupportedGeometry))?;
            if chosen.len() == 3 {
                let center = cone_planes(
                    cone,
                    radius,
                    [(pa.0, pa.1 - radius), (pb.0, pb.1 - radius)],
                    vertex.point(),
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let normals: HashMap<_, _> = [(a, pa.0), (b, pb.0), (f, cone_normal(cone, center))]
                    .into_iter()
                    .collect();
                let points: HashMap<_, _> = faces
                    .iter()
                    .map(|&j| (j, Vertex::new(center + radius * normals[&j])))
                    .collect();
                for &k in &chosen {
                    for j in data.sides[k] {
                        contact(k, j, &points[&j]);
                    }
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                }
                spheres.push((
                    v,
                    center,
                    1.,
                    faces.iter().map(|j| normals[j]).collect::<Vec<_>>(),
                ));
            } else {
                let k = chosen[0];
                let edge = &data.original_edges[k];
                let plane = *data.sides[k].iter().find(|&&j| j != f).unwrap();
                let terminal = *planes.iter().find(|&&j| j != plane).unwrap();
                let (n, d) = data.planes[plane].unwrap();
                let (m, e) = data.planes[terminal].unwrap();
                let (center, cone_point, plane_point) = if n.cross(axis).magnitude() < TOLERANCE {
                    let center =
                        cone_planes(cone, radius, [(n, d - radius), (m, e)], vertex.point())
                            .ok_or_else(|| error(Code::OutsideNeighbour))?;
                    (
                        center,
                        center + radius * cone_normal(cone, center),
                        center + radius * n,
                    )
                } else {
                    if n.dot(axis).abs() > TOLERANCE || (d - n.dot(apex.to_vec())).abs() > TOLERANCE
                    {
                        return Err(error(Code::UnsupportedGeometry));
                    }
                    let u = cone_lateral(cone, n, edge)
                        .ok_or_else(|| error(Code::UnsupportedGeometry))?;
                    let center = |h: f64| -> Option<Point3> {
                        let rho = h * angle.0.tan() - radius / angle.0.cos();
                        let square = rho * rho - radius * radius;
                        (rho > 0. && square >= 0.)
                            .then(|| apex + axis * h - n * radius + u * square.sqrt())
                    };
                    if m.cross(axis).magnitude() < TOLERANCE {
                        let h = (e - m.dot(apex.to_vec())) / m.dot(axis);
                        let cp = center(h + radius * angle.0.sin())
                            .ok_or_else(|| error(Code::OutsideNeighbour))?;
                        let cq = center(h).ok_or_else(|| error(Code::OutsideNeighbour))?;
                        cone_terminals.insert((k, end_index(k)), terminal);
                        (cq, cp + radius * cone_normal(cone, cp), cq + radius * n)
                    } else if m.dot(axis).abs() < TOLERANCE
                        && (e - m.dot(apex.to_vec())).abs() < TOLERANCE
                        && m.dot(n).abs() < TOLERANCE
                    {
                        let h = (radius + radius / angle.0.cos()) / angle.0.tan();
                        let c = apex + axis * h - n * radius;
                        (c, c + radius * cone_normal(cone, c), c + radius * n)
                    } else {
                        return Err(error(Code::UnsupportedGeometry));
                    }
                };
                let points: HashMap<_, _> = [
                    (f, Vertex::new(cone_point)),
                    (plane, Vertex::new(plane_point)),
                ]
                .into_iter()
                .collect();
                for &j in &[f, plane] {
                    contact(k, j, &points[&j]);
                    let other = *incident
                        .iter()
                        .find(|&&i| i != k && data.sides[i].contains(&j))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    for side in data.sides[other] {
                        contact(other, side, &points[&j]);
                    }
                }
                ends[k][end_index(k)] = Some(End {
                    center,
                    miter: None,
                });
            }
            continue;
        }
        let sphere = faces.iter().copied().find(|&f| {
            data.sphere_supports[f].is_some() && chosen.iter().any(|&k| data.sides[k].contains(&f))
        });
        if let Some(f) = sphere {
            if faces.len() != 3 || !(chosen.len() == 1 || chosen.len() == 3) {
                return Err(error(Code::UnsupportedTopology));
            }
            let (origin, outer) = data.sphere_supports[f].unwrap();
            let inner = outer - radius;
            if inner <= 0. {
                return Err(error(Code::OutsideNeighbour));
            }
            let planes: Vec<_> = faces.iter().copied().filter(|&j| j != f).collect();
            let [a, b] = [planes[0], planes[1]];
            let [pa, pb] =
                [a, b].map(|j| data.planes[j].ok_or_else(|| error(Code::UnsupportedGeometry)));
            let (pa, pb) = (pa?, pb?);
            if chosen.len() == 3 {
                let center = sphere_planes(
                    origin,
                    inner,
                    [(pa.0, pa.1 - radius), (pb.0, pb.1 - radius)],
                    vertex.point(),
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let normals: HashMap<_, _> =
                    [(a, pa.0), (b, pb.0), (f, (center - origin).normalize())]
                        .into_iter()
                        .collect();
                let points: HashMap<_, _> = faces
                    .iter()
                    .map(|&j| (j, Vertex::new(center + radius * normals[&j])))
                    .collect();
                for &k in &chosen {
                    for j in data.sides[k] {
                        contact(k, j, &points[&j]);
                    }
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                }
                spheres.push((
                    v,
                    center,
                    1.,
                    faces.iter().map(|j| normals[j]).collect::<Vec<_>>(),
                ));
            } else {
                let k = chosen[0];
                let edge = &data.original_edges[k];
                let plane = *data.sides[k].iter().find(|&&j| j != f).unwrap();
                let terminal = *planes.iter().find(|&&j| j != plane).unwrap();
                let (n, d) = data.planes[plane].unwrap();
                let terminal_plane = data.planes[terminal].unwrap();
                let h = d - radius - n.dot(origin.to_vec());
                let sphere_point = sphere_planes(
                    origin,
                    outer,
                    [
                        (n, n.dot(origin.to_vec()) + outer / inner * h),
                        terminal_plane,
                    ],
                    vertex.point(),
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let plane_point = sphere_planes(
                    origin + n * radius,
                    inner,
                    [(n, d), terminal_plane],
                    vertex.point(),
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let points: HashMap<_, _> = [
                    (f, Vertex::new(sphere_point)),
                    (plane, Vertex::new(plane_point)),
                ]
                .into_iter()
                .collect();
                for &j in &[f, plane] {
                    contact(k, j, &points[&j]);
                    let other = *incident
                        .iter()
                        .find(|&&i| i != k && data.sides[i].contains(&j))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    for side in data.sides[other] {
                        contact(other, side, &points[&j]);
                    }
                }
                let support = sphere_rim_surface(edge, (origin, outer), (n, d), radius)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let [a, b] = data.sides[k];
                let arc = terminal_cross_arc(
                    &points[&a],
                    &points[&b],
                    &support,
                    &shell[terminal].oriented_surface(),
                )
                .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                cache_arc(&mut shared_arcs, &arc);
                complex_ends.insert((k, end_index(k)), Wire::from(vec![arc]));
                ends[k][end_index(k)] = Some(End {
                    center: origin + (sphere_point - origin) * (inner / outer),
                    miter: None,
                });
            }
            continue;
        }
        let cylinder = faces.iter().copied().find(|&f| {
            data.cylinders[f].is_some() && chosen.iter().any(|&k| data.sides[k].contains(&f))
        });
        if let Some(f) = cylinder {
            if faces.len() != 3 || !(chosen.len() == 1 || chosen.len() == 3) {
                return Err(error(Code::UnsupportedTopology));
            }
            let (origin, axis, outer) = data.cylinders[f].unwrap();
            if outer <= 2. * radius {
                return Err(error(Code::OutsideNeighbour));
            }
            let cap = *faces
                .iter()
                .find(|&&j| {
                    data.planes[j].is_some_and(|(n, _)| n.cross(axis).magnitude() < TOLERANCE)
                })
                .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let wall = *faces.iter().find(|&&j| j != f && j != cap).unwrap();
            let (n, d) = data.planes[cap].unwrap();
            let (m, e) = data.planes[wall].ok_or_else(|| error(Code::UnsupportedGeometry))?;
            if m.dot(axis).abs() > TOLERANCE || (e - m.dot(origin.to_vec())).abs() > TOLERANCE {
                return Err(error(Code::UnsupportedGeometry));
            }
            let touched: HashSet<_> = chosen.iter().flat_map(|&k| data.sides[k]).collect();
            let offset = |f| if touched.contains(&f) { radius } else { 0. };
            let axial = origin + axis * ((d - offset(cap) - n.dot(origin.to_vec())) / n.dot(axis));
            let h = e - offset(wall) - m.dot(origin.to_vec());
            let radial = vertex.point() - origin - axis * (vertex.point() - origin).dot(axis);
            let lateral = radial - m * radial.dot(m);
            let square = (outer - radius).powi(2) - h * h;
            if !square.is_finite() || square <= 0. || lateral.so_small() {
                return Err(error(Code::OutsideNeighbour));
            }
            let center = axial + m * h + lateral.normalize() * square.sqrt();
            let delta = center - origin;
            let cylinder_normal = (delta - axis * delta.dot(axis)).normalize();
            let normal = |j: usize| {
                if j == f {
                    cylinder_normal
                } else {
                    data.planes[j].unwrap().0
                }
            };
            let face_contacts: HashMap<_, _> = touched
                .iter()
                .map(|&j| (j, Vertex::new(center + radius * normal(j))))
                .collect();
            for &k in incident {
                if chosen.contains(&k) {
                    for f in data.sides[k] {
                        contact(k, f, &face_contacts[&f]);
                    }
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                } else {
                    let p = data.sides[k]
                        .iter()
                        .find_map(|f| face_contacts.get(f))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    for f in data.sides[k] {
                        contact(k, f, p);
                    }
                }
            }
            if chosen.len() == 3 {
                spheres.push((
                    v,
                    center,
                    1.,
                    faces.iter().map(|&f| normal(f)).collect::<Vec<_>>(),
                ));
            }
            continue;
        }
        if faces.iter().any(|&f| data.planes[f].is_none()) {
            let [k] = chosen[..] else {
                return Err(error(Code::UnsupportedTopology));
            };
            let [a, b] = data.sides[k];
            if faces.len() != 3 || data.planes[a].is_none() || data.planes[b].is_none() {
                return Err(error(Code::UnsupportedTopology));
            }
            let end = *faces.iter().find(|&&f| f != a && f != b).unwrap();
            let edge = &data.original_edges[k];
            let delta = edge.back().point() - edge.front().point();
            let axis = delta.normalize();
            let s = sign(k);
            let center = solve([
                (normal(a), data.planes[a].unwrap().1 - s * radius),
                (normal(b), data.planes[b].unwrap().1 - s * radius),
                (axis, axis.dot(vertex.point().to_vec())),
            ])
            .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let wall = shell[end].oriented_surface();
            let mut points = Vec::new();
            for side in [a, b] {
                let origin = center + s * radius * normal(side);
                let line = Line(origin, origin + axis);
                let point = super::planar_edge::line_intersection(&wall, &line, vertex.point())
                    .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let point = Vertex::new(point);
                contact(k, side, &point);
                let other = *incident
                    .iter()
                    .find(|&&j| j != k && data.sides[j].contains(&side))
                    .ok_or_else(|| error(Code::UnsupportedTopology))?;
                for f in data.sides[other] {
                    contact(other, f, &point);
                }
                points.push(point);
            }
            let base = center + (edge.front().point() - vertex.point());
            let section = circle(
                &Vertex::new(base + s * radius * normal(a)),
                &Vertex::new(base + s * radius * normal(b)),
                base,
            );
            let support: Surface = ExtrudedCurve::by_extrusion(section.curve(), delta).into();
            let [p, q] = [&points[0], &points[1]];
            let direction = q.point() - p.point();
            let uv = |v: &Vertex| support.search_parameter(v.point(), None, 100);
            let [uv0, uv1] =
                [uv(p), uv(q)].map(|v| v.ok_or_else(|| error(Code::BlendConstructionFailed)));
            let (uv0, uv1) = (uv0?, uv1?);
            let tangent = |v: &Vertex, (u, w): (f64, f64)| {
                let (x, y) = wall.search_parameter(v.point(), None, 100)?;
                let normal = support.normal(u, w).normalize();
                let mut tangent = normal.cross(wall.normal(x, y));
                if tangent.so_small() {
                    tangent = direction - normal * direction.dot(normal);
                }
                if tangent.dot(direction) < 0. {
                    tangent = -tangent;
                }
                (!tangent.so_small()).then_some(tangent)
            };
            let arc: Edge = super::create_pcurve_edge(
                (
                    p,
                    uv0,
                    tangent(p, uv0).ok_or_else(|| error(Code::UnsupportedGeometry))?,
                ),
                (
                    q,
                    uv1,
                    tangent(q, uv1).ok_or_else(|| error(Code::UnsupportedGeometry))?,
                ),
                support.clone(),
            )
            .ok_or_else(|| error(Code::BlendConstructionFailed))?;
            arc.set_curve(Curve::IntersectionCurve(IntersectionCurve::new(
                Box::new(wall),
                Box::new(support),
                Box::new(arc.curve()),
            )));
            cache_arc(&mut shared_arcs, &arc);
            complex_ends.insert((k, end_index(k)), Wire::from(vec![arc]));
            curved_terminals.insert(k);
            ends[k][end_index(k)] = Some(End {
                center,
                miter: None,
            });
            continue;
        }
        if faces.len() != 3 {
            let [k] = chosen[..] else {
                return Err(error(Code::UnsupportedTopology));
            };
            let edge = &data.original_edges[k];
            let [a, b] = data.sides[k];
            let axis = (edge.back().point() - edge.front().point()).normalize();
            let s = sign(k);
            let center = solve([
                (normal(a), data.planes[a].unwrap().1 - s * radius),
                (normal(b), data.planes[b].unwrap().1 - s * radius),
                (axis, axis.dot(vertex.point().to_vec())),
            ])
            .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let section = circle(
                &Vertex::new(center + s * radius * normal(a)),
                &Vertex::new(center + s * radius * normal(b)),
                center,
            );
            let cylinder: Surface =
                ExtrudedCurve::by_extrusion(section.oriented_curve(), axis).into();
            let mut points = HashMap::default();
            for &other in incident.iter().filter(|&&j| j != k) {
                let old = &data.original_edges[other];
                let far = if old.front() == vertex {
                    old.back()
                } else {
                    old.front()
                };
                let delta = far.point() - vertex.point();
                let offset = vertex.point() - center;
                let radial = delta - axis * delta.dot(axis);
                let offset = offset - axis * offset.dot(axis);
                let quadratic = radial.magnitude2();
                if quadratic <= TOLERANCE * TOLERANCE {
                    return Err(error(Code::UnsupportedGeometry));
                }
                let t = if let Some(f) = data.sides[other].into_iter().find(|f| *f == a || *f == b)
                {
                    // The support contact is tangent: solve its known radial position directly
                    // instead of subtracting nearly equal terms in a double-root discriminant.
                    (s * radius * normal(f) - offset).dot(radial) / quadratic
                } else {
                    let linear = offset.dot(radial);
                    let discriminant =
                        linear * linear - quadratic * (offset.magnitude2() - radius * radius);
                    if discriminant < 0. {
                        return Err(error(Code::UnsupportedGeometry));
                    }
                    [
                        (-linear - discriminant.sqrt()) / quadratic,
                        (-linear + discriminant.sqrt()) / quadratic,
                    ]
                    .into_iter()
                    .filter(|t| *t > 0. && *t < 1.)
                    .min_by(f64::total_cmp)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?
                };
                if !t.is_finite() || t <= 0. || t >= 1. {
                    return Err(error(Code::OutsideNeighbour));
                }
                let point = Vertex::new(vertex.point() + delta * t);
                for f in data.sides[other] {
                    contact(other, f, &point);
                    if f == a || f == b {
                        contact(k, f, &point);
                    }
                }
                points.insert(old.id(), point);
            }
            let (initial, _, walk) = super::planar_edge::end_walk(shell, [a, b], edge.id(), vertex)
                .ok_or_else(|| error(Code::UnsupportedTopology))?;
            let adjacent = |f: usize| {
                shell[f]
                    .edge_iter()
                    .find(|e| e.id() != edge.id() && (e.front() == vertex || e.back() == vertex))
                    .unwrap()
            };
            let mut from = points[&adjacent(a).id()].clone();
            let mut wall = initial;
            let mut cross = Wire::new();
            for (to, next) in walk
                .iter()
                .map(|(e, _, f)| (points[&e.id()].clone(), *f))
                .chain(std::iter::once((points[&adjacent(b).id()].clone(), b)))
            {
                let Surface::Plane(plane) = shell[wall].oriented_surface() else {
                    unreachable!()
                };
                let curve = super::projected_section::projected_section(
                    &cylinder,
                    plane,
                    from.point(),
                    to.point(),
                )
                .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                let arc = Edge::new(&from, &to, curve);
                cache_arc(&mut shared_arcs, &arc);
                cross.push_back(arc);
                from = to;
                wall = next;
            }
            complex_ends.insert((k, end_index(k)), cross);
            ends[k][end_index(k)] = Some(End {
                center,
                miter: None,
            });
            continue;
        }
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
            spheres.push((
                v,
                center,
                s,
                faces.iter().map(|&f| s * normal(f)).collect::<Vec<_>>(),
            ));
        }
    }
    // Intersect fitted contacts with terminal planes before any shared trim edge is assembled.
    let mut cone_fits = HashMap::default();
    for (k, edge) in data.original_edges.iter().enumerate() {
        if !selected.contains(&edge.id()) {
            continue;
        }
        let [a, b] = data.sides[k];
        let Some(f) = [a, b]
            .into_iter()
            .find(|&f| data.cone_supports[f].is_some())
        else {
            continue;
        };
        let cone = data.cone_supports[f].unwrap();
        let (apex, axis, angle) = cone;
        let plane = if a == f { b } else { a };
        let (n, d) = data.planes[plane].ok_or_else(|| error(Code::UnsupportedGeometry))?;
        if n.cross(axis).magnitude() < TOLERANCE {
            continue;
        }
        if n.dot(axis).abs() > TOLERANCE || (d - n.dot(apex.to_vec())).abs() > TOLERANCE {
            return Err(error(Code::UnsupportedGeometry));
        }
        let u = cone_lateral(cone, n, edge).ok_or_else(|| error(Code::UnsupportedGeometry))?;
        let stations: Vec<_> = (0..2)
            .map(|end| {
                [a, b].map(|j| {
                    let p = contacts[k][usize::from(j != a)][end].point();
                    let c = p - radius * if j == f { cone_normal(cone, p) } else { n };
                    (c - apex).dot(u)
                })
            })
            .collect();
        let increasing = (edge.back().point() - edge.front().point()).dot(u) > 0.;
        let xs = std::array::from_fn::<_, 2, _>(|end| {
            if (end == 1) == increasing {
                stations[end].into_iter().fold(f64::NEG_INFINITY, f64::max)
            } else {
                stations[end].into_iter().fold(f64::INFINITY, f64::min)
            }
        });
        if !xs[0].is_finite() || !xs[1].is_finite() || (xs[1] - xs[0]).abs() <= TOLERANCE {
            return Err(error(Code::OutsideNeighbour));
        }
        let surfaces = [shell[a].oriented_surface(), shell[b].oriented_surface()];
        let fit = ApproxFilletSurface::approx_contact_circles(
            &surfaces[0],
            &surfaces[1],
            (0., 1.),
            tol / 100.,
            |t| {
                let x = xs[0] + (xs[1] - xs[0]) * t;
                let rho = (x * x + radius * radius).sqrt();
                let h = (rho + radius / angle.0.cos()) / angle.0.tan();
                let c = apex + axis * h - n * radius + u * x;
                let dc = (u + axis * x / (angle.0.tan() * rho)) * (xs[1] - xs[0]);
                let points =
                    [a, b].map(|j| c + radius * if j == f { cone_normal(cone, c) } else { n });
                let hints = [
                    surfaces[0].search_parameter(points[0], None, 100)?,
                    surfaces[1].search_parameter(points[1], None, 100)?,
                ]
                .map(|(u, v)| Point2::new(u, v));
                ContactCircle::try_from_contacts(
                    (c, dc),
                    t,
                    &surfaces[0],
                    &surfaces[1],
                    -radius,
                    hints,
                )
            },
        )
        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
        let mut curves: [Curve; 2] = [
            fit.side_pcurve0().to_same_geometry(),
            fit.side_pcurve1().to_same_geometry(),
        ];
        let surface: Surface = fit.to_same_geometry();
        let mut lines = Vec::new();
        for (side, curve) in curves.iter_mut().enumerate() {
            let mut ts = [0., 1.];
            for (end, t) in ts.iter_mut().enumerate() {
                if let Some(&terminal) = cone_terminals.get(&(k, end)) {
                    let wall = data.planes[terminal].unwrap();
                    *t = cone_curve_plane(curve, wall, *t)
                        .ok_or_else(|| error(Code::OutsideNeighbour))?;
                    contacts[k][side][end].set_point(curve.subs(*t));
                }
            }
            if ts[1] <= ts[0] {
                return Err(error(Code::OutsideNeighbour));
            }
            if ts[1] < 1. {
                curve.cut(ts[1]);
            }
            if ts[0] > 0. {
                *curve = curve.cut(ts[0]);
            }
            lines.push(Edge::new(
                &contacts[k][side][0],
                &contacts[k][side][1],
                curve.clone(),
            ));
        }
        for (end, (p, q)) in contacts[k][0].iter().zip(&contacts[k][1]).enumerate() {
            if let Some(&terminal) = cone_terminals.get(&(k, end)) {
                let arc = terminal_cross_arc(p, q, &surface, &shell[terminal].oriented_surface())
                    .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                cache_arc(&mut shared_arcs, &arc);
                complex_ends.insert((k, end), Wire::from(vec![arc]));
            }
        }
        cone_fits.insert(k, (surface, [lines[0].clone(), lines[1].clone()]));
    }
    let mut trims = Vec::new();
    let mut arcs: Vec<Option<[Wire; 2]>> = vec![None; data.original_edges.len()];
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
        if !selected.contains(&edge.id()) {
            if matches!(edge.curve(), Curve::Line(_)) {
                let piece = make_line(&a0, &a1)?;
                trims.push([piece.clone(), piece]);
                continue;
            }
            let mut piece = edge.clone();
            if &a0 != piece.front() {
                piece = super::planar_edge::trim_edge(&piece, piece.front(), &a0, tol, false)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?;
            }
            if &a1 != piece.back() {
                piece = super::planar_edge::trim_edge(&piece, piece.back(), &a1, tol, false)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?;
            }
            trims.push([piece.clone(), piece]);
            continue;
        }
        let [a, b] = data.sides[k];
        let cone = [a, b].iter().find_map(|&f| data.cone_supports[f]);
        let cone_rim = cone.filter(|(_, axis, _)| {
            [a, b].iter().any(|&f| {
                data.planes[f].is_some_and(|(n, _)| n.cross(*axis).magnitude() < TOLERANCE)
            })
        });
        let sphere = [a, b].iter().find_map(|&f| data.sphere_supports[f]);
        let sphere_plane = sphere.and_then(|_| [a, b].iter().find_map(|&f| data.planes[f]));
        let cylinder = [a, b].iter().find_map(|&f| data.cylinders[f]);
        let rim = cylinder.filter(|(_, axis, _)| {
            [a, b].iter().any(|&f| {
                data.planes[f].is_some_and(|(n, _)| n.cross(*axis).magnitude() < TOLERANCE)
            })
        });
        let make_contact = |p: &Vertex, q: &Vertex| {
            if let Some(((origin, _), (axis, _))) = sphere.zip(sphere_plane) {
                circular_contact(edge, p, q, origin, axis)
                    .ok_or_else(|| error(Code::OutsideNeighbour))
            } else if let Some((origin, axis, _)) = rim.or(cone_rim.map(|(p, a, _)| (p, a, 0.))) {
                circular_contact(edge, p, q, origin, axis)
                    .ok_or_else(|| error(Code::OutsideNeighbour))
            } else {
                make_line(p, q)
            }
        };
        let [line_a, line_b] = if let Some((_, lines)) = cone_fits.get(&k) {
            lines.clone()
        } else {
            [make_contact(&a0, &a1)?, make_contact(&b0, &b1)?]
        };
        let [n, m] = [a, b].map(|f| data.planes[f].map_or(Vector3::zero(), |(n, _)| n));
        let s = if cylinder.is_some() || sphere.is_some() || cone.is_some() {
            1.
        } else {
            sign(k)
        };
        let arc = |end: End, p: &Vertex, q: &Vertex| -> Edge {
            if cylinder.is_some() || sphere.is_some() || cone.is_some() {
                return circle(p, q, end.center);
            }
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
        let complex = complex_ends.contains_key(&(k, 0)) || complex_ends.contains_key(&(k, 1));
        let arc0 = complex_ends
            .remove(&(k, 0))
            .unwrap_or_else(|| vec![reuse_arc(&mut shared_arcs, arc(e0, &a0, &b0))].into());
        let arc1 = complex_ends
            .remove(&(k, 1))
            .unwrap_or_else(|| vec![reuse_arc(&mut shared_arcs, arc(e1, &a1, &b1))].into());
        let mut surface: Surface = if let Some((surface, _)) = cone_fits.remove(&k) {
            surface
        } else if let Some((apex, axis, _)) = cone_rim {
            Processor::new(RevolutedCurve::by_revolution(arc0[0].curve(), apex, axis)).into()
        } else if let Some((sphere, plane)) = sphere.zip(sphere_plane) {
            sphere_rim_surface(edge, sphere, plane, radius)
                .ok_or_else(|| error(Code::OutsideNeighbour))?
        } else if let Some((origin, axis, _)) = rim {
            Processor::new(RevolutedCurve::by_revolution(arc0[0].curve(), origin, axis)).into()
        } else if complex {
            let direction = axis.normalize();
            let mut positions: Vec<_> = arc0
                .vertex_iter()
                .chain(arc1.vertex_iter())
                .map(|v| (v.point() - e0.center).dot(direction))
                .collect();
            if curved_terminals.contains(&k) {
                positions.extend(
                    [edge.front(), edge.back()].map(|v| (v.point() - e0.center).dot(direction)),
                );
            }
            let low = positions.iter().copied().fold(f64::INFINITY, f64::min);
            let high = positions.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let center = e0.center + direction * low;
            let section = circle(
                &Vertex::new(center + s * radius * n),
                &Vertex::new(center + s * radius * m),
                center,
            );
            ExtrudedCurve::by_extrusion(section.curve(), direction * (high - low)).into()
        } else if e0.miter.is_none() && e1.miter.is_none() {
            ExtrudedCurve::by_extrusion(arc0[0].curve(), e1.center - e0.center).into()
        } else {
            let c0 = arc0[0].oriented_curve().lift_up();
            let c1 = arc1[0].oriented_curve().lift_up();
            NurbsSurface::new(BSplineSurface::homotopy(c0, c1)).into()
        };
        let mut outward = n + m;
        if let Some((origin, axis, _)) = cylinder {
            let delta = surface.subs(0.5, 0.5) - origin;
            outward += (delta - axis * delta.dot(axis)).normalize();
        }
        if let Some((origin, _)) = sphere {
            outward += (surface.subs(0.5, 0.5) - origin).normalize();
        }
        if let Some(cone) = cone {
            outward += cone_normal(cone, surface.subs(0.5, 0.5));
        }
        if surface.normal(0.5, 0.5).dot(outward) < 0. {
            surface.invert();
        }
        let mut boundary = Wire::from(vec![line_a.inverse()]);
        boundary.extend(arc0.clone());
        boundary.push_back(line_b.clone());
        boundary.extend(arc1.inverse());
        generated.push(
            Face::try_new(vec![boundary], surface)
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
        if let Some((normal, _)) = data.planes[f] {
            if !valid_boundaries(face, &boundaries, normal, tol) {
                return Err(error(Code::OutsideNeighbour).face(f));
            }
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
            .flat_map(|&k| {
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
        let section = arcs[a].as_ref().unwrap()[end][0].oriented_curve();
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
    for (v, center, s, normals) in spheres {
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

fn circular_contact(
    edge: &Edge,
    a: &Vertex,
    b: &Vertex,
    origin: Point3,
    axis: Vector3,
) -> Option<Edge> {
    let curve = edge.curve();
    let (lo, hi) = curve.range_tuple();
    let mid = curve.subs((lo + hi) / 2.);
    let radial = mid - origin - axis * (mid - origin).dot(axis);
    let sense = axis.cross(radial).dot(curve.der((lo + hi) / 2.)).signum();
    let center = origin + axis * (a.point() - origin).dot(axis);
    let u = a.point() - center;
    let v = b.point() - center;
    let angle = (sense * axis.dot(u.cross(v)).atan2(u.dot(v))).rem_euclid(std::f64::consts::TAU);
    if !angle.is_finite()
        || angle <= TOLERANCE
        || u.so_small()
        || !u.magnitude().near(&v.magnitude())
    {
        return None;
    }
    let half = sense * angle / 2.;
    Some(builder::circle_arc(
        a,
        b,
        center + u * half.cos() + axis.cross(u) * half.sin(),
    ))
}

fn sphere_planes(
    origin: Point3,
    radius: f64,
    planes: [(Vector3, f64); 2],
    hint: Point3,
) -> Option<Point3> {
    let [(n, d), (m, e)] = planes;
    let axis = n.cross(m);
    if axis.so_small() {
        return None;
    }
    let direction = axis.normalize();
    let foot = solve([(n, d), (m, e), (direction, direction.dot(origin.to_vec()))])?;
    let square = radius * radius - foot.distance2(origin);
    if square <= 0. || !square.is_finite() {
        return None;
    }
    let offset = direction * square.sqrt();
    let points = [foot + offset, foot - offset];
    Some(if points[0].distance2(hint) < points[1].distance2(hint) {
        points[0]
    } else {
        points[1]
    })
}
fn sphere_rim_surface(
    edge: &Edge,
    (origin, outer): (Point3, f64),
    (axis, d): (Vector3, f64),
    radius: f64,
) -> Option<Surface> {
    let inner = outer - radius;
    let h = d - radius - axis.dot(origin.to_vec());
    let square = inner * inner - h * h;
    if inner <= 0. || square <= 0. || !square.is_finite() {
        return None;
    }
    let curve = edge.curve();
    let (lo, hi) = curve.range_tuple();
    let p = curve.subs((lo + hi) / 2.);
    let lateral = p - origin - axis * (p - origin).dot(axis);
    if lateral.so_small() {
        return None;
    }
    let center = origin + axis * h + lateral.normalize() * square.sqrt();
    let sphere = Vertex::new(origin + (center - origin) * (outer / inner));
    let plane = Vertex::new(center + axis * radius);
    let section = circle(&sphere, &plane, center);
    Some(Processor::new(RevolutedCurve::by_revolution(section.curve(), origin, axis)).into())
}
fn terminal_cross_arc(p: &Vertex, q: &Vertex, support: &Surface, wall: &Surface) -> Option<Edge> {
    let direction = q.point() - p.point();
    let uv0 = support.search_parameter(p.point(), None, 100)?;
    let uv1 = support.search_parameter(q.point(), None, 100)?;
    let tangent = |v: &Vertex, (u, w): (f64, f64)| {
        let (x, y) = wall.search_parameter(v.point(), None, 100)?;
        let mut tangent = support.normal(u, w).cross(wall.normal(x, y));
        if tangent.dot(direction) < 0. {
            tangent = -tangent;
        }
        (!tangent.so_small()).then_some(tangent)
    };
    let arc: Edge = super::create_pcurve_edge(
        (p, uv0, tangent(p, uv0)?),
        (q, uv1, tangent(q, uv1)?),
        support.clone(),
    )?;
    arc.set_curve(Curve::IntersectionCurve(IntersectionCurve::new(
        Box::new(wall.clone()),
        Box::new(support.clone()),
        Box::new(arc.curve()),
    )));
    Some(arc)
}

type ConeSupport = (Point3, Vector3, Rad<f64>);
fn cone_normal((apex, axis, angle): ConeSupport, point: Point3) -> Vector3 {
    let delta = point - apex;
    (delta - axis * delta.dot(axis)).normalize() * angle.0.cos() - axis * angle.0.sin()
}
fn cone_planes(
    cone: ConeSupport,
    radius: f64,
    planes: [(Vector3, f64); 2],
    hint: Point3,
) -> Option<Point3> {
    let (apex, axis, angle) = cone;
    let shifted = apex + axis * (radius / angle.0.sin());
    let [(n, d), (m, e)] = planes;
    let direction = n.cross(m);
    if direction.so_small() {
        return None;
    }
    let direction = direction.normalize();
    let foot = solve([(n, d), (m, e), (direction, direction.dot(shifted.to_vec()))])?;
    let delta = foot - shifted;
    let h = delta.dot(axis);
    let along = direction.dot(axis);
    let sec2 = 1. + angle.0.tan().powi(2);
    let a = 1. - sec2 * along * along;
    let b = 2. * (delta.dot(direction) - sec2 * h * along);
    let c = delta.magnitude2() - sec2 * h * h;
    let roots = if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            return None;
        }
        vec![-c / b]
    } else {
        let disc = b * b - 4. * a * c;
        if disc < 0. || !disc.is_finite() {
            return None;
        }
        let q = -0.5 * (b + disc.sqrt().copysign(b));
        if q == 0. {
            vec![-b / (2. * a)]
        } else {
            vec![q / a, c / q]
        }
    };
    roots
        .into_iter()
        .filter(|t| t.is_finite())
        .map(|t| foot + direction * t)
        .filter(|p| (*p - shifted).dot(axis) > 0.)
        .min_by(|p, q| p.distance2(hint).total_cmp(&q.distance2(hint)))
}

fn cone_lateral((apex, axis, _): ConeSupport, n: Vector3, edge: &Edge) -> Option<Vector3> {
    let delta = (edge.front().point() + edge.back().point().to_vec()) * 0.5 - apex;
    let lateral = delta - axis * delta.dot(axis) - n * delta.dot(n);
    (!lateral.so_small()).then(|| lateral.normalize())
}
fn cone_curve_plane(curve: &Curve, (n, d): (Vector3, f64), hint: f64) -> Option<f64> {
    let mut t = hint;
    for _ in 0..30 {
        let residual = n.dot(curve.subs(t).to_vec()) - d;
        if residual.abs() < TOLERANCE / 10. {
            return Some(t);
        }
        let deriv = n.dot(curve.der(t));
        if deriv.abs() < 1e-12 {
            return None;
        }
        t -= residual / deriv;
        if !(-1e-8..=1. + 1e-8).contains(&t) {
            return None;
        }
        t = t.clamp(0., 1.);
    }
    None
}
