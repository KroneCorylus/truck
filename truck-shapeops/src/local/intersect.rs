//! The curves where two surfaces meet, on their own.

use crate::transversal::{intersection_curve::intersection_curves, smooth_leader};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_modeling::{Curve, Elementary, Face, Surface};

/// A parameter rectangle `((u0, u1), (v0, v1))` of a surface.
pub type Domain = ((f64, f64), (f64, f64));

/// The parameter rectangle of `face` on its surface, each side enlarged by `margin` times the
/// rectangle's diagonal: a side the surface bounds itself is that bound, a side it leaves open
/// is the range of the projected loops. An angular side is never extended past a turn. `None`
/// when a boundary point cannot be projected.
pub fn parameter_domain(face: &Face, margin: f64) -> Option<Domain> {
    domain_on(&face.surface(), face, margin)
}

/// The parameter rectangle of `face` on `surface`, as [`parameter_domain`]. An angular side is
/// measured geometrically along the loops and unwrapped, since a revolved surface's own range
/// is a full turn whatever the face covers and the nearest-parameter search of a periodic
/// surface is not to be trusted with the turn; a side the surface bounds is that bound; only
/// what is left is found by projecting the loops.
pub(super) fn domain_on(surface: &Surface, face: &Face, margin: f64) -> Option<Domain> {
    let angular = angular(surface);
    let own = surface.try_range_tuple();
    let mut range: [Option<(f64, f64)>; 2] = [None, None];
    for (side, own) in [own.0, own.1].into_iter().enumerate() {
        range[side] = match angular == Some(side) {
            true => angular_range(surface, face).or(own),
            false => own,
        };
    }
    if range.iter().any(Option::is_none) {
        let mut projected = [(f64::MAX, f64::MIN); 2];
        let mut hint = None;
        for p in loop_samples(face) {
            let (u, v) = surface.search_nearest_parameter(p, hint, 100)?;
            hint = Some((u, v));
            for (side, value) in [u, v].into_iter().enumerate() {
                projected[side] = (projected[side].0.min(value), projected[side].1.max(value));
            }
        }
        for side in 0..2 {
            range[side].get_or_insert(projected[side]);
        }
    }
    let [u, v] = range.map(Option::unwrap);
    let diagonal = f64::hypot(u.1 - u.0, v.1 - v.0);
    let enlarge = |side: usize, (a, b): (f64, f64)| {
        let mut extension = margin * diagonal;
        if angular == Some(side) {
            extension = extension.min(0.45 * (2.0 * PI - (b - a)).max(0.0));
        }
        (a - extension, b + extension)
    };
    Some((enlarge(0, u), enlarge(1, v)))
}

/// Sixteen points on every edge of `face`, in loop order and direction.
fn loop_samples(face: &Face) -> impl Iterator<Item = Point3> + '_ {
    face.edge_iter().flat_map(|edge| {
        let curve = edge.oriented_curve();
        let (t0, t1) = curve.range_tuple();
        (0..16).map(move |i| curve.subs(t0 + (t1 - t0) * i as f64 / 16.0))
    })
}

/// The range of the angle about the axis of `surface`, a cylinder or cone, covered by the loops
/// of `face`, measured from the surface's angular origin and unwrapped along the loops.
fn angular_range(surface: &Surface, face: &Face) -> Option<(f64, f64)> {
    let (origin, axis) = match surface.elementary()? {
        (Elementary::Cylinder { origin, axis, .. }, _) => (origin, axis),
        (Elementary::Cone { apex, axis, .. }, _) => (apex, axis),
        _ => return None,
    };
    let (u0, v0) = surface.try_range_tuple();
    let mid = |r: Option<(f64, f64)>| r.map_or(0.0, |(a, b)| (a + b) / 2.0);
    let start = match angular(surface)? {
        0 => surface.subs(0.0, mid(v0)),
        _ => surface.subs(mid(u0), 0.0),
    };
    let x = radial(start, origin, axis).normalize();
    let y = axis.cross(x);
    let mut range = (f64::MAX, f64::MIN);
    let mut last = 0.0;
    for p in loop_samples(face) {
        let r = radial(p, origin, axis);
        if r.so_small() {
            continue;
        }
        let mut angle = f64::atan2(y.dot(r), x.dot(r));
        angle -= 2.0 * PI * ((angle - last) / (2.0 * PI)).round();
        last = angle;
        range = (range.0.min(angle), range.1.max(angle));
    }
    (range.0 <= range.1).then_some(range)
}

/// Which parameter of `surface` is an angle, if one is; it must not be extended past a turn.
fn angular(surface: &Surface) -> Option<usize> {
    match surface {
        Surface::Extruded(extruded) => {
            matches!(extruded.entity_curve(), Curve::Conic(_)).then_some(0)
        }
        Surface::RevolutedCurve(_) => Some(1),
        _ => None,
    }
}

/// The curves where `surface0` over `domain0` meets `surface1` over `domain1`. A plane against
/// a plane gives an exact `Line`, clipped to both rectangles; a plane against a cylinder or a
/// cone whose axis is normal to it gives an exact `Conic` over the angular range of its domain.
/// Anything else is tessellated at `tol` over its rectangle and the interference of the two
/// meshes is lifted onto both surfaces as `IntersectionCurve`s with a smooth leader. Empty when
/// the surfaces do not meet, or only touch; `None` when a curve cannot be lifted.
pub fn intersect_surfaces(
    surface0: &Surface,
    domain0: Domain,
    surface1: &Surface,
    domain1: Domain,
    tol: f64,
) -> Option<Vec<Curve>> {
    if let Some(curves) = exact(surface0, domain0, surface1, domain1) {
        return Some(curves);
    }
    if let Some(curves) = exact(surface1, domain1, surface0, domain0) {
        return Some(curves);
    }
    let polygon0 = StructuredMesh::from_surface(surface0, domain0, tol).destruct();
    let polygon1 = StructuredMesh::from_surface(surface1, domain1, tol).destruct();
    let curves = intersection_curves(surface0.clone(), &polygon0, surface1.clone(), &polygon1)?;
    curves
        .into_iter()
        .map(|(_, ic)| {
            let ic: IntersectionCurve<PolylineCurve<Point3>, Surface, Surface> = ic.into();
            let leader = smooth_leader(&ic)?;
            let (surface0, surface1, _) = ic.destruct();
            Some(IntersectionCurve::new(surface0, surface1, leader).into())
        })
        .collect()
}

/// The exact intersection when `surface0` is a plane and `surface1` a plane, or a cylinder or
/// cone with its axis normal to it.
fn exact(
    surface0: &Surface,
    domain0: Domain,
    surface1: &Surface,
    domain1: Domain,
) -> Option<Vec<Curve>> {
    let Surface::Plane(plane) = surface0 else {
        return None;
    };
    let normal = plane.normal();
    let origin = plane.subs(0.0, 0.0);
    match surface1 {
        Surface::Plane(other) => {
            let direction = normal.cross(other.normal());
            if direction.so_small() {
                return Some(Vec::new());
            }
            // a point on both planes: from the origin of the first, along the in-plane direction
            // towards the second
            let towards = direction.cross(normal).normalize();
            let other_origin = other.subs(0.0, 0.0);
            let step = (other_origin - origin).dot(other.normal()) / towards.dot(other.normal());
            let point = origin + towards * step;
            let direction = direction.normalize();
            let range = |plane: &Plane, domain: Domain| {
                let (u, v) = plane.search_parameter(point, None, 1)?;
                let (du, dv) = plane.search_parameter(point + direction, None, 1)?;
                clip((u, v), (du - u, dv - v), domain)
            };
            let (Some((a0, b0)), Some((a1, b1))) = (range(plane, domain0), range(other, domain1))
            else {
                return Some(Vec::new());
            };
            let (a, b) = (a0.max(a1), b0.min(b1));
            Some(match a < b {
                true => vec![Curve::Line(Line(
                    point + direction * a,
                    point + direction * b,
                ))],
                false => Vec::new(),
            })
        }
        _ => {
            let (elementary, _) = surface1.elementary()?;
            let (axis, on_axis) = match &elementary {
                Elementary::Cylinder { origin, axis, .. } => (*axis, *origin),
                Elementary::Cone { apex, axis, .. } => (*axis, *apex),
                _ => return None,
            };
            if !normal.cross(axis).so_small() {
                return None;
            }
            let height = origin.to_vec().dot(axis);
            let radius = match &elementary {
                Elementary::Cylinder { radius, .. } => *radius,
                Elementary::Cone { half_angle, .. } => {
                    (height - on_axis.to_vec().dot(axis)) * half_angle.0.tan()
                }
                _ => return None,
            };
            if radius <= TOLERANCE {
                return Some(Vec::new());
            }
            let center = on_axis + axis * (height - on_axis.to_vec().dot(axis));
            // the angle origin of the surface: its point at the start of its angular range,
            // projected to the circle
            let angle = angular(surface1)?;
            let (a0, a1) = [domain1.0, domain1.1][angle];
            // the other parameter at the middle of the surface's own range, a point of
            // positive radius on the surface's own nappe
            let own = surface1.try_range_tuple();
            let other = match angle {
                0 => own.1,
                _ => own.0,
            };
            let other = other.map_or(0.0, |(a, b)| (a + b) / 2.0);
            let at = |t: f64| match angle {
                0 => surface1.subs(t, other),
                _ => surface1.subs(other, t),
            };
            let start = at(a0);
            let x = radial(start, center, axis).normalize();
            let y = axis.cross(x);
            let matrix = Matrix4::from_cols(
                (x * radius).extend(0.0),
                (y * radius).extend(0.0),
                axis.extend(0.0),
                center.to_homogeneous(),
            );
            let range = if at(a1).near(&start) {
                2.0 * PI
            } else {
                let end = radial(at(a1), center, axis).normalize();
                let angle = x.angle(end).0;
                match y.dot(end) >= 0.0 {
                    true => angle,
                    false => 2.0 * PI - angle,
                }
            };
            let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, range));
            Some(vec![Curve::Conic(Processor::with_transform(
                circle, matrix,
            ))])
        }
    }
}

pub(super) fn radial(p: Point3, origin: Point3, axis: Vector3) -> Vector3 {
    let r = p - origin;
    r - axis * r.dot(axis)
}

/// The parameter interval of the 2D line `start + t · direction` inside the rectangle.
fn clip(start: (f64, f64), direction: (f64, f64), domain: Domain) -> Option<(f64, f64)> {
    let (mut a, mut b) = (f64::NEG_INFINITY, f64::INFINITY);
    for (s, d, (lo, hi)) in [
        (start.0, direction.0, domain.0),
        (start.1, direction.1, domain.1),
    ] {
        if d.so_small() {
            if s < lo || s > hi {
                return None;
            }
            continue;
        }
        let (t0, t1) = ((lo - s) / d, (hi - s) / d);
        a = a.max(t0.min(t1));
        b = b.min(t0.max(t1));
    }
    (a < b).then_some((a, b))
}

/// The nearest point of `surface` to `p`, by the geometry of a plane, cylinder or cone.
pub(super) fn project(surface: &Surface, p: Point3) -> Option<Point3> {
    Some(match surface.elementary()?.0 {
        Elementary::Plane(plane) => {
            let (origin, normal) = (plane.subs(0.0, 0.0), plane.normal());
            p - normal * (p - origin).dot(normal)
        }
        Elementary::Cylinder {
            origin,
            axis,
            radius,
        } => {
            let r = radial(p, origin, axis);
            p - r + r.normalize() * radius
        }
        Elementary::Cone {
            apex,
            axis,
            half_angle,
        } => {
            let r = radial(p, apex, axis).normalize();
            let generator = axis * half_angle.0.cos() + r * half_angle.0.sin();
            apex + generator * (p - apex).dot(generator)
        }
        _ => return None,
    })
}
