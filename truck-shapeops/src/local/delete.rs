//! Removing a face and closing the hole with its neighbours.

use super::{
    intersect::domain_on,
    replace::{rebuild_face, rebuild_solid, Incidence, MARGIN},
    LocalOpError,
};
use super::{locate_failure, Failure};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_geometry::prelude::*;
use truck_modeling::*;

/// Removes `face` from `solid` and closes the hole with its neighbours: the two neighbours
/// across a pair of opposite edges are extended to meet along a new edge, and the neighbours
/// across the other two edges lose those edges and meet the new edge at a vertex each. That is
/// a fillet or chamfer of one edge removed and the sharp edge restored. The input is not
/// modified; the result shares with it whatever did not change.
///
/// This first version takes a face with four edges whose neighbours are all planes, with
/// exactly three faces at each of its vertices; anything else is [`LocalOpError::Unsupported`].
/// The pair of neighbours that meets is the first of the two opposite pairs whose planes
/// intersect; when neither does, [`LocalOpError::NoIntersection`] names one of them.
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::delete_face;
/// // a box with one vertical edge rounded, the round face removed
/// let s = 0.5 / f64::sqrt(2.0);
/// let v: Vec<Vertex> = [(0.0, 0.0), (1.5, 0.0), (2.0, 0.5), (2.0, 3.0), (0.0, 3.0)]
///     .iter()
///     .map(|&(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
///     .collect();
/// let wire: Wire = wire![
///     builder::line(&v[0], &v[1]),
///     builder::circle_arc(&v[1], &v[2], Point3::new(1.5 + s, 0.5 - s, 0.0)),
///     builder::line(&v[2], &v[3]),
///     builder::line(&v[3], &v[4]),
///     builder::line(&v[4], &v[0]),
/// ];
/// let face: Face = builder::try_attach_plane(&[wire]).unwrap();
/// let rounded: Solid = builder::tsweep(&face, Vector3::unit_z());
/// let round = rounded
///     .face_iter()
///     .find(|face| matches!(face.surface(), Surface::Extruded(_)))
///     .unwrap()
///     .id();
/// let sharp = delete_face(&rounded, round).unwrap();
/// assert_eq!(sharp.face_iter().count(), 6);
/// assert!(sharp.vertex_iter().any(|v| v.point().near(&Point3::new(2.0, 0.0, 1.0))));
/// ```
pub fn delete_face(solid: &Solid, face: FaceID) -> Result<Solid, LocalOpError<Surface>> {
    delete_face_impl(solid, face).map_err(|e| e.legacy)
}

/// Diagnostic variant of [`delete_face`], with input-relative locations and retained causes.
pub fn try_delete_face(solid: &Solid, face: FaceID) -> Result<Solid, Diagnostic> {
    super::validate_solid(solid, "delete_face")?;
    delete_face_impl(solid, face)
        .map_err(|e| locate_failure(solid.face_iter(), &[face], "delete_face", e))
}

pub(super) fn delete_face_impl(solid: &Solid, face: FaceID) -> Result<Solid, Failure> {
    let Incidence {
        faces,
        index,
        faces_of_edge,
        edges_of_vertex,
    } = Incidence::new(solid);
    let &removed = index.get(&face).ok_or(LocalOpError::UnknownFace { face })?;
    let unsupported = || LocalOpError::Unsupported { face };
    let boundaries = faces[removed].boundaries();
    let [wire] = boundaries.as_slice() else {
        return Err(Failure::new(
            unsupported(),
            Code::UnsupportedTopology,
            "validate_incidence",
        ));
    };
    let loop_edges: Vec<Edge> = wire.iter().map(Edge::absolute_clone).collect();
    let loop_ids: HashSet<EdgeID> = loop_edges.iter().map(Edge::id).collect();
    if loop_edges.len() != 4 {
        return Err(Failure::new(
            unsupported(),
            Code::UnsupportedTopology,
            "validate_incidence",
        ));
    }
    // the neighbour across each edge of the loop
    let neighbours: Vec<usize> = loop_edges
        .iter()
        .map(|edge| {
            let across = &faces_of_edge[&edge.id()];
            match across[..] {
                [a, b] if a == removed => Some(b),
                [a, b] if b == removed => Some(a),
                _ => None,
            }
        })
        .collect::<Option<_>>()
        .ok_or_else(|| {
            Failure::new(
                unsupported(),
                Code::UnsupportedTopology,
                "validate_incidence",
            )
        })?;
    if neighbours
        .iter()
        .any(|&n| !matches!(faces[n].surface(), Surface::Plane(_)))
    {
        return Err(unsupported().into());
    }
    for edge in wire.iter() {
        let edges = &edges_of_vertex[&edge.front().id()];
        let faces_here: HashSet<usize> = edges
            .iter()
            .flat_map(|e| faces_of_edge[&e.id()].iter().copied())
            .collect();
        if edges.len() != 3 || faces_here.len() != 3 {
            return Err(Failure::new(
                unsupported(),
                Code::UnsupportedTopology,
                "validate_incidence",
            ));
        }
    }

    // the opposite pair whose planes meet: their edges become one new edge, the other two vanish
    let meet = |i: usize, j: usize| -> Result<Option<Curve>, Failure> {
        let (a, b) = (&faces[neighbours[i]], &faces[neighbours[j]]);
        let (sa, sb) = (a.surface(), b.surface());
        let domain = |face: &Face, s: &Surface| {
            domain_on(s, face, MARGIN).ok_or_else(|| {
                Failure::new(
                    unsupported(),
                    Code::ProjectionFailed,
                    "parameterize_boundary",
                )
            })
        };
        let curves =
            super::try_intersect_surfaces(&sa, domain(a, &sa)?, &sb, domain(b, &sb)?, 0.01)
                .map_err(|diagnostic| Failure {
                    legacy: unsupported(),
                    diagnostic,
                })?;
        Ok(match curves.as_slice() {
            [curve @ Curve::Line(_)] => Some(curve.clone()),
            _ => None,
        })
    };
    let (sides, line) = match (meet(1, 3)?, meet(0, 2)?) {
        (Some(line), _) => ([1, 3], line),
        (None, Some(line)) => ([0, 2], line),
        (None, None) => {
            return Err(LocalOpError::NoIntersection {
                face,
                neighbour: faces[neighbours[1]].id(),
            }
            .into())
        }
    };
    let ends = [(sides[0] + 1) % 4, (sides[0] + 3) % 4];

    // each end edge collapses to the vertex where the new line crosses its spokes
    let mut new_vertices: HashMap<VertexID, Vertex> = HashMap::default();
    for &k in &ends {
        let end_edge = &wire[k];
        let (v0, v1) = (end_edge.front(), end_edge.back());
        let spoke = edges_of_vertex[&v0.id()]
            .iter()
            .find(|e| !loop_ids.contains(&e.id()))
            .ok_or_else(unsupported)?;
        let hint = (
            line.search_nearest_parameter(v0.point(), None, 100),
            spoke
                .curve()
                .search_nearest_parameter(v0.point(), None, 100),
        );
        let met = match hint {
            (Some(t0), Some(t1)) => {
                algo::curve::search_intersection_parameter(&line, &spoke.curve(), (t0, t1), 100)
            }
            _ => None,
        };
        let Some((t, _)) = met.filter(|&(t, s)| line.subs(t).near(&spoke.curve().subs(s))) else {
            return Err(Failure::new(
                LocalOpError::NoIntersection {
                    face,
                    neighbour: faces[neighbours[k]].id(),
                },
                Code::IntersectionFailed,
                "find_vertex",
            ));
        };
        let vertex = Vertex::new(line.subs(t));
        new_vertices.insert(v0.id(), vertex.clone());
        new_vertices.insert(v1.id(), vertex);
    }
    let end = |v: &Vertex| {
        new_vertices
            .get(&v.id())
            .cloned()
            .unwrap_or_else(|| v.clone())
    };

    let mut new_edges: HashMap<EdgeID, Option<Edge>> = HashMap::default();
    for &k in &ends {
        new_edges.insert(loop_edges[k].id(), None);
    }
    let side = &loop_edges[sides[0]];
    let (front, back) = (end(side.front()), end(side.back()));
    let new_edge = Edge::new(
        &front,
        &back,
        Curve::Line(Line(front.point(), back.point())),
    );
    new_edges.insert(side.id(), Some(new_edge.clone()));
    let other = &loop_edges[sides[1]];
    let along = end(other.front()).id() == front.id();
    new_edges.insert(
        other.id(),
        Some(match along {
            true => new_edge.clone(),
            false => new_edge.inverse(),
        }),
    );
    for edge in wire.iter() {
        for spoke in edges_of_vertex[&edge.front().id()]
            .iter()
            .filter(|e| !loop_ids.contains(&e.id()))
        {
            let (front, back) = (end(spoke.front()), end(spoke.back()));
            let line = Curve::Line(Line(front.point(), back.point()));
            new_edges.insert(spoke.id(), Some(Edge::new(&front, &back, line)));
        }
    }

    let rebuilt: HashMap<usize, Face> = neighbours
        .iter()
        .map(|&n| {
            rebuild_face(&faces[n], &new_edges, faces[n].oriented_surface()).map(|face| (n, face))
        })
        .collect::<Result<_, _>>()
        .map_err(|e| {
            Failure::new(
                unsupported(),
                Code::InvalidOutputTopology,
                "validate_output",
            )
            .source(e)
        })?;
    let dropped = HashSet::from_iter([removed]);
    rebuild_solid(solid, &rebuilt, &dropped).map_err(|e| {
        Failure::new(
            unsupported(),
            Code::InvalidOutputTopology,
            "validate_output",
        )
        .source(e)
    })
}
