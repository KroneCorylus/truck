//! Hollowing a solid.

use super::{replace::Incidence, replace_surfaces, LocalOpError};
use rustc_hash::FxHashSet as HashSet;
use std::result::Result;
use truck_geometry::prelude::*;
use truck_modeling::*;

/// Hollows `solid` to walls of `thickness`, with the faces in `removed` opened. Every kept face
/// is offset inward with [`Surface::offset`](truck_modeling::Surface::offset) and the offsets
/// are re-intersected with each other through [`replace_surfaces`], which gives the cavity; a
/// removed face is the opening, kept as an annulus around the cavity's mouth. With nothing
/// removed the cavity is a second, inner shell of the result. The input is not modified.
///
/// This first version takes a positive thickness, going inward, and convex edges only: an
/// offset across a concave edge leaves a gap that needs a blend. The faces must be planes,
/// cylinders or cones as `replace_surfaces` accepts, meeting three at every vertex. Faces that
/// were not neighbours and come to interfere are not checked; a wall thinner than the solid's
/// features is the caller's to avoid.
/// # Failures
/// - [`LocalOpError::NotInward`] for a thickness that is not positive
/// - [`LocalOpError::Concave`] naming the faces of a concave edge
/// - [`LocalOpError::NoOffset`] naming a face whose surface cannot be offset that far
/// - those of [`replace_surfaces`]
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::shell;
/// let cube: Solid = primitive::cuboid(BoundingBox::from_iter([Point3::origin(), Point3::new(1.0, 1.0, 1.0)]));
/// let top = cube.face_iter().find(|f| f.surface().subs(0.0, 0.0).z > 0.5).unwrap().id();
/// let open_box = shell(&cube, &[top], 0.1).unwrap();
/// assert_eq!(open_box.boundaries().len(), 1);
/// assert_eq!(open_box.face_iter().count(), 11);
/// let closed_box = shell(&cube, &[], 0.1).unwrap();
/// assert_eq!(closed_box.boundaries().len(), 2);
/// ```
pub fn shell(
    solid: &Solid,
    removed: &[FaceID],
    thickness: f64,
) -> Result<Solid, LocalOpError<Surface>> {
    if thickness <= 0.0 {
        return Err(LocalOpError::NotInward);
    }
    let Incidence {
        faces,
        index,
        faces_of_edge,
        ..
    } = Incidence::new(solid);
    let mut opened = HashSet::default();
    for &id in removed {
        let &i = index
            .get(&id)
            .ok_or(LocalOpError::UnknownFace { face: id })?;
        opened.insert(i);
    }
    // every edge convex: the outward normals turn the right way about the edge as its face
    // traverses it
    for (i, face) in faces.iter().enumerate() {
        let surface = face.oriented_surface();
        for edge in face.edge_iter() {
            let &[a, b] = &faces_of_edge[&edge.id()][..] else {
                return Err(LocalOpError::Unsupported { face: face.id() });
            };
            let other = if a == i { b } else { a };
            let curve = edge.oriented_curve();
            let (t0, t1) = curve.range_tuple();
            let t = (t0 + t1) / 2.0;
            let (p, tangent) = (curve.subs(t), curve.der(t));
            let normal_of = |s: &Surface| {
                let (u, v) = s.search_nearest_parameter(p, None, 100)?;
                Some(s.normal(u, v))
            };
            let (Some(n), Some(m)) = (
                normal_of(&surface),
                normal_of(&faces[other].oriented_surface()),
            ) else {
                return Err(LocalOpError::Unsupported { face: face.id() });
            };
            let turn = n.cross(m).dot(tangent);
            if turn < -TOLERANCE {
                return Err(LocalOpError::Concave {
                    face: face.id(),
                    neighbour: faces[other].id(),
                });
            }
        }
    }

    // the cavity: every kept face offset inward, the openings left where they are
    let replacements = faces
        .iter()
        .enumerate()
        .map(|(i, face)| {
            let surface = match opened.contains(&i) {
                true => face.oriented_surface(),
                false => face
                    .oriented_surface()
                    .offset(-thickness)
                    .map_err(|_| LocalOpError::NoOffset { face: face.id() })?,
            };
            Ok((face.id(), surface))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cavity = replace_surfaces(solid, &replacements)?;
    let cavity_faces: Vec<&Face> = cavity.face_iter().collect();

    let mut outer: Shell = Shell::new();
    let mut inner: Shell = Shell::new();
    for (i, face) in faces.iter().enumerate() {
        match opened.contains(&i) {
            true => {
                // the opening: the face with the cavity's mouth as a hole
                let mut loops = face.boundaries();
                loops.extend(cavity_faces[i].boundaries().iter().map(Wire::inverse));
                outer.push(Face::new(loops, face.oriented_surface()));
            }
            false => {
                outer.push(face.clone());
                inner.push(cavity_faces[i].inverse());
            }
        }
    }
    let shells = match opened.is_empty() {
        true => vec![outer, inner],
        false => {
            outer.append(&mut inner);
            vec![outer]
        }
    };
    Solid::try_new(shells).map_err(|_| LocalOpError::Unsupported {
        face: faces[0].id(),
    })
}
