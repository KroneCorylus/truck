use super::*;
use std::collections::HashMap;

/// Edges of a boundary loop after trimming: one entry per original edge, `None` to keep it.
type LoopReplacement<C> = Vec<Option<Vec<Edge<Point3, C>>>>;

/// The radius function of the wire restricted to chain edge `k`: `t ↦ R(k + (t − t0) · scale)`.
#[derive(Clone, Debug)]
struct EdgeRadius<R> {
    radius: R,
    k: f64,
    t0: f64,
    scale: f64,
}

impl<R: ScalarFunctionD1> ScalarFunctionD1 for EdgeRadius<R> {
    fn der_n(&self, n: usize, t: f64) -> f64 {
        let t = self.k + (t - self.t0) * self.scale;
        self.radius.der_n(n, t) * self.scale.powi(n as i32)
    }
}

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

pub(super) fn is_tangent_continuous<C: ParametricCurve3D + BoundedCurve + Invertible>(
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

/// Fillets a tangent-continuous chain of edges of `shell` with `radius`, a function of the wire
/// parameter `t ∈ [0, n]`, where edge `k` of the `n` edges covers `[k, k + 1]` in its own
/// parameter scaled to unit length. A constant `f64` is such a function. The rolling ball uses
/// derivatives of the radius up to second order, so the function must supply them. At the open
/// ends of a chain the fillet is extended to meet the end faces, and the radius is evaluated a
/// little beyond `[0, n]`.
///
/// `wire` must be continuous and simple, and consecutive edges must share their tangent at the
/// common vertex. It may be closed. Every edge of the chain must be shared by two faces of the
/// shell, and at every interior vertex of the chain the faces on each side must either be the
/// same face or be separated by exactly one edge. An open chain may end at vertices with any
/// number of faces: the fillet is trimmed against every face between the two side faces, and the
/// edges between those faces are cut where they meet the fillet. Blending several fillets that
/// meet at a vertex is not supported.
///
/// Returns the shell with the faces along the chain trimmed and one fillet face per chain edge
/// appended, in chain order. The fillet faces share their cross edges, so the blend is tangent
/// continuous along the chain up to `tol`. Returns `None` if the chain does not satisfy the
/// conditions above or if a contact curve cannot be found.
pub fn fillet_along_wire<C, S, R>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    radius: R,
    tol: f64,
) -> Option<Shell<Point3, C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    R: ScalarFunctionD1,
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

    let rbfs: Vec<_> = (0..n)
        .map(|k| {
            let curve = wire[k].oriented_curve();
            let (t0, t1) = curve.range_tuple();
            RbfSurface::new(
                curve,
                shell[sides[k][0]].oriented_surface(),
                shell[sides[k][1]].oriented_surface(),
                EdgeRadius {
                    radius: radius.clone(),
                    k: k as f64,
                    t0,
                    scale: 1.0 / (t1 - t0),
                },
            )
        })
        .collect();
    let mut ranges: Vec<(f64, f64)> = rbfs
        .iter()
        .map(|rbf| rbf.edge_curve().range_tuple())
        .collect();
    // For an open chain, walk around each end vertex and extend the parameter range of the end
    // fillet to where the contact curves and the fillet meet the edges leaving the chain.
    let mut ends: [Option<EndWalk<C>>; 2] = [None, None];
    if !closed {
        for (which, j, k) in [(0, 0, 0), (1, n, n - 1)] {
            let vertex = if j == 0 {
                wire.front_vertex()?
            } else {
                wire.back_vertex()?
            };
            let mut walk = walk_around_vertex(
                shell,
                sides[k],
                vertex,
                adjacent_edge(j, 0)?,
                &adjacent_edge(j, 1)?,
            )?;
            let rbf = &rbfs[k];
            let t = if j == 0 { ranges[k].0 } else { ranges[k].1 };
            let (curve0, hint0) = adjacent_hint(&walk.edges[0], vertex);
            let (curve1, hint1) = adjacent_hint(&walk.edges[walk.faces.len()], vertex);
            let (_, _, v0, _) =
                rbf.search_contact_curve0_cross_point_with_adjacent_edge(t, &curve0, hint0, 100)?;
            let (_, _, v1, _) =
                rbf.search_contact_curve1_cross_point_with_adjacent_edge(t, &curve1, hint1, 100)?;
            let mut vs = vec![v0, v1];
            for edge in &walk.edges[1..walk.faces.len()] {
                let curve = edge.curve();
                let (s0, s1) = curve.range_tuple();
                let hint = if curve.subs(s0).near(&vertex.point()) {
                    s0
                } else {
                    s1
                };
                let ((u, v), s) =
                    algo::surface::search_intersection_parameter(rbf, (0.5, t), &curve, hint, 100)?;
                walk.crossings.push((Vertex::new(curve.subs(s)), s, (u, v)));
                vs.push(v);
            }
            if j == 0 {
                ranges[k].0 = vs.into_iter().fold(f64::INFINITY, f64::min);
            } else {
                ranges[k].1 = vs.into_iter().fold(f64::NEG_INFINITY, f64::max);
            }
            ends[which] = Some(walk);
        }
    }
    let mut surfaces = Vec::with_capacity(n);
    for k in 0..n {
        surfaces.push(ApproxFilletSurface::approx_rolling_ball_fillet(
            &rbfs[k], ranges[k], tol,
        )?);
    }

    let contact_curves = surfaces
        .iter()
        .map(|surface| {
            [
                surface.side_pcurve0().to_same_geometry(),
                surface.side_pcurve1().to_same_geometry(),
            ]
        })
        .collect::<Vec<[C; 2]>>();
    let TrimmedChain {
        mut faces,
        contacts,
        vertices,
        cuts,
    } = trim_runs(shell, wire, &runs, &contact_curves)?;
    let blend_surfaces: Vec<S> = surfaces.iter().map(|af| af.to_same_geometry()).collect();

    let mut cross: Vec<Vec<Edge<Point3, C>>> = vec![Vec::new(); nv];
    for (j, edges) in cross.iter_mut().enumerate() {
        if closed || (0 < j && j < n) {
            let k = (j + n - 1) % n;
            let v = surfaces[k].range_tuple().1 .1;
            let curve = surfaces[k].fillet_bezier(v).to_same_geometry();
            *edges = vec![Edge::new(&vertices[j][0], &vertices[j][1], curve)];
        }
    }

    if !closed {
        for (which, j, k) in [(0, 0, 0), (1, n, n - 1)] {
            let walk = ends[which].take()?;
            let vertex = if j == 0 {
                wire.front_vertex()?
            } else {
                wire.back_vertex()?
            };
            let m = walk.faces.len();
            let ((u0, u1), (v0, v1)) = surfaces[k].range_tuple();
            let v_end = if j == 0 { v0 } else { v1 };
            // Points of the cross edge from side 0 to side 1 with their parameters on the fillet,
            // and the kept piece of the edge leaving the chain at each of them.
            let mut points = vec![(vertices[j][0].clone(), (u0, v_end))];
            let mut pieces = vec![cuts.piece(&walk.edges[0])?];
            for (i, (w, s, uv)) in walk.crossings.iter().enumerate() {
                let edge = &walk.edges[i + 1];
                let uv = surfaces[k].search_parameter(w.point(), Some(*uv), 100)?;
                points.push((w.clone(), uv));
                let (front, back) = edge.cut_with_parameter(w, *s)?;
                pieces.push(if edge.back() == vertex { front } else { back });
            }
            points.push((vertices[j][1].clone(), (u1, v_end)));
            pieces.push(cuts.piece(&walk.edges[m])?);

            let blend = &blend_surfaces[k];
            for i in 0..m {
                let idx = walk.faces[i];
                let face_surface = shell[idx].oriented_surface();
                let ((w0, uv0), (w1, uv1)) = (&points[i], &points[i + 1]);
                let direction = w1.point() - w0.point();
                let der0 = cross_tangent(blend, *uv0, &face_surface, w0.point(), direction)?;
                let der1 = cross_tangent(blend, *uv1, &face_surface, w1.point(), direction)?;
                let piece = create_pcurve_edge((w0, *uv0, der0), (w1, *uv1, der1), blend.clone())?;
                let (fillet_edge, left, right) = match j {
                    0 => (
                        piece.clone(),
                        toward(&pieces[i], w0, true),
                        toward(&pieces[i + 1], w1, false),
                    ),
                    _ => (
                        piece.inverse(),
                        toward(&pieces[i + 1], w1, true),
                        toward(&pieces[i], w0, false),
                    ),
                };
                faces[idx] =
                    create_new_side(&faces[idx], &fillet_edge, blend, vertex.id(), &left, &right)?;
                cross[j].push(piece);
            }
        }
    }

    let blends: Vec<Face<Point3, C, S>> = (0..n)
        .map(|k| {
            let boundary: Vec<_> = std::iter::once(contacts[k][0].inverse())
                .chain(cross[k].iter().cloned())
                .chain(std::iter::once(contacts[k][1].clone()))
                .chain(cross[(k + 1) % nv].iter().rev().map(Edge::inverse))
                .collect();
            Face::new(vec![boundary.into()], blend_surfaces[k].clone())
        })
        .collect();

    faces.extend(blends);
    Some(faces.into())
}

pub(super) struct TrimmedChain<C, S> {
    pub(super) faces: Vec<Face<Point3, C, S>>,
    pub(super) contacts: Vec<[Edge<Point3, C>; 2]>,
    pub(super) vertices: Vec<[Vertex<Point3>; 2]>,
    cuts: Cuts<C>,
}

pub(super) fn trim_closed_chain<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    contact_curves: &[[C; 2]],
) -> Option<TrimmedChain<C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
{
    let chain_index = wire.iter().enumerate().map(|(k, e)| (e.id(), k)).collect();
    let runs = shell
        .iter()
        .enumerate()
        .map(|(i, f)| find_runs(i, f, wire, &chain_index))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    trim_runs(shell, wire, &runs, contact_curves)
}

fn trim_runs<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    runs: &[Run],
    contact_curves: &[[C; 2]],
) -> Option<TrimmedChain<C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
{
    let n = wire.len();
    let nv = if wire.is_cyclic() { n } else { n + 1 };
    let boundaries: Vec<_> = shell.iter().map(|face| face.boundaries()).collect();
    let mut cuts = Cuts {
        vertices: vec![[None, None]; nv],
        pieces: HashMap::new(),
    };
    let mut contacts: Vec<[Option<Edge<Point3, C>>; 2]> = vec![[None, None]; n];
    let mut faces: Vec<Face<Point3, C, S>> = shell.iter().cloned().collect();
    // Loops touched by runs, as a replacement list per original edge index.
    let mut new_loops: HashMap<(usize, usize), LoopReplacement<C>> = HashMap::new();

    for run in runs {
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
                0 => contact_curves[k][0].clone(),
                _ => contact_curves[k][1].inverse(),
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
                    let (piece, _) = cut_at_front(&mut curves[0], prev)?;
                    cuts.insert(prev, piece.clone())?;
                    cuts.vertices[start_vertex][side] = Some(piece.back().clone());
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
                    let (piece, _) = cut_at_back(&mut curves[m - 1], next)?;
                    cuts.insert(next, piece.clone())?;
                    cuts.vertices[end_vertex][side] = Some(piece.front().clone());
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
            if !p0.near(&p1) {
                return None;
            }
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
        let mut loops = faces[face_idx].boundaries();
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
    Some(TrimmedChain {
        faces,
        contacts,
        vertices,
        cuts,
    })
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

/// The faces around an end vertex of an open chain between the two side faces, the edges crossed
/// walking from side 0 to side 1, and where the fillet meets the edges between those faces.
struct EndWalk<C> {
    faces: Vec<usize>,
    edges: Vec<Edge<Point3, C>>,
    /// per edge between two faces: the crossing vertex, its parameter on the edge's curve and
    /// its parameters on the fillet
    crossings: Vec<(Vertex<Point3>, f64, (f64, f64))>,
}

/// Walks the faces around `vertex` from side 0, leaving through `edge0`, until side 1 is reached,
/// which must happen through `edge1`.
fn walk_around_vertex<C: Clone, S>(
    shell: &Shell<Point3, C, S>,
    sides: [usize; 2],
    vertex: &Vertex<Point3>,
    edge0: Edge<Point3, C>,
    edge1: &Edge<Point3, C>,
) -> Option<EndWalk<C>> {
    let mut faces = Vec::new();
    let mut edges = vec![edge0];
    let mut current = sides[0];
    for _ in 0..shell.len() {
        let edge_id = edges.last()?.id();
        let next = (0..shell.len())
            .find(|&idx| idx != current && shell[idx].edge_iter().any(|e| e.id() == edge_id))?;
        if next == sides[1] {
            let crossings = Vec::new();
            return (edge_id == edge1.id()).then_some(EndWalk {
                faces,
                edges,
                crossings,
            });
        }
        let other = shell[next]
            .edge_iter()
            .find(|e| e.id() != edge_id && (e.front() == vertex || e.back() == vertex))?;
        faces.push(next);
        edges.push(other);
        current = next;
    }
    None
}

/// Tangent of the intersection of `blend` and `surface` at `point`, pointing along `direction`.
fn cross_tangent<S>(
    blend: &S,
    (u, v): (f64, f64),
    surface: &S,
    point: Point3,
    direction: Vector3,
) -> Option<Vector3>
where
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    let (s, t) = surface.search_parameter(point, None, 100)?;
    let tangent = blend.normal(u, v).cross(surface.normal(s, t));
    Some(if tangent.dot(direction) >= 0.0 {
        tangent
    } else {
        -tangent
    })
}

/// `piece` oriented so that it arrives at `vertex` if `arriving`, or leaves it otherwise.
fn toward<C: Clone>(
    piece: &Edge<Point3, C>,
    vertex: &Vertex<Point3>,
    arriving: bool,
) -> Edge<Point3, C> {
    match (piece.back() == vertex) == arriving {
        true => piece.clone(),
        false => piece.inverse(),
    }
}
