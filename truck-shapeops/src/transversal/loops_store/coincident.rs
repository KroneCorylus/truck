//! Cutting two faces that lie on the same surface along each other's boundaries.
//!
//! Faces meeting along a common edge and faces overlapping in a region look alike here: the
//! vertices and crossings of each boundary are put on the other, pieces of boundary the faces
//! share become one edge, and pieces of one boundary inside the other face cut it. A wire whose
//! region lies under the other face gets the overlap status. Everything else stays `Unknown`.
//!
//! Boundaries are compared through their polylines, so two faces sharing a piece of boundary
//! must discretize it identically; that holds for planar faces and for shared curves.

use super::*;
use rustc_hash::FxHashMap as HashMap;

type PolyFace = Face<Point3, PolylineCurve, Option<PolygonMesh>>;
type Boundary2D = truck_meshalgo::prelude::PolylineCurve<Point2>;

/// The loops of one face in both representations.
pub(super) struct FaceLoops<'a, C> {
    pub geom: &'a mut LoopsStore<Point3, C>,
    pub poly: &'a mut LoopsStore<Point3, PolylineCurve>,
    pub index: usize,
}

/// Whether every mesh vertex of each face lies on the other's surface with a parallel normal,
/// and if so whether the normals point the same way.
pub(super) fn coincidence<S>(
    surface0: &S,
    polygon0: &PolygonMesh,
    surface1: &S,
    polygon1: &PolygonMesh,
) -> Option<bool>
where
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let sense = |p: Point3, own: &S, other: &S| -> Option<bool> {
        let (u0, v0) = own.search_nearest_parameter(p, None, 100)?;
        let (u1, v1) = other.search_nearest_parameter(p, None, 100)?;
        let (n0, n1) = (own.normal(u0, v0), other.normal(u1, v1));
        (other.subs(u1, v1).near(&p) && n0.cross(n1).so_small()).then(|| n0.dot(n1) > 0.0)
    };
    let mut senses = polygon0
        .positions()
        .iter()
        .map(|&p| sense(p, surface0, surface1))
        .chain(
            polygon1
                .positions()
                .iter()
                .map(|&p| sense(p, surface1, surface0)),
        );
    let first = senses.next()??;
    senses.all(|sense| sense == Some(first)).then_some(first)
}

/// Cuts the faces along each other's boundaries and marks their overlap.
///
/// `same_normal`: the surface normals agree. `overlap`: the status of the overlap on each face.
pub(super) fn cut<C, S>(
    a: &mut FaceLoops<'_, C>,
    b: &mut FaceLoops<'_, C>,
    surface: &S,
    poly_faces: [&PolyFace; 2],
    same_normal: bool,
    overlap: [ShapesOpStatus; 2],
) -> Option<()>
where
    C: Cut<Point = Point3, Vector = Vector3> + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    unify_vertices(a, b)?;
    unify_vertices(b, a)?;
    add_crossings(a, b)?;
    merge_shared_pieces(a, b, same_normal);
    let region_a = parameter_boundaries(poly_faces[0], surface, true)?;
    let region_b = parameter_boundaries(poly_faces[1], surface, same_normal)?;
    // A partial cut can enclose more than the overlap. Classify only after every boundary
    // piece has been inserted, so a later cut cannot inherit a premature overlap status.
    add_cut_edges(a, b, surface, &region_a, same_normal)?;
    add_cut_edges(b, a, surface, &region_b, same_normal)?;
    mark_overlap(a, b, same_normal, overlap[0]);
    mark_overlap(b, a, same_normal, overlap[1]);
    Some(())
}

/// Puts the vertices of `from` that lie on the boundary of `into` onto that boundary, so both
/// faces use the same vertex there.
fn unify_vertices<C>(from: &FaceLoops<'_, C>, into: &mut FaceLoops<'_, C>) -> Option<()>
where C: Cut<Point = Point3, Vector = Vector3> + SearchNearestParameter<D1, Point = Point3> {
    let mut vertices: Vec<(Vertex<Point3>, Vertex<Point3>)> = Vec::new();
    for (w, wire) in from.poly[from.index].iter().enumerate() {
        for (e, edge) in wire.iter().enumerate() {
            if !vertices.iter().any(|(pv, _)| pv == edge.front()) {
                let gv = from.geom[from.index][w][e].front().clone();
                vertices.push((edge.front().clone(), gv));
            }
        }
    }
    for (pv, gv) in vertices {
        let mut pemap = HashMap::default();
        let Some((w, e, kind)) = into.poly.add_polygon_vertex(into.index, &pv, &mut pemap) else {
            continue;
        };
        into.geom.add_geom_vertex(
            (into.index, w, e),
            &gv,
            kind,
            |curve| curve.search_nearest_parameter(gv.point(), None, 100),
            &mut HashMap::default(),
        )?;
    }
    Some(())
}

/// Puts a vertex on both boundaries where they cross.
fn add_crossings<C>(a: &mut FaceLoops<'_, C>, b: &mut FaceLoops<'_, C>) -> Option<()>
where C: Cut<Point = Point3, Vector = Vector3> + SearchNearestParameter<D1, Point = Point3> {
    let mut points: Vec<Point3> = Vec::new();
    for edge_a in a.poly[a.index].iter().flat_map(|wire| wire.iter()) {
        for edge_b in b.poly[b.index].iter().flat_map(|wire| wire.iter()) {
            let ends = [edge_a.front(), edge_a.back(), edge_b.front(), edge_b.back()];
            for seg_a in edge_a.curve().windows(2) {
                for seg_b in edge_b.curve().windows(2) {
                    let Some(p) = segments_cross([seg_a[0], seg_a[1]], [seg_b[0], seg_b[1]]) else {
                        continue;
                    };
                    if ends.iter().any(|v| v.point().near(&p)) || points.iter().any(|q| q.near(&p))
                    {
                        continue;
                    }
                    points.push(p);
                }
            }
        }
    }
    for p in points {
        let (pv, gv) = (Vertex::new(p), Vertex::new(p));
        let mut pemap = HashMap::default();
        let (wa, ea, kind_a) = a.poly.add_polygon_vertex(a.index, &pv, &mut pemap)?;
        let (wb, eb, kind_b) = b.poly.add_polygon_vertex(b.index, &pv, &mut pemap)?;
        let curve_a = a.geom[a.index][wa][ea].curve();
        let curve_b = b.geom[b.index][wb][eb].curve();
        let s = curve_a.search_nearest_parameter(p, None, 100)?;
        let t = curve_b.search_nearest_parameter(p, None, 100)?;
        let (s, t) = algo::curve::search_intersection_parameter(&curve_a, &curve_b, (s, t), 100)?;
        gv.set_point(curve_a.subs(s));
        let mut gemap = HashMap::default();
        a.geom
            .add_geom_vertex((a.index, wa, ea), &gv, kind_a, |_| Some(s), &mut gemap)?;
        b.geom
            .add_geom_vertex((b.index, wb, eb), &gv, kind_b, |_| Some(t), &mut gemap)?;
    }
    Some(())
}

/// Where two segments cross, if they do.
fn segments_cross([a0, a1]: [Point3; 2], [b0, b1]: [Point3; 2]) -> Option<Point3> {
    let (da, db, r) = (a1 - a0, b1 - b0, a0 - b0);
    let (aa, bb, ab) = (da.dot(da), db.dot(db), da.dot(db));
    let denom = aa * bb - ab * ab;
    if denom <= TOLERANCE2 * aa * bb {
        return None;
    }
    let s = (ab * db.dot(r) - bb * da.dot(r)) / denom;
    let t = (aa * db.dot(r) - ab * da.dot(r)) / denom;
    if !(0.0..=1.0).contains(&s) || !(0.0..=1.0).contains(&t) {
        return None;
    }
    let (p, q) = (a0 + da * s, b0 + db * t);
    p.near(&q).then(|| p.midpoint(q))
}

/// Replaces each piece of the boundary of `b` that coincides with a piece of the boundary of
/// `a` by that piece, in every loop of the shell of `b`.
///
/// Two faces with opposite normals whose regions lie on opposite sides of the piece fold onto
/// each other there; their solids touch only along the piece, and a merged edge would make the
/// result non-manifold. Their pieces stay apart. Faces running the piece the same way relative
/// to their normals have their regions on the same side.
fn merge_shared_pieces<C: Clone>(
    a: &FaceLoops<'_, C>,
    b: &mut FaceLoops<'_, C>,
    same_normal: bool,
) {
    let poly_a = &a.poly[a.index];
    let mut pairs = Vec::new();
    for (wb, wire) in b.poly[b.index].iter().enumerate() {
        for (eb, edge_b) in wire.iter().enumerate() {
            let found = poly_a.iter().enumerate().find_map(|(wa, wire)| {
                wire.iter().enumerate().find_map(|(ea, edge_a)| {
                    let same_direction =
                        edge_a.front() == edge_b.front() && edge_a.back() == edge_b.back();
                    let same_ends = same_direction
                        || (edge_a.front() == edge_b.back() && edge_a.back() == edge_b.front());
                    let same_middle = midpoint(&edge_a.curve()).near(&midpoint(&edge_b.curve()));
                    let merge = edge_a.id() != edge_b.id()
                        && same_ends
                        && same_middle
                        && (same_normal || !same_direction);
                    merge.then_some((wa, ea))
                })
            });
            if let Some((wa, ea)) = found {
                pairs.push(((wa, ea), (wb, eb)));
            }
        }
    }
    for ((wa, ea), (wb, eb)) in pairs {
        let poly_b = b.poly[b.index][wb][eb].absolute_clone();
        let mut poly_a = a.poly[a.index][wa][ea].absolute_clone();
        let mut geom_a = a.geom[a.index][wa][ea].absolute_clone();
        if poly_a.front() != poly_b.front() {
            poly_a.invert();
            geom_a.invert();
        }
        let geom_id = b.geom[b.index][wb][eb].id();
        b.poly.swap_edge_into_wire(poly_b.id(), &wire![poly_a]);
        b.geom.swap_edge_into_wire(geom_id, &wire![geom_a]);
    }
}

fn midpoint(polyline: &PolylineCurve) -> Point3 {
    let (t0, t1) = polyline.range_tuple();
    polyline.subs((t0 + t1) / 2.0)
}

/// The boundaries of `face` in the parameter space of `surface`, counterclockwise.
fn parameter_boundaries<S>(
    face: &PolyFace,
    surface: &S,
    counterclockwise: bool,
) -> Option<Vec<Boundary2D>>
where
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    face.absolute_boundaries()
        .iter()
        .map(|wire| {
            let mut hint = None;
            let mut boundary = Vec::new();
            for edge in wire.iter() {
                let polyline = edge.oriented_curve();
                for &p in &polyline[..polyline.len() - 1] {
                    let uv = surface.search_nearest_parameter(p, hint, 100)?;
                    hint = Some(uv);
                    boundary.push(Point2::from(uv));
                }
            }
            let mut boundary = Boundary2D::from(boundary);
            if !counterclockwise {
                boundary.invert();
            }
            Some(boundary)
        })
        .collect()
}

/// Adds the pieces of the boundary of `from` inside the face of `into` as cut edges of `into`,
/// oriented so that the region of `from` is on their left.
fn add_cut_edges<C: Clone, S>(
    into: &mut FaceLoops<'_, C>,
    from: &FaceLoops<'_, C>,
    surface: &S,
    region: &[Boundary2D],
    same_normal: bool,
) -> Option<()>
where
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let present: Vec<_> = into.poly[into.index]
        .iter()
        .flat_map(|wire| wire.iter().map(|edge| edge.id()))
        .collect();
    let inside = |edge: &Edge<Point3, PolylineCurve>| {
        surface
            .search_nearest_parameter(midpoint(&edge.curve()), None, 100)
            .is_some_and(|uv| polyline_curve::include(region, Point2::from(uv)))
    };
    let mut pieces = Vec::new();
    for (w, wire) in from.poly[from.index].iter().enumerate() {
        for (e, edge) in wire.iter().enumerate() {
            if !present.contains(&edge.id()) && inside(edge) {
                pieces.push((w, e));
            }
        }
    }
    for (w, e) in pieces {
        let mut poly = from.poly[from.index][w][e].clone();
        let mut geom = from.geom[from.index][w][e].clone();
        if !same_normal {
            poly.invert();
            geom.invert();
        }
        let positions =
            into.poly[into.index].add_edge(poly, ShapesOpStatus::Unknown, normal_at(surface))?;
        into.geom[into.index].add_edge_at(geom, ShapesOpStatus::Unknown, positions);
    }
    Some(())
}

/// Gives `overlap` to every wire of `a` whose region lies under the face of `b`.
///
/// A piece of the boundary of `a` decides it: a piece missing from the loops of `b` lies
/// outside `b`; a piece present once is boundary of `b`, which lies on its left if the two
/// faces run it the same way relative to their normals; a piece present twice cuts `b`, so it
/// is inside.
fn mark_overlap<C>(
    a: &mut FaceLoops<'_, C>,
    b: &FaceLoops<'_, C>,
    same_normal: bool,
    overlap: ShapesOpStatus,
) {
    let poly_b = &b.poly[b.index];
    let mut wires = Vec::new();
    for (w, wire) in a.poly[a.index].iter().enumerate() {
        for edge in wire.iter() {
            let in_b: Vec<bool> = poly_b
                .iter()
                .flat_map(|wire| wire.iter())
                .filter(|other| other.id() == edge.id())
                .map(|other| other.orientation())
                .collect();
            let b_on_left = match in_b[..] {
                [] => false,
                [orientation] => (orientation == edge.orientation()) == same_normal,
                _ => true,
            };
            if b_on_left && !wires.contains(&w) {
                wires.push(w);
            }
        }
    }
    for w in wires {
        a.poly[a.index][w].status = overlap;
        a.geom[a.index][w].status = overlap;
    }
}
