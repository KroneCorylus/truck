use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_modeling::*;

pub(super) fn fillet<R: ScalarFunctionD1>(shell: &Shell, selected: &Wire, law: R) -> Option<Shell> {
    let radius = law.subs(0.);
    if (0..=selected.len() * 32).any(|i| {
        let t = i as f64 / 32.;
        !law.subs(t).is_finite()
            || (law.subs(t) - radius).abs() > radius * 1e-12
            || law.der_n(1, t) != 0.
            || law.der_n(2, t) != 0.
    }) {
        return None;
    }
    if selected.is_empty()
        || !selected.is_continuous()
        || !selected.is_simple()
        || !radius.is_finite()
        || radius <= 0.
    {
        return None;
    }
    let ids: HashSet<_> = selected.iter().map(Edge::id).collect();
    let (cap, normal) = shell.iter().enumerate().find_map(|(i, f)| {
        let Some((Elementary::Plane(plane), sense)) = f.oriented_surface().elementary() else {
            return None;
        };
        ids.iter()
            .all(|id| f.edge_iter().any(|e| e.id() == *id))
            .then_some((i, plane.normal() * if sense { 1. } else { -1. }))
    })?;
    let first = selected.front()?;
    let cap_edge = shell[cap].edge_iter().find(|e| e.id() == first.id())?;
    let wire = if first.front() == cap_edge.front() {
        selected.clone()
    } else {
        selected.inverse()
    };
    let n = wire.len();
    let closed = wire.is_cyclic();
    let nv = if closed { n } else { n + 1 };
    let vertices: Vec<_> = wire
        .iter()
        .map(|e| e.front().clone())
        .chain((!closed).then(|| wire.back_vertex().unwrap().clone()))
        .collect();
    let points: Vec<_> = vertices.iter().map(Vertex::point).collect();
    let mut cap_points: Vec<Option<Point3>> = vec![None; nv];
    let mut collapsed = vec![false; n];
    let mut cylinders = vec![None; n];
    let mut walls = Vec::new();
    for (k, e) in wire.iter().enumerate() {
        let wall = shell.iter().enumerate().find_map(|(i, f)| {
            (i != cap && f.edge_iter().any(|q| q.id() == e.id())).then_some(i)
        })?;
        let (kind, sense) = shell[wall].oriented_surface().elementary()?;
        let curve = e.oriented_curve();
        let (a, b) = curve.range_tuple();
        match kind {
            Elementary::Plane(plane) => {
                let w = plane.normal() * if sense { 1. } else { -1. };
                if w.dot(normal).abs() > 1e-8 {
                    return None;
                }
                let axis = (e.back().point() - e.front().point()).normalize();
                if (0..=16).any(|i| {
                    (curve.subs(a + (b - a) * i as f64 / 16.) - e.front().point())
                        .cross(axis)
                        .magnitude()
                        > 1e-8
                }) {
                    return None;
                }
            }
            Elementary::Cylinder {
                origin,
                axis,
                radius: r,
            } => {
                if !sense || r < radius - radius * 1e-10 || axis.cross(normal).magnitude() > 1e-8 {
                    return None;
                }
                collapsed[k] = (r - radius).abs() <= radius * 1e-10;
                let center = origin + axis * (e.front().point() - origin).dot(axis);
                cylinders[k] = Some((center, r));
            }
            _ => return None,
        };
        let wall_normal = |p: Point3| match kind {
            Elementary::Plane(plane) => plane.normal() * if sense { 1. } else { -1. },
            Elementary::Cylinder { origin, axis, .. } => {
                let q = p - origin;
                (q - axis * q.dot(axis)).normalize()
            }
            _ => unreachable!(),
        };
        for (j, p) in [(k, e.front().point()), ((k + 1) % nv, e.back().point())] {
            let w = wall_normal(p);
            let cp = p - w * radius;
            if let Some(old) = cap_points[j] {
                if (old - cp).magnitude() > 1e-8 {
                    return None;
                }
            }
            cap_points[j] = Some(cp);
        }
        for i in 0..=16 {
            let t = a + (b - a) * i as f64 / 16.;
            let tangent = curve.der(t).normalize();
            let p = curve.subs(t);
            let alignment = normal.cross(tangent).dot(-wall_normal(p));
            if !alignment.is_finite() || alignment < 1. - 1e-8 {
                return None;
            }
            if collapsed[k] && (p - wall_normal(p) * radius - cap_points[k]?).magnitude() > 1e-8 {
                return None;
            }
        }
        walls.push(wall);
    }
    if !collapsed.iter().any(|x| *x) || collapsed.iter().all(|x| *x) {
        return None;
    }
    let cap_points: Vec<_> = cap_points.into_iter().collect::<Option<_>>()?;
    // The missing cap arc must leave one shared vertex, including across a closed-wire seam.
    let mut groups: Vec<usize> = (0..nv).collect();
    for k in (0..n).filter(|&k| collapsed[k]) {
        let a = groups[k];
        let b = groups[(k + 1) % nv];
        for group in &mut groups {
            if *group == b {
                *group = a;
            }
        }
    }
    let mut cap_vertices: Vec<Vertex> = cap_points.iter().copied().map(Vertex::new).collect();
    for j in 0..nv {
        cap_vertices[j] = cap_vertices[groups[j]].clone();
    }
    let wall_vertices: Vec<_> = points
        .iter()
        .map(|p| Vertex::new(*p - normal * radius))
        .collect();
    let sections: Vec<Edge> = (0..nv)
        .map(|j| {
            let center = cap_points[j] - normal * radius;
            let mid = center
                + ((cap_vertices[j].point() - center) + (wall_vertices[j].point() - center))
                    .normalize()
                    * radius;
            builder::circle_arc(&cap_vertices[j], &wall_vertices[j], mid)
        })
        .collect();
    let mut cross = HashMap::default();
    for e in &sections {
        cross.insert((e.front().id(), e.back().id()), e.clone());
        cross.insert((e.back().id(), e.front().id()), e.inverse());
    }
    let mut cap_edges = Vec::new();
    let mut wall_edges = Vec::new();
    let mut generated = Vec::new();
    for (k, e) in wire.iter().enumerate() {
        let j = (k + 1) % nv;
        let wall = Edge::new(
            &wall_vertices[k],
            &wall_vertices[j],
            e.oriented_curve()
                .transformed(Matrix4::from_translation(-normal * radius)),
        );
        let cap_edge = if collapsed[k] {
            None
        } else {
            let edge = match cylinders[k] {
                Some((center, r)) => {
                    let transform = Matrix4::from_translation(center.to_vec())
                        * Matrix4::from_scale((r - radius) / r)
                        * Matrix4::from_translation(-center.to_vec());
                    Edge::new(
                        &cap_vertices[k],
                        &cap_vertices[j],
                        e.oriented_curve().transformed(transform),
                    )
                }
                None => builder::line(&cap_vertices[k], &cap_vertices[j]),
            };
            if (edge.back().point() - edge.front().point())
                .dot(e.back().point() - e.front().point())
                <= 0.
            {
                return None;
            }
            Some(edge)
        };
        let mut boundary = Wire::new();
        if let Some(edge) = &cap_edge {
            boundary.push_back(edge.inverse());
        }
        boundary.extend([sections[k].clone(), wall.clone(), sections[j].inverse()]);
        let mut surface: Surface = if collapsed[k] {
            let center = cap_points[k] - normal * radius;
            let curve = e.oriented_curve();
            let (a, b) = curve.range_tuple();
            let y = (curve.subs((a + b) * 0.5) - center).normalize();
            let z = normal.cross(y).normalize();
            let x = y.cross(z);
            let transform = Matrix4::from_cols(
                x.extend(0.),
                y.extend(0.),
                z.extend(0.),
                center.to_vec().extend(1.),
            );
            Processor::new(Sphere::new(Point3::origin(), radius))
                .transformed(transform)
                .into()
        } else if let Some((center, _)) = cylinders[k] {
            Processor::new(RevolutedCurve::by_revolution(
                sections[k].oriented_curve(),
                center,
                normal,
            ))
            .into()
        } else {
            ExtrudedCurve::by_extrusion(sections[k].oriented_curve(), points[j] - points[k]).into()
        };
        let sec = sections[k].oriented_curve();
        let (a, b) = sec.range_tuple();
        let p = sec.subs((a + b) * 0.5);
        let uv = surface.search_parameter(p, None, 100)?;
        let center = cap_points[k] - normal * radius;
        if surface.normal(uv.0, uv.1).dot(p - center) < 0. {
            surface.invert();
        }
        generated.push(Face::try_new(vec![boundary], surface).ok()?);
        cap_edges.push(cap_edge);
        wall_edges.push(wall);
    }
    let vertex_index: HashMap<_, _> = vertices
        .iter()
        .enumerate()
        .map(|(j, v)| (v.id(), j))
        .collect();
    let selected_index: HashMap<_, _> = wire.iter().enumerate().map(|(k, e)| (e.id(), k)).collect();
    let mut remaining = HashMap::default();
    for e in shell.edge_iter().filter(|e| !ids.contains(&e.id())) {
        if remaining.contains_key(&e.id()) {
            continue;
        }
        let a = vertex_index.get(&e.front().id());
        let b = vertex_index.get(&e.back().id());
        if a.is_none() && b.is_none() {
            remaining.insert(e.id(), (e.clone(), e.front().id()));
            continue;
        }
        let vector = e.back().point() - e.front().point();
        let dir = vector.normalize();
        let curve = e.oriented_curve();
        let (t0, t1) = curve.range_tuple();
        if (0..=16).any(|i| {
            (curve.subs(t0 + (t1 - t0) * i as f64 / 16.) - e.front().point())
                .cross(dir)
                .magnitude()
                > 1e-8
        }) {
            return None;
        }
        let pool = if dir.cross(normal).magnitude() < 1e-8 {
            &wall_vertices
        } else if dir.dot(normal).abs() < 1e-8 {
            &cap_vertices
        } else {
            return None;
        };
        let front = a.map_or(e.front(), |&j| &pool[j]);
        let back = b.map_or(e.back(), |&j| &pool[j]);
        if (back.point() - front.point()).dot(vector) <= 0. {
            return None;
        }
        remaining.insert(e.id(), (builder::line(front, back), e.front().id()));
    }
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for old in face.boundaries() {
            let mut edges = Vec::new();
            for e in old {
                let replacement = if let Some(&k) = selected_index.get(&e.id()) {
                    if f == cap {
                        cap_edges[k].clone()
                    } else if f == walls[k] {
                        Some(wall_edges[k].clone())
                    } else {
                        return None;
                    }
                } else {
                    Some(remaining.get(&e.id())?.0.clone())
                };
                if let Some(mut q) = replacement {
                    let forward = if let Some(&k) = selected_index.get(&e.id()) {
                        e.front() == wire[k].front()
                    } else {
                        e.front().id() == remaining.get(&e.id())?.1
                    };
                    if !forward {
                        q.invert();
                    }
                    edges.push(q);
                }
            }
            if edges.is_empty() {
                return None;
            }
            let mut boundary = Wire::new();
            for k in 0..edges.len() {
                boundary.push_back(edges[k].clone());
                let a = edges[k].back();
                let b = edges[(k + 1) % edges.len()].front();
                if a != b {
                    boundary.push_back(cross.get(&(a.id(), b.id()))?.clone());
                }
            }
            boundaries.push(boundary);
        }
        if boundaries == face.boundaries() {
            result.push(face.clone());
        } else {
            result.push(Face::try_new(boundaries, face.oriented_surface()).ok()?);
        }
    }
    result.extend(generated);
    (result.shell_condition() == ShellCondition::Closed).then_some(result)
}
