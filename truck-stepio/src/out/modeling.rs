//! Preparing procedural modeling geometry for STEP.

use truck_geometry::prelude::*;
use truck_modeling::{Curve, Elementary, Surface};
use truck_topology::compress::CompressedSolid;

/// Prepares a modeling solid for [`super::StepModel`] at the given geometric tolerance.
///
/// Contact and intersection curves become spatial splines (exact forms are retained when
/// available); rolling-ball surfaces become rational B-spline surfaces. Exact circular
/// quadratic splines become circles, and spherical octants align their parameter axes with
/// their circular boundaries. Fillet surfaces use normalized longitudinal parameters and
/// include the sampled extent of curved trims beyond their contact endpoints.
/// Other geometry stays exact. Topology, indices, face order and
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
    let mut prepared =
        solid.try_mapped(|p| Some(*p), |c| prepare_curve(c, tol), |s| Some(s.clone()))?;
    for shell in &mut prepared.boundaries {
        for face in &mut shell.faces {
            let curves: Vec<_> = face
                .boundaries
                .iter()
                .flatten()
                .map(|edge| &shell.edges[edge.index].curve)
                .collect();
            face.surface = prepare_surface(&face.surface, &curves, tol)?;
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
            let (a, b) = curve.range_tuple();
            // A coarse adaptive fit can miss a localized deviation. Retry with finer
            // initial intervals and tighter fitting error, keeping total dyadic depth 16.
            for level in 0..=4 {
                let segments = 1 << level;
                let mut fitted: Option<BSplineCurve<Point3>> = None;
                for i in 0..segments {
                    let range = (
                        a + (b - a) * i as f64 / segments as f64,
                        if i + 1 == segments {
                            b
                        } else {
                            a + (b - a) * (i + 1) as f64 / segments as f64
                        },
                    );
                    let piece = BSplineCurve::cubic_approximation(
                        curve,
                        range,
                        tol * 0.25 / 4.0_f64.powi(level as i32),
                        tol.sqrt(),
                        16 - level,
                    );
                    let piece = piece?;
                    fitted = Some(match fitted {
                        None => piece,
                        Some(previous) => previous.try_concat(&piece).ok()?,
                    });
                }
                let spline = fitted?;
                let valid = (0..=256).all(|i| {
                    let t = a + (b - a) * i as f64 / 256.;
                    let error = spline.subs(t).distance(curve.subs(t));
                    error.is_finite() && error <= tol
                });
                if valid {
                    return Some(spline.into());
                }
            }
            None
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

fn prepare_surface(surface: &Surface, curves: &[&Curve], tol: f64) -> Option<Surface> {
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
            let mut surface = processor.entity().clone();
            surface.try_vknot_normalize()?;
            let range = fillet_parameter_range(&surface, transform, curves, tol)?;
            let mut nurbs = fillet_to_nurbs(&surface, range, tol / scale)?;
            nurbs.transform_by(transform);
            if !processor.orientation() {
                nurbs.invert();
            }
            Some(nurbs.into())
        }
        Surface::RevolutedCurve(processor) => {
            let entity = processor.entity();
            let mut curve = prepare_curve(entity.entity_curve(), tol)?;
            // A STEP ellipse has no trim range. Retain a non-elementary revolution's
            // meridian domain as an exact rational arc instead of exporting its full ellipse.
            if surface.elementary().is_none() && matches!(curve, Curve::Conic(_)) {
                curve = Curve::NurbsCurve(NurbsCurve::try_from(&curve).ok()?);
            }
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

fn fillet_parameter_range(
    surface: &ApproxFilletSurface<Box<Surface>, Box<Surface>>,
    transform: Matrix4,
    curves: &[&Curve],
    tol: f64,
) -> Option<(f64, f64)> {
    let inverse = transform.invert()?;
    let (mut lo, mut hi, mut step) = (0.0_f64, 1.0_f64, 0.0_f64);
    for curve in curves {
        let (a, b) = curve.range_tuple();
        let mut hint = None;
        let mut previous: Option<f64> = None;
        for i in 0..=256 {
            let point = curve.subs(a + (b - a) * i as f64 / 256.);
            let local = inverse.transform_point(point);
            if !point.to_vec().is_finite() || !local.to_vec().is_finite() {
                return None;
            }
            let (u, mut v) = surface
                .search_nearest_parameter(local, hint, 100)
                .or_else(|| surface.search_nearest_parameter(local, None, 100))?;
            if !u.is_finite() || !v.is_finite() {
                return None;
            }
            let u = u.clamp(0., 1.);
            let bounded = surface.subs(u, v.clamp(0., 1.));
            if transform.transform_point(bounded).distance(point) <= 2. * tol {
                v = v.clamp(0., 1.);
            } else {
                let error = transform
                    .transform_point(surface.subs(u, v))
                    .distance(point);
                if !error.is_finite() || error > 2. * tol {
                    return None;
                }
            }
            hint = Some((u, v));
            lo = lo.min(v);
            hi = hi.max(v);
            if let Some(previous) = previous {
                step = step.max((v - previous).abs());
            }
            previous = Some(v);
        }
    }
    // Curved trims can turn beyond the contact endpoints. Leave one sample step
    // beyond their measured extent; this remains a sampled fitting bound.
    let range = (
        if lo < 0. { lo - step } else { 0. },
        if hi > 1. { hi + step } else { 1. },
    );
    (range.0.is_finite() && range.1.is_finite()).then_some(range)
}

fn fillet_to_nurbs(
    surface: &ApproxFilletSurface<Box<Surface>, Box<Surface>>,
    (a, b): (f64, f64),
    tol: f64,
) -> Option<NurbsSurface<Vector4>> {
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
        let Some(rows) = (0..4)
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
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let mut spans = knots.as_slice().to_vec();
        spans.dedup();
        let nurbs: NurbsSurface<Vector4> =
            NurbsSurface::new(BSplineSurface::new((KnotVec::bezier_knot(3), knots), rows));
        let mut samples = (0..=(n - 1) * 4)
            .map(|j| a + (b - a) * j as f64 / ((n - 1) * 4) as f64)
            .chain(spans.windows(2).flat_map(|span| {
                (0..=4).map(move |i| span[0] + (span[1] - span[0]) * i as f64 / 4.0)
            }));
        let valid = samples.all(|v| {
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
    // Retain source derivative breaks and spend the same 193-control budget only
    // on spans that miss the tolerance. Each cubic interpolates four exact sections.
    let mut breaks = vec![a];
    breaks.extend(
        surface
            .vknot_vec()
            .iter()
            .copied()
            .filter(|&v| a < v && v < b),
    );
    breaks.push(b);
    breaks.dedup();
    let mut spans: Vec<_> = breaks.windows(2).map(|v| (v[0], v[1])).rev().collect();
    let mut pieces = Vec::new();
    while let Some((left, right)) = spans.pop() {
        if (pieces.len() + spans.len() + 1) * 3 + 1 > 193 {
            return None;
        }
        let sections: Vec<_> = (0..=3)
            .map(|i| {
                let v = match i {
                    0 => left,
                    3 => right,
                    _ => left + (right - left) * i as f64 / 3.,
                };
                surface
                    .fillet_bezier(v)
                    .non_rationalized()
                    .control_points()
                    .to_vec()
            })
            .collect();
        let rows: Vec<_> = (0..4)
            .map(|i| {
                let (p, q, r, s) = (
                    sections[0][i],
                    sections[1][i],
                    sections[2][i],
                    sections[3][i],
                );
                vec![
                    p,
                    (-5. * p + 18. * q - 9. * r + 2. * s) / 6.,
                    (2. * p - 9. * q + 18. * r - 5. * s) / 6.,
                    s,
                ]
            })
            .collect();
        let mut knots = KnotVec::bezier_knot(3);
        knots.transform(right - left, left);
        let patch = NurbsSurface::new(BSplineSurface::new(
            (KnotVec::bezier_knot(3), knots),
            rows.clone(),
        ));
        let valid = (0..=16).all(|j| {
            let v = left + (right - left) * j as f64 / 16.;
            (0..=8).all(|i| {
                let u = i as f64 / 8.;
                let error = patch.subs(u, v).distance(surface.subs(u, v));
                error.is_finite() && error <= tol
            })
        });
        if valid {
            pieces.push((left, right, rows));
        } else {
            let mid = (left + right) / 2.;
            if mid == left || mid == right {
                return None;
            }
            spans.push((mid, right));
            spans.push((left, mid));
        }
    }
    let mut knots = vec![a; 4];
    let mut rows = vec![Vec::new(); 4];
    for (i, (_, right, piece)) in pieces.iter().enumerate() {
        for row in 0..4 {
            rows[row].extend(piece[row].iter().skip(usize::from(i != 0)).copied());
        }
        knots.extend(std::iter::repeat_n(
            *right,
            if i + 1 == pieces.len() { 4 } else { 3 },
        ));
    }
    Some(NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(3), knots.into()),
        rows,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fillet_export_refines_local_error_without_relaxing_tolerance() {
        let Surface::Fillet(surface) = serde_json::from_str(include_str!(
            "../../tests/fixtures/spherical-pole-fillet.json"
        ))
        .unwrap() else {
            panic!("expected procedural fillet");
        };
        let mut surface = surface.entity().clone();
        surface.try_vknot_normalize().unwrap();
        let tol = 0.00005 / 3_f64.sqrt();
        let fitted = fillet_to_nurbs(&surface, (0., 1.), tol).expect("locally refined fillet");
        let knots = fitted.non_rationalized().vknot_vec();
        assert!(knots.len() - 4 <= 193);
        let (unique, counts) = knots.to_single_multi();
        assert!(counts[1..counts.len() - 1].iter().all(|&count| count <= 3));
        let samples = (0..=4096)
            .map(|i| i as f64 / 4096.)
            .chain(unique.windows(2).flat_map(|span| {
                (0..=32).map(move |i| span[0] + (span[1] - span[0]) * i as f64 / 32.)
            }));
        for v in samples {
            for i in 0..=16 {
                let u = i as f64 / 16.;
                assert!(
                    fitted.subs(u, v).distance(surface.subs(u, v)) <= tol,
                    "{u} {v}"
                );
            }
        }
    }

    #[test]
    fn procedural_curve_export_refines_a_missed_interior_bulge() {
        use truck_base::hash::HashGen;
        let h: f64 = 0.5 + (0.2 * <Point3 as HashGen<f64>>::hash1(Point3::new(0.5, 0., 0.)) - 0.1);
        // t²(1-t)²(t-h)² has matching endpoint tangents and a double root at h.
        let heights = [
            0.,
            0.,
            h * h / 15.,
            -h * (1. - h) / 10.,
            (1. - h).powi(2) / 15.,
            0.,
            0.,
        ];
        let surface = BSplineSurface::new(
            (KnotVec::bezier_knot(6), KnotVec::bezier_knot(1)),
            heights
                .into_iter()
                .enumerate()
                .map(|(i, z)| {
                    vec![
                        Point3::new(i as f64 / 6., 0., z * 100.),
                        Point3::new(i as f64 / 6., 1., z * 100.),
                    ]
                })
                .collect(),
        );
        let curve = Curve::PCurve(PCurve::new(
            BSplineCurve::from(Line(Point2::new(0., 0.), Point2::new(1., 0.))),
            Box::new(Surface::BSplineSurface(surface)),
        ));
        let tol = 0.00001;
        let prepared = prepare_curve(&curve, tol).expect("refined procedural boundary");
        for i in 0..=4096 {
            let t = i as f64 / 4096.;
            assert!(curve.subs(t).distance(prepared.subs(t)) <= tol);
        }
        assert_eq!(prepared.front(), curve.front());
        assert_eq!(prepared.back(), curve.back());
    }

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
