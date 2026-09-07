//! Replacing the surfaces of faces and re-intersecting them with their neighbours.

use super::{intersect::domain_on, intersect_surfaces, LocalOpError};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::result::Result;
use truck_geometry::prelude::*;
use truck_modeling::*;

/// How far, as a fraction of each side, the parameter rectangle of a face is extended when its
/// surface is intersected with a neighbour's.
const MARGIN: f64 = 10.0;

/// Gives each face in `replacements` its new surface and rebuilds its boundary as the
/// intersection of that surface with the surfaces of its neighbours, with new vertices where the
/// new edges meet; the neighbours' loops follow, and their edges leaving the replaced face are
/// trimmed to the new vertices. Faces replaced together meet each other's new surfaces. The
/// input is not modified; the result shares with it whatever did not change.
///
/// This first version takes planes only: the new surfaces, the neighbours, and the three faces
/// that must meet at every vertex of a replaced face. Anything else is
/// [`LocalOpError::Unsupported`]. A neighbour whose surface no longer meets the new one, or two
/// new edges that no longer meet at a vertex, is [`LocalOpError::NoIntersection`].
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::replace_surfaces;
/// let cube: Solid = primitive::cuboid(BoundingBox::from_iter([Point3::origin(), Point3::new(1.0, 1.0, 1.0)]));
/// let top = cube
///     .face_iter()
///     .find(|face| face.surface().subs(0.0, 0.0).z > 0.5)
///     .unwrap()
///     .id();
/// let higher = Surface::Plane(Plane::new(
///     Point3::new(0.0, 0.0, 2.0),
///     Point3::new(1.0, 0.0, 2.0),
///     Point3::new(0.0, 1.0, 2.0),
/// ));
/// let taller = replace_surfaces(&cube, &[(top, higher)]).unwrap();
/// assert_eq!(taller.face_iter().count(), 6);
/// assert!(taller.vertex_iter().any(|v| v.point().z == 2.0));
/// ```
pub fn replace_surfaces(
    solid: &Solid,
    replacements: &[(FaceID, Surface)],
) -> Result<Solid, LocalOpError<Surface>> {
    let faces: Vec<Face> = solid.face_iter().cloned().collect();
    let index: HashMap<FaceID, usize> =
        faces.iter().enumerate().map(|(i, f)| (f.id(), i)).collect();
    let mut new_surface: HashMap<usize, &Surface> = HashMap::default();
    for (face, surface) in replacements {
        let &i = index
            .get(face)
            .ok_or(LocalOpError::UnknownFace { face: *face })?;
        new_surface.insert(i, surface);
    }
    let effective = |i: usize| -> Surface {
        match new_surface.get(&i) {
            Some(&surface) => surface.clone(),
            None => faces[i].surface(),
        }
    };

    let mut faces_of_edge: HashMap<EdgeID, Vec<usize>> = HashMap::default();
    let mut edges_of_vertex: HashMap<VertexID, Vec<Edge>> = HashMap::default();
    for (i, face) in faces.iter().enumerate() {
        for edge in face.edge_iter() {
            faces_of_edge.entry(edge.id()).or_default().push(i);
            for vertex in [edge.front(), edge.back()] {
                let edges = edges_of_vertex.entry(vertex.id()).or_default();
                if edges.iter().all(|e| e.id() != edge.id()) {
                    edges.push(edge.absolute_clone());
                }
            }
        }
    }

    // the vertices of the replaced faces, the edges at them, and the faces across those edges
    let touched_vertices: Vec<Vertex> = {
        let mut seen = HashSet::default();
        new_surface
            .keys()
            .flat_map(|&i| faces[i].boundaries())
            .flat_map(|wire| wire.vertex_iter().collect::<Vec<_>>())
            .filter(|v| seen.insert(v.id()))
            .collect()
    };
    let touched_edges: Vec<Edge> = {
        let mut seen = HashSet::default();
        touched_vertices
            .iter()
            .flat_map(|v| edges_of_vertex[&v.id()].iter().cloned())
            .filter(|e| seen.insert(e.id()))
            .collect()
    };
    let touched_faces: HashSet<usize> = touched_edges
        .iter()
        .flat_map(|e| faces_of_edge[&e.id()].iter().copied())
        .collect();
    // the replaced face an offending face or edge is reported against
    let replaced_at = |edge: &Edge| -> usize {
        let across = &faces_of_edge[&edge.id()];
        across
            .iter()
            .copied()
            .find(|i| new_surface.contains_key(i))
            .unwrap_or(across[0])
    };
    let unsupported = |i: usize| LocalOpError::Unsupported {
        face: faces[i].id(),
    };
    for &i in &touched_faces {
        if !matches!(effective(i), Surface::Plane(_)) {
            let edge = faces[i]
                .edge_iter()
                .find(|e| touched_edges.iter().any(|t| t.id() == e.id()))
                .unwrap();
            return Err(unsupported(replaced_at(&edge)));
        }
    }

    let mut new_curves: HashMap<EdgeID, Curve> = HashMap::default();
    for edge in &touched_edges {
        let across = &faces_of_edge[&edge.id()];
        let [f0, f1] = across[..] else {
            return Err(unsupported(replaced_at(edge)));
        };
        let (s0, s1) = (effective(f0), effective(f1));
        let domain = |i: usize, s: &Surface| domain_on(s, &faces[i], MARGIN).ok_or(unsupported(i));
        let curves = intersect_surfaces(&s0, domain(f0, &s0)?, &s1, domain(f1, &s1)?, 0.01);
        let (face, neighbour) = match new_surface.contains_key(&f0) {
            true => (f0, f1),
            false => (f1, f0),
        };
        let no_intersection = LocalOpError::NoIntersection {
            face: faces[face].id(),
            neighbour: faces[neighbour].id(),
        };
        match curves.as_deref() {
            Some([curve @ Curve::Line(_)]) => new_curves.insert(edge.id(), curve.clone()),
            Some([]) | None => return Err(no_intersection),
            Some(_) => return Err(unsupported(face)),
        };
    }

    let mut new_vertices: HashMap<VertexID, Vertex> = HashMap::default();
    for vertex in &touched_vertices {
        let edges = &edges_of_vertex[&vertex.id()];
        let faces_here: HashSet<usize> = edges
            .iter()
            .flat_map(|e| faces_of_edge[&e.id()].iter().copied())
            .collect();
        if faces_here.len() != 3 || edges.len() != 3 {
            return Err(unsupported(replaced_at(&edges[0])));
        }
        let (c0, c1) = (&new_curves[&edges[0].id()], &new_curves[&edges[1].id()]);
        let point = vertex.point();
        let hint = (
            c0.search_nearest_parameter(point, None, 100),
            c1.search_nearest_parameter(point, None, 100),
        );
        let met = match hint {
            (Some(t0), Some(t1)) => {
                algo::curve::search_intersection_parameter(c0, c1, (t0, t1), 100)
            }
            _ => None,
        };
        let Some((t0, _)) = met.filter(|&(t0, t1)| c0.subs(t0).near(&c1.subs(t1))) else {
            let across = &faces_of_edge[&edges[1].id()];
            let face = replaced_at(&edges[0]);
            let neighbour = across
                .iter()
                .copied()
                .find(|&i| i != face)
                .unwrap_or(across[0]);
            return Err(LocalOpError::NoIntersection {
                face: faces[face].id(),
                neighbour: faces[neighbour].id(),
            });
        };
        new_vertices.insert(vertex.id(), Vertex::new(c0.subs(t0)));
    }

    let new_edges: HashMap<EdgeID, Edge> = touched_edges
        .iter()
        .map(|edge| {
            let end = |v: &Vertex| {
                new_vertices
                    .get(&v.id())
                    .cloned()
                    .unwrap_or_else(|| v.clone())
            };
            let (front, back) = (end(edge.front()), end(edge.back()));
            let line = Curve::Line(Line(front.point(), back.point()));
            (edge.id(), Edge::new(&front, &back, line))
        })
        .collect();

    let rebuilt = |i: usize| -> Face {
        let loops: Vec<Wire> = faces[i]
            .boundaries()
            .into_iter()
            .map(|wire| {
                wire.iter()
                    .map(|edge| match new_edges.get(&edge.id()) {
                        Some(new) if edge.orientation() => new.clone(),
                        Some(new) => new.inverse(),
                        None => edge.clone(),
                    })
                    .collect()
            })
            .collect();
        match new_surface.get(&i) {
            None => Face::new(loops, faces[i].oriented_surface()),
            Some(&surface) => {
                // the loops run counterclockwise about the old outward normal
                let old = faces[i].oriented_surface();
                let (u, v) = old.try_range_tuple();
                let mid = |r: Option<(f64, f64)>| r.map_or(0.5, |(a, b)| (a + b) / 2.0);
                let outward = old.normal(mid(u), mid(v));
                match surface.normal(0.0, 0.0).dot(outward) > 0.0 {
                    true => Face::new(loops, surface.clone()),
                    false => Face::new(loops.iter().map(Wire::inverse).collect(), surface.clone())
                        .inverse(),
                }
            }
        }
    };
    let mut next = 0;
    let shells: Vec<Shell> = solid
        .boundaries()
        .iter()
        .map(|shell| {
            shell
                .iter()
                .map(|face| {
                    let i = next;
                    next += 1;
                    match touched_faces.contains(&i) {
                        true => rebuilt(i),
                        false => face.clone(),
                    }
                })
                .collect()
        })
        .collect();
    let first = replacements.first().map(|(face, _)| *face);
    Solid::try_new(shells).map_err(|_| LocalOpError::Unsupported {
        face: first.unwrap_or_else(|| faces[0].id()),
    })
}
