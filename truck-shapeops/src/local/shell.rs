//! Hollowing a solid.

use super::{locate_failure, Failure};
use super::{replace::Incidence, LocalOpError};
use rustc_hash::FxHashSet as HashSet;
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_geometry::prelude::*;
use truck_modeling::*;

/// Hollows `solid` to walls of `thickness`, with the faces in `removed` opened. Every kept face
/// is offset inward with [`Surface::offset`](truck_modeling::Surface::offset) and the offsets
/// are re-intersected with each other through [`super::replace_surfaces`], which gives the cavity; a
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
/// - those of [`super::replace_surfaces`]
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
    shell_impl(solid, removed, thickness).map_err(|e| e.legacy)
}

/// Diagnostic variant of [`shell()`], with input-relative locations and retained causes.
pub fn try_shell(solid: &Solid, removed: &[FaceID], thickness: f64) -> Result<Solid, Diagnostic> {
    super::validate_solid(solid, "shell")?;
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(
            Diagnostic::new(Code::InvalidParameter, "shell", "validate_input")
                .parameter("thickness", thickness),
        );
    }
    shell_impl(solid, removed, thickness)
        .map_err(|e| locate_failure(solid.face_iter(), removed, "shell", e))
}

pub(super) fn shell_impl(
    solid: &Solid,
    removed: &[FaceID],
    thickness: f64,
) -> Result<Solid, Failure> {
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(LocalOpError::NotInward.into());
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
                return Err(Failure::new(
                    LocalOpError::Unsupported { face: face.id() },
                    Code::UnsupportedTopology,
                    "validate_incidence",
                ));
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
                return Err(Failure::new(
                    LocalOpError::Unsupported { face: face.id() },
                    Code::ProjectionFailed,
                    "evaluate_normal",
                ));
            };
            let turn = n.cross(m).dot(tangent);
            if turn < -TOLERANCE {
                return Err(LocalOpError::Concave {
                    face: face.id(),
                    neighbour: faces[other].id(),
                }
                .into());
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
                false => face.oriented_surface().offset(-thickness).map_err(|e| {
                    Failure::new(
                        LocalOpError::NoOffset { face: face.id() },
                        Code::NoOffset,
                        "offset_surface",
                    )
                    .source(e)
                })?,
            };
            Ok((face.id(), surface))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let cavity = super::replace::replace_surfaces_impl(solid, &replacements)?;
    let cavity_faces: Vec<&Face> = cavity.face_iter().collect();

    let mut outer: Shell = Shell::new();
    let mut inner: Shell = Shell::new();
    for (i, face) in faces.iter().enumerate() {
        match opened.contains(&i) {
            true => {
                // the opening: the face with the cavity's mouth as a hole
                let mut loops = face.boundaries();
                loops.extend(cavity_faces[i].boundaries().iter().map(Wire::inverse));
                outer.push(Face::try_new(loops, face.oriented_surface()).map_err(|e| {
                    Failure::new(
                        LocalOpError::Unsupported { face: face.id() },
                        Code::InvalidOutputTopology,
                        "validate_output",
                    )
                    .source(e)
                })?);
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
    Solid::try_new(shells).map_err(|e| {
        Failure::new(
            LocalOpError::Unsupported {
                face: faces[0].id(),
            },
            Code::InvalidOutputTopology,
            "validate_output",
        )
        .source(e)
    })
}

/// Thickens the open `shell` by `thickness` along the normals of its faces into a solid: the
/// shell, its offset, and a wall along every free edge. This first version takes a planar
/// shell, all faces planes with one normal, whose walls are exact extrusions of the free edges;
/// a face of another kind, or in another plane, is [`LocalOpError::Unsupported`] naming it, so
/// a bent shell with a concave interior edge is refused by the face beyond the bend. A
/// thickness that is not positive is [`LocalOpError::NotInward`]. The input is not modified.
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::thicken;
/// let v: Vec<Vertex> = [(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (1.0, 1.0), (1.0, 2.0), (0.0, 2.0)]
///     .iter()
///     .map(|&(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
///     .collect();
/// let wire: Wire = (0..6).map(|i| builder::line(&v[i], &v[(i + 1) % 6])).collect();
/// let plate: Shell = vec![builder::try_attach_plane(&[wire]).unwrap()].into();
/// let slab = thicken(&plate, 0.5).unwrap();
/// assert_eq!(slab.face_iter().count(), 8);
/// ```
pub fn thicken(shell: &Shell, thickness: f64) -> Result<Solid, LocalOpError<Surface>> {
    thicken_impl(shell, thickness).map_err(|e| e.legacy)
}

/// Diagnostic variant of [`thicken`], with input-relative locations and retained causes.
pub fn try_thicken(shell: &Shell, thickness: f64) -> Result<Solid, Diagnostic> {
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(
            Diagnostic::new(Code::InvalidParameter, "thicken", "validate_input")
                .parameter("thickness", thickness),
        );
    }
    if shell.is_empty() {
        return Err(Diagnostic::new(
            Code::EmptySelection,
            "thicken",
            "validate_input",
        ));
    }
    thicken_impl(shell, thickness).map_err(|e| locate_failure(shell.face_iter(), &[], "thicken", e))
}

pub(super) fn thicken_impl(shell: &Shell, thickness: f64) -> Result<Solid, Failure> {
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(LocalOpError::NotInward.into());
    }
    let mut normal: Option<Vector3> = None;
    for face in shell.face_iter() {
        let unsupported = LocalOpError::Unsupported { face: face.id() };
        let Surface::Plane(_) = face.surface() else {
            return Err(unsupported.into());
        };
        let n = face.oriented_surface().normal(0.0, 0.0);
        match normal {
            Some(m) if !(n - m).so_small() => return Err(unsupported.into()),
            Some(_) => {}
            None => normal = Some(n),
        }
    }
    let Some(normal) = normal else {
        return Err(LocalOpError::NotInward.into());
    };
    let solids: Vec<Result<Solid, truck_topology::errors::Error>> =
        builder::tsweep(shell, normal * thickness);
    let unsupported = LocalOpError::Unsupported {
        face: shell.face_iter().next().unwrap().id(),
    };
    match <[_; 1]>::try_from(solids) {
        Ok([Ok(solid)]) => Ok(solid),
        Ok([Err(e)]) => {
            Err(Failure::new(unsupported, Code::InvalidOutputTopology, "validate_output").source(e))
        }
        _ => Err(unsupported.into()),
    }
}
