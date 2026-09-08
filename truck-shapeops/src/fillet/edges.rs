use super::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};

/// Fillets selected straight edges of a closed convex planar shell at a common radius.
/// Three selected edges at a vertex receive an exact spherical corner; one selected edge
/// ends on the remaining plane. Two selected edges at an orthogonal trihedral corner meet
/// along an exact elliptical miter: the rounds are tangent to their supporting planes,
/// but meet with a crease along the miter. The third edge is only shortened, never rounded.
/// Collinear chain subdivisions are retained. Other junctions,
/// curved faces, concave shells and radii that collapse an edge return `None`. Input topology
/// and geometry are not modified. Planarity and straightness are checked at sampled points.
///
/// Planar contact faces, rational cylindrical fillets and spherical triangles are exact.
/// `tol` is the geometric validation tolerance; the tessellated Steiner-volume error is
/// bounded separately by the tessellation tolerance times the curved area.
pub fn fillet_edges<C, S>(
    shell: &Shell<Point3, C, S>,
    edges: &[EdgeID<C>],
    radius: f64,
    tol: f64,
) -> Option<Shell<Point3, C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    Line<Point3>: ToSameGeometry<C>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
    NurbsSurface<Vector4>: ToSameGeometry<S>,
    Processor<Sphere, Matrix4>: ToSameGeometry<S>,
{
    try_fillet_edges(shell, edges, radius, tol).ok()
}

/// Equal-radius edge fillets with stable failure codes and face context.
/// Unsupported junction valence returns [`Code::UnsupportedTopology`], nonorthogonal
/// two-edge corners return [`Code::UnsupportedGeometry`], and radii that leave the
/// available faces or collapse trimmed edges return [`Code::OutsideNeighbour`].
pub fn try_fillet_edges<C, S>(
    shell: &Shell<Point3, C, S>,
    edges: &[EdgeID<C>],
    radius: f64,
    tol: f64,
) -> Result<Shell<Point3, C, S>, Diagnostic>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    Line<Point3>: ToSameGeometry<C>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
    NurbsSurface<Vector4>: ToSameGeometry<S>,
    Processor<Sphere, Matrix4>: ToSameGeometry<S>,
{
    validate_tolerance(tol, "fillet_edges")?;
    let error = |code| Diagnostic::new(code, "fillet_edges", "validate_input");
    let failed = || {
        Diagnostic::new(
            Code::BlendConstructionFailed,
            "fillet_edges",
            "construct_blend",
        )
    };
    if !radius.is_finite() || radius <= 0.0 {
        return Err(error(Code::InvalidParameter).parameter("radius", radius));
    }
    if shell.shell_condition() != shell::ShellCondition::Closed {
        return Err(error(Code::InvalidInputTopology));
    }
    let selected: HashSet<_> = edges.iter().copied().collect();
    if selected.len() != edges.len() {
        return Err(error(Code::DuplicateSelection));
    }
    if edges.is_empty() {
        return Ok(shell.clone());
    }
    let convex::ConvexShell {
        vertex_index,
        vertices,
        edge_index,
        original_edges,
        planes,
        incident,
        sides,
    } = convex::validate(shell, &selected, tol, "fillet_edges")?;
    let mut centers = Vec::new();
    let mut contacts: Vec<HashMap<usize, Vertex<Point3>>> = Vec::new();
    let mut miters = vec![None; vertices.len()];
    for (i, vertex) in vertices.iter().enumerate() {
        let chosen: Vec<_> = incident[i]
            .iter()
            .copied()
            .filter(|&k| selected.contains(&original_edges[k].id()))
            .collect();
        if chosen.is_empty() {
            centers.push(vertex.point());
            contacts.push(HashMap::default());
            continue;
        }
        let face_set: HashSet<_> = incident[i].iter().flat_map(|&k| sides[k]).collect();
        let mut faces: Vec<_> = face_set.into_iter().collect();
        faces.sort_unstable();
        let touched: HashSet<_> = chosen.iter().flat_map(|&k| sides[k]).collect();
        let mut equations: Vec<_> = faces
            .iter()
            .map(|&f| {
                let (n, d) = planes[f];
                (n, d - if touched.contains(&f) { radius } else { 0.0 })
            })
            .collect();
        match (chosen.len(), incident[i].len(), faces.len()) {
            (1, 3, 3) | (3, 3, 3) => {}
            (2, 3, 3) => {
                if faces
                    .iter()
                    .array_combinations()
                    .any(|[&a, &b]| planes[a].0.dot(planes[b].0).abs() > TOLERANCE)
                {
                    return Err(error(Code::UnsupportedGeometry));
                }
                let away = |k: usize| {
                    let e = &original_edges[k];
                    (if e.front() == vertex {
                        e.back().point() - vertex.point()
                    } else {
                        e.front().point() - vertex.point()
                    })
                    .normalize()
                };
                miters[i] = Some((away(chosen[0]) - away(chosen[1])).normalize());
            }
            (2, 2, 2) => {
                let away = |k: usize| {
                    let e = &original_edges[k];
                    if e.front() == vertex {
                        e.back().point() - vertex.point()
                    } else {
                        e.front().point() - vertex.point()
                    }
                };
                let axis = away(chosen[0]).normalize();
                if !axis.near(&(-away(chosen[1]).normalize())) {
                    return Err(error(Code::UnsupportedTopology));
                }
                equations.push((axis, axis.dot(vertex.point().to_vec())));
            }
            _ => return Err(error(Code::UnsupportedTopology)),
        }
        let center = Point3::from_vec(
            Matrix3::from_cols(equations[0].0, equations[1].0, equations[2].0)
                .transpose()
                .invert()
                .ok_or_else(failed)?
                * Vector3::new(equations[0].1, equations[1].1, equations[2].1),
        );
        for (f, &(n, d)) in planes.iter().enumerate() {
            let offset = if faces.contains(&f) && !touched.contains(&f) {
                0.0
            } else {
                radius
            };
            if n.dot(center.to_vec()) > d - offset + TOLERANCE {
                return Err(error(Code::OutsideNeighbour).parameter("radius", radius));
            }
        }
        contacts.push(
            touched
                .into_iter()
                .map(|f| (f, Vertex::new(center + radius * planes[f].0)))
                .collect(),
        );
        if miters[i].is_some() {
            let shared = sides[chosen[0]]
                .into_iter()
                .find(|f| sides[chosen[1]].contains(f))
                .ok_or_else(failed)?;
            let tip = Vertex::new(vertex.point() - radius * planes[shared].0);
            for &f in &faces {
                if f != shared {
                    contacts[i].insert(f, tip.clone());
                }
            }
        }
        centers.push(center);
    }
    let mut trims: Vec<[Edge<Point3, C>; 2]> = Vec::new();
    let mut arcs: Vec<Option<[Edge<Point3, C>; 2]>> = Vec::new();
    let mut fillets = Vec::new();
    let mut arc_map = HashMap::default();
    for (k, edge) in original_edges.iter().enumerate() {
        let [a, b] = sides[k];
        let [v0, v1] = [edge.front(), edge.back()].map(|v| vertex_index[&v.id()]);
        let point = |v: usize, face: usize, other: usize| {
            contacts[v]
                .get(&face)
                .or_else(|| contacts[v].get(&other))
                .unwrap_or(&vertices[v])
                .clone()
        };
        let make_line = |p: &Vertex<Point3>, q: &Vertex<Point3>| {
            let direction = q.point() - p.point();
            if direction.dot(edge.back().point() - edge.front().point()) <= TOLERANCE2 {
                return Err(error(Code::OutsideNeighbour).parameter("radius", radius));
            }
            Ok(Edge::new(
                p,
                q,
                Line(p.point(), q.point()).to_same_geometry(),
            ))
        };
        if !selected.contains(&edge.id()) {
            let line = make_line(&point(v0, a, b), &point(v1, a, b))?;
            trims.push([line.clone(), line]);
            arcs.push(None);
            continue;
        }
        let [a0, b0, a1, b1] = [
            contacts[v0][&a].clone(),
            contacts[v0][&b].clone(),
            contacts[v1][&a].clone(),
            contacts[v1][&b].clone(),
        ];
        let line_a = make_line(&a0, &a1)?;
        let line_b = make_line(&b0, &b1)?;
        let arc_curve = |v: usize| {
            let center = centers[v];
            let (n, m) = (planes[a].0, planes[b].0);
            let w = ((1.0 + n.dot(m)) / 2.0).sqrt();
            let middle = center + radius * (n + m) / (1.0 + n.dot(m));
            let mut curve = NurbsCurve::new(BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![
                    (center + radius * n).to_vec().extend(1.0),
                    middle.to_vec().extend(1.0) * w,
                    (center + radius * m).to_vec().extend(1.0),
                ],
            ));
            if let Some(normal) = miters[v] {
                let axis = (edge.back().point() - edge.front().point()).normalize();
                // Project the circular section along its cylinder axis onto the miter plane.
                for p in curve.control_points_mut() {
                    let distance = normal.dot(p.truncate() - vertices[v].point().to_vec() * p.w);
                    *p -= (axis * (distance / normal.dot(axis))).extend(0.0);
                }
            }
            curve
        };
        let curve0 = arc_curve(v0);
        let curve1 = arc_curve(v1);
        let mut make_arc = |v: usize,
                            front: &Vertex<Point3>,
                            back: &Vertex<Point3>,
                            curve: &NurbsCurve<Vector4>| {
            let pair = miters[v].is_none().then_some((a.min(b), a.max(b)));
            let edge = arc_map
                .entry((v, pair))
                .or_insert_with(|| Edge::new(front, back, curve.to_same_geometry()));
            if edge.front() == front {
                edge.clone()
            } else {
                edge.inverse()
            }
        };
        let arc0 = make_arc(v0, &a0, &b0, &curve0);
        let arc1 = make_arc(v1, &a1, &b1, &curve1);
        let surface = NurbsSurface::new(BSplineSurface::new(
            (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
            curve0
                .control_points()
                .iter()
                .zip(curve1.control_points())
                .map(|(&p, &q)| vec![p, q])
                .collect(),
        ));
        fillets.push(Face::new(
            vec![wire![
                line_a.inverse(),
                arc0.clone(),
                line_b.clone(),
                arc1.inverse()
            ]],
            surface.to_same_geometry(),
        ));
        trims.push([line_a, line_b]);
        arcs.push(Some([arc0, arc1]));
    }
    let mut result = Shell::new();
    for (f, face) in shell.iter().enumerate() {
        let boundary = face.boundaries().pop().ok_or_else(failed)?;
        let lines: Vec<_> = boundary
            .iter()
            .map(|edge| {
                let k = edge_index[&edge.id()];
                let line = &trims[k][usize::from(sides[k][1] == f)];
                if edge.orientation() {
                    line.clone()
                } else {
                    line.inverse()
                }
            })
            .collect();
        let mut wire = Wire::new();
        for i in 0..lines.len() {
            wire.push_back(lines[i].clone());
            let next = &lines[(i + 1) % lines.len()];
            if lines[i].back() == next.front() {
                continue;
            }
            let v = vertex_index[&boundary[i].back().id()];
            let k = *incident[v]
                .iter()
                .find(|&&k| arcs[k].is_some())
                .ok_or_else(failed)?;
            let end = usize::from(original_edges[k].back() == &vertices[v]);
            let arc = &arcs[k].as_ref().ok_or_else(failed)?[end];
            let arc = if arc.front() == lines[i].back() {
                arc.clone()
            } else {
                arc.inverse()
            };
            if arc.back() != next.front() {
                return Err(failed());
            }
            wire.push_back(arc);
        }
        result.push(
            Face::try_new(vec![wire], face.oriented_surface()).map_err(|e| {
                Diagnostic::new(
                    Code::InvalidOutputTopology,
                    "fillet_edges",
                    "validate_output",
                )
                .face(f)
                .with_coded_source(e)
            })?,
        );
    }
    result.extend(fillets);
    for (v, vertex) in vertices.iter().enumerate() {
        if contacts[v].len() != 3 || miters[v].is_some() {
            continue;
        }
        let mut remaining: Vec<_> = incident[v]
            .iter()
            .map(|&k| {
                let end = usize::from(original_edges[k].back() == vertex);
                let arc = &arcs[k].as_ref().unwrap()[end];
                if end == 0 {
                    arc.inverse()
                } else {
                    arc.clone()
                }
            })
            .collect();
        let mut boundary = wire![remaining.pop().ok_or_else(failed)?];
        while !remaining.is_empty() {
            let i = remaining
                .iter()
                .position(|edge| Some(edge.front()) == boundary.back_vertex())
                .ok_or_else(failed)?;
            boundary.push_back(remaining.remove(i));
        }
        let normals: Vec<_> = contacts[v].keys().map(|&f| planes[f].0).collect();
        let y = (Matrix3::from_cols(normals[0], normals[1], normals[2])
            .transpose()
            .invert()
            .ok_or_else(failed)?
            * Vector3::new(1.0, 1.0, 1.0))
        .normalize();
        let z = normals[0].cross(y).normalize();
        let x = y.cross(z);
        let transform = Matrix4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            centers[v].to_vec().extend(1.0),
        );
        let sphere = Processor::new(Sphere::new(Point3::origin(), radius)).transformed(transform);
        result.push(
            Face::try_new(vec![boundary], sphere.to_same_geometry()).map_err(|e| {
                Diagnostic::new(
                    Code::InvalidOutputTopology,
                    "fillet_edges",
                    "validate_output",
                )
                .with_coded_source(e)
            })?,
        );
    }
    (result.shell_condition() == shell::ShellCondition::Closed)
        .then_some(result)
        .ok_or_else(|| {
            Diagnostic::new(
                Code::InvalidOutputTopology,
                "fillet_edges",
                "validate_output",
            )
        })
}
