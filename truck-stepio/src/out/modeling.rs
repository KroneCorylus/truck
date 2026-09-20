//! Preparing procedural modeling geometry for STEP.

use truck_geometry::prelude::*;
use truck_modeling::{Curve, Elementary, Surface};
use truck_topology::compress::CompressedSolid;

/// Prepares a modeling solid for [`super::StepModel`] at the given geometric tolerance.
///
/// Contact and intersection curves become spatial splines (exact forms are retained when
/// available); rolling-ball surfaces become rational B-spline surfaces. Exact circular
/// quadratic splines become circles, and spherical octants align their parameter axes with
/// their circular boundaries. Other geometry stays exact. Topology, indices, face order and
/// orientation are preserved, and the input is not modified. Use this before exporting a blend:
/// the STEP formatter cannot directly represent [`Surface::Fillet`] or [`Curve::PCurve`].
///
/// `tol` is a positive finite distance in model units. Approximation is checked at sampled
/// parameters, not a certified global error bound. A curve and its supporting surface can each
/// deviate by `tol`, so allow up to `2 * tol` for their agreement in the exported model.
/// Returns `None` for invalid tolerances or if approximation fails within the subdivision limit.
/// Keep the original solid for further modeling and topology naming. Use
/// `truck_meshalgo::tessellation::RobustMeshableShape::robust_triangulation` when tessellating a
/// STEP round trip: the fitted boundary curves need not lie exactly on their fitted surfaces.
pub fn prepare_for_step(
    solid: &CompressedSolid<Point3, Curve, Surface>,
    tol: f64,
) -> Option<CompressedSolid<Point3, Curve, Surface>> {
    if !tol.is_finite() || tol <= 0.0 {
        return None;
    }
    let mut prepared = solid.try_mapped(
        |p| Some(*p),
        |c| prepare_curve(c, tol),
        |s| prepare_surface(s, tol),
    )?;
    for shell in &mut prepared.boundaries {
        for face in &mut shell.faces {
            let [boundary] = face.boundaries.as_slice() else {
                continue;
            };
            let [a, b, c] = boundary.as_slice() else {
                continue;
            };
            if let Some(surface) = spherical_octant(
                &face.surface,
                [a, b, c].map(|edge| &shell.edges[edge.index].curve),
            ) {
                face.surface = surface;
            }
        }
    }
    Some(prepared)
}

fn spherical_octant(surface: &Surface, curves: [&Curve; 3]) -> Option<Surface> {
    let (Elementary::Sphere { center, radius }, same_sense) = surface.elementary()? else {
        return None;
    };
    let mut normals = [Vector3::zero(); 3];
    for (normal, curve) in normals.iter_mut().zip(curves) {
        let Curve::Conic(circle) = curve else {
            return None;
        };
        let transform = circle.transform();
        let x = transform[0].truncate();
        let y = transform[1].truncate();
        if !transform[3].to_point().near2(&center)
            || !x.magnitude().near2(&radius)
            || !y.magnitude().near2(&radius)
            || !x.dot(y).so_small2()
        {
            return None;
        }
        *normal = x.cross(y).normalize();
    }
    if (0..3).any(|i| !normals[i].dot(normals[(i + 1) % 3]).so_small2()) {
        return None;
    }
    // Align great-circle trims with meridians and the equator. Otherwise STEP readers
    // must fit curved sphere pcurves, whose error can exceed a near-limit face's width.
    let x = normals[1];
    let z = normals[0];
    let y = z.cross(x);
    let mut sphere = Processor::with_transform(
        Sphere::new(Point3::origin(), radius),
        Matrix4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            center.to_homogeneous(),
        ),
    );
    if !same_sense {
        sphere.invert();
    }
    Some(Surface::Sphere(sphere))
}

fn prepare_curve(curve: &Curve, tol: f64) -> Option<Curve> {
    if let Curve::NurbsCurve(spline) = curve {
        if let Some(circle) = circular_bezier(spline) {
            return Some(circle);
        }
    }
    if let Curve::IntersectionCurve(intersection) = curve {
        if let Some(circle) =
            circular_intersection(curve, intersection.surface0(), intersection.surface1())
        {
            return Some(circle);
        }
        let leader = intersection.leader();
        let (a, b) = curve.range_tuple();
        if (0..=64).all(|i| {
            let t = a + (b - a) * i as f64 / 64.0;
            curve.subs(t).distance(leader.subs(t)) <= tol * 0.5
        }) {
            return prepare_curve(leader, tol * 0.5);
        }
    }
    if let Curve::PCurve(pcurve) = curve {
        if let Surface::Plane(plane) = pcurve.surface().as_ref() {
            return Some(
                BSplineCurve::new(
                    pcurve.curve().knot_vec().clone(),
                    pcurve
                        .curve()
                        .control_points()
                        .iter()
                        .map(|p| plane.subs(p.x, p.y))
                        .collect(),
                )
                .into(),
            );
        }
    }
    match curve {
        Curve::PCurve(_) | Curve::IntersectionCurve(_) => {
            let spline = BSplineCurve::cubic_approximation(
                curve,
                curve.range_tuple(),
                tol * 0.25,
                tol.sqrt(),
                16,
            )?;
            let (a, b) = curve.range_tuple();
            for i in 0..=256 {
                let t = a + (b - a) * i as f64 / 256.0;
                let error = spline.subs(t).distance(curve.subs(t));
                if !error.is_finite() || error > tol {
                    return None;
                }
            }
            Some(spline.into())
        }
        _ => Some(curve.clone()),
    }
}

fn circular_bezier(curve: &NurbsCurve<Vector4>) -> Option<Curve> {
    let [a, b, c] = curve.control_points().as_slice() else {
        return None;
    };
    if curve.degree() != 2 || a.w != c.w || a.w <= 0.0 || b.w <= 0.0 {
        return None;
    }
    let (knots, mults) = curve.knot_vec().to_single_multi();
    if knots.len() != 2 || mults != [3, 3] {
        return None;
    }
    let weight = b.w / a.w;
    if !(0.0..1.0).contains(&weight) {
        return None;
    }
    let [p, q, r] = [a, b, c].map(|p| p.to_point());
    // A circular rational quadratic has equal endpoint weights. Its tangent
    // intersection and chord midpoint determine the center algebraically.
    let center = Point3::from_vec(
        (p.midpoint(r).to_vec() - weight * weight * q.to_vec()) / (1.0 - weight * weight),
    );
    let x = p - center;
    let normal = (q - p).cross(r - q).normalize();
    let y = normal.cross(x);
    let angle = 2.0 * weight.acos();
    let end = center + angle.cos() * x + angle.sin() * y;
    if !x.magnitude2().is_finite()
        || x.magnitude2() <= TOLERANCE2
        || !normal.x.is_finite()
        || !end.near2(&r)
        || x.dot(q - p).abs() > TOLERANCE2
    {
        return None;
    }
    Some(Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(UnitCircle::new(), (0.0, angle)),
        Matrix4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            normal.extend(0.0),
            center.to_homogeneous(),
        ),
    )))
}

fn circular_intersection(curve: &Curve, a: &Surface, b: &Surface) -> Option<Curve> {
    let (plane, origin, axis, radius) = match (a.elementary()?.0, b.elementary()?.0) {
        (
            Elementary::Plane(plane),
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            },
        )
        | (
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            },
            Elementary::Plane(plane),
        ) => (plane, origin, axis, radius),
        _ => return None,
    };
    if !radius.is_finite()
        || radius <= TOLERANCE
        || !axis.magnitude2().is_finite()
        || axis.so_small()
    {
        return None;
    }
    let axis = axis.normalize();
    if !plane.normal().cross(axis).so_small() {
        return None;
    }
    let center = origin + axis * (plane.subs(0.0, 0.0) - origin).dot(axis);
    let (start, end) = curve.range_tuple();
    if !start.is_finite() || !end.is_finite() || start >= end {
        return None;
    }
    let x = (curve.subs(start) - center).normalize();
    let mut y = axis.cross(x);
    if curve.der(start).dot(y) < 0.0 {
        y = -y;
    }
    let mut previous = 0.0;
    for i in 0..=64 {
        let t = start + (end - start) * i as f64 / 64.0;
        let radial = curve.subs(t) - center;
        if !radial.magnitude().is_finite()
            || radial.dot(axis).abs() > TOLERANCE
            || (radial.magnitude() - radius).abs() > TOLERANCE
        {
            return None;
        }
        let mut angle = radial.dot(y).atan2(radial.dot(x));
        while angle < previous - std::f64::consts::PI {
            angle += std::f64::consts::TAU;
        }
        if !angle.is_finite() || angle < previous - TOLERANCE {
            return None;
        }
        previous = angle;
    }
    if previous <= 0.0 || previous > std::f64::consts::TAU + TOLERANCE {
        return None;
    }
    Some(Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(UnitCircle::new(), (0.0, previous)),
        Matrix4::from_cols(
            (x * radius).extend(0.0),
            (y * radius).extend(0.0),
            x.cross(y).extend(0.0),
            center.to_homogeneous(),
        ),
    )))
}

fn prepare_surface(surface: &Surface, tol: f64) -> Option<Surface> {
    match surface {
        Surface::Fillet(processor) => {
            let transform = *processor.transform();
            let scale = (0..3)
                .map(|i| transform[i].truncate().magnitude2())
                .sum::<f64>()
                .sqrt();
            if !scale.is_finite() || scale <= 0.0 {
                return None;
            }
            let mut nurbs = fillet_to_nurbs(processor.entity(), tol / scale)?;
            nurbs.transform_by(transform);
            if !processor.orientation() {
                nurbs.invert();
            }
            Some(nurbs.into())
        }
        Surface::RevolutedCurve(processor) => {
            let entity = processor.entity();
            let curve = prepare_curve(entity.entity_curve(), tol)?;
            let mut prepared = Processor::with_transform(
                RevolutedCurve::by_revolution(curve, entity.origin(), entity.axis()),
                *processor.transform(),
            );
            if !processor.orientation() {
                prepared.invert();
            }
            Some(Surface::RevolutedCurve(prepared))
        }
        Surface::Extruded(extruded) => Some(Surface::Extruded(ExtrudedCurve::by_extrusion(
            prepare_curve(extruded.entity_curve(), tol)?,
            extruded.extruding_vector(),
        ))),
        _ => Some(surface.clone()),
    }
}

fn fillet_to_nurbs(
    surface: &ApproxFilletSurface<Box<Surface>, Box<Surface>>,
    tol: f64,
) -> Option<NurbsSurface<Vector4>> {
    let (_, (a, b)) = surface.range_tuple();
    // Interpolate the four homogeneous control points of each exact cross-section. Only the
    // longitudinal direction is approximated; the rational cubic across the fillet is retained.
    for level in 0..7 {
        let n = 3 * (1 << level) + 1;
        let mut knots = KnotVec::uniform_knot(3, n - 3);
        knots.transform(b - a, a);
        // Greville stations keep the clamped cubic interpolation system well-conditioned.
        let params: Vec<_> = (0..n)
            .map(|i| (knots[i + 1] + knots[i + 2] + knots[i + 3]) / 3.0)
            .collect();
        let sections: Vec<_> = params.iter().map(|&v| surface.fillet_bezier(v)).collect();
        let rows = (0..4)
            .map(|row| {
                Some(
                    BSplineCurve::try_interpole(
                        knots.clone(),
                        params
                            .iter()
                            .copied()
                            .zip(
                                sections
                                    .iter()
                                    .map(|c| c.non_rationalized().control_points()[row]),
                            )
                            .collect::<Vec<_>>(),
                    )
                    .ok()?
                    .control_points()
                    .to_vec(),
                )
            })
            .collect::<Option<Vec<_>>>()?;
        let nurbs: NurbsSurface<Vector4> =
            NurbsSurface::new(BSplineSurface::new((KnotVec::bezier_knot(3), knots), rows));
        let valid = (0..=(n - 1) * 4).all(|j| {
            let v = a + (b - a) * j as f64 / ((n - 1) * 4) as f64;
            (0..=8).all(|i| {
                let u = i as f64 / 8.0;
                let error = nurbs.subs(u, v).distance(surface.subs(u, v));
                error.is_finite() && error <= tol
            })
        });
        if valid {
            return Some(nurbs);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noncircular_quadratics_remain_splines() {
        for (height, weight) in [(2.0, std::f64::consts::FRAC_1_SQRT_2), (1.0, 0.6)] {
            let spline = NurbsCurve::new(BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![
                    Vector4::new(1., 0., 0., 1.),
                    Vector4::new(weight, height * weight, 0., weight),
                    Vector4::new(0., height, 0., 1.),
                ],
            ));
            let original = Curve::NurbsCurve(spline);
            let prepared = prepare_curve(&original, 0.00005).unwrap();
            assert!(matches!(prepared, Curve::NurbsCurve(_)));
            for i in 0..=16 {
                assert_eq!(original.subs(i as f64 / 16.), prepared.subs(i as f64 / 16.));
            }
        }
    }
}
