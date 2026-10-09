use super::planar::valid_boundaries;
use rustc_hash::FxHashMap as HashMap;
use truck_modeling::*;

// When an inset consumes a convex circular corner, its cone ends at an apex.
// Below that apex the two planar chamfers meet along a new straight ridge.
pub(super) fn chamfer(shell: &Shell, wire: &Wire, d0: f64, d1: f64, tol: f64) -> Option<Shell> {
    if wire.len() < 3 {
        return None;
    }
    let (cap, face) = shell.iter().enumerate().find(|(_, face)| {
        matches!(face.surface(), Surface::Plane(_))
            && wire
                .iter()
                .all(|edge| face.edge_iter().any(|e| e.id() == edge.id()))
    })?;
    let along = face.edge_iter().find(|e| e.id() == wire[0].id())?.front() == wire[0].front();
    let (wire, inset, depth) = if along {
        (wire.clone(), d0, d1)
    } else {
        (wire.inverse(), d1, d0)
    };
    let closed = wire.is_cyclic();
    if !super::along_wire::is_tangent_continuous(&wire, closed) {
        return None;
    }
    let n = wire.len();
    let nv = if closed { n } else { n + 1 };
    let normal = face.oriented_surface().normal(0., 0.).normalize();
    let points: Vec<_> = wire.vertex_iter().collect();
    let direction = |i: usize, end: bool| {
        let curve = wire[i].oriented_curve();
        let (a, b) = curve.range_tuple();
        curve.der(if end { b } else { a }).normalize()
    };
    let shifted = |i: usize| wire[i].front().point() + normal.cross(direction(i, false)) * inset;
    let mut corners = Vec::new();
    for (i, edge) in wire.iter().enumerate() {
        let side = shell
            .iter()
            .enumerate()
            .find(|(f, side)| *f != cap && side.edge_iter().any(|e| e.id() == edge.id()))?
            .1;
        match side.oriented_surface().elementary()? {
            (Elementary::Plane(plane), _) => {
                if !matches!(edge.curve(), Curve::Line(_))
                    || plane.normal().normalize().dot(normal).abs() > TOLERANCE
                {
                    return None;
                }
                corners.push(None);
            }
            (
                Elementary::Cylinder {
                    origin,
                    axis,
                    radius,
                },
                true,
            ) => {
                if axis.cross(normal).magnitude() > TOLERANCE {
                    return None;
                }
                let center = origin + axis * (edge.front().point() - origin).dot(axis);
                if !normal
                    .cross(direction(i, false))
                    .near(&((center - edge.front().point()) / radius))
                {
                    return None;
                }
                corners.push(Some((center, radius)));
            }
            _ => return None,
        }
    }
    let collapsed: Vec<_> = corners
        .iter()
        .map(|c| c.is_some_and(|(_, r)| inset >= r || inset.near(&r)))
        .collect();
    if !collapsed.iter().any(|c| *c) {
        return None;
    }
    let mut cap_vertices: Vec<_> = (0..nv)
        .map(|i| {
            let point = if i < n {
                shifted(i)
            } else {
                points[i].point() + normal.cross(direction(n - 1, true)) * inset
            };
            Vertex::new(point)
        })
        .collect();
    let mut apexes = vec![None; n];
    for i in 0..n {
        let Some((center, radius)) = corners[i] else {
            continue;
        };
        if points
            .iter()
            .any(|p| (p.point() - center).dot(normal).abs() > TOLERANCE)
        {
            return None;
        }
        if !collapsed[i] {
            continue;
        }
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        if (!closed && (i == 0 || i == n - 1)) || corners[prev].is_some() || corners[next].is_some()
        {
            return None;
        }
        let corner = Vertex::new(intersection(
            shifted(prev),
            direction(prev, false),
            shifted(next),
            direction(next, false),
            normal,
        )?);
        let position = center - normal * (depth * (1. - radius / inset));
        apexes[i] = Some(if corner.point().near(&position) {
            corner.clone()
        } else {
            Vertex::new(position)
        });
        cap_vertices[i] = corner.clone();
        cap_vertices[(i + 1) % nv] = corner;
    }
    let mut replacements = HashMap::default();
    if !closed {
        for (j, i) in [(0, 0), (n, n - 1)] {
            if corners[i].is_some() {
                return None;
            }
            let adjacent = face.edge_iter().find(|e| {
                !wire.iter().any(|w| w.id() == e.id())
                    && (e.front() == &points[j] || e.back() == &points[j])
            })?;
            if !matches!(adjacent.curve(), Curve::Line(_)) {
                return None;
            }
            let point = intersection(
                shifted(i),
                direction(i, false),
                adjacent.front().point(),
                adjacent.back().point() - adjacent.front().point(),
                normal,
            )?;
            cap_vertices[j] = Vertex::new(point);
            replacements.insert(
                adjacent.id(),
                trim(&adjacent, &points[j], &cap_vertices[j])?,
            );
        }
    }
    let side_vertices: Vec<_> = points
        .iter()
        .take(nv)
        .map(|p| Vertex::new(p.point() - normal * depth))
        .collect();
    for (point, vertex) in points.iter().zip(&side_vertices) {
        let candidates: Vec<_> = shell
            .edge_iter()
            .filter(|e| {
                !wire.iter().any(|w| w.id() == e.id())
                    && !face.edge_iter().any(|c| c.id() == e.id())
                    && (e.front() == point || e.back() == point)
            })
            .collect();
        let old = candidates.first()?;
        if candidates.iter().any(|e| e.id() != old.id()) {
            return None;
        }
        replacements.insert(old.id(), trim(old, point, vertex)?);
    }
    let side_contacts: Vec<_> = wire
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let curve = e
                .oriented_curve()
                .transformed(Matrix4::from_translation(-normal * depth));
            Edge::new(&side_vertices[i], &side_vertices[(i + 1) % nv], curve)
        })
        .collect();
    let mut cap_contacts = Vec::new();
    let mut cross: Vec<Wire> = (0..nv)
        .map(|i| vec![builder::line(&cap_vertices[i], &side_vertices[i])].into())
        .collect();
    for i in 0..n {
        let j = (i + 1) % nv;
        let [a, b] = [&cap_vertices[i], &cap_vertices[j]];
        if let Some(apex) = &apexes[i] {
            let ridge = (a != apex).then(|| builder::line(a, apex));
            for index in [i, j] {
                cross[index] = ridge
                    .iter()
                    .cloned()
                    .chain(std::iter::once(builder::line(apex, &side_vertices[index])))
                    .collect();
            }
            cap_contacts.push(None);
        } else if let Some((center, radius)) = corners[i] {
            let scale = 1. - inset / radius;
            let transform = Matrix4::from_translation(center.to_vec())
                * Matrix4::from_scale(scale)
                * Matrix4::from_translation(-center.to_vec());
            cap_contacts.push(Some(Edge::new(
                a,
                b,
                wire[i].oriented_curve().transformed(transform),
            )));
        } else {
            if (b.point() - a.point()).dot(direction(i, false)) <= TOLERANCE {
                return None;
            }
            cap_contacts.push(Some(builder::line(a, b)));
        }
    }
    let mut connectors = HashMap::default();
    if !closed {
        for i in [0, n] {
            let edge = &cross[i][0];
            connectors.insert((edge.front().id(), edge.back().id()), edge.clone());
            connectors.insert((edge.back().id(), edge.front().id()), edge.inverse());
        }
    }
    let mut result = Shell::new();
    for (index, original) in shell.iter().enumerate() {
        let mut boundaries = Vec::new();
        for boundary in original.boundaries() {
            let mut pieces = Vec::new();
            for edge in boundary {
                if let Some(k) = wire.iter().position(|e| e.id() == edge.id()) {
                    if index == cap {
                        pieces.extend(cap_contacts[k].iter().cloned());
                    } else {
                        pieces.push(side_contacts[k].inverse());
                    }
                } else if let Some(new) = replacements.get(&edge.id()) {
                    pieces.push(if edge.orientation() {
                        new.clone()
                    } else {
                        new.inverse()
                    });
                } else {
                    pieces.push(edge);
                }
            }
            let mut boundary = Wire::new();
            for i in 0..pieces.len() {
                boundary.push_back(pieces[i].clone());
                let a = pieces[i].back();
                let b = pieces[(i + 1) % pieces.len()].front();
                if a != b {
                    boundary.push_back(connectors.get(&(a.id(), b.id()))?.clone());
                }
            }
            boundaries.push(boundary);
        }
        if boundaries == original.boundaries() {
            result.push(original.clone());
            continue;
        }
        if let Surface::Plane(plane) = original.oriented_surface() {
            if !valid_boundaries(original, &boundaries, plane.normal().normalize(), tol) {
                return None;
            }
        }
        result.push(Face::try_new(boundaries, original.oriented_surface()).ok()?);
    }
    for i in 0..n {
        let j = (i + 1) % nv;
        let mut boundary: Wire = cap_contacts[i].iter().map(Edge::inverse).collect();
        if collapsed[i] {
            boundary.push_back(cross[i].back()?.clone());
        } else {
            boundary.extend(cross[i].clone());
        }
        boundary.push_back(side_contacts[i].clone());
        if collapsed[i] {
            boundary.push_back(cross[j].back()?.inverse());
        } else {
            boundary.extend(cross[j].inverse());
        }
        let mut surface: Surface = if let Some((center, radius)) = corners[i] {
            let apex = center - normal * (depth * (1. - radius / inset));
            let mut cone: Surface = Processor::new(RevolutedCurve::by_revolution(
                Curve::Line(Line(apex, side_vertices[i].point())),
                center,
                normal,
            ))
            .into();
            if cone
                .normal(0.5, 0.)
                .dot(normal + (points[i].point() - center) / radius)
                < 0.
            {
                cone.invert();
            }
            cone
        } else {
            Plane::new(
                cap_vertices[i].point(),
                cap_vertices[j].point(),
                side_vertices[i].point(),
            )
            .into()
        };
        if let Surface::Plane(plane) = &surface {
            if plane.normal().dot(normal) < 0. {
                surface.invert();
            }
        }
        result.push(Face::try_new(vec![boundary], surface).ok()?);
    }
    Some(result)
}

fn intersection(p: Point3, a: Vector3, q: Point3, b: Vector3, normal: Vector3) -> Option<Point3> {
    let denominator = a.cross(b).dot(normal);
    if denominator.abs() <= TOLERANCE {
        return None;
    }
    Some(p + a * ((q - p).cross(b).dot(normal) / denominator))
}

fn trim(edge: &Edge, removed: &Vertex, vertex: &Vertex) -> Option<Edge> {
    let curve = edge.curve();
    let (a, b) = curve.range_tuple();
    let t = curve.search_parameter(vertex.point(), None, 100)?;
    if t <= a + TOLERANCE || t >= b - TOLERANCE {
        return None;
    }
    let absolute = if edge.orientation() {
        edge.clone()
    } else {
        edge.inverse()
    };
    let (front, back) = absolute.cut_with_parameter(vertex, t)?;
    Some(if absolute.front() == removed {
        back
    } else {
        front
    })
}
