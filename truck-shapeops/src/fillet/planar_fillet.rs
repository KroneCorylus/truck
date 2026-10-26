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
    let mut extrusion_parameters = HashMap::default();
    let mut extrusion_miters = Vec::<ExtrusionMiter>::new();
    let mut spheres = Vec::new();
    let mut tori = Vec::new();
    let mut shared_arcs = HashMap::default();
    let mut complex_ends = HashMap::default();
    let mut planar_terminals = HashMap::default();
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

        if let Some(f) = faces.iter().copied().find(|&f| {
            data.extrusion_supports[f] && chosen.iter().any(|&k| data.sides[k].contains(&f))
        }) {
            if chosen.len() == 2 {
                if faces.len() != 3 || incident.len() != 3 {
                    return Err(error(Code::UnsupportedTopology));
                }
                let cap = data.sides[chosen[0]]
                    .iter()
                    .copied()
                    .find(|j| data.sides[chosen[1]].contains(j) && data.planes[*j].is_some())
                    .ok_or_else(|| error(Code::UnsupportedTopology))?;
                let wall = *faces
                    .iter()
                    .find(|&&j| j != f && j != cap)
                    .ok_or_else(|| error(Code::UnsupportedTopology))?;
                let curved = *chosen
                    .iter()
                    .find(|&&k| data.sides[k].contains(&f))
                    .ok_or_else(|| error(Code::UnsupportedTopology))?;
                let straight = *chosen.iter().find(|&&k| k != curved).unwrap();
                let ruling = *incident.iter().find(|k| !chosen.contains(k)).unwrap();
                if !data.sides[ruling].contains(&f)
                    || !data.sides[ruling].contains(&wall)
                    || !matches!(data.original_edges[straight].curve(), Curve::Line(_))
                    || !matches!(data.original_edges[ruling].curve(), Curve::Line(_))
                {
                    return Err(error(Code::UnsupportedTopology));
                }
                let support = shell[f].oriented_surface();
                let hint = support
                    .search_parameter(vertex.point(), None, 100)
                    .ok_or_else(|| error(Code::UnsupportedGeometry))?;
                let ExtrusionMiterGeometry {
                    uvp,
                    uvq,
                    center,
                    points,
                    cylinder,
                    guides,
                } = extrusion_miter_geometry(
                    &support,
                    data.planes[cap].unwrap(),
                    data.planes[wall].ok_or_else(|| error(Code::UnsupportedGeometry))?,
                    [
                        &data.original_edges[curved],
                        &data.original_edges[straight],
                        &data.original_edges[ruling],
                    ],
                    hint,
                    radius,
                    tol.min(TOLERANCE),
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let [p, q, r] = points.map(Vertex::new);
                contact(curved, cap, &p);
                contact(curved, f, &q);
                contact(straight, cap, &p);
                contact(straight, wall, &r);
                for j in data.sides[ruling] {
                    contact(ruling, j, &r);
                }
                for k in [curved, straight] {
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                }
                extrusion_parameters.insert((curved, end_index(curved)), [uvq, uvp]);
                curved_terminals.insert(straight);
                extrusion_miters.push(ExtrusionMiter {
                    curved: (curved, end_index(curved)),
                    straight: (straight, end_index(straight)),
                    support,
                    cylinder,
                    points: [p, q, r],
                    guides,
                    witness_axes: [
                        data.planes[cap].unwrap().0,
                        (data.original_edges[straight].back().point()
                            - data.original_edges[straight].front().point())
                        .normalize(),
                    ],
                });
                continue;
            }
            if faces.len() != 3 || !(chosen.len() == 1 || chosen.len() == 3) {
                return Err(error(Code::UnsupportedTopology));
            }
            let support = shell[f].oriented_surface();
            let hint = support
                .search_parameter(vertex.point(), None, 100)
                .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let plane_faces: Vec<_> = faces.iter().copied().filter(|&j| j != f).collect();
            let planes: Vec<_> = plane_faces
                .iter()
                .map(|&j| data.planes[j].ok_or_else(|| error(Code::UnsupportedGeometry)))
                .collect::<Result<_, _>>()?;
            let mut points = HashMap::default();
            let mut normals = HashMap::default();
            let center;
            if chosen.len() == 3 {
                let uv = extrusion_corner_parameter(
                    &support,
                    radius,
                    hint,
                    planes[0],
                    (planes[1].0, planes[1].1 - radius),
                    false,
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let q = support.subs(uv.0, uv.1);
                let n = support.normal(uv.0, uv.1).normalize();
                center = q - radius * n;
                points.insert(f, Vertex::new(q));
                normals.insert(f, n);
                for (&j, &(n, _)) in plane_faces.iter().zip(&planes) {
                    points.insert(j, Vertex::new(center + radius * n));
                    normals.insert(j, n);
                }
                for &k in &chosen {
                    if data.sides[k].contains(&f) {
                        extrusion_parameters.insert((k, end_index(k)), [uv, uv]);
                    }
                }
            } else {
                let k = chosen[0];
                let active = *data.sides[k].iter().find(|&&j| j != f).unwrap();
                let terminal = *plane_faces.iter().find(|&&j| j != active).unwrap();
                let plane = data.planes[active].unwrap();
                let cap = data.planes[terminal].unwrap();
                let curved_uv =
                    extrusion_corner_parameter(&support, radius, hint, plane, cap, true)
                        .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let planar_uv = extrusion_corner_parameter(
                    &support,
                    radius,
                    hint,
                    plane,
                    (cap.0, cap.1 - radius * cap.0.dot(plane.0)),
                    false,
                )
                .ok_or_else(|| error(Code::OutsideNeighbour))?;
                let curved = support.subs(curved_uv.0, curved_uv.1);
                center = support.subs(planar_uv.0, planar_uv.1)
                    - radius * support.normal(planar_uv.0, planar_uv.1).normalize();
                points.insert(f, Vertex::new(curved));
                points.insert(active, Vertex::new(center + radius * plane.0));
                extrusion_parameters.insert((k, end_index(k)), [curved_uv, planar_uv]);
                planar_terminals.insert((k, end_index(k)), terminal);
            }
            for &k in incident {
                if chosen.contains(&k) {
                    for j in data.sides[k] {
                        contact(k, j, &points[&j]);
                    }
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                } else {
                    let p = data.sides[k]
                        .iter()
                        .find_map(|j| points.get(j))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    for j in data.sides[k] {
                        contact(k, j, p);
                    }
                }
            }
            if chosen.len() == 3 {
                spheres.push((
                    v,
                    center,
                    1.,
                    faces.iter().map(|j| normals[j]).collect::<Vec<_>>(),
                ));
            }
            continue;
        }
        let torus_face = faces.iter().copied().find(|&f| {
            data.torus_supports[f].is_some() && chosen.iter().any(|&k| data.sides[k].contains(&f))
        });
        if let Some(f) = torus_face {
            if faces.len() != 3 || !(chosen.len() == 1 || chosen.len() == 3) {
                return Err(error(Code::UnsupportedTopology));
            }
            let torus = data.torus_supports[f].unwrap();
            let (origin, axis, major, minor) = torus;
            let inner = minor - radius;
            if inner <= 0. {
                return Err(error(Code::OutsideNeighbour));
            }
            let planes: Vec<_> = faces.iter().copied().filter(|&j| j != f).collect();
            let cap = *planes
                .iter()
                .find(|&&j| {
                    data.planes[j].is_some_and(|(n, _)| n.cross(axis).magnitude() < TOLERANCE)
                })
                .ok_or_else(|| error(Code::UnsupportedGeometry))?;
            let wall = *planes.iter().find(|&&j| j != cap).unwrap();
            let (cap_n, cap_d) = data.planes[cap].unwrap();
            let (wall_n, wall_d) =
                data.planes[wall].ok_or_else(|| error(Code::UnsupportedGeometry))?;
            if wall_n.dot(axis).abs() > TOLERANCE
                || (wall_d - wall_n.dot(origin.to_vec())).abs() > TOLERANCE
            {
                return Err(error(Code::UnsupportedGeometry));
            }
            let delta = vertex.point() - origin;
            let radial = delta - axis * delta.dot(axis);
            let lateral = radial - wall_n * radial.dot(wall_n);
            if lateral.so_small() {
                return Err(error(Code::UnsupportedGeometry));
            }
            let u = lateral.normalize();
            let touched: HashSet<_> = chosen.iter().flat_map(|&k| data.sides[k]).collect();
            let mut normals = HashMap::default();
            let mut points = HashMap::default();
            let center;
            if chosen.len() == 3 || touched.contains(&cap) {
                let h = (cap_d - radius - cap_n.dot(origin.to_vec())) / cap_n.dot(axis);
                let square = inner * inner - h * h;
                if square <= 0. {
                    return Err(error(Code::OutsideNeighbour));
                }
                let rho = major + square.sqrt().copysign(radial.magnitude() - major);
                let w = if touched.contains(&wall) { -radius } else { 0. };
                let square = rho * rho - w * w;
                if rho <= 0. || square <= 0. {
                    return Err(error(Code::OutsideNeighbour));
                }
                center = origin + axis * h + wall_n * w + u * square.sqrt();
                normals.insert(f, torus_normal(torus, center));
                normals.insert(cap, cap_n);
                normals.insert(wall, wall_n);
                for &j in &touched {
                    points.insert(j, Vertex::new(center + radius * normals[&j]));
                }
            } else {
                let k = chosen[0];
                let h = (cap_d - cap_n.dot(origin.to_vec())) / cap_n.dot(axis);
                let hint = torus_station(torus, vertex.point());
                let angles = [minor, inner]
                    .map(|q| torus_angle(h / q, hint).ok_or_else(|| error(Code::OutsideNeighbour)));
                let [theta, phi] = angles;
                let c = torus_center(torus, wall_n, u, radius, theta?)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?
                    .0;
                center = torus_center(torus, wall_n, u, radius, phi?)
                    .ok_or_else(|| error(Code::OutsideNeighbour))?
                    .0;
                points.insert(f, Vertex::new(c + radius * torus_normal(torus, c)));
                points.insert(wall, Vertex::new(center + radius * wall_n));
                planar_terminals.insert((k, end_index(k)), cap);
            }
            for &k in incident {
                if chosen.contains(&k) {
                    for j in data.sides[k] {
                        contact(k, j, &points[&j]);
                    }
                    ends[k][end_index(k)] = Some(End {
                        center,
                        miter: None,
                    });
                } else {
                    let p = data.sides[k]
                        .iter()
                        .find_map(|j| points.get(j))
                        .ok_or_else(|| error(Code::UnsupportedTopology))?;
                    for j in data.sides[k] {
                        contact(k, j, p);
                    }
                }
            }
            if chosen.len() == 3 {
                spheres.push((
                    v,
                    center,
                    1.,
                    faces.iter().map(|j| normals[j]).collect::<Vec<_>>(),
                ));
            }
            continue;
        }
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
                        planar_terminals.insert((k, end_index(k)), terminal);
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
    let mut curved_fits = HashMap::default();
    for (k, edge) in data.original_edges.iter().enumerate() {
        if !selected.contains(&edge.id()) {
            continue;
        }
        let [a, b] = data.sides[k];
        let Some(f) = [a, b]
            .into_iter()
            .find(|&f| data.cone_supports[f].is_some() || data.torus_supports[f].is_some())
        else {
            continue;
        };
        let cone = data.cone_supports[f];
        let torus = data.torus_supports[f];
        let (origin, axis) = if let Some((p, a, _)) = cone {
            (p, a)
        } else {
            let (p, a, _, _) = torus.unwrap();
            (p, a)
        };
        let plane = if a == f { b } else { a };
        let (n, d) = data.planes[plane].ok_or_else(|| error(Code::UnsupportedGeometry))?;
        if n.cross(axis).magnitude() < TOLERANCE {
            continue;
        }
        if n.dot(axis).abs() > TOLERANCE || (d - n.dot(origin.to_vec())).abs() > TOLERANCE {
            return Err(error(Code::UnsupportedGeometry));
        }
        let curve = edge.curve();
        let (lo, hi) = curve.range_tuple();
        let delta = curve.subs((lo + hi) / 2.) - origin;
        let u = if let Some(cone) = cone {
            cone_lateral(cone, n, edge).ok_or_else(|| error(Code::UnsupportedGeometry))?
        } else {
            (delta - axis * delta.dot(axis) - n * delta.dot(n)).normalize()
        };
        let normal = |p| {
            if let Some(cone) = cone {
                cone_normal(cone, p)
            } else {
                torus_normal(torus.unwrap(), p)
            }
        };
        let station = |p: Point3| {
            if let Some(torus) = torus {
                torus_station(torus, p)
            } else {
                (p - origin).dot(u)
            }
        };
        let stations: Vec<_> = (0..2)
            .map(|end| {
                [a, b].map(|j| {
                    let p = contacts[k][usize::from(j != a)][end].point();
                    station(p - radius * if j == f { normal(p) } else { n })
                })
            })
            .collect();
        let increasing = if cone.is_some() {
            (edge.back().point() - edge.front().point()).dot(u) > 0.
        } else {
            station(edge.back().point()) > station(edge.front().point())
        };
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
                let (c, dc) = if let Some((apex, axis, angle)) = cone {
                    let rho = (x * x + radius * radius).sqrt();
                    let h = (rho + radius / angle.0.cos()) / angle.0.tan();
                    (
                        apex + axis * h - n * radius + u * x,
                        u + axis * x / (angle.0.tan() * rho),
                    )
                } else {
                    torus_center(torus.unwrap(), n, u, radius, x)?
                };
                let dc = dc * (xs[1] - xs[0]);
                let points = [a, b].map(|j| c + radius * if j == f { normal(c) } else { n });
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
                if let Some(&terminal) = planar_terminals.get(&(k, end)) {
                    let wall = data.planes[terminal].unwrap();
                    *t = contact_curve_plane(curve, wall, *t)
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
            if let Some(&terminal) = planar_terminals.get(&(k, end)) {
                let arc = terminal_cross_arc(p, q, &surface, &shell[terminal].oriented_surface())
                    .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                cache_arc(&mut shared_arcs, &arc);
                complex_ends.insert((k, end), Wire::from(vec![arc]));
            }
        }
        curved_fits.insert(k, (surface, [lines[0].clone(), lines[1].clone()]));
    }

    for (k, edge) in data.original_edges.iter().enumerate() {
        if !selected.contains(&edge.id()) {
            continue;
        }
        let [a, b] = data.sides[k];
        let Some(f) = [a, b].into_iter().find(|&f| data.extrusion_supports[f]) else {
            continue;
        };
        let plane = if a == f { b } else { a };
        let (n, d) = data.planes[plane].ok_or_else(|| error(Code::UnsupportedGeometry))?;
        let support = shell[f].oriented_surface();
        let mut uv = [extrusion_parameters[&(k, 0)], extrusion_parameters[&(k, 1)]];
        if let Some(period) = support.u_period() {
            let c = edge.curve();
            let (a, b) = c.range_tuple();
            let start = support
                .search_parameter(c.subs(a), None, 100)
                .ok_or_else(|| error(Code::UnsupportedGeometry))?
                .0;
            let mut previous = start;
            for i in 1..=32 {
                let value = support
                    .search_parameter(c.subs(a + (b - a) * i as f64 / 32.), None, 100)
                    .ok_or_else(|| error(Code::UnsupportedGeometry))?
                    .0;
                previous = value + ((previous - value) / period).round() * period;
            }
            for (end, target) in [start, previous].into_iter().enumerate() {
                for point in &mut uv[end] {
                    point.0 += ((target - point.0) / period).round() * period;
                }
            }
        }
        let vary_u = n.dot(support.vder(uv[0][0].0, uv[0][0].1)).abs()
            > 1e-8 * support.vder(uv[0][0].0, uv[0][0].1).magnitude();
        let station = |p: (f64, f64)| if vary_u { p.0 } else { p.1 };
        let increasing = station(uv[1][0]) > station(uv[0][0]);
        let xs = std::array::from_fn::<_, 2, _>(|end| {
            let values = uv[end].map(station);
            if (end == 1) == increasing {
                values.into_iter().fold(f64::NEG_INFINITY, f64::max)
            } else {
                values.into_iter().fold(f64::INFINITY, f64::min)
            }
        });
        if !xs.iter().all(|x| x.is_finite()) || (xs[1] - xs[0]).abs() < 1e-10 {
            return Err(error(Code::OutsideNeighbour));
        }
        let surfaces = [shell[a].oriented_surface(), shell[b].oriented_surface()];
        let solve_circle = |t: f64, parameter_span: f64| {
            let x = xs[0] + (xs[1] - xs[0]) * t;
            let (mut u, mut v) = (
                uv[0][0].0 * (1. - t) + uv[1][0].0 * t,
                uv[0][0].1 * (1. - t) + uv[1][0].1 * t,
            );
            if vary_u {
                u = x;
            } else {
                v = x;
            }
            for _ in 0..20 {
                if !u.is_finite() || !v.is_finite() {
                    return None;
                }
                let c = support.subs(u, v) - radius * support.normal(u, v).normalize();
                let residual = n.dot(c.to_vec()) - d + radius;
                if ![c.x, c.y, c.z, residual].into_iter().all(f64::is_finite) {
                    return None;
                }
                if residual.abs() < 1e-10 {
                    break;
                }
                let du = support.uder(u, v) - radius * support.normal_uder(u, v);
                let dv = support.vder(u, v) - radius * support.normal_vder(u, v);
                let divisor = n.dot(if vary_u { dv } else { du });
                if !divisor.is_finite() || divisor.abs() < 1e-12 {
                    return None;
                }
                if vary_u {
                    v -= residual / divisor;
                } else {
                    u -= residual / divisor;
                }
            }
            if !u.is_finite() || !v.is_finite() {
                return None;
            }
            let c = support.subs(u, v) - radius * support.normal(u, v).normalize();
            let residual = n.dot(c.to_vec()) - d + radius;
            if ![c.x, c.y, c.z, residual].into_iter().all(f64::is_finite) || residual.abs() > 1e-9 {
                return None;
            }
            let du = support.uder(u, v) - radius * support.normal_uder(u, v);
            let dv = support.vder(u, v) - radius * support.normal_vder(u, v);
            let divisor = n.dot(if vary_u { dv } else { du });
            if ![du.x, du.y, du.z, dv.x, dv.y, dv.z, divisor]
                .into_iter()
                .all(f64::is_finite)
                || divisor.abs() < 1e-12
            {
                return None;
            }
            let dc = if vary_u {
                du - dv * n.dot(du) / n.dot(dv)
            } else {
                dv - du * n.dot(dv) / n.dot(du)
            } * ((xs[1] - xs[0]) / parameter_span);
            if ![dc.x, dc.y, dc.z, dc.magnitude2()]
                .into_iter()
                .all(f64::is_finite)
                || dc.magnitude2() == 0.
            {
                return None;
            }
            let point = c + radius * n;
            if ![point.x, point.y, point.z].into_iter().all(f64::is_finite) {
                return None;
            }
            let plane_uv = surfaces[usize::from(b == plane)].search_parameter(point, None, 100)?;
            if !plane_uv.0.is_finite() || !plane_uv.1.is_finite() {
                return None;
            }
            let hints = if a == f {
                [Point2::new(u, v), Point2::from(plane_uv)]
            } else {
                [Point2::from(plane_uv), Point2::new(u, v)]
            };
            let circle = ContactCircle::try_from_contacts(
                (c, dc),
                t * parameter_span,
                &surfaces[0],
                &surfaces[1],
                -radius,
                hints,
            )?;
            Some((circle, dc))
        };
        let (circle, center_derivative) =
            solve_circle(0.5, 1.).ok_or_else(|| error(Code::BlendConstructionFailed))?;
        let section: NurbsCurve<Vector4> = circle.to_same_geometry();
        let parameter_span = center_derivative.magnitude() / section.der(0.5).magnitude();
        if !parameter_span.is_finite() || parameter_span <= 0. {
            return Err(error(Code::BlendConstructionFailed));
        }
        let fitting_tol = if extrusion_miters.iter().any(|m| m.curved.0 == k) {
            tol.min(TOLERANCE)
        } else {
            tol
        };
        let fit = ApproxFilletSurface::approx_contact_circles(
            &surfaces[0],
            &surfaces[1],
            (0., parameter_span),
            fitting_tol / 100.,
            |parameter| solve_circle(parameter / parameter_span, parameter_span).map(|x| x.0),
        )
        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
        let mut curves: [Curve; 2] = [
            fit.side_pcurve0().to_same_geometry(),
            fit.side_pcurve1().to_same_geometry(),
        ];
        let surface: Surface = fit.to_same_geometry();
        let mut lines = Vec::new();
        for (side, curve) in curves.iter_mut().enumerate() {
            let mut ts = [0., parameter_span];
            for (end, t) in ts.iter_mut().enumerate() {
                if let Some(&terminal) = planar_terminals.get(&(k, end)) {
                    *t = contact_curve_plane(curve, data.planes[terminal].unwrap(), *t)
                        .ok_or_else(|| error(Code::OutsideNeighbour))?;
                    contacts[k][side][end].set_point(curve.subs(*t));
                }
                if let Some(miter) = extrusion_miters.iter().find(|m| m.curved == (k, end)) {
                    let original = contacts[k][side][end].point();
                    *t = curve
                        .search_nearest_parameter(original, Some(*t), 100)
                        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                    if !t.is_finite()
                        || *t < 0.
                        || *t > parameter_span
                        || !finite_point(original)
                        || !finite_point(curve.subs(*t))
                        || !curve.subs(*t).distance(original).is_finite()
                        || curve.subs(*t).distance(original) > tol / 10.
                    {
                        return Err(error(Code::BlendConstructionFailed));
                    }
                    // The two side contacts have different stations. Retain each
                    // side only to its own endpoint, before assembling the miter.
                    contacts[k][side][end].set_point(curve.subs(*t));
                    let cylinder = &miter.cylinder;
                    let uv = cylinder
                        .search_nearest_parameter(curve.subs(*t), None, 100)
                        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                    if !finite_uv(uv)
                        || !finite_point(cylinder.subs(uv.0, uv.1))
                        || !cylinder
                            .subs(uv.0, uv.1)
                            .distance(curve.subs(*t))
                            .is_finite()
                        || cylinder.subs(uv.0, uv.1).distance(curve.subs(*t)) > tol / 10.
                    {
                        return Err(error(Code::BlendConstructionFailed));
                    }
                }
            }
            if ts[1] <= ts[0] {
                return Err(error(Code::OutsideNeighbour));
            }
            if ts[1] < parameter_span {
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
            if let Some(&terminal) = planar_terminals.get(&(k, end)) {
                let arc = terminal_cross_arc(p, q, &surface, &shell[terminal].oriented_surface())
                    .ok_or_else(|| error(Code::BlendConstructionFailed))?;
                cache_arc(&mut shared_arcs, &arc);
                complex_ends.insert((k, end), Wire::from(vec![arc]));
            }
        }
        curved_fits.insert(k, (surface, [lines[0].clone(), lines[1].clone()]));
    }
    for miter in extrusion_miters {
        let [p, q, r] = miter.points;
        let blend = &curved_fits
            .get(&miter.curved.0)
            .ok_or_else(|| error(Code::BlendConstructionFailed))?
            .0;
        let [seam_guide, continuation_guide] = miter.guides;
        let seam = guided_intersection(
            &p,
            &q,
            blend,
            &miter.cylinder,
            seam_guide,
            miter.witness_axes,
            tol,
        )
        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
        let continuation = guided_intersection(
            &q,
            &r,
            &miter.cylinder,
            &miter.support,
            continuation_guide,
            miter.witness_axes,
            tol,
        )
        .ok_or_else(|| error(Code::BlendConstructionFailed))?;
        cache_arc(&mut shared_arcs, &continuation);
        let mut curved_end = Wire::from(vec![seam.clone()]);
        if contacts[miter.curved.0][0][miter.curved.1] != p {
            curved_end.invert();
        }
        let mut straight_end = Wire::from(vec![seam, continuation]);
        if contacts[miter.straight.0][0][miter.straight.1] != p {
            straight_end.invert();
        }
        complex_ends.insert(miter.curved, curved_end);
        complex_ends.insert(miter.straight, straight_end);
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
        let extrusion = [a, b].into_iter().find(|&f| data.extrusion_supports[f]);
        let torus = [a, b].iter().find_map(|&f| data.torus_supports[f]);
        let torus_rim = torus.filter(|(_, axis, _, _)| {
            [a, b].iter().any(|&f| {
                data.planes[f].is_some_and(|(n, _)| n.cross(*axis).magnitude() < TOLERANCE)
            })
        });
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
            } else if let Some((origin, axis, _)) = rim
                .or(cone_rim.map(|(p, a, _)| (p, a, 0.)))
                .or(torus_rim.map(|(p, a, _, _)| (p, a, 0.)))
            {
                circular_contact(edge, p, q, origin, axis)
                    .ok_or_else(|| error(Code::OutsideNeighbour))
            } else {
                make_line(p, q)
            }
        };
        let [line_a, line_b] = if let Some((_, lines)) = curved_fits.get(&k) {
            lines.clone()
        } else {
            [make_contact(&a0, &a1)?, make_contact(&b0, &b1)?]
        };
        let [n, m] = [a, b].map(|f| data.planes[f].map_or(Vector3::zero(), |(n, _)| n));
        let s = if cylinder.is_some()
            || sphere.is_some()
            || cone.is_some()
            || torus.is_some()
            || extrusion.is_some()
        {
            1.
        } else {
            sign(k)
        };
        let arc = |end: End, p: &Vertex, q: &Vertex| -> Edge {
            if cylinder.is_some()
                || sphere.is_some()
                || cone.is_some()
                || torus.is_some()
                || extrusion.is_some()
            {
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
        let mut surface: Surface = if let Some((surface, _)) = curved_fits.remove(&k) {
            surface
        } else if let Some((apex, axis, _)) =
            cone_rim.or(torus_rim.map(|(p, a, _, _)| (p, a, Rad(0.))))
        {
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
        if let Some(torus) = torus {
            outward += torus_normal(torus, surface.subs(0.5, 0.5));
        }
        if let Some(f) = extrusion {
            let support = shell[f].oriented_surface();
            let (u, v) = support
                .search_nearest_parameter(surface.subs(0.5, 0.5), None, 100)
                .ok_or_else(|| error(Code::BlendConstructionFailed))?;
            outward += support.normal(u, v).normalize();
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
    let leader = arc.curve();
    let hint = match &leader {
        Curve::PCurve(pcurve) => Some(pcurve.curve().clone()),
        _ => None,
    };
    let mut intersection = IntersectionCurve::new(
        Box::new(wall.clone()),
        Box::new(support.clone()),
        Box::new(leader),
    );
    if let Some(hint) = hint {
        intersection = intersection.with_surface1_parameter_hint(hint);
    }
    arc.set_curve(Curve::IntersectionCurve(intersection));
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
fn contact_curve_plane(curve: &Curve, (n, d): (Vector3, f64), hint: f64) -> Option<f64> {
    let (lo, hi) = curve.range_tuple();
    if !lo.is_finite() || !hi.is_finite() || !(hi - lo).is_finite() || lo >= hi || !hint.is_finite()
    {
        return None;
    }
    let padding = (hi - lo) * 1e-8;
    let mut t = hint.clamp(lo, hi);
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
        if !(lo - padding..=hi + padding).contains(&t) {
            return None;
        }
        t = t.clamp(lo, hi);
    }
    None
}

type TorusSupport = (Point3, Vector3, f64, f64);
fn torus_normal((origin, axis, major, _): TorusSupport, p: Point3) -> Vector3 {
    let delta = p - origin;
    let radial = delta - axis * delta.dot(axis);
    (delta - radial.normalize() * major).normalize()
}
fn torus_station((origin, axis, major, minor): TorusSupport, point: Point3) -> f64 {
    let d = point - origin;
    let z = d.dot(axis);
    let rho = (d - axis * z).magnitude();
    (z / minor).atan2((rho - major) / minor)
}
fn torus_center(
    torus: TorusSupport,
    n: Vector3,
    u: Vector3,
    radius: f64,
    theta: f64,
) -> Option<(Point3, Vector3)> {
    let (origin, axis, major, minor) = torus;
    let q = minor - radius;
    if q <= 0. {
        return None;
    }
    let rho = major + q * theta.cos();
    let square = rho * rho - radius * radius;
    if square <= 0. || !square.is_finite() {
        return None;
    }
    let x = square.sqrt();
    let c = origin + u * x - n * radius + axis * q * theta.sin();
    let dc = -u * (rho * q * theta.sin() / x) + axis * q * theta.cos();
    Some((c, dc))
}

fn torus_angle(sine: f64, hint: f64) -> Option<f64> {
    if !(-1. ..=1.).contains(&sine) {
        return None;
    }
    let a = sine.asin();
    [a, std::f64::consts::PI - a]
        .into_iter()
        .map(|a| a + std::f64::consts::TAU * ((hint - a) / std::f64::consts::TAU).round())
        .min_by(|a, b| (a - hint).abs().total_cmp(&(b - hint).abs()))
}

fn extrusion_corner_parameter(
    surface: &Surface,
    radius: f64,
    hint: (f64, f64),
    plane: (Vector3, f64),
    terminal: (Vector3, f64),
    on_surface: bool,
) -> Option<(f64, f64)> {
    let (mut u, mut v) = hint;
    let residual = |u, v| {
        let n = surface.normal(u, v).normalize();
        let c = surface.subs(u, v) - radius * n;
        let q = if on_surface { surface.subs(u, v) } else { c };
        [
            plane.0.dot(c.to_vec()) - plane.1 + radius,
            terminal.0.dot(q.to_vec()) - terminal.1,
        ]
    };
    for _ in 0..60 {
        if !u.is_finite() || !v.is_finite() {
            return None;
        }
        let f = residual(u, v);
        if !f.into_iter().all(f64::is_finite) {
            return None;
        }
        if f[0].abs().max(f[1].abs()) < 1e-10 {
            return Some((u, v));
        }
        let su = surface.uder(u, v);
        let sv = surface.vder(u, v);
        let du = su - radius * surface.normal_uder(u, v);
        let dv = sv - radius * surface.normal_vder(u, v);
        let terminal_du = if on_surface { su } else { du };
        let terminal_dv = if on_surface { sv } else { dv };
        let a = plane.0.dot(du);
        let b = plane.0.dot(dv);
        let c = terminal.0.dot(terminal_du);
        let d = terminal.0.dot(terminal_dv);
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant.abs() < 1e-15 {
            return None;
        }
        let dx = (d * f[0] - b * f[1]) / determinant;
        let dy = (-c * f[0] + a * f[1]) / determinant;
        if !dx.is_finite() || !dy.is_finite() {
            return None;
        }
        let mut step = 1.;
        while step > 1e-8 {
            let g = residual(u - step * dx, v - step * dy);
            if g.iter().all(|x| x.is_finite())
                && g[0] * g[0] + g[1] * g[1] < f[0] * f[0] + f[1] * f[1]
            {
                break;
            }
            step *= 0.5;
        }
        if step <= 1e-8 {
            return None;
        }
        u -= step * dx;
        v -= step * dy;
    }
    None
}

#[derive(Clone)]
struct ExtrusionMiter {
    curved: (usize, usize),
    straight: (usize, usize),
    support: Surface,
    cylinder: Surface,
    points: [Vertex; 3],
    guides: [Vec<Point3>; 2],
    witness_axes: [Vector3; 2],
}

fn finite_point(p: Point3) -> bool { [p.x, p.y, p.z].into_iter().all(f64::is_finite) }
fn finite_vector(v: Vector3) -> bool { [v.x, v.y, v.z].into_iter().all(f64::is_finite) }
fn finite_uv(uv: (f64, f64)) -> bool { uv.0.is_finite() && uv.1.is_finite() }

fn bounded_surface_point(
    surface: &Surface,
    uv: (f64, f64),
    witness: Point3,
    tol: f64,
) -> Option<Point3> {
    if !finite_uv(uv) {
        return None;
    }
    let (ur, vr) = surface.try_range_tuple();
    let ((u0, u1), (v0, v1)) = (ur?, vr?);
    if ![u0, u1, v0, v1].into_iter().all(f64::is_finite) || u0 > u1 || v0 > v1 {
        return None;
    }
    // Project endpoint roundoff back into the original domain, then require
    // the bounded point itself to agree with the intersection witness.
    let point = surface.subs(uv.0.clamp(u0, u1), uv.1.clamp(v0, v1));
    (finite_point(point) && point.distance(witness) <= tol).then_some(point)
}

fn bracketed_root(
    mut f: impl FnMut(f64) -> Option<f64>,
    (mut a, mut b): (f64, f64),
    tol: f64,
) -> Option<f64> {
    if !a.is_finite() || !b.is_finite() || !tol.is_finite() || tol <= 0. {
        return None;
    }
    let mut fa = f(a)?;
    let fb = f(b)?;
    if !fa.is_finite() || !fb.is_finite() {
        return None;
    }
    if fa.abs() <= tol {
        return Some(a);
    }
    if fb.abs() <= tol {
        return Some(b);
    }
    if fa * fb >= 0. {
        return None;
    }
    for _ in 0..80 {
        let t = (a + b) / 2.;
        let ft = f(t)?;
        if !ft.is_finite() {
            return None;
        }
        if ft.abs() <= tol {
            return Some(t);
        }
        if fa * ft > 0. {
            a = t;
            fa = ft;
        } else {
            b = t;
        }
    }
    None
}

struct ExtrusionMiterGeometry {
    uvp: (f64, f64),
    uvq: (f64, f64),
    center: Point3,
    points: [Point3; 3],
    cylinder: Surface,
    guides: [Vec<Point3>; 2],
}

// This branch needs no extension of the generating curve or change of chart.
fn extrusion_miter_geometry(
    support: &Surface,
    cap: (Vector3, f64),
    wall: (Vector3, f64),
    [edge, straight, ruling]: [&Edge; 3],
    hint: (f64, f64),
    radius: f64,
    tol: f64,
) -> Option<ExtrusionMiterGeometry> {
    let Surface::Extruded(extrusion) = support else {
        return None;
    };
    let Curve::BSplineCurve(basis) = extrusion.entity_curve() else {
        return None;
    };
    if basis.degree() != 2 || basis.control_points().len() != 3 {
        return None;
    }
    let (lo, hi) = basis.range_tuple();
    if !lo.is_finite()
        || !hi.is_finite()
        || lo >= hi
        || (0..3).any(|i| basis.knot(i) != lo || basis.knot(i + 3) != hi)
    {
        return None;
    }
    if !finite_uv(hint) || !radius.is_finite() || radius <= 0. || !tol.is_finite() || tol <= 0. {
        return None;
    }
    let (n, d) = cap;
    let (m, e) = wall;
    let a = extrusion.extruding_vector();
    if !finite_vector(n)
        || !finite_vector(m)
        || !finite_vector(a)
        || !d.is_finite()
        || !e.is_finite()
        || a.so_small()
    {
        return None;
    }
    let controls = basis.control_points();
    if controls.iter().any(|p| !finite_point(*p)) {
        return None;
    }
    let [bottom, top] = [ruling.front().point(), ruling.back().point()];
    if !finite_point(bottom) || !finite_point(top) {
        return None;
    }
    let heights = [n.dot(bottom.to_vec()), n.dot(top.to_vec())];
    let (height_lo, height_hi) = (heights[0].min(heights[1]), heights[0].max(heights[1]));
    let in_original_slab = |p: Point3| {
        let h = n.dot(p.to_vec());
        finite_point(p) && h.is_finite() && h >= height_lo - tol / 10. && h <= height_hi + tol / 10.
    };
    let chord = controls[2] - controls[0];
    let sag = controls[1] - controls[0].midpoint(controls[2]);
    // Reflection symmetry bounds the remote branch by the original endpoint
    // and the profile midpoint. Non-symmetric profiles need a separate contract.
    if chord.so_small()
        || sag.so_small()
        || a.dot(chord).abs() > TOLERANCE * a.magnitude() * chord.magnitude()
        || sag.dot(chord).abs() > TOLERANCE * sag.magnitude() * chord.magnitude()
        || n.dot(chord).abs() > TOLERANCE * chord.magnitude()
        || n.dot(sag).abs() > TOLERANCE * sag.magnitude()
    {
        return None;
    }
    let na = n.dot(a);
    let nm = n.dot(m);
    if !na.is_finite()
        || !nm.is_finite()
        || na.abs() <= TOLERANCE * a.magnitude()
        || m.dot(a).abs() > TOLERANCE * a.magnitude()
        || 1. - nm * nm <= TOLERANCE
    {
        return None;
    }
    // A quadratic's projected derivative is linear. Its minimum speed gives
    // the exact maximum curvature of the cross-section perpendicular to A.
    let axis = a.normalize();
    let project = |x: Vector3| x - axis * x.dot(axis);
    let b0 = project(basis.der(lo));
    let b1 = project(basis.der2(lo));
    if b1.so_small() {
        return None;
    }
    let critical = (lo - b0.dot(b1) / b1.magnitude2()).clamp(lo, hi);
    let speed = project(basis.der(critical)).magnitude();
    let curvature = b0.cross(b1).magnitude() / speed.powi(3);
    if !curvature.is_finite() || radius * curvature >= 1. {
        return None;
    }
    let vw = (m - n * nm).normalize();
    let center_line = Point3::from_vec(
        n * ((d - radius - nm * (e - radius)) / (1. - nm * nm))
            + m * ((e - radius - nm * (d - radius)) / (1. - nm * nm)),
    );
    if !finite_point(center_line) || !finite_vector(vw) {
        return None;
    }
    let frame = |u: f64| {
        if !u.is_finite() || u < lo || u > hi {
            return None;
        }
        let normal = support.normal(u, hint.1);
        let point = support.subs(u, hint.1);
        let v = hint.1 + (d - radius - n.dot((point - radius * normal).to_vec())) / na;
        let center = support.subs(u, v) - radius * normal;
        let lateral = normal - n * normal.dot(n);
        let cosine = n.dot(normal);
        if !v.is_finite()
            || !finite_vector(normal)
            || !finite_point(center)
            || !finite_vector(lateral)
            || !in_original_slab(support.subs(u, v))
            || lateral.so_small()
            || !cosine.is_finite()
            || cosine.abs() >= 1.
        {
            return None;
        }
        Some(((u, v), center, normal, lateral.normalize(), cosine.acos()))
    };
    let curve = edge.curve();
    let (t0, t1) = curve.range_tuple();
    let mid = support
        .search_parameter(curve.subs((t0 + t1) / 2.), None, 100)?
        .0;
    let interval = (hint.0, mid);
    let seam = |u: f64, q: f64| -> Option<f64> {
        let (_, c, _, v, alpha) = frame(u)?;
        // E-first, with no sine turning point on the common seam.
        if alpha >= std::f64::consts::FRAC_PI_2 || alpha >= nm.acos() {
            return None;
        }
        Some(vw.dot(c - center_line) + radius * (q * alpha).sin() * (vw.dot(v) - 1.))
    };
    let mut seam_points = Vec::new();
    let mut stations = Vec::new();
    for i in 0..=64 {
        let q = i as f64 / 64.;
        let u = bracketed_root(|u| seam(u, q), interval, tol / 100.)?;
        let (uv, c, _, v, alpha) = frame(u)?;
        let phi = q * alpha;
        seam_points.push(c + radius * (n * phi.cos() + v * phi.sin()));
        stations.push(uv);
    }
    let uvp = stations[0];
    let uvq = stations[64];
    // The quadratic E-first branch must be monotone between P and Q; fit
    // bounds are then exactly the endpoint bounds, with no hidden extension.
    let direction = (uvq.0 - uvp.0).signum();
    if direction == 0.
        || stations
            .windows(2)
            .any(|w| direction * (w[1].0 - w[0].0) < 0.)
    {
        return None;
    }
    let (_, center, _, _, _) = frame(uvp.0)?;
    let phi0 = frame(uvq.0)?.4;
    let phi1 = nm.acos();
    let mut continuation = Vec::new();
    for i in 0..=64 {
        let phi = phi0 + (phi1 - phi0) * i as f64 / 64.;
        let point = |u| {
            let b = support.subs(u, hint.1);
            b + a * ((d - radius + radius * phi.cos() - n.dot(b.to_vec())) / na)
        };
        let u = bracketed_root(
            |u| Some(vw.dot(point(u) - center_line) - radius * phi.sin()),
            (hint.0, uvq.0),
            tol / 100.,
        )?;
        continuation.push(point(u));
    }
    let points = [seam_points[0], seam_points[64], continuation[64]];
    if seam_points
        .iter()
        .chain(&continuation)
        .any(|p| !in_original_slab(*p))
    {
        return None;
    }
    let endpoint_gap = points[1].distance(continuation[0]);
    if !endpoint_gap.is_finite() || endpoint_gap > tol {
        return None;
    }
    let ruling_axis = top - bottom;
    if !finite_vector(ruling_axis) || ruling_axis.so_small() {
        return None;
    }
    let ruling_parameter = (points[2] - bottom).dot(ruling_axis) / ruling_axis.magnitude2();
    let on_ruling = bottom + ruling_axis * ruling_parameter.clamp(0., 1.);
    let ruling_gap = points[2].distance(on_ruling);
    if !ruling_parameter.is_finite() || !ruling_gap.is_finite() || ruling_gap > tol / 10. {
        return None;
    }
    continuation[0] = points[1];
    // Keep the full original straight-edge span, which bounds these branches.
    let along = (straight.back().point() - straight.front().point()).normalize();
    let positions = [straight.front(), straight.back()].map(|v| (v.point() - center).dot(along));
    for p in seam_points.iter().chain(&continuation) {
        let t = (*p - center).dot(along);
        if !t.is_finite() || t < positions[0] - tol || t > positions[1] + tol {
            return None;
        }
    }
    let origin = center + along * positions[0];
    let section = circle(
        &Vertex::new(origin + radius * n),
        &Vertex::new(origin + radius * m),
        origin,
    );
    let cylinder =
        ExtrudedCurve::by_extrusion(section.curve(), along * (positions[1] - positions[0])).into();
    Some(ExtrusionMiterGeometry {
        uvp,
        uvq,
        center,
        points,
        cylinder,
        guides: [seam_points, continuation],
    })
}

fn guided_intersection(
    p: &Vertex,
    q: &Vertex,
    a: &Surface,
    b: &Surface,
    mut guide: Vec<Point3>,
    axes: [Vector3; 2],
    tol: f64,
) -> Option<Edge> {
    if guide.len() < 2
        || !finite_point(p.point())
        || !finite_point(q.point())
        || guide.iter().any(|p| !finite_point(*p))
        || axes.iter().any(|v| !finite_vector(*v) || v.so_small())
    {
        return None;
    }
    guide[0] = p.point();
    let last = guide.len() - 1;
    guide[last] = q.point();
    let knots = std::iter::once(0.)
        .chain((0..=last).map(|i| i as f64 / last as f64))
        .chain(std::iter::once(1.));
    let leader = Curve::BSplineCurve(BSplineCurve::new(KnotVec::from_iter(knots), guide.clone()));
    let curve = IntersectionCurve::new(Box::new(a.clone()), Box::new(b.clone()), Box::new(leader));
    for i in 0..=2 * last {
        let t = i as f64 / (2 * last) as f64;
        let (point, uv0, uv1) = curve.search_triple(t, 100)?;
        if !finite_point(point) {
            return None;
        }
        let pa = bounded_surface_point(a, (uv0.x, uv0.y), point, tol / 10.)?;
        let pb = bounded_surface_point(b, (uv1.x, uv1.y), point, tol / 10.)?;
        let residual = pa.distance(pb);
        if !residual.is_finite() || residual > tol / 10. {
            return None;
        }
        // At analytic nodes the projection must stay near that exact witness.
        // Between nodes, cap height and cylinder-axis position bound the local
        // monotone segment. Together with the bounded cylinder arc, these
        // are sampled branch witnesses, not a continuous uniqueness proof.
        if i % 2 == 0 {
            let distance = point.distance(guide[i / 2]);
            if !distance.is_finite() || distance > tol / 10. {
                return None;
            }
        }
        let segment = (i / 2).min(last - 1);
        for axis in axes {
            let x = axis.dot(point - guide[segment]);
            let y = axis.dot(guide[segment + 1] - guide[segment]);
            if !x.is_finite()
                || !y.is_finite()
                || x < y.min(0.) - tol / 10.
                || x > y.max(0.) + tol / 10.
            {
                return None;
            }
        }
    }
    Some(Edge::new(p, q, Curve::IntersectionCurve(curve)))
}
