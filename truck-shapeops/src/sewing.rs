use truck_geometry::prelude::*;
use truck_topology::compress::*;

/// Welds near-coincident boundary edges without adding faces or changing surface geometry.
///
/// Vertices move at most `tolerance` to an existing representative. Curves are compared
/// bidirectionally as polylines at one quarter of that tolerance. Only boundary edges
/// with opposite face uses are joined. Missing faces remain open. Invalid indices,
/// non-finite input, collapsed edges and non-manifold input return `None`.
pub fn sew_shell<C, S>(
    input: &CompressedShell<Point3, C, S>,
    tolerance: f64,
) -> Option<CompressedShell<Point3, C, S>>
where
    C: Clone + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: Clone,
{
    if !tolerance.is_finite() || tolerance < TOLERANCE || input.vertices.iter().any(|p| !finite(*p))
    {
        return None;
    }
    let mut usage = vec![Vec::new(); input.edges.len()];
    for face in &input.faces {
        for edge in face.boundaries.iter().flatten() {
            usage
                .get_mut(edge.index)?
                .push(edge.orientation == face.orientation);
        }
    }
    if usage
        .iter()
        .any(|u| u.len() > 2 || (u.len() == 2 && u[0] == u[1]))
    {
        return None;
    }
    let mut vertices: Vec<Point3> = Vec::new();
    let mut mapping = Vec::new();
    for point in &input.vertices {
        let index = vertices
            .iter()
            .position(|other| other.distance(*point) <= tolerance)
            .unwrap_or_else(|| {
                vertices.push(*point);
                vertices.len() - 1
            });
        mapping.push(index);
    }
    let mut edges: Vec<CompressedEdge<C>> = Vec::new();
    let mut owners = Vec::<usize>::new();
    let mut samples = Vec::<Vec<Point3>>::new();
    let mut used = Vec::<bool>::new();
    let mut edge_map = Vec::new();
    for (i, edge) in input.edges.iter().enumerate() {
        let ends = (
            *mapping.get(edge.vertices.0)?,
            *mapping.get(edge.vertices.1)?,
        );
        if ends.0 == ends.1 {
            return None;
        }
        let range = edge.curve.range_tuple();
        if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
            return None;
        }
        let (_, polyline) = edge.curve.parameter_division(range, tolerance * 0.25);
        if polyline.len() < 2 || polyline.iter().any(|p| !finite(*p)) {
            return None;
        }
        let candidate = if usage[i].len() == 1 {
            edges.iter().enumerate().find_map(|(j, other)| {
                if used[j] || usage[owners[j]].len() != 1 {
                    return None;
                }
                let same = other.vertices == ends;
                if !same && other.vertices != (ends.1, ends.0) {
                    return None;
                }
                if usage[i][0] == (usage[owners[j]][0] == same) {
                    return None;
                }
                if !close(&polyline, &samples[j], tolerance * 0.5) {
                    return None;
                }
                Some((j, same))
            })
        } else {
            None
        };
        if let Some((j, same)) = candidate {
            used[j] = true;
            edge_map.push((j, same));
        } else {
            edge_map.push((edges.len(), true));
            edges.push(CompressedEdge {
                vertices: ends,
                curve: edge.curve.clone(),
            });
            owners.push(i);
            samples.push(polyline);
            used.push(false);
        }
    }
    let mut faces = input.faces.clone();
    for face in &mut faces {
        for edge in face.boundaries.iter_mut().flatten() {
            let (index, same) = edge_map[edge.index];
            edge.index = index;
            edge.orientation = edge.orientation == same;
        }
    }
    Some(CompressedShell {
        vertices,
        edges,
        faces,
    })
}

fn finite(p: Point3) -> bool { p.x.is_finite() && p.y.is_finite() && p.z.is_finite() }
fn close(a: &[Point3], b: &[Point3], tol: f64) -> bool {
    let within = |one: &[Point3], other: &[Point3]| {
        one.iter().all(|p| {
            other.windows(2).any(|w| {
                let d = w[1] - w[0];
                let length = d.magnitude2();
                let t = if length == 0.0 {
                    0.0
                } else {
                    ((*p - w[0]).dot(d) / length).clamp(0.0, 1.0)
                };
                p.distance(w[0] + d * t) <= tol
            })
        })
    };
    within(a, b) && within(b, a)
}
