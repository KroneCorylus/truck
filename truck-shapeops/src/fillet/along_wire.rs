use super::*;
use std::collections::HashMap;
use std::result::Result;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};

/// Continues a closed rim through neighboring edges with matching parameter speed.
#[derive(Clone)]
struct ExtendedSpine<C> {
    current: C,
    neighbors: Option<[(C, f64); 2]>,
}

impl<C: ParametricCurve3D + BoundedCurve> ExtendedSpine<C> {
    fn station(&self, t: f64) -> (&C, f64, f64) {
        let (a, b) = self.current.range_tuple();
        if let Some([(previous, before), (next, after)]) = &self.neighbors {
            if t < a {
                return (
                    previous,
                    previous.range_tuple().1 + (t - a) * before,
                    *before,
                );
            }
            if t > b {
                return (next, next.range_tuple().0 + (t - b) * after, *after);
            }
        }
        (&self.current, t, 1.0)
    }
}

impl<C: ParametricCurve3D + BoundedCurve> ParametricCurve for ExtendedSpine<C> {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Point3 {
        let (curve, t, _) = self.station(t);
        curve.subs(t)
    }
    fn der(&self, t: f64) -> Vector3 { self.der_n(1, t) }
    fn der2(&self, t: f64) -> Vector3 { self.der_n(2, t) }
    fn der_n(&self, n: usize, t: f64) -> Vector3 {
        let (curve, t, scale) = self.station(t);
        curve.der_n(n, t) * scale.powi(n as i32)
    }
    fn parameter_range(&self) -> ParameterRange { self.current.parameter_range() }
}
impl<C: ParametricCurve3D + BoundedCurve> BoundedCurve for ExtendedSpine<C> {}
impl<C> SearchNearestParameter<D1> for ExtendedSpine<C>
where C: ParametricCurve3D + BoundedCurve + SearchNearestParameter<D1, Point = Point3>
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = self.current.search_nearest_parameter(point, hint, trials)?;
        algo::curve::search_nearest_parameter(self, point, hint, trials)
    }
}

/// Edges of a boundary loop after trimming: one entry per original edge, `None` to keep it.
type LoopReplacement<C> = Vec<Option<Vec<Edge<Point3, C>>>>;

/// The radius function of the wire restricted to chain edge `k`: `t ↦ R(k + (t − t0) · scale)`.
#[derive(Clone, Debug)]
pub(super) struct EdgeRadius<R> {
    pub(super) radius: R,
    pub(super) k: f64,
    pub(super) t0: f64,
    pub(super) scale: f64,
}

impl<R: ScalarFunctionD1> ScalarFunctionD1 for EdgeRadius<R> {
    fn der_n(&self, n: usize, t: f64) -> f64 {
        let t = self.k + (t - self.t0) * self.scale;
        self.radius.der_n(n, t) * self.scale.powi(n as i32)
    }
}

#[derive(Clone)]
struct ContinuedRadius<R> {
    current: EdgeRadius<R>,
    range: (f64, f64),
    neighbors: Option<[EdgeRadius<R>; 2]>,
}

impl<R: ScalarFunctionD1> ScalarFunctionD1 for ContinuedRadius<R> {
    fn der_n(&self, n: usize, t: f64) -> f64 {
        let law = match &self.neighbors {
            Some([previous, _]) if t < self.range.0 => previous,
            Some([_, next]) if t > self.range.1 => next,
            _ => &self.current,
        };
        law.der_n(n, t)
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
pub(super) struct Cuts<C> {
    vertices: Vec<[Option<Vertex<Point3>>; 2]>,
    /// the kept piece of each cut edge, oriented like the underlying edge
    pieces: HashMap<EdgeID<C>, Option<Edge<Point3, C>>>,
}

impl<C: Clone> Cuts<C> {
    /// The kept piece of `edge`, oriented like `edge`.
    pub(super) fn piece(&self, edge: &Edge<Point3, C>) -> Option<Option<Edge<Point3, C>>> {
        let piece = self.pieces.get(&edge.id())?;
        Some(piece.as_ref().map(|piece| match edge.orientation() {
            true => piece.clone(),
            false => piece.inverse(),
        }))
    }
    fn insert(&mut self, edge: &Edge<Point3, C>, piece: Option<Edge<Point3, C>>) -> Option<()> {
        let absolute = match edge.orientation() {
            true => piece,
            false => piece.map(|piece| piece.inverse()),
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
/// little beyond `[0, n]`. At closed-chain seams, the guide and radius law continue on the
/// neighboring chain edge until the contacts meet the side-face seam.
///
/// `wire` must be continuous and simple, and consecutive edges must share their tangent at the
/// common vertex. It may be closed. Every edge of the chain must be shared by two faces of the
/// shell, and at every interior vertex of the chain the faces on each side must either be the
/// same face or be separated by exactly one edge. An open chain may end at vertices with any
/// number of faces: the fillet is trimmed against every face between the two side faces, and the
/// edges between those faces are cut where they meet the fillet. An existing round can serve as
/// an end face for a miter; tangent junction patches between meeting rounds are not constructed.
/// An exact straight edge leaving an open end may extend toward that end to meet a contact.
/// The curve's `TryInto<Line<Point3>>` conversion must succeed only for exact lines; curved
/// edges retain their trimmed domains. Interior seams and closed chains are never extended.
///
/// Contact curves must remain regular: rounding the rim where an existing convex cylindrical
/// fillet meets a perpendicular plane requires a smaller radius. Equal radii collapse one
/// contact curve to a point and require a spherical corner with different topology; larger
/// radii fold the offset surface. These cases return `None`.
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
    C: FilletedCurve<S> + TryInto<Line<Point3>>,
    S: FilletedSurface<C>,
    R: ScalarFunctionD1,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
{
    try_fillet_along_wire(shell, wire, radius, tol).ok()
}

/// Chain fillets with selection, contact, fitting and trimming diagnostics.
pub fn try_fillet_along_wire<C, S, R>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    radius: R,
    tol: f64,
) -> Result<Shell<Point3, C, S>, Diagnostic>
where
    C: FilletedCurve<S> + TryInto<Line<Point3>>,
    S: FilletedSurface<C>,
    R: ScalarFunctionD1,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
{
    let operation = "fillet_along_wire";
    validate_tolerance(tol, operation)?;
    let failed = || Diagnostic::new(Code::BlendConstructionFailed, operation, "construct_blend");
    let n = wire.len();
    if n == 0 {
        return Err(Diagnostic::new(
            Code::EmptySelection,
            operation,
            "validate_input",
        ));
    }
    if !wire.is_continuous() || !wire.is_simple() {
        return Err(Diagnostic::new(
            Code::UnsupportedTopology,
            operation,
            "validate_wire",
        ));
    }
    for i in 0..=n * 32 {
        let t = i as f64 / 32.0;
        for (name, value, positive) in [
            ("radius", radius.subs(t), true),
            ("radius_derivative", radius.der_n(1, t), false),
            ("radius_second_derivative", radius.der_n(2, t), false),
        ] {
            if !value.is_finite() || (positive && value <= 0.0) {
                let mut error =
                    Diagnostic::new(Code::InvalidParameter, operation, "validate_input")
                        .parameter(name, value);
                error.context.station = Some(t);
                return Err(error);
            }
        }
    }
    let closed = wire.is_cyclic();
    if !is_tangent_continuous(wire, closed) {
        let mut error = Diagnostic::new(Code::UnsupportedGeometry, operation, "validate_wire");
        error.message = "The selected wire is not tangent continuous.".into();
        return Err(error);
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
        .map(|(idx, face)| {
            find_runs(idx, face, wire, &chain_index).ok_or_else(|| {
                Diagnostic::new(Code::UnsupportedTopology, operation, "validate_wire").face(idx)
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    let mut sides = vec![[None; 2]; n];
    for run in &runs {
        for &k in &run.chain {
            if sides[k][run.side].replace(run.face).is_some() {
                return Err(
                    Diagnostic::new(Code::UnsupportedTopology, operation, "validate_wire")
                        .selection(k)
                        .face(run.face),
                );
            }
        }
    }
    let sides: Vec<[usize; 2]> = sides
        .into_iter()
        .map(|[s0, s1]| Some([s0?, s1?]))
        .collect::<Option<_>>()
        .ok_or_else(|| Diagnostic::new(Code::UnsupportedTopology, operation, "validate_wire"))?;

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
            if !t0.is_finite() || !t1.is_finite() || t0 >= t1 {
                return Err(failed().stage("validate_wire").selection(k));
            }
            let neighbors = if closed {
                let previous = wire[(k + n - 1) % n].oriented_curve();
                let next = wire[(k + 1) % n].oriented_curve();
                let before =
                    curve.der(t0).magnitude() / previous.der(previous.range_tuple().1).magnitude();
                let after = curve.der(t1).magnitude() / next.der(next.range_tuple().0).magnitude();
                if [before, after].iter().any(|s| !s.is_finite() || *s <= 0.0) {
                    return Err(failed().stage("validate_wire").selection(k));
                }
                Some([(previous, before), (next, after)])
            } else {
                None
            };
            let neighbor_radii = neighbors
                .as_ref()
                .map(|[(previous, before), (next, after)]| {
                    let (p0, p1) = previous.range_tuple();
                    let (n0, n1) = next.range_tuple();
                    [
                        EdgeRadius {
                            radius: radius.clone(),
                            k: ((k + n - 1) % n) as f64,
                            t0: t0 - (p1 - p0) / before,
                            scale: before / (p1 - p0),
                        },
                        EdgeRadius {
                            radius: radius.clone(),
                            k: ((k + 1) % n) as f64,
                            t0: t1,
                            scale: after / (n1 - n0),
                        },
                    ]
                });
            Ok(RbfSurface::new(
                ExtendedSpine {
                    current: curve,
                    neighbors,
                },
                shell[sides[k][0]].oriented_surface(),
                shell[sides[k][1]].oriented_surface(),
                ContinuedRadius {
                    current: EdgeRadius {
                        radius: radius.clone(),
                        k: k as f64,
                        t0,
                        scale: 1.0 / (t1 - t0),
                    },
                    range: (t0, t1),
                    neighbors: neighbor_radii,
                },
            ))
        })
        .collect::<Result<_, _>>()?;
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
                wire.front_vertex().ok_or_else(failed)?
            } else {
                wire.back_vertex().ok_or_else(failed)?
            };
            let mut walk = walk_around_vertex(
                shell,
                sides[k],
                vertex,
                adjacent_edge(j, 0).ok_or_else(failed)?,
                &adjacent_edge(j, 1).ok_or_else(failed)?,
            )
            .ok_or_else(failed)?;
            let rbf = &rbfs[k];
            let t = if j == 0 { ranges[k].0 } else { ranges[k].1 };
            let (curve0, hint0) = adjacent_hint(&walk.edges[0], vertex);
            let (curve1, hint1) = adjacent_hint(&walk.edges[walk.faces.len()], vertex);
            let (_, _, v0, _) = rbf
                .search_contact_curve0_cross_point_with_adjacent_edge(t, &curve0, hint0, 100)
                .ok_or_else(|| failed().stage("find_contact").selection(k))?;
            let (_, _, v1, _) = rbf
                .search_contact_curve1_cross_point_with_adjacent_edge(t, &curve1, hint1, 100)
                .ok_or_else(|| failed().stage("find_contact").selection(k))?;
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
                    intersect_rolling_fillet(rbf, (0.5, t), &curve, hint).ok_or_else(failed)?;
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
    if closed {
        for k in 0..n {
            for (at_start, j) in [(true, k), (false, (k + 1) % n)] {
                let edges = [adjacent_edge(j, 0), adjacent_edge(j, 1)];
                let (side, edge) = match &edges {
                    [Some(edge), None] => (0, edge),
                    [None, Some(edge)] => (1, edge),
                    _ => continue,
                };
                let vertex = if at_start {
                    wire[k].front()
                } else {
                    wire[k].back()
                };
                let (curve, hint) = adjacent_hint(edge, vertex);
                let t = if at_start { ranges[k].0 } else { ranges[k].1 };
                let crossing = if side == 0 {
                    rbfs[k]
                        .search_contact_curve0_cross_point_with_adjacent_edge(t, &curve, hint, 100)
                } else {
                    rbfs[k]
                        .search_contact_curve1_cross_point_with_adjacent_edge(t, &curve, hint, 100)
                };
                if let Some((_, _, parameter, _)) = crossing {
                    let spine = rbfs[k].edge_curve();
                    if !spine.subs(parameter).near2(&spine.subs(t)) {
                        if at_start {
                            ranges[k].0 = parameter;
                        } else {
                            ranges[k].1 = parameter;
                        }
                    }
                }
            }
        }
    }
    let mut surfaces = Vec::with_capacity(n);
    for k in 0..n {
        let rbf = &rbfs[k];
        let spine = rbf.edge_curve();
        let (a, b) = spine.range_tuple();
        if let Some([(previous, before), (next, after)]) = &spine.neighbors {
            let (p0, p1) = previous.range_tuple();
            let (n0, n1) = next.range_tuple();
            if ranges[k].0 < a - (p1 - p0) / before || ranges[k].1 > b + (n1 - n0) / after {
                return Err(failed().stage("find_contact").selection(k));
            }
        }
        let polar = !closed
            && [rbf.surface0(), rbf.surface1()].into_iter().any(|surface| {
                [a, b].into_iter().any(|t| {
                    surface
                        .search_parameter(spine.subs(t), None, 100)
                        .is_some_and(|(u, v)| {
                            surface.uder(u, v).so_small() || surface.vder(u, v).so_small()
                        })
                })
            });
        let continued = polar || closed && ranges[k] != (a, b);
        let mut contacts = if continued {
            vec![rbf
                .contact_circle((a + b) / 2.0)
                .ok_or_else(|| failed().stage("find_contact").selection(k))?]
        } else {
            Vec::new()
        };
        surfaces.push(
            ApproxFilletSurface::approx_contact_circles(
                rbf.surface0(),
                rbf.surface1(),
                ranges[k],
                tol,
                |t| {
                    if !continued {
                        return rbf.contact_circle(t);
                    }
                    // Follow one UV branch through guide poles and displaced closed seams.
                    let mut previous = *contacts.iter().min_by(|x, y| {
                        (x.curve_parameter() - t)
                            .abs()
                            .total_cmp(&(y.curve_parameter() - t).abs())
                    })?;
                    let start = previous.curve_parameter();
                    if start == t {
                        return Some(previous);
                    }
                    let divisions = if polar { 64.0 } else { 8.0 };
                    let step = (b - a).max(ranges[k].1 - ranges[k].0) / divisions;
                    let steps = ((t - start).abs() / step).ceil().max(1.0) as usize;
                    for i in 1..=steps {
                        let v = start + (t - start) * i as f64 / steps as f64;
                        let [cp0, cp1] = [previous.contact_point0(), previous.contact_point1()];
                        let sign = (previous.center() - cp0.point)
                            .dot(rbf.surface0().normal(cp0.uv.x, cp0.uv.y))
                            .signum();
                        previous = ContactCircle::try_from_contacts(
                            (rbf.edge_curve().subs(v), rbf.edge_curve().der(v)),
                            v,
                            rbf.surface0(),
                            rbf.surface1(),
                            sign * rbf.radius().subs(v),
                            [cp0.uv, cp1.uv],
                        )?;
                    }
                    contacts.push(previous);
                    Some(previous)
                },
            )
            .ok_or_else(|| failed().stage("approximate_surface").selection(k))?,
        );
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
    } = trim_runs(shell, wire, &runs, &contact_curves, |curve, edge, start| {
        let open_end = ends.iter().flatten().any(|walk| {
            [0, walk.faces.len()]
                .into_iter()
                .any(|i| walk.edges[i].id() == edge.id())
        });
        if open_end {
            trim_extended_contact_edge(curve, edge, start)
        } else {
            trim_contact_edge(curve, edge, start)
        }
    })
    .ok_or_else(|| failed().stage("trim_faces"))?;
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
        let mut end_pieces: HashMap<EdgeID<C>, Option<Edge<Point3, C>>> = HashMap::new();
        let mut end_edges: HashMap<usize, Vec<Edge<Point3, C>>> = HashMap::new();
        for (which, j, k) in [(0, 0, 0), (1, n, n - 1)] {
            let walk = ends[which].take().ok_or_else(failed)?;
            let vertex = if j == 0 {
                wire.front_vertex().ok_or_else(failed)?
            } else {
                wire.back_vertex().ok_or_else(failed)?
            };
            let m = walk.faces.len();
            let ((u0, u1), (v0, v1)) = surfaces[k].range_tuple();
            let v_end = if j == 0 { v0 } else { v1 };
            // Points of the cross edge from side 0 to side 1 with their parameters on the fillet,
            // and the kept piece of the edge leaving the chain at each of them.
            let mut points = vec![(vertices[j][0].clone(), (u0, v_end))];
            let mut pieces = vec![cuts.piece(&walk.edges[0]).ok_or_else(failed)?];
            for (i, (w, s, uv)) in walk.crossings.iter().enumerate() {
                let edge = &walk.edges[i + 1];
                let uv = surfaces[k]
                    .search_parameter(w.point(), Some(*uv), 100)
                    .ok_or_else(failed)?;
                points.push((w.clone(), uv));
                let (front, back) = edge.cut_with_parameter(w, *s).ok_or_else(failed)?;
                pieces.push(Some(if edge.back() == vertex { front } else { back }));
            }
            points.push((vertices[j][1].clone(), (u1, v_end)));
            pieces.push(cuts.piece(&walk.edges[m]).ok_or_else(failed)?);
            for (edge, piece) in walk.edges.iter().zip(&pieces) {
                let canonical = piece.as_ref().map(|piece| {
                    if edge.orientation() {
                        piece.clone()
                    } else {
                        piece.inverse()
                    }
                });
                if let Some(previous) = end_pieces.insert(edge.id(), canonical.clone()) {
                    if previous != canonical {
                        return Err(failed().stage("trim_end_faces"));
                    }
                }
            }

            let blend = &blend_surfaces[k];
            for i in 0..m {
                let idx = walk.faces[i];
                let face_surface = shell[idx].oriented_surface();
                let ((w0, uv0), (w1, uv1)) = (&points[i], &points[i + 1]);
                let direction = w1.point() - w0.point();
                let der0 = cross_tangent(blend, *uv0, &face_surface, w0.point(), direction)
                    .ok_or_else(failed)?;
                let der1 = cross_tangent(blend, *uv1, &face_surface, w1.point(), direction)
                    .ok_or_else(failed)?;
                let piece = create_pcurve_edge((w0, *uv0, der0), (w1, *uv1, der1), blend.clone())
                    .ok_or_else(failed)?;
                let fillet_edge = if j == 0 {
                    piece.clone()
                } else {
                    piece.inverse()
                };
                piece.set_curve(
                    IntersectionCurve::new(face_surface, blend.clone(), piece.curve())
                        .to_same_geometry(),
                );
                end_edges
                    .entry(idx)
                    .or_default()
                    .push(fillet_edge.inverse());
                cross[j].push(piece);
            }
        }
        // Both ends may trim the same face, so close its gaps only after collecting both.
        for (idx, bridges) in end_edges {
            let mut used = vec![false; bridges.len()];
            let mut loops = Vec::new();
            for boundary in faces[idx].boundaries() {
                let kept: Vec<_> = boundary
                    .into_iter()
                    .filter_map(|edge| match end_pieces.get(&edge.id()) {
                        Some(piece) => piece.as_ref().map(|piece| {
                            if edge.orientation() {
                                piece.clone()
                            } else {
                                piece.inverse()
                            }
                        }),
                        None => Some(edge),
                    })
                    .collect();
                if kept.is_empty() {
                    return Err(failed().stage("trim_end_faces"));
                }
                let mut boundary = Wire::new();
                for index in 0..kept.len() {
                    boundary.push_back(kept[index].clone());
                    let (a, b) = (kept[index].back(), kept[(index + 1) % kept.len()].front());
                    if a != b {
                        let (i, edge) = bridges
                            .iter()
                            .enumerate()
                            .find(|(i, edge)| !used[*i] && edge.front() == a && edge.back() == b)
                            .ok_or_else(|| failed().stage("trim_end_faces"))?;
                        boundary.push_back(edge.clone());
                        used[i] = true;
                    }
                }
                loops.push(boundary);
            }
            if used.iter().any(|used| !used) {
                return Err(failed().stage("trim_end_faces"));
            }
            faces[idx] = Face::try_new(loops, faces[idx].oriented_surface())
                .map_err(|_| failed().stage("trim_end_faces"))?;
        }
    }

    let blends: Vec<Face<Point3, C, S>> = (0..n)
        .map(|k| {
            let boundary: Vec<_> = std::iter::once(contacts[k][0].inverse())
                .chain(cross[k].iter().cloned())
                .chain(std::iter::once(contacts[k][1].clone()))
                .chain(cross[(k + 1) % nv].iter().rev().map(Edge::inverse))
                .collect();
            Face::try_new(vec![boundary.into()], blend_surfaces[k].clone()).map_err(|e| {
                Diagnostic::new(Code::InvalidOutputTopology, operation, "validate_output")
                    .selection(k)
                    .with_coded_source(e)
            })
        })
        .collect::<Result<_, _>>()?;

    faces.extend(blends);
    Ok(faces.into())
}

pub(super) struct TrimmedChain<C, S> {
    pub(super) faces: Vec<Face<Point3, C, S>>,
    pub(super) contacts: Vec<[Edge<Point3, C>; 2]>,
    pub(super) vertices: Vec<[Vertex<Point3>; 2]>,
    pub(super) cuts: Cuts<C>,
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
    let trimmed = trim_runs(shell, wire, &runs, contact_curves, trim_contact_edge)?;
    trimmed
        .cuts
        .pieces
        .values()
        .all(Option::is_some)
        .then_some(trimmed)
}

fn trim_runs<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    runs: &[Run],
    contact_curves: &[[C; 2]],
    trim: impl Fn(&mut C, &Edge<Point3, C>, bool) -> Option<TrimmedContact<C>>,
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
        if !whole && len < m + 1 {
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
        if !whole && len == m + 1 {
            if cuts.vertices[start_vertex][side].is_some()
                || cuts.vertices[end_vertex][side].is_some()
            {
                return None;
            }
            let adjacent = &boundary[(run.start + len - 1) % len];
            let (before, start) = trim(&mut curves[0], adjacent, true)?;
            let (_, end) = trim(&mut curves[m - 1], adjacent, false)?;
            let before = before?;
            let middle = if before.front() == &end {
                before
            } else {
                let parameter = before
                    .curve()
                    .search_nearest_parameter(end.point(), None, 100)?;
                before.cut_with_parameter(&end, parameter)?.1
            };
            cuts.insert(adjacent, Some(middle.clone()))?;
            cuts.vertices[start_vertex][side] = Some(start);
            cuts.vertices[end_vertex][side] = Some(end);
            piece_prev = Some(Some(middle));
        } else if !whole {
            let prev = &boundary[(run.start + len - 1) % len];
            let next = &boundary[(run.start + m) % len];
            piece_prev = Some(match cuts.vertices[start_vertex][side].clone() {
                Some(vertex) => {
                    let piece = cuts.piece(prev)?;
                    if piece.as_ref().is_some_and(|piece| piece.back() != &vertex) {
                        return None;
                    }
                    trim_to_point(&mut curves[0], vertex.point(), true)?;
                    piece
                }
                None => {
                    let (piece, vertex) = trim(&mut curves[0], prev, true)?;
                    cuts.insert(prev, piece.clone())?;
                    cuts.vertices[start_vertex][side] = Some(vertex);
                    piece
                }
            });
            piece_next = Some(match cuts.vertices[end_vertex][side].clone() {
                Some(vertex) => {
                    let piece = cuts.piece(next)?;
                    if piece.as_ref().is_some_and(|piece| piece.front() != &vertex) {
                        return None;
                    }
                    trim_to_point(&mut curves[m - 1], vertex.point(), false)?;
                    piece
                }
                None => {
                    let (piece, vertex) = trim(&mut curves[m - 1], next, false)?;
                    cuts.insert(next, piece.clone())?;
                    cuts.vertices[end_vertex][side] = Some(vertex);
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
            set(run.start + len - 1, piece.into_iter().collect())?;
        }
        set(run.start, loop_edges)?;
        for i in 1..m {
            set(run.start + i, Vec::new())?;
        }
        if let Some(piece) = piece_next {
            set(run.start + m, piece.into_iter().collect())?;
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
    let hint = if edge.back() == vertex {
        t1 - (t1 - t0) * 0.05
    } else {
        t0 + (t1 - t0) * 0.05
    };
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
    let normal = blend.normal(u, v);
    let mut tangent = normal.cross(surface.normal(s, t));
    if tangent.so_small() {
        // At a tangent endpoint the chord chooses the intersection branch entering the face.
        tangent = direction - normal * direction.dot(normal);
    }
    Some(if tangent.dot(direction) >= 0.0 {
        tangent
    } else {
        -tangent
    })
}

type TrimmedContact<C> = (Option<Edge<Point3, C>>, Vertex<Point3>);

fn intersect_rolling_fillet<C, S, R, E>(
    surface: &RbfSurface<C, S, S, R>,
    (u, v): (f64, f64),
    curve: &E,
    t: f64,
) -> Option<((f64, f64), f64)>
where
    C: ParametricCurve3D,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
    E: ParametricCurve3D,
{
    let mut parameters = Vector3::new(u, v, t);
    for _ in 0..=100 {
        if !parameters.is_finite() {
            return None;
        }
        let Vector3 { x: u, y: v, z: t } = parameters;
        // A Newton step can leave the part of the guide where rolling contacts exist.
        let circle = surface.contact_circle(v)?;
        let residual = circle.subs(u) - curve.subs(t);
        let jacobian = Matrix3::from_cols(circle.der(u), surface.vder(u, v), -curve.der(t));
        let next = parameters - jacobian.invert()? * residual;
        if next.near2(&parameters) {
            return residual.so_small().then_some(((u, v), t));
        }
        parameters = next;
    }
    None
}

fn trim_extended_contact_edge<C, S>(
    curve: &mut C,
    edge: &Edge<Point3, C>,
    at_start: bool,
) -> Option<TrimmedContact<C>>
where
    C: FilletedCurve<S> + TryInto<Line<Point3>>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
{
    let mut trimmed = curve.clone();
    if let Some(result) = trim_contact_edge(&mut trimmed, edge, at_start) {
        *curve = trimmed;
        return Some(result);
    }
    let line: Line<Point3> = edge.curve().try_into().ok()?;
    let axis = line.1 - line.0;
    let length2 = axis.magnitude2();
    if !length2.is_finite() || length2 <= TOLERANCE2 {
        return None;
    }
    let range = curve.range_tuple();
    let mut bounds = (0.0_f64, 1.0_f64);
    // Only the end next to the chain may extend; contact evaluations remain inside their trim.
    for i in 0..=16 {
        let point = curve.subs(range.0 + (range.1 - range.0) * i as f64 / 16.);
        let s = (point - line.0).dot(axis) / length2;
        if !s.is_finite() {
            return None;
        }
        if edge.orientation() == at_start {
            bounds.1 = bounds.1.max(s);
        } else {
            bounds.0 = bounds.0.min(s);
        }
    }
    if bounds == (0., 1.) {
        return None;
    }
    let hint = if at_start { range.0 } else { range.1 };
    let end = if edge.orientation() == at_start {
        1.
    } else {
        0.
    };
    let (t, s) = intersect_samples(
        range,
        bounds,
        (hint, end),
        |t| Some(curve.subs(t)),
        |s| Some(line.subs(s)),
        |t| Some(curve.der(t)),
        |_| Some(axis),
    )?;
    if (0. ..=1.).contains(&s) {
        return None;
    }
    let vertex = Vertex::new(curve.subs(t).midpoint(line.subs(s)));
    if !t.near(&hint) {
        if at_start {
            *curve = curve.cut(t);
        } else {
            curve.cut(t);
        }
    }
    let (front, back) = if at_start {
        (edge.front(), &vertex)
    } else {
        (&vertex, edge.back())
    };
    let extended = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![
            front.point().to_homogeneous(),
            back.point().to_homogeneous(),
        ],
    ));
    Some((
        Some(Edge::new(front, back, extended.to_same_geometry())),
        vertex,
    ))
}

fn trim_contact_edge<C: FilletedCurve<S>, S>(
    curve: &mut C,
    edge: &Edge<Point3, C>,
    at_start: bool,
) -> Option<TrimmedContact<C>> {
    let adjacent = edge.curve();
    let (a, b) = adjacent.range_tuple();
    let adjacent_hint = if edge.orientation() == at_start { b } else { a };
    let range = curve.range_tuple();
    let hint = if at_start { range.0 } else { range.1 };
    let (t, s) = intersect_trimmed_curves(curve, &adjacent, (hint, adjacent_hint))?;
    let p = curve.subs(t).midpoint(adjacent.subs(s));
    let vertex = if p.near(&edge.front().point()) {
        edge.front().clone()
    } else if p.near(&edge.back().point()) {
        edge.back().clone()
    } else {
        Vertex::new(p)
    };
    if !t.near(&hint) {
        if at_start {
            *curve = curve.cut(t);
        } else {
            curve.cut(t);
        }
    }
    let piece = if at_start {
        if &vertex == edge.front() {
            None
        } else if &vertex == edge.back() {
            Some(edge.clone())
        } else {
            Some(edge.cut_with_parameter(&vertex, s)?.0)
        }
    } else if &vertex == edge.back() {
        None
    } else if &vertex == edge.front() {
        Some(edge.clone())
    } else {
        Some(edge.cut_with_parameter(&vertex, s)?.1)
    };
    Some((piece, vertex))
}

fn intersect_trimmed_curves<C: ParametricCurve3D + BoundedCurve>(
    curve0: &C,
    curve1: &C,
    hint: (f64, f64),
) -> Option<(f64, f64)> {
    intersect_samples(
        curve0.range_tuple(),
        curve1.range_tuple(),
        hint,
        |t| Some(curve0.subs(t)),
        |t| Some(curve1.subs(t)),
        |t| Some(curve0.der(t)),
        |t| Some(curve1.der(t)),
    )
}

pub(super) fn intersect_samples(
    (a, b): (f64, f64),
    (c, d): (f64, f64),
    (t, s): (f64, f64),
    curve0: impl Fn(f64) -> Option<Point3>,
    curve1: impl Fn(f64) -> Option<Point3>,
    tangent0: impl Fn(f64) -> Option<Vector3>,
    tangent1: impl Fn(f64) -> Option<Vector3>,
) -> Option<(f64, f64)> {
    if ![a, b, c, d, t, s].iter().all(|x| x.is_finite()) || a >= b || c >= d {
        return None;
    }
    let clamp = |p: Vector2| Vector2::new(p.x.clamp(a, b), p.y.clamp(c, d));
    let solve = |mut param: Vector2| {
        for _ in 0..=100 {
            let (p, q) = (curve0(param.x)?, curve1(param.y)?);
            if p.near2(&q) {
                return Some(param.into());
            }
            let diff = p - q;
            let (du, dv) = (tangent0(param.x)?, -tangent1(param.y)?);
            let uv = du.dot(dv);
            let jacobian = Matrix2::new(du.dot(du), uv, uv, dv.dot(dv));
            let Some(inverse) = jacobian.invert() else {
                return p.near(&q).then_some(param.into());
            };
            let step = inverse * Vector2::new(du.dot(diff), dv.dot(diff));
            if !step.x.is_finite() || !step.y.is_finite() {
                return None;
            }
            // Trimmed intersections have no guaranteed continuation outside either curve's range.
            let mut next = None;
            let mut fraction = 1.;
            for _ in 0..16 {
                let candidate = clamp(param - fraction * step);
                let residual = curve0(candidate.x)? - curve1(candidate.y)?;
                if residual.magnitude2() < diff.magnitude2() {
                    next = Some(candidate);
                    break;
                }
                fraction *= 0.5;
            }
            match next {
                Some(next) => param = next,
                None => return p.near(&q).then_some(param.into()),
            }
        }
        None
    };
    solve(clamp(Vector2::new(t, s))).or_else(|| {
        fn samples(
            curve: impl Fn(f64) -> Option<Point3>,
            a: f64,
            b: f64,
        ) -> Option<[(f64, Point3); 5]> {
            let mut result = [(0., Point3::origin()); 5];
            for (i, item) in result.iter_mut().enumerate() {
                let t = (a + (b - a) * i as f64 / 4.).clamp(a, b);
                *item = (t, curve(t)?);
            }
            Some(result)
        }
        let samples0 = samples(&curve0, a, b)?;
        let samples1 = samples(&curve1, c, d)?;
        let mut nearest = (f64::INFINITY, Vector2::new(a, c));
        for (t, p) in samples0 {
            for &(s, q) in &samples1 {
                let distance = p.distance2(q);
                if distance < nearest.0 {
                    nearest = (distance, Vector2::new(t, s));
                }
            }
        }
        solve(nearest.1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_contact_edges_extend_only_toward_the_chain_endpoint() {
        use truck_modeling::{Curve, Surface};
        let a = Vertex::new(Point3::origin());
        let b = Vertex::new(Point3::new(1., 0., 0.));
        for orientation in [false, true] {
            for at_start in [false, true] {
                let edge = Edge::new(&a, &b, Curve::Line(Line(a.point(), b.point())));
                let edge = if orientation { edge } else { edge.inverse() };
                let toward_end = orientation == at_start;
                for allowed in [false, true] {
                    let x = if toward_end == allowed { 1.2 } else { -0.2 };
                    let mut contact =
                        Curve::Line(Line(Point3::new(x, -0.2, 0.), Point3::new(x, 0.2, 0.)));
                    let before = serde_json::to_string(&contact).unwrap();
                    let result =
                        trim_extended_contact_edge::<_, Surface>(&mut contact, &edge, at_start);
                    if !allowed {
                        assert!(result.is_none());
                        assert_eq!(before, serde_json::to_string(&contact).unwrap());
                        continue;
                    }
                    let (piece, vertex) = result.unwrap();
                    let piece = piece.unwrap();
                    assert_near!(vertex.point(), Point3::new(x, 0., 0.));
                    let bounds = contact.range_tuple();
                    assert_near!(
                        contact.subs(if at_start { bounds.0 } else { bounds.1 }),
                        vertex.point()
                    );
                    assert!(piece.is_geometric_consistent());
                    if at_start {
                        assert_eq!(piece.front(), edge.front());
                        assert_eq!(piece.back(), &vertex);
                    } else {
                        assert_eq!(piece.front(), &vertex);
                        assert_eq!(piece.back(), edge.back());
                    }
                    assert_near!(edge.absolute_front().point(), Point3::origin());
                    assert_near!(edge.absolute_back().point(), Point3::new(1., 0., 0.));
                }
            }
        }
    }

    #[test]
    fn curved_contact_edges_keep_their_trimmed_domains() {
        use truck_modeling::{Curve, Surface};
        let a = Vertex::new(Point3::origin());
        let b = Vertex::new(Point3::new(1., 0., 0.));
        let edge = Edge::new(
            &a,
            &b,
            Curve::BSplineCurve(BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![a.point(), Point3::new(0.5, 0.01, 0.), b.point()],
            )),
        );
        let mut contact = Curve::Line(Line(Point3::new(1.2, -0.2, 0.), Point3::new(1.2, 0.2, 0.)));
        let before = serde_json::to_string(&contact).unwrap();
        assert!(trim_extended_contact_edge::<_, Surface>(&mut contact, &edge, true).is_none());
        assert_eq!(before, serde_json::to_string(&contact).unwrap());
    }

    #[derive(Clone)]
    struct Trimmed(BSplineCurve<Point3>);

    impl ParametricCurve for Trimmed {
        type Point = Point3;
        type Vector = Vector3;
        fn subs(&self, t: f64) -> Point3 {
            assert!((0. ..=1.).contains(&t), "evaluation outside trim: {t}");
            self.0.subs(t)
        }
        fn der(&self, t: f64) -> Vector3 { self.der_n(1, t) }
        fn der2(&self, t: f64) -> Vector3 { self.der_n(2, t) }
        fn der_n(&self, n: usize, t: f64) -> Vector3 {
            assert!((0. ..=1.).contains(&t), "derivative outside trim: {t}");
            self.0.der_n(n, t)
        }
        fn parameter_range(&self) -> ParameterRange { self.0.parameter_range() }
    }
    impl BoundedCurve for Trimmed {}

    fn quadratic(points: [[f64; 3]; 3]) -> Trimmed {
        Trimmed(BSplineCurve::new(
            KnotVec::bezier_knot(2),
            points.into_iter().map(Point3::from).collect(),
        ))
    }

    #[test]
    fn contact_intersections_stay_inside_trimmed_domains() {
        let parabola = quadratic([[0., 0., 0.], [0., 0., 0.], [1., 0., 0.]]);
        for x in [0.5, 1.] {
            let line = quadratic([[x, -0.5, 0.], [x, 0., 0.], [x, 0.5, 0.]]);
            // At t = 0.1, an unrestricted Newton step jumps beyond the curve's domain.
            for hint in [(0.1, 0.5), (-1., 2.)] {
                let (t, s) = intersect_trimmed_curves(&parabola, &line, hint).unwrap();
                assert_near!(t, x.sqrt());
                assert_near!(s, 0.5);
                assert_near!(parabola.subs(t), line.subs(s));
            }
        }
    }

    #[test]
    fn contact_intersections_reseed_parallel_endpoint_tangents() {
        let parabola = quadratic([[0., 1., 0.], [0.5, 1., 0.], [1., 0., 0.]]);
        let line = quadratic([[0., 0.5, 0.], [0.5, 0.5, 0.], [1., 0.5, 0.]]);
        let (t, s) = intersect_trimmed_curves(&parabola, &line, (0., 0.)).unwrap();
        assert_near!(t, 0.5_f64.sqrt());
        assert_near!(s, t);
    }

    #[test]
    fn contact_intersections_reject_disjoint_curves_and_invalid_hints() {
        let parabola = quadratic([[0., 0., 0.], [0., 0., 0.], [1., 0., 0.]]);
        let line = quadratic([[2., -0.5, 0.], [2., 0., 0.], [2., 0.5, 0.]]);
        for hint in [(0.1, 0.5), (f64::NAN, 0.5), (0.1, f64::INFINITY)] {
            assert!(intersect_trimmed_curves(&parabola, &line, hint).is_none());
        }
        let parallel = quadratic([[0., 1., 0.], [0., 1., 0.], [1., 1., 0.]]);
        assert!(intersect_trimmed_curves(&parabola, &parallel, (0.5, 0.5)).is_none());
    }
}
