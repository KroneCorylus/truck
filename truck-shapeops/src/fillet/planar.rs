use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

fn crosses(a: Point3, b: Point3, c: Point3, d: Point3, normal: Vector3, epsilon: f64) -> bool {
    let side = |a: Point3, b: Point3, p: Point3| (b - a).normalize().cross(p - a).dot(normal);
    let opposite = |x: f64, y: f64| (x > epsilon && y < -epsilon) || (y > epsilon && x < -epsilon);
    opposite(side(a, b, c), side(a, b, d)) && opposite(side(c, d, a), side(c, d, b))
}

fn any_segment_pair(
    a: &[Point3],
    b: Option<&[Point3]>,
    normal: Vector3,
    clearance: f64,
    mut test: impl FnMut(usize, usize) -> bool,
) -> bool {
    if a.len() + b.map_or(0, <[_]>::len) <= 32
        || !normal.magnitude2().is_finite()
        || normal.so_small()
    {
        return (0..a.len()).any(|i| {
            let range = b.map_or(i + 1..a.len(), |b| 0..b.len());
            range.into_iter().any(|j| test(i, j))
        });
    }
    if a.is_empty() {
        return false;
    }
    let normal = normal.normalize();
    let axis = if normal.x.abs() < 0.9 {
        Vector3::unit_x()
    } else {
        Vector3::unit_y()
    };
    let x = normal.cross(axis).normalize();
    let y = normal.cross(x);
    let project = |p: Point3| [(p - a[0]).dot(x), (p - a[0]).dot(y)];
    let mut segments: Vec<_> = std::iter::once(a)
        .chain(b)
        .enumerate()
        .flat_map(|(group, points)| {
            (0..points.len()).map(move |i| {
                let (p, q) = (project(points[i]), project(points[(i + 1) % points.len()]));
                (
                    [p[0].min(q[0]), p[1].min(q[1])],
                    [p[0].max(q[0]), p[1].max(q[1])],
                    group,
                    i,
                )
            })
        })
        .collect();
    let extent = segments.iter().fold(0.0_f64, |extent, (min, max, _, _)| {
        min.iter().chain(max).fold(extent, |m, v| m.max(v.abs()))
    });
    // Projected bounds only discard distant pairs; the original predicates decide contact.
    let padding = clearance + 64.0 * f64::EPSILON * (1.0 + extent);
    segments.sort_unstable_by(|a, b| a.0[0].total_cmp(&b.0[0]));
    for (i, &(min_a, max_a, group_a, index_a)) in segments.iter().enumerate() {
        for &(min_b, max_b, group_b, index_b) in &segments[i + 1..] {
            if min_b[0] > max_a[0] + padding {
                break;
            }
            if (b.is_some() && group_a == group_b)
                || min_b[1] > max_a[1] + padding
                || min_a[1] > max_b[1] + padding
            {
                continue;
            }
            let (i, j) = if b.is_none() {
                (index_a.min(index_b), index_a.max(index_b))
            } else if group_a == 0 {
                (index_a, index_b)
            } else {
                (index_b, index_a)
            };
            if test(i, j) {
                return true;
            }
        }
    }
    false
}

pub(super) fn valid_polygon(points: &[Point3], normal: Vector3, epsilon: f64) -> bool {
    if points.len() < 3 {
        return false;
    }
    polygon_area(points).dot(normal) > epsilon * epsilon
        && (0..points.len()).all(|i| points[i].distance(points[(i + 1) % points.len()]) > epsilon)
        && !any_segment_pair(points, None, normal, epsilon, |i, j| {
            if j == i + 1 || (i == 0 && j == points.len() - 1) {
                return false;
            }
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            let (c, d) = (points[j], points[(j + 1) % points.len()]);
            crosses(a, b, c, d, normal, epsilon)
                || ![(a, c, d), (b, c, d), (c, a, b), (d, a, b)]
                    .iter()
                    .all(|&(p, q, r)| {
                        let axis = r - q;
                        let t = ((p - q).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
                        p.distance(q + t * axis) > epsilon
                    })
        })
}

pub(super) fn polygon_area(points: &[Point3]) -> Vector3 {
    (0..points.len())
        .map(|i| (points[i] - points[0]).cross(points[(i + 1) % points.len()] - points[0]))
        .sum()
}

pub(super) fn sample_wire(wire: &Wire, tolerance: f64) -> Vec<Point3> {
    wire.iter()
        .flat_map(|edge| {
            let curve = edge.oriented_curve();
            let (_, mut points) = curve.parameter_division(curve.range_tuple(), tolerance);
            points.pop();
            points
        })
        .collect()
}

fn loops_intersect(a: &[Point3], b: &[Point3], normal: Vector3, epsilon: f64) -> bool {
    any_segment_pair(a, Some(b), normal, epsilon, |i, j| {
        crosses(
            a[i],
            a[(i + 1) % a.len()],
            b[j],
            b[(j + 1) % b.len()],
            normal,
            epsilon,
        )
    })
}

fn inside(p: Point3, original: &[Point3], normal: Vector3, epsilon: f64) -> bool {
    all_inside(std::iter::once(p), original, normal, epsilon)
}

fn all_inside(
    mut points: impl Iterator<Item = Point3>,
    original: &[Point3],
    normal: Vector3,
    epsilon: f64,
) -> bool {
    let x = (original[1] - original[0]).normalize();
    let y = normal.cross(x);
    let project = |p: Point3| Point2::new((p - original[0]).dot(x), (p - original[0]).dot(y));
    let original: Vec<_> = original.iter().map(|&p| project(p)).collect();
    let extent = original
        .iter()
        .fold(0.0_f64, |m, p| m.max(p.x.abs()).max(p.y.abs()));
    let padding = epsilon + 64.0 * f64::EPSILON * (1.0 + extent);
    points.all(|p| {
        let p = project(p);
        let mut winding = false;
        for j in 0..original.len() {
            let (a, b) = (original[j], original[(j + 1) % original.len()]);
            let axis = b - a;
            if p.x >= a.x.min(b.x) - padding
                && p.x <= a.x.max(b.x) + padding
                && p.y >= a.y.min(b.y) - padding
                && p.y <= a.y.max(b.y) + padding
            {
                let t = ((p - a).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
                if p.distance(a + t * axis) <= epsilon {
                    return true;
                }
            }
            if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) * axis.x / axis.y {
                winding = !winding;
            }
        }
        winding
    })
}

pub(super) fn contained(
    points: &[Point3],
    original: &[Point3],
    normal: Vector3,
    epsilon: f64,
) -> bool {
    if points.is_empty() {
        return true;
    }
    let samples = (0..points.len()).flat_map(|i| {
        [
            points[i],
            points[i] + (points[(i + 1) % points.len()] - points[i]) / 2.0,
        ]
    });
    all_inside(samples, original, normal, epsilon)
        && !loops_intersect(points, original, normal, epsilon)
}

pub(super) fn contains_patch(face: &Face, points: &[Point3], normal: Vector3, tol: f64) -> bool {
    let loops: Vec<_> = face
        .boundaries()
        .iter()
        .map(|w| sample_wire(w, tol))
        .collect();
    contained(points, &loops[0], normal, TOLERANCE)
        && loops[1..].iter().all(|hole| {
            !loops_touch(points, hole, normal, TOLERANCE)
                && !inside(hole[0], points, normal, TOLERANCE)
                && !inside(points[0], hole, normal, TOLERANCE)
        })
}

pub(super) struct Neighborhood {
    pub vertices: Vec<Vertex>,
    pub original_edges: Vec<Edge>,
    pub edge_index: HashMap<EdgeID, usize>,
    pub planes: Vec<Option<(Vector3, f64)>>,
    pub incident: Vec<Vec<usize>>,
    pub sides: Vec<[usize; 2]>,
}

pub(super) fn neighborhood(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    operation: &'static str,
) -> Result<Neighborhood, Diagnostic> {
    let error = |code| Diagnostic::new(code, operation, "validate_neighborhood");
    let mut vertices = Vec::new();
    let mut vertex_index = HashMap::default();
    let mut original_edges = Vec::new();
    let mut edge_index = HashMap::default();
    let mut edge_faces: Vec<Vec<(usize, bool)>> = Vec::new();
    let planes: Vec<_> = shell
        .iter()
        .map(|face| match face.oriented_surface() {
            Surface::Plane(plane) => {
                let normal = plane.normal().normalize();
                Some((normal, normal.dot(plane.origin().to_vec())))
            }
            _ => None,
        })
        .collect();
    for (f, face) in shell.iter().enumerate() {
        for edge in face.edge_iter() {
            for vertex in [edge.front(), edge.back()] {
                vertex_index.entry(vertex.id()).or_insert_with(|| {
                    let index = vertices.len();
                    vertices.push(vertex.clone());
                    index
                });
            }
            let k = *edge_index.entry(edge.id()).or_insert_with(|| {
                let index = original_edges.len();
                original_edges.push(edge.absolute_clone());
                edge_faces.push(Vec::new());
                index
            });
            edge_faces[k].push((f, edge.orientation()));
        }
    }
    if selected.iter().any(|id| !edge_index.contains_key(id)) {
        return Err(error(Code::UnknownEdge));
    }
    let mut incident = vec![Vec::new(); vertices.len()];
    let mut sides = Vec::new();
    for (k, edge) in original_edges.iter().enumerate() {
        let [(a, forward), (b, backward)] = edge_faces[k][..] else {
            return Err(error(Code::UnsupportedTopology));
        };
        if forward == backward {
            return Err(error(Code::UnsupportedTopology));
        }
        sides.push(if forward { [a, b] } else { [b, a] });
        for v in [edge.front(), edge.back()] {
            incident[vertex_index[&v.id()]].push(k);
        }
    }
    for edges in &incident {
        if !edges
            .iter()
            .any(|&k| selected.contains(&original_edges[k].id()))
        {
            continue;
        }
        if edges.len() != 3
            && edges
                .iter()
                .filter(|&&k| selected.contains(&original_edges[k].id()))
                .count()
                != 1
        {
            return Err(error(Code::UnsupportedTopology));
        }
        for &k in edges {
            for f in sides[k] {
                if planes[f].is_none() {
                    return Err(error(Code::NonPlanarFace).face(f));
                }
            }
            let edge = &original_edges[k];
            if !matches!(edge.curve(), Curve::Line(_)) {
                return Err(error(Code::UnsupportedGeometry));
            }
            let axis = edge.back().point() - edge.front().point();
            if axis.so_small() {
                return Err(error(Code::DegenerateCurve));
            }
            let [a, b] = sides[k];
            if selected.contains(&edge.id())
                && planes[a]
                    .unwrap()
                    .0
                    .cross(planes[b].unwrap().0)
                    .dot(axis.normalize())
                    .abs()
                    <= TOLERANCE
            {
                return Err(error(Code::BlendConstructionFailed));
            }
        }
    }
    Ok(Neighborhood {
        vertices,
        original_edges,
        edge_index,
        planes,
        incident,
        sides,
    })
}

fn loops_touch(a: &[Point3], b: &[Point3], normal: Vector3, clearance: f64) -> bool {
    let distance = |p: Point3, q: Point3, r: Point3| {
        let axis = r - q;
        let t = ((p - q).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
        p.distance(q + t * axis)
    };
    any_segment_pair(a, Some(b), normal, clearance, |i, j| {
        let (a, b, c, d) = (a[i], a[(i + 1) % a.len()], b[j], b[(j + 1) % b.len()]);
        crosses(a, b, c, d, normal, TOLERANCE)
            || [(a, c, d), (b, c, d), (c, a, b), (d, a, b)]
                .iter()
                .any(|&(p, q, r)| distance(p, q, r) <= clearance)
    })
}

pub(crate) fn valid_boundaries(
    face: &Face,
    boundaries: &[Wire],
    normal: Vector3,
    tol: f64,
) -> bool {
    let epsilon = tol.min(TOLERANCE);
    let original = face.boundaries();
    let loops: Vec<_> = boundaries.iter().map(|w| sample_wire(w, tol)).collect();
    for (i, points) in loops.iter().enumerate() {
        let old = sample_wire(&original[i], tol);
        let orientation = polygon_area(&old).dot(normal).signum();
        if boundaries[i] != original[i] && !valid_polygon(points, normal * orientation, epsilon) {
            return false;
        }
    }
    for i in 1..loops.len() {
        if (boundaries[i] != original[i] || boundaries[0] != original[0])
            && (!contained(&loops[i], &loops[0], normal, epsilon)
                || loops_touch(&loops[i], &loops[0], normal, 2.0 * tol))
        {
            return false;
        }
        for j in 1..i {
            if (boundaries[i] != original[i] || boundaries[j] != original[j])
                && (loops_touch(&loops[i], &loops[j], normal, 2.0 * tol)
                    || inside(loops[i][0], &loops[j], normal, epsilon)
                    || inside(loops[j][0], &loops[i], normal, epsilon))
            {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests;
