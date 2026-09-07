//! The curves where two surfaces meet, on their own.

use crate::transversal::{intersection_curve::intersection_curves, smooth_leader};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_modeling::{Curve, Elementary, Face, Surface};

/// A parameter rectangle `((u0, u1), (v0, v1))` of a surface.
pub type Domain = ((f64, f64), (f64, f64));

/// The parameter rectangle of the loops of `face` on its surface, each side enlarged by
/// `margin` times its length. `None` when a boundary point cannot be found on the surface.
pub fn parameter_domain(face: &Face, margin: f64) -> Option<Domain> {
    let surface = face.surface();
    let (mut u, mut v) = ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN));
    let mut hint = None;
    for edge in face.edge_iter() {
        let curve = edge.curve();
        let (t0, t1) = curve.range_tuple();
        for i in 0..16 {
            let p = curve.subs(t0 + (t1 - t0) * i as f64 / 16.0);
            let (a, b) = surface.search_parameter(p, hint, 100)?;
            hint = Some((a, b));
            u = (u.0.min(a), u.1.max(a));
            v = (v.0.min(b), v.1.max(b));
        }
    }
    let enlarge = |(a, b): (f64, f64)| (a - margin * (b - a), b + margin * (b - a));
    Some((enlarge(u), enlarge(v)))
}

/// The curves where `surface0` over `domain0` meets `surface1` over `domain1`. A plane against
/// a plane gives an exact `Line`, clipped to both rectangles; a plane against a cylinder or a
/// cone whose axis is normal to it gives an exact `Conic` over the `u` range of its domain.
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
            let (a0, b0) = range(plane, domain0)?;
            let (a1, b1) = range(other, domain1)?;
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
            // the angle origin of the surface: its point at `u = 0`, projected to the circle
            let (u0, u1) = domain1.0;
            let start = surface1.subs(u0, domain1.1 .0);
            let x = radial(start, center, axis).normalize();
            let y = axis.cross(x);
            let matrix = Matrix4::from_cols(
                (x * radius).extend(0.0),
                (y * radius).extend(0.0),
                axis.extend(0.0),
                center.to_homogeneous(),
            );
            let range = if surface1.subs(u1, domain1.1 .0).near(&start) {
                2.0 * PI
            } else {
                let end = radial(surface1.subs(u1, domain1.1 .0), center, axis).normalize();
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

fn radial(p: Point3, origin: Point3, axis: Vector3) -> Vector3 {
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
