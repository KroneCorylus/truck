#![allow(clippy::many_single_char_names)]

use super::*;
use crate::profile::{self, Stage};
use rustc_hash::FxHashMap as HashMap;
use truck_base::cgmath64::*;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::{Vertex, *};

type PolylineCurve = truck_meshalgo::prelude::PolylineCurve<Point3>;

/// Which set operations keep a piece of a face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapesOpStatus {
    Unknown,
    And,
    Or,
    /// The overlap of two coincident faces whose outward normals agree: kept by both operations.
    Both,
    /// A copy of an overlap that is dropped by both operations.
    Neither,
}

impl ShapesOpStatus {
    /// The status seen from the other side of a boundary. The far side of a coincident overlap
    /// is undetermined.
    fn not(self) -> Self {
        match self {
            Self::And => Self::Or,
            Self::Or => Self::And,
            Self::Unknown | Self::Both | Self::Neither => Self::Unknown,
        }
    }

    /// The status of the region across a new cut edge, given the status `prev` the region had.
    /// A transversal cut swaps inside and outside; the boundary of a coincident overlap tells
    /// nothing about the region beyond it.
    fn across(self, prev: Self) -> Self {
        match self {
            Self::Both | Self::Neither => prev,
            _ => self.not(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoundaryWire<P, C> {
    wire: Wire<P, C>,
    status: ShapesOpStatus,
}

impl<P, C> BoundaryWire<P, C> {
    #[inline(always)]
    pub fn new(wire: Wire<P, C>, status: ShapesOpStatus) -> Self { Self { wire, status } }
    #[inline(always)]
    pub fn status(&self) -> ShapesOpStatus { self.status }
    #[inline(always)]
    pub fn invert(&mut self) {
        self.wire.invert();
        self.status = self.status.not();
    }
    #[inline(always)]
    pub fn inverse(&self) -> Self {
        Self {
            wire: self.wire.inverse(),
            status: self.status.not(),
        }
    }
}

impl ShapesOpStatus {
    fn from_is_curve<C, S0, S1>(curve: &IntersectionCurve<C, S0, S1>) -> Option<ShapesOpStatus>
    where
        C: ParametricCurve3D + BoundedCurve,
        S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
        S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
        let (t0, t1) = curve.range_tuple();
        let t = (t0 + t1) / 2.0;
        let (_, pt0, pt1) = curve.search_triple(t, 100)?;
        let der = curve.leader().der(t);
        let normal0 = curve.surface0().normal(pt0[0], pt0[1]);
        let normal1 = curve.surface1().normal(pt1[0], pt1[1]);
        match normal0.cross(der).dot(normal1) > 0.0 {
            true => Some(ShapesOpStatus::Or),
            false => Some(ShapesOpStatus::And),
        }
    }
}

impl<P, C> std::ops::Deref for BoundaryWire<P, C> {
    type Target = Wire<P, C>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.wire }
}

impl<P, C> std::ops::DerefMut for BoundaryWire<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.wire }
}

#[derive(Clone, Debug)]
pub struct Loops<P, C>(Vec<BoundaryWire<P, C>>);
#[derive(Clone, Debug)]
pub struct LoopsStore<P, C>(Vec<Loops<P, C>>);

impl<P, C> std::ops::Deref for Loops<P, C> {
    type Target = Vec<BoundaryWire<P, C>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P, C> std::ops::DerefMut for Loops<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P, C> std::ops::Deref for LoopsStore<P, C> {
    type Target = Vec<Loops<P, C>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P, C> std::ops::DerefMut for LoopsStore<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P, C> FromIterator<BoundaryWire<P, C>> for Loops<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = BoundaryWire<P, C>>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl<'a, P, C, S> From<&'a Face<P, C, S>> for Loops<P, C> {
    #[inline(always)]
    fn from(face: &'a Face<P, C, S>) -> Loops<P, C> {
        face.absolute_boundaries()
            .iter()
            .map(|wire| BoundaryWire::new(wire.clone(), ShapesOpStatus::Unknown))
            .collect()
    }
}

impl<'a, P: 'a, C: 'a, S: 'a> FromIterator<&'a Face<P, C, S>> for LoopsStore<P, C> {
    fn from_iter<I: IntoIterator<Item = &'a Face<P, C, S>>>(iter: I) -> Self {
        Self(iter.into_iter().map(Loops::from).collect())
    }
}

impl<'a, P, C> IntoIterator for &'a LoopsStore<P, C> {
    type Item = <&'a Vec<Loops<P, C>> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<Loops<P, C>> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum ParameterKind {
    Front,
    Back,
    Inner(f64),
}

impl ParameterKind {
    fn try_new(t: f64, (t0, t1): (f64, f64)) -> Option<ParameterKind> {
        if t0.near(&t) {
            Some(ParameterKind::Front)
        } else if t1.near(&t) {
            Some(ParameterKind::Back)
        } else if t0 < t && t < t1 {
            Some(ParameterKind::Inner(t))
        } else {
            None
        }
    }
}

impl<P: Copy, C: Clone> Loops<P, C> {
    fn search_parameter(&self, pt: P) -> Option<(usize, usize, ParameterKind)>
    where C: BoundedCurve<Point = P> + SearchParameter<D1, Point = P> {
        self.iter()
            .enumerate()
            .flat_map(move |(i, wire)| wire.iter().enumerate().map(move |(j, edge)| (i, j, edge)))
            .find_map(|(i, j, edge)| {
                let curve = edge.curve();
                curve.search_parameter(pt, None, 1).and_then(|t| {
                    let kind = ParameterKind::try_new(t, curve.range_tuple())?;
                    Some((i, j, kind))
                })
            })
    }

    fn change_vertex(
        &mut self,
        old_vertex: &Vertex<P>,
        new_vertex: &Vertex<P>,
        emap: &mut HashMap<EdgeID<C>, Edge<P, C>>,
    ) {
        if old_vertex == new_vertex {
            return;
        }
        self.iter_mut()
            .flat_map(|wire| wire.iter_mut())
            .for_each(|edge| {
                let mut new_edge = if edge.absolute_front() == old_vertex {
                    emap.entry(edge.id()).or_insert_with(|| {
                        Edge::new(new_vertex, edge.absolute_back(), edge.curve())
                    })
                } else if edge.absolute_back() == old_vertex {
                    emap.entry(edge.id()).or_insert_with(|| {
                        Edge::new(edge.absolute_front(), new_vertex, edge.curve())
                    })
                } else {
                    return;
                }
                .clone();
                if !edge.orientation() {
                    new_edge.invert();
                }
                // Remove the edge from the HashMap when it is no longer there because ID reassignment will occur.
                if edge.count() == 1 {
                    emap.remove(&edge.id());
                }
                *edge = new_edge;
            })
    }

    fn swap_edge_into_wire(&mut self, edge_id: EdgeID<C>, new_wire: &Wire<P, C>) {
        self.iter_mut().for_each(|wire| {
            let mut iter = wire.iter().enumerate();
            if let Some((idx, edge)) = iter.find(|(_, edge)| edge.id() == edge_id) {
                if edge.orientation() {
                    wire.swap_edge_into_wire(idx, new_wire.clone());
                } else {
                    wire.swap_edge_into_wire(idx, new_wire.inverse());
                }
            }
        });
    }

    #[inline(always)]
    fn add_independent_loop(&mut self, r#loop: BoundaryWire<P, C>) {
        self.push(r#loop.inverse());
        self.push(r#loop);
    }

    fn add_edge(
        &mut self,
        edge0: Edge<P, C>,
        status: ShapesOpStatus,
        normal: impl Fn(P) -> Option<Vector3>,
    ) -> Option<[Option<(usize, usize)>; 2]>
    where
        P: std::ops::Sub<P, Output = Vector3>,
        C: BoundedCurve<Point = P, Vector = Vector3>,
    {
        let direction = |edge: &Edge<P, C>| {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            let t = if edge.orientation() {
                a + (b - a) * 1.0e-4
            } else {
                b - (b - a) * 1.0e-4
            };
            curve.subs(t) - edge.front().point()
        };
        let position = |incoming: &Edge<P, C>| {
            let candidates: Vec<_> = self
                .iter()
                .enumerate()
                .flat_map(|(i, wire)| {
                    wire.iter().enumerate().filter_map(move |(j, edge)| {
                        (edge.front() == incoming.back()).then_some((i, j))
                    })
                })
                .collect();
            if candidates.len() < 2 {
                return Some(candidates.first().copied());
            }
            // Follow the next clockwise ray from the incoming edge's reverse. This
            // selects the face sector even when earlier cuts repeat the vertex.
            let reverse = direction(&incoming.inverse());
            let n = normal(incoming.back().point())?;
            Some(candidates.into_iter().min_by(|&(i, j), &(k, l)| {
                let angle = |d: Vector3| {
                    (-n.dot(reverse.cross(d)))
                        .atan2(reverse.dot(d))
                        .rem_euclid(std::f64::consts::TAU)
                };
                angle(direction(&self[i][j])).total_cmp(&angle(direction(&self[k][l])))
            }))
        };
        let a = position(&edge0)?;
        let b = position(&edge0.inverse())?;
        self.add_edge_at(edge0, status, [a, b]);
        Some([a, b])
    }

    fn add_edge_at(
        &mut self,
        edge0: Edge<P, C>,
        status: ShapesOpStatus,
        [a, b]: [Option<(usize, usize)>; 2],
    ) {
        if let Some((wire_index0, edge_index0)) = a {
            self[wire_index0].rotate_left(edge_index0);
            self[wire_index0].push_front(edge0.clone());
            self[wire_index0].push_back(edge0.inverse());
        }
        match (a, b) {
            (Some((wire_index0, edge_index0)), Some((wire_index1, edge_index1))) => {
                if wire_index0 == wire_index1 {
                    let len = self[wire_index0].len() - 2;
                    let edge_index1 = (len + edge_index1 - edge_index0) % len + 1;
                    let new_wire = self[wire_index0].split_off(edge_index1);
                    let prev = self[wire_index0].status;
                    self[wire_index0].status = status;
                    self.push(BoundaryWire::new(new_wire, status.across(prev)));
                } else {
                    let mut new_wire0 = self[wire_index1].clone();
                    let mut new_wire1 = new_wire0.split_off(edge_index1);
                    new_wire0.append(&mut self[wire_index0]);
                    new_wire0.append(&mut new_wire1);
                    self[wire_index0] = new_wire0;
                    self.swap_remove(wire_index1);
                }
            }
            (None, Some((wire_index1, edge_index1))) => {
                self[wire_index1].rotate_left(edge_index1);
                self[wire_index1].push_front(edge0.inverse());
                self[wire_index1].push_back(edge0);
            }
            (None, None) => self.push(BoundaryWire::new(
                vec![edge0.inverse(), edge0].into(),
                ShapesOpStatus::Unknown,
            )),
            _ => {}
        }
    }
}

impl<P: Copy + Tolerance, C: Clone> LoopsStore<P, C> {
    #[inline(always)]
    fn change_vertex(
        &mut self,
        old_vertex: &Vertex<P>,
        new_vertex: &Vertex<P>,
        emap: &mut HashMap<EdgeID<C>, Edge<P, C>>,
    ) {
        self.iter_mut()
            .for_each(|loops| loops.change_vertex(old_vertex, new_vertex, emap));
    }

    #[inline(always)]
    fn swap_edge_into_wire(&mut self, edge_id: EdgeID<C>, new_wire: &Wire<P, C>) {
        self.iter_mut()
            .for_each(|loops| loops.swap_edge_into_wire(edge_id, new_wire))
    }

    fn add_polygon_vertex(
        &mut self,
        loops_index: usize,
        v: &Vertex<P>,
        emap: &mut HashMap<EdgeID<C>, Edge<P, C>>,
    ) -> Option<(usize, usize, ParameterKind)>
    where
        C: Cut<Point = P> + SearchParameter<D1, Point = P>,
    {
        let position = self[loops_index].search_parameter(v.point())?;
        self.add_polygon_vertex_at(loops_index, v, position, emap)
    }

    fn add_polygon_vertex_at(
        &mut self,
        loops_index: usize,
        v: &Vertex<P>,
        (wire_index, edge_index, kind): (usize, usize, ParameterKind),
        emap: &mut HashMap<EdgeID<C>, Edge<P, C>>,
    ) -> Option<(usize, usize, ParameterKind)>
    where
        C: Cut<Point = P> + SearchParameter<D1, Point = P>,
    {
        match kind {
            ParameterKind::Front => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_front()
                    .clone();
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Back => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_back()
                    .clone();
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Inner(t) => {
                let edge = self[loops_index][wire_index][edge_index].absolute_clone();
                let edge_id = edge.id();
                let (edge0, edge1) = edge.cut_with_parameter(v, t)?;
                let new_wire: Wire<_, _> = vec![edge0, edge1].into();
                self.swap_edge_into_wire(edge_id, &new_wire);
            }
        }
        Some((wire_index, edge_index, kind))
    }
}

impl<C> LoopsStore<Point3, C> {
    /// Puts `v` on the edge at the given indices, where `kind` came from the polyline
    /// counterpart. `locate` gives the parameter of `v` on the curve of that edge and may move
    /// `v` onto the curve.
    fn add_geom_vertex(
        &mut self,
        (loops_index, wire_index, edge_index): (usize, usize, usize),
        v: &Vertex<Point3>,
        kind: ParameterKind,
        locate: impl FnOnce(&C) -> Option<f64>,
        emap: &mut HashMap<EdgeID<C>, Edge<Point3, C>>,
    ) -> Option<()>
    where
        C: Cut<Point = Point3, Vector = Vector3>,
    {
        match kind {
            ParameterKind::Front => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_front()
                    .clone();
                v.set_point(old_vertex.point());
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Back => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_back()
                    .clone();
                v.set_point(old_vertex.point());
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Inner(_) => {
                let edge = self[loops_index][wire_index][edge_index].absolute_clone();
                let t = locate(&edge.curve())?;
                let (edge0, edge1) = edge.cut_with_parameter(v, t)?;
                let new_wire: Wire<_, _> = vec![edge0, edge1].into();
                self.swap_edge_into_wire(edge.id(), &new_wire);
            }
        }
        Some(())
    }
}

/// The parameter on `curve` of the point of `curve` on `surface` closest to `v`, moving `v`
/// onto it.
fn projected_parameter<C, S>(curve: &C, surface: &S, v: &Vertex<Point3>) -> Option<f64>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    let (pt, t, _) = curve_surface_projection(curve, None, surface, None, v.point(), 100)?;
    v.set_point(pt);
    Some(t)
}

fn curve_surface_projection<C, S>(
    curve: &C,
    curve_hint: Option<f64>,
    surface: &S,
    surface_hint: Option<(f64, f64)>,
    point: Point3,
    trials: usize,
) -> Option<(Point3, f64, Point2)>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    if trials == 0 {
        return None;
    }
    let t = curve.search_nearest_parameter(point, curve_hint, 10)?;
    let pt0 = curve.subs(t);
    let (u, v) = surface.search_nearest_parameter(point, surface_hint, 10)?;
    let pt1 = surface.subs(u, v);
    if point.near(&pt0) && point.near(&pt1) && pt0.near(&pt1) {
        Some((point, t, Point2::new(u, v)))
    } else {
        let l = curve.der(t);
        let n = surface.normal(u, v);
        let t0 = (pt1 - pt0).dot(n) / l.dot(n);
        curve_surface_projection(
            curve,
            Some(t),
            surface,
            Some((u, v)),
            pt0 + t0 * l,
            trials - 1,
        )
    }
}

fn create_independent_loop<P, C, D>(mut poly_curve0: C) -> Wire<P, D>
where
    C: Cut<Point = P>,
    D: From<C>, {
    let (t0, t1) = poly_curve0.range_tuple();
    let t = (t0 + t1) / 2.0;
    let poly_curve1 = poly_curve0.cut(t);
    let v0 = Vertex::new(poly_curve0.front());
    let v1 = Vertex::new(poly_curve1.front());
    let edge0 = Edge::new(&v0, &v1, poly_curve0.into());
    let edge1 = Edge::new(&v1, &v0, poly_curve1.into());
    wire![edge0, edge1]
}

#[allow(dead_code)]
pub struct LoopsStoreQuadruple<C> {
    pub geom_loops_store0: LoopsStore<Point3, C>,
    pub poly_loops_store0: LoopsStore<Point3, PolylineCurve>,
    pub geom_loops_store1: LoopsStore<Point3, C>,
    pub poly_loops_store1: LoopsStore<Point3, PolylineCurve>,
}

pub fn create_loops_stores<C, S>(
    geom_shell0: &Shell<Point3, C, S>,
    poly_shell0: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
    geom_shell1: &Shell<Point3, C, S>,
    poly_shell1: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
) -> Option<LoopsStoreQuadruple<C>>
where
    C: SearchNearestParameter<D1, Point = Point3>
        + SearchParameter<D1, Point = Point3>
        + Cut<Point = Point3, Vector = Vector3>
        + From<IntersectionCurve<PolylineCurve, S, S>>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let mut geom_loops_store0: LoopsStore<_, _> = geom_shell0.face_iter().collect();
    let mut poly_loops_store0: LoopsStore<_, _> = poly_shell0.face_iter().collect();
    let mut geom_loops_store1: LoopsStore<_, _> = geom_shell1.face_iter().collect();
    let mut poly_loops_store1: LoopsStore<_, _> = poly_shell1.face_iter().collect();
    let store0_len = geom_loops_store0.len();
    let store1_len = geom_loops_store1.len();
    let bounding_box = |face: &Face<Point3, PolylineCurve, Option<PolygonMesh>>| {
        face.surface().map(|mesh| mesh.bounding_box())
    };
    let bboxes0: Vec<_> = poly_shell0.iter().map(bounding_box).collect();
    let bboxes1: Vec<_> = poly_shell1.iter().map(bounding_box).collect();
    let mut overlapping = 0;
    (0..store0_len)
        .flat_map(move |i| (0..store1_len).map(move |j| (i, j)))
        .try_for_each(|(face_index0, face_index1)| {
            let (Some(bbox0), Some(bbox1)) = (&bboxes0[face_index0], &bboxes1[face_index1]) else {
                return None;
            };
            if !bounding_boxes_overlap(bbox0, bbox1) {
                return Some(());
            }
            overlapping += 1;
            let start = profile::now();
            let result = (|| {
                let ori0 = geom_shell0[face_index0].orientation();
                let ori1 = geom_shell1[face_index1].orientation();
                let surface0 = geom_shell0[face_index0].surface();
                let surface1 = geom_shell1[face_index1].surface();
                let polygon0 = poly_shell0[face_index0].surface()?;
                let polygon1 = poly_shell1[face_index1].surface()?;
                if let Some(same_normal) =
                    coincident::coincidence(&surface0, &polygon0, &surface1, &polygon1)
                {
                    let overlap0 = match same_normal == (ori0 == ori1) {
                        true => ShapesOpStatus::Both,
                        false => ShapesOpStatus::Neither,
                    };
                    let mut a = coincident::FaceLoops {
                        geom: &mut geom_loops_store0,
                        poly: &mut poly_loops_store0,
                        index: face_index0,
                    };
                    let mut b = coincident::FaceLoops {
                        geom: &mut geom_loops_store1,
                        poly: &mut poly_loops_store1,
                        index: face_index1,
                    };
                    return coincident::cut(
                        &mut a,
                        &mut b,
                        &surface0,
                        [&poly_shell0[face_index0], &poly_shell1[face_index1]],
                        same_normal,
                        [overlap0, ShapesOpStatus::Neither],
                    );
                }
                intersection_curve::intersection_curves(
                    surface0.clone(),
                    &polygon0,
                    surface1.clone(),
                    &polygon1,
                )?
                .into_iter()
                .flat_map(|(mut polyline, mut curve)| {
                    let mut pieces = Vec::new();
                    {
                        let on_boundary = |point| {
                            [&poly_shell0[face_index0], &poly_shell1[face_index1]]
                                .into_iter()
                                .flat_map(|face| face.absolute_boundaries().iter())
                                .flat_map(|wire| wire.iter())
                                .any(|edge| edge.curve().search_parameter(point, None, 1).is_some())
                        };
                        let cuts: Vec<_> = (1..polyline.len() - 1)
                            .filter(|&i| on_boundary(polyline[i]))
                            .collect();
                        for i in cuts.into_iter().rev() {
                            pieces.push((polyline.cut(i as f64), curve.cut(i as f64)));
                        }
                    }
                    pieces.push((polyline, curve));
                    pieces.reverse();
                    pieces
                })
                .filter(|(polyline, _)| {
                    !runs_along_boundary(polyline, &poly_shell0[face_index0])
                        && !runs_along_boundary(polyline, &poly_shell1[face_index1])
                })
                .try_for_each(|(polyline, intersection_curve)| {
                    let mut intersection_curve = intersection_curve.into();
                    let status = ShapesOpStatus::from_is_curve(&intersection_curve)?;
                    let (status0, status1) = match (ori0, ori1) {
                        (true, true) => (status, status.not()),
                        (true, false) => (status.not(), status.not()),
                        (false, true) => (status, status),
                        (false, false) => (status.not(), status),
                    };
                    if polyline.front().near(&polyline.back()) {
                        let poly_wire = create_independent_loop(polyline);
                        poly_loops_store0[face_index0]
                            .add_independent_loop(BoundaryWire::new(poly_wire.clone(), status0));
                        poly_loops_store1[face_index1]
                            .add_independent_loop(BoundaryWire::new(poly_wire, status1));
                        let geom_wire = create_independent_loop(intersection_curve);
                        geom_loops_store0[face_index0]
                            .add_independent_loop(BoundaryWire::new(geom_wire.clone(), status0));
                        geom_loops_store1[face_index1]
                            .add_independent_loop(BoundaryWire::new(geom_wire, status1));
                    } else {
                        let pv0 = Vertex::new(polyline.front());
                        let pv1 = Vertex::new(polyline.back());
                        let gv0 = Vertex::new(polyline.front());
                        let gv1 = Vertex::new(polyline.back());
                        for (pv, gv) in [(&pv0, &gv0), (&pv1, &gv1)] {
                            let mut pemap = HashMap::default();
                            let mut gemap = HashMap::default();
                            insert_intersection_vertex(
                                &mut poly_loops_store0,
                                &mut geom_loops_store0,
                                face_index0,
                                pv,
                                gv,
                                &surface1,
                                &mut pemap,
                                &mut gemap,
                            )?;
                            insert_intersection_vertex(
                                &mut poly_loops_store1,
                                &mut geom_loops_store1,
                                face_index1,
                                pv,
                                gv,
                                &surface0,
                                &mut pemap,
                                &mut gemap,
                            )?;
                        }
                        *intersection_curve.leader_mut().first_mut().unwrap() = gv0.point();
                        *intersection_curve.leader_mut().last_mut().unwrap() = gv1.point();
                        let mut polyline = polyline;
                        *polyline.first_mut().unwrap() = pv0.point();
                        *polyline.last_mut().unwrap() = pv1.point();
                        let pedge = Edge::new(&pv0, &pv1, polyline);
                        let gedge = Edge::new(&gv0, &gv1, intersection_curve.into());
                        let positions = poly_loops_store0[face_index0].add_edge(
                            pedge.clone(),
                            status0,
                            normal_at(&surface0),
                        )?;
                        geom_loops_store0[face_index0].add_edge_at(
                            gedge.clone(),
                            status0,
                            positions,
                        );
                        let positions = poly_loops_store1[face_index1].add_edge(
                            pedge,
                            status1,
                            normal_at(&surface1),
                        )?;
                        geom_loops_store1[face_index1].add_edge_at(gedge, status1, positions);
                    }
                    Some(())
                })
            })();
            profile::lap(Stage::Interference, start);
            result
        })?;
    profile::pairs(store0_len * store1_len, overlapping);
    Some(LoopsStoreQuadruple {
        geom_loops_store0,
        poly_loops_store0,
        geom_loops_store1,
        poly_loops_store1,
    })
}

mod coincident;

#[cfg(test)]
mod tests;

/// Whether two bounding boxes overlap, with a margin of `TOLERANCE`.
fn bounding_boxes_overlap(bbox0: &BoundingBox<Point3>, bbox1: &BoundingBox<Point3>) -> bool {
    (0..3).all(|i| {
        bbox0.min()[i] <= bbox1.max()[i] + TOLERANCE && bbox1.min()[i] <= bbox0.max()[i] + TOLERANCE
    })
}

/// Whether every segment of `polyline` lies on the boundary of `face`. Faces meeting along a
/// common edge interfere along that edge; that is contact, not a cut.
fn runs_along_boundary(
    polyline: &PolylineCurve,
    face: &Face<Point3, PolylineCurve, Option<PolygonMesh>>,
) -> bool {
    let on_boundary = |p: Point3| {
        face.edge_iter().any(|edge| {
            edge.curve()
                .windows(2)
                .any(|seg| distance_to_segment(p, seg[0], seg[1]) < TOLERANCE)
        })
    };
    polyline
        .windows(2)
        .all(|seg| on_boundary(seg[0].midpoint(seg[1])))
}

fn distance_to_segment(p: Point3, a: Point3, b: Point3) -> f64 {
    let ab = b - a;
    let t = (p - a).dot(ab) / ab.magnitude2().max(TOLERANCE2);
    p.distance(a + ab * t.clamp(0.0, 1.0))
}

fn normal_at<S>(surface: &S) -> impl Fn(Point3) -> Option<Vector3> + '_
where S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3> {
    move |p| {
        let (u, v) = surface.search_nearest_parameter(p, None, 100)?;
        let normal = surface.normal(u, v);
        (normal.magnitude2().is_finite() && normal.magnitude2() > 0.0).then(|| normal.normalize())
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_intersection_vertex<C, S>(
    poly: &mut LoopsStore<Point3, PolylineCurve>,
    geom: &mut LoopsStore<Point3, C>,
    index: usize,
    pv: &Vertex<Point3>,
    gv: &Vertex<Point3>,
    other: &S,
    pemap: &mut HashMap<EdgeID<PolylineCurve>, Edge<Point3, PolylineCurve>>,
    gemap: &mut HashMap<EdgeID<C>, Edge<Point3, C>>,
) -> Option<()>
where
    C: Cut<Point = Point3, Vector = Vector3> + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let Some((w, e, mut kind)) = poly[index].search_parameter(pv.point()) else {
        return Some(());
    };
    let mut parameter = None;
    if matches!(kind, ParameterKind::Inner(_)) {
        let curve = geom[index][w][e].curve();
        let t = projected_parameter(&curve, other, gv)?;
        parameter = Some(t);
        let geometric_kind = ParameterKind::try_new(t, curve.range_tuple())?;
        match geometric_kind {
            ParameterKind::Front => pv.set_point(poly[index][w][e].absolute_front().point()),
            ParameterKind::Back => pv.set_point(poly[index][w][e].absolute_back().point()),
            ParameterKind::Inner(_) => {}
        }
        if !matches!(geometric_kind, ParameterKind::Inner(_)) {
            kind = geometric_kind;
        }
    }
    poly.add_polygon_vertex_at(index, pv, (w, e, kind), pemap)?;
    geom.add_geom_vertex(
        (index, w, e),
        gv,
        kind,
        |curve| parameter.or_else(|| projected_parameter(curve, other, gv)),
        gemap,
    )
}
