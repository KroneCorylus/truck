use super::*;
use std::collections::HashMap;

/// Edges of a boundary loop after trimming: one entry per original edge, `None` to keep it.
type LoopReplacement<C> = Vec<Option<Vec<Edge<Point3, C>>>>;

/// A maximal sequence of chain edges met consecutively in one boundary loop of one face.
struct Run {
    face: usize,
    boundary: usize,
    /// index in the loop of the first chain edge
    start: usize,
    /// chain indices in loop order
    chain: Vec<usize>,
    /// whether the loop edges run along the chain (side 0) or against it (side 1)
    side: usize,
}

impl Run {
    fn whole(&self, len: usize) -> bool { self.chain.len() == len }
    /// Chain vertex index at the loop start and end of this run.
    fn ends(&self, nv: usize) -> (usize, usize) {
        let (first, last) = (self.chain[0], self.chain[self.chain.len() - 1]);
        match self.side {
            0 => (first, (last + 1) % nv),
            _ => ((first + 1) % nv, last),
        }
    }
}

fn find_runs<C: Clone, S>(
    face_idx: usize,
    face: &Face<Point3, C, S>,
    wire: &Wire<Point3, C>,
    chain_index: &HashMap<EdgeID<C>, usize>,
) -> Option<Vec<Run>> {
    let mut runs = Vec::new();
    for (boundary_idx, boundary) in face.boundaries().into_iter().enumerate() {
        let flags: Vec<Option<(usize, usize)>> = boundary
            .iter()
            .map(|edge| {
                let k = *chain_index.get(&edge.id())?;
                let side = if edge.front() == wire[k].front() {
                    0
                } else {
                    1
                };
                Some((k, side))
            })
            .collect();
        let len = flags.len();
        let start = (0..len)
            .find(|&i| flags[i].is_some() && flags[(i + len - 1) % len].is_none())
            .or(flags[0].map(|_| 0));
        let Some(start) = start else { continue };
        let mut current: Option<Run> = None;
        for i in 0..len {
            let idx = (start + i) % len;
            match (flags[idx], &mut current) {
                (Some((k, side)), Some(run)) => {
                    let expected = match run.side {
                        0 => (run.chain[run.chain.len() - 1] + 1) % wire.len(),
                        _ => (run.chain[run.chain.len() - 1] + wire.len() - 1) % wire.len(),
                    };
                    if side != run.side || k != expected {
                        return None;
                    }
                    run.chain.push(k);
                }
                (Some((k, side)), None) => {
                    current = Some(Run {
                        face: face_idx,
                        boundary: boundary_idx,
                        start: idx,
                        chain: vec![k],
                        side,
                    });
                }
                (None, Some(_)) => runs.push(current.take().unwrap()),
                (None, None) => {}
            }
        }
        runs.extend(current);
    }
    Some(runs)
}

fn is_tangent_continuous<C: ParametricCurve3D + BoundedCurve + Invertible>(
    wire: &Wire<Point3, C>,
    closed: bool,
) -> bool {
    let tangent = |edge: &Edge<Point3, C>, at_end: bool| {
        let curve = edge.oriented_curve();
        let (t0, t1) = curve.range_tuple();
        curve.der(if at_end { t1 } else { t0 }).normalize()
    };
    let pairs = wire
        .iter()
        .tuple_windows()
        .map(|(e0, e1)| (e0.clone(), e1.clone()));
    let wrap = closed.then(|| {
        (
            wire.back_edge().unwrap().clone(),
            wire.front_edge().unwrap().clone(),
        )
    });
    pairs
        .chain(wrap)
        .all(|(e0, e1)| tangent(&e0, true).near(&tangent(&e1, false)))
}

fn trim_to_point<C: FilletedCurve<S>, S>(
    curve: &mut C,
    point: Point3,
    at_start: bool,
) -> Option<()> {
    let (t0, t1) = curve.range_tuple();
    let hint = if at_start { t0 } else { t1 };
    let t = curve.search_nearest_parameter(point, Some(hint), 100)?;
    if !t.near(&hint) {
        if at_start {
            *curve = curve.cut(t);
        } else {
            curve.cut(t);
        }
    }
    Some(())
}

/// The edges leaving the chain that get cut, and the resulting vertices, per chain vertex and
/// side. Cuts are shared between the two faces of a cut edge.
struct Cuts<C> {
    vertices: Vec<[Option<Vertex<Point3>>; 2]>,
    ders: Vec<[Option<Vector3>; 2]>,
    /// the kept piece of each cut edge, oriented like the underlying edge
    pieces: HashMap<EdgeID<C>, Edge<Point3, C>>,
}

impl<C: Clone> Cuts<C> {
    /// The kept piece of `edge`, oriented like `edge`.
    fn piece(&self, edge: &Edge<Point3, C>) -> Option<Edge<Point3, C>> {
        let piece = self.pieces.get(&edge.id())?;
        Some(match edge.orientation() {
            true => piece.clone(),
            false => piece.inverse(),
        })
    }
    fn insert(&mut self, edge: &Edge<Point3, C>, piece: Edge<Point3, C>) -> Option<()> {
        let absolute = match edge.orientation() {
            true => piece,
            false => piece.inverse(),
        };
        match self.pieces.insert(edge.id(), absolute) {
            None => Some(()),
            Some(_) => None,
        }
    }
}

/// Fillets a tangent-continuous chain of edges of `shell` with constant `radius`.
///
/// `wire` must be continuous and simple, and consecutive edges must share their tangent at the
/// common vertex. It may be closed. Every edge of the chain must be shared by two faces of the
/// shell, and at every interior vertex of the chain the faces on each side must either be the
/// same face or be separated by exactly one edge. An open chain must end at vertices adjacent to
/// exactly three faces, as for [`fillet_with_side`].
///
/// Returns the shell with the faces along the chain trimmed and one fillet face per chain edge
/// appended, in chain order. The fillet faces share their cross edges, so the blend is tangent
/// continuous along the chain up to `tol`. Returns `None` if the chain does not satisfy the
/// conditions above or if a contact curve cannot be found.
pub fn fillet_along_wire<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    radius: f64,
    tol: f64,
) -> Option<Shell<Point3, C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
{
    let n = wire.len();
    if n == 0 || !wire.is_continuous() || !wire.is_simple() {
        return None;
    }
    let closed = wire.is_cyclic();
    if !is_tangent_continuous(wire, closed) {
        return None;
    }
    let nv = if closed { n } else { n + 1 };
    let chain_index: HashMap<EdgeID<C>, usize> = wire
        .iter()
        .enumerate()
        .map(|(k, edge)| (edge.id(), k))
        .collect();

    let runs: Vec<Run> = shell
        .iter()
        .enumerate()
        .map(|(idx, face)| find_runs(idx, face, wire, &chain_index))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    let mut sides = vec![[None; 2]; n];
    for run in &runs {
        for &k in &run.chain {
            if sides[k][run.side].replace(run.face).is_some() {
                return None;
            }
        }
    }
    let sides: Vec<[usize; 2]> = sides
        .into_iter()
        .map(|[s0, s1]| Some([s0?, s1?]))
        .collect::<Option<_>>()?;

    let boundaries: Vec<Vec<Wire<Point3, C>>> =
        shell.iter().map(|face| face.boundaries()).collect();
    // The loop edge next to the run at the chain vertex `j` on side `side`, if the run ends there.
    let adjacent_edge = |j: usize, side: usize| -> Option<Edge<Point3, C>> {
        runs.iter().find_map(|run| {
            let len = boundaries[run.face][run.boundary].len();
            if run.side != side || run.whole(len) {
                return None;
            }
            let (start, end) = run.ends(nv);
            let boundary = &boundaries[run.face][run.boundary];
            let m = run.chain.len();
            if start == j {
                Some(boundary[(run.start + len - 1) % len].clone())
            } else if end == j {
                Some(boundary[(run.start + m) % len].clone())
            } else {
                None
            }
        })
    };

    let mut surfaces = Vec::with_capacity(n);
    for k in 0..n {
        let rbf = RbfSurface::new(
            wire[k].oriented_curve(),
            shell[sides[k][0]].oriented_surface(),
            shell[sides[k][1]].oriented_surface(),
            radius,
        );
        let (t0, t1) = rbf.edge_curve().range_tuple();
        let (mut v0, mut v1) = (t0, t1);
        if !closed && k == 0 {
            let vertex = wire.front_vertex()?;
            let (curve0, hint0) = adjacent_hint(&adjacent_edge(0, 0)?, vertex);
            let (curve1, hint1) = adjacent_hint(&adjacent_edge(0, 1)?, vertex);
            let (_, _, v00, _) =
                rbf.search_contact_curve0_cross_point_with_adjacent_edge(t0, &curve0, hint0, 100)?;
            let (_, _, v01, _) =
                rbf.search_contact_curve1_cross_point_with_adjacent_edge(t0, &curve1, hint1, 100)?;
            v0 = v00.min(v01);
        }
        if !closed && k == n - 1 {
            let vertex = wire.back_vertex()?;
            let (curve0, hint0) = adjacent_hint(&adjacent_edge(n, 0)?, vertex);
            let (curve1, hint1) = adjacent_hint(&adjacent_edge(n, 1)?, vertex);
            let (_, _, v10, _) =
                rbf.search_contact_curve0_cross_point_with_adjacent_edge(t1, &curve0, hint0, 100)?;
            let (_, _, v11, _) =
                rbf.search_contact_curve1_cross_point_with_adjacent_edge(t1, &curve1, hint1, 100)?;
            v1 = v10.max(v11);
        }
        surfaces.push(ApproxFilletSurface::approx_rolling_ball_fillet(
            &rbf,
            (v0, v1),
            tol,
        )?);
    }

    let mut cuts = Cuts {
        vertices: vec![[None, None]; nv],
        ders: vec![[None, None]; nv],
        pieces: HashMap::new(),
    };
    let mut contacts: Vec<[Option<Edge<Point3, C>>; 2]> = vec![[None, None]; n];
    let mut faces: Vec<Face<Point3, C, S>> = shell.iter().cloned().collect();
    // Loops touched by runs, as a replacement list per original edge index.
    let mut new_loops: HashMap<(usize, usize), LoopReplacement<C>> = HashMap::new();

    for run in &runs {
        let side = run.side;
        let boundary = &boundaries[run.face][run.boundary];
        let len = boundary.len();
        let m = run.chain.len();
        let whole = run.whole(len);
        if !whole && len < m + 2 {
            return None;
        }
        let mut curves: Vec<C> = run
            .chain
            .iter()
            .map(|&k| match side {
                0 => surfaces[k].side_pcurve0().to_same_geometry(),
                _ => surfaces[k].side_pcurve1().to_same_geometry().inverse(),
            })
            .collect();
        let (start_vertex, end_vertex) = run.ends(nv);

        let mut piece_prev = None;
        let mut piece_next = None;
        if !whole {
            let prev = &boundary[(run.start + len - 1) % len];
            let next = &boundary[(run.start + m) % len];
            piece_prev = Some(match cuts.vertices[start_vertex][side].clone() {
                Some(vertex) => {
                    let piece = cuts.piece(prev)?;
                    (piece.back() == &vertex).then_some(())?;
                    trim_to_point(&mut curves[0], vertex.point(), true)?;
                    piece
                }
                None => {
                    let (piece, der) = cut_at_front(&mut curves[0], prev)?;
                    cuts.insert(prev, piece.clone())?;
                    cuts.vertices[start_vertex][side] = Some(piece.back().clone());
                    cuts.ders[start_vertex][side] = Some(der);
                    piece
                }
            });
            piece_next = Some(match cuts.vertices[end_vertex][side].clone() {
                Some(vertex) => {
                    let piece = cuts.piece(next)?;
                    (piece.front() == &vertex).then_some(())?;
                    trim_to_point(&mut curves[m - 1], vertex.point(), false)?;
                    piece
                }
                None => {
                    let (piece, der) = cut_at_back(&mut curves[m - 1], next)?;
                    cuts.insert(next, piece.clone())?;
                    cuts.vertices[end_vertex][side] = Some(piece.front().clone());
                    cuts.ders[end_vertex][side] = Some(der);
                    piece
                }
            });
        }
        // Interior junctions of the run.
        let junctions = if whole { m } else { m - 1 };
        for i in 0..junctions {
            let (c0, c1) = (&curves[i], &curves[(i + 1) % m]);
            let p0 = c0.subs(c0.range_tuple().1);
            let p1 = c1.subs(c1.range_tuple().0);
            let j = match side {
                0 => (run.chain[i] + 1) % nv,
                _ => run.chain[i],
            };
            if cuts.vertices[j][side]
                .replace(Vertex::new(p0.midpoint(p1)))
                .is_some()
            {
                return None;
            }
        }

        let vertex_at = |i: usize, at_start: bool| -> Vertex<Point3> {
            let k = run.chain[i];
            let j = match (side, at_start) {
                (0, true) | (1, false) => k,
                _ => (k + 1) % nv,
            };
            cuts.vertices[j][side].clone().unwrap()
        };
        let mut loop_edges = Vec::with_capacity(m);
        for (i, curve) in curves.into_iter().enumerate() {
            let k = run.chain[i];
            let (a, b) = (vertex_at(i, true), vertex_at(i, false));
            let (contact, loop_edge) = match side {
                0 => {
                    let edge = Edge::new(&a, &b, curve);
                    (edge.clone(), edge)
                }
                _ => {
                    let edge = Edge::new(&b, &a, curve.inverse());
                    (edge.clone(), edge.inverse())
                }
            };
            contacts[k][side] = Some(contact);
            loop_edges.push(loop_edge);
        }

        let replacement = new_loops
            .entry((run.face, run.boundary))
            .or_insert_with(|| vec![None; len]);
        let mut set = |idx: usize, edges: Vec<Edge<Point3, C>>| -> Option<()> {
            match replacement[idx % len].replace(edges) {
                None => Some(()),
                Some(_) => None,
            }
        };
        if let Some(piece) = piece_prev {
            set(run.start + len - 1, vec![piece])?;
        }
        set(run.start, loop_edges)?;
        for i in 1..m {
            set(run.start + i, Vec::new())?;
        }
        if let Some(piece) = piece_next {
            set(run.start + m, vec![piece])?;
        }
    }

    for ((face_idx, boundary_idx), replacement) in new_loops {
        let mut loops = boundaries[face_idx].clone();
        loops[boundary_idx] = replacement
            .into_iter()
            .enumerate()
            .flat_map(|(idx, edges)| {
                edges.unwrap_or_else(|| vec![boundaries[face_idx][boundary_idx][idx].clone()])
            })
            .collect();
        faces[face_idx] = Face::new(loops, shell[face_idx].oriented_surface());
    }

    let contacts: Vec<[Edge<Point3, C>; 2]> = contacts
        .into_iter()
        .map(|[c0, c1]| Some([c0?, c1?]))
        .collect::<Option<_>>()?;
    let vertices: Vec<[Vertex<Point3>; 2]> = cuts
        .vertices
        .iter()
        .map(|[v0, v1]| Some([v0.clone()?, v1.clone()?]))
        .collect::<Option<_>>()?;
    let blend_surfaces: Vec<S> = surfaces.iter().map(|af| af.to_same_geometry()).collect();

    let mut cross: Vec<Edge<Point3, C>> = Vec::with_capacity(nv);
    let mut end_edge = None;
    for j in 0..nv {
        let interior = closed || (0 < j && j < n);
        let edge = if interior {
            let k = (j + n - 1) % n;
            let v = surfaces[k].range_tuple().1 .1;
            let curve = surfaces[k].fillet_bezier(v).to_same_geometry();
            Edge::new(&vertices[j][0], &vertices[j][1], curve)
        } else if j == 0 {
            let ((u0, u1), (v0, _)) = surfaces[0].range_tuple();
            create_pcurve_edge(
                (&vertices[0][0], (u0, v0), cuts.ders[0][0]?),
                (&vertices[0][1], (u1, v0), cuts.ders[0][1]?),
                blend_surfaces[0].clone(),
            )?
        } else {
            let ((u0, u1), (_, v1)) = surfaces[n - 1].range_tuple();
            let edge = create_pcurve_edge(
                (&vertices[n][1], (u1, v1), cuts.ders[n][1]?),
                (&vertices[n][0], (u0, v1), cuts.ders[n][0]?),
                blend_surfaces[n - 1].clone(),
            )?;
            end_edge = Some(edge.clone());
            edge.inverse()
        };
        cross.push(edge);
    }

    let blends: Vec<Face<Point3, C, S>> = (0..n)
        .map(|k| {
            let boundary = [
                contacts[k][0].inverse(),
                cross[k].clone(),
                contacts[k][1].clone(),
                cross[(k + 1) % nv].inverse(),
            ];
            Face::new(vec![boundary.into()], blend_surfaces[k].clone())
        })
        .collect();

    if !closed {
        let end_face = |j: usize| -> Option<usize> {
            let edge0 = adjacent_edge(j, 0)?;
            let edge1 = adjacent_edge(j, 1)?;
            let k = if j == 0 { 0 } else { n - 1 };
            let face_idx = shell.iter().position(|face| {
                let contains =
                    |edge: &Edge<Point3, C>| face.edge_iter().any(|e| e.id() == edge.id());
                contains(&edge0) && contains(&edge1)
            })?;
            (face_idx != sides[k][0] && face_idx != sides[k][1]).then_some(face_idx)
        };
        let piece = |j: usize, side: usize| cuts.piece(&adjacent_edge(j, side)?);
        let idx = end_face(0)?;
        faces[idx] = create_new_side(
            &faces[idx],
            &cross[0],
            &blend_surfaces[0],
            wire.front_vertex()?.id(),
            &piece(0, 0)?,
            &piece(0, 1)?,
        )?;
        let idx = end_face(n)?;
        faces[idx] = create_new_side(
            &faces[idx],
            end_edge.as_ref()?,
            &blend_surfaces[n - 1],
            wire.back_vertex()?.id(),
            &piece(n, 1)?,
            &piece(n, 0)?,
        )?;
    }

    faces.extend(blends);
    Some(faces.into())
}

/// The oriented curve of an edge next to the chain and the parameter of its end at `vertex`.
fn adjacent_hint<C: ParametricCurve3D + BoundedCurve + Invertible>(
    edge: &Edge<Point3, C>,
    vertex: &Vertex<Point3>,
) -> (C, f64) {
    let curve = edge.oriented_curve();
    let (t0, t1) = curve.range_tuple();
    let hint = if edge.back() == vertex { t1 } else { t0 };
    (curve, hint)
}
