//! Tilting faces about a neutral plane.

use super::{intersect::radial, replace_surfaces, LocalOpError};
use std::result::Result;
use truck_geometry::prelude::*;
use truck_modeling::*;

/// Tilts `faces` of `solid` by `angle` about their traces on `neutral`, so that nothing on
/// that plane moves, and re-intersects them with their neighbours through
/// [`replace_surfaces`]. A positive angle tilts each face's outward normal `n` away from
/// `pull`: it becomes `n cos α − p sin α` with `p` the unit component of `pull` normal to `n`,
/// so a boss widens and a hole narrows along `pull`. A planar face becomes a rotated plane and
/// a cylindrical face whose axis is along `pull` becomes a cone; any other face, a face normal
/// to `pull`, or a neutral plane that is not normal to a cylinder's axis, is
/// [`LocalOpError::Unsupported`] naming the face, found before anything is built.
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::draft;
/// let cube: Solid = primitive::cuboid(BoundingBox::from_iter([Point3::origin(), Point3::new(1.0, 1.0, 1.0)]));
/// let sides: Vec<_> = cube
///     .face_iter()
///     .filter(|face| face.oriented_surface().normal(0.0, 0.0).z.abs() < 0.5)
///     .map(|face| face.id())
///     .collect();
/// let bottom = Plane::new(Point3::origin(), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
/// let drafted = draft(&cube, &sides, &bottom, Vector3::unit_z(), Rad(0.1)).unwrap();
/// // the top got wider, the bottom stayed
/// assert!(drafted.vertex_iter().any(|v| v.point().z > 0.5 && v.point().x > 1.0));
/// assert!(drafted.vertex_iter().all(|v| v.point().z > 0.5 || (0.0..=1.0).contains(&v.point().x)));
/// ```
pub fn draft(
    solid: &Solid,
    faces: &[FaceID],
    neutral: &Plane,
    pull: Vector3,
    angle: Rad<f64>,
) -> Result<Solid, LocalOpError<Surface>> {
    let pull = pull.normalize();
    let (sin, cos) = angle.0.sin_cos();
    let mut replacements = Vec::with_capacity(faces.len());
    for &id in faces {
        let face = solid
            .face_iter()
            .find(|face| face.id() == id)
            .ok_or(LocalOpError::UnknownFace { face: id })?;
        let unsupported = LocalOpError::Unsupported { face: id };
        let oriented = face.oriented_surface();
        let surface = match face.surface() {
            Surface::Plane(_) => {
                let n = oriented.normal(0.0, 0.0);
                let trace = n.cross(neutral.normal());
                let p = pull - n * pull.dot(n);
                if trace.so_small() || p.so_small() {
                    return Err(unsupported);
                }
                let tilted = n * cos - p.normalize() * sin;
                // a point of the trace: from the face's origin, within the face, to the neutral plane
                let origin = oriented.subs(0.0, 0.0);
                let towards = trace.cross(n).normalize();
                let step = (neutral.subs(0.0, 0.0) - origin).dot(neutral.normal())
                    / towards.dot(neutral.normal());
                let point = origin + towards * step;
                let trace = trace.normalize();
                Surface::Plane(Plane::new(
                    point,
                    point + trace,
                    point + tilted.cross(trace),
                ))
            }
            Surface::Extruded(_) => {
                let Some((
                    Elementary::Cylinder {
                        origin,
                        axis,
                        radius,
                    },
                    _,
                )) = oriented.elementary()
                else {
                    return Err(unsupported);
                };
                let along = axis * axis.dot(pull).signum();
                if along.cross(pull).so_small() && neutral.normal().cross(along).so_small() {
                    // the generator at the start of the face's arc, tilted so that the radius
                    // changes with the outward normal turned away from pull
                    let (Some((u0, _)), _) = oriented.try_range_tuple() else {
                        return Err(unsupported);
                    };
                    let start = oriented.subs(u0, 0.0);
                    let outward_radial = radial(start, origin, along).normalize();
                    let out = oriented.normal(u0, 0.0).dot(outward_radial).signum();
                    let neutral_height = (neutral.subs(0.0, 0.0) - origin).dot(along);
                    let on_axis = origin + along * neutral_height;
                    let on_neutral = on_axis + outward_radial * radius;
                    let direction = along * cos + outward_radial * (out * sin);
                    // the generator over the face's own height and a margin, kept on the
                    // side of the apex where the radius is positive
                    let heights = face.edge_iter().flat_map(|edge| {
                        let curve = edge.curve();
                        let (t0, t1) = curve.range_tuple();
                        (0..=8).map(move |i| {
                            (curve.subs(t0 + (t1 - t0) * i as f64 / 8.0) - origin).dot(along)
                        })
                    });
                    let (lo, hi) =
                        heights.fold((f64::MAX, f64::MIN), |(lo, hi), z| (lo.min(z), hi.max(z)));
                    let margin = (hi - lo).max(radius);
                    let radius_at = |z: f64| radius + (z - neutral_height) * out * sin / cos;
                    let mut span = (lo - margin, hi + margin);
                    for z in [&mut span.0, &mut span.1] {
                        if radius_at(*z) <= radius * 0.1 {
                            // the height where the radius is a tenth of the cylinder's
                            *z = neutral_height - 0.9 * radius * cos / (out * sin);
                        }
                    }
                    let at = |z: f64| on_neutral + direction * ((z - neutral_height) / cos);
                    let generator = Curve::Line(Line(at(span.0), at(span.1)));
                    RevolutedCurve::by_revolution(generator, on_axis, along).to_same_geometry()
                } else {
                    return Err(unsupported);
                }
            }
            _ => return Err(unsupported),
        };
        replacements.push((id, surface));
    }
    replace_surfaces(solid, &replacements)
}
