//! Preparing procedural modeling geometry for STEP.

use truck_geometry::prelude::*;
use truck_modeling::{Curve, Elementary, Surface};
use truck_topology::compress::CompressedSolid;

mod boundary;
use boundary::BoundaryCertificate;

/// Prepares a modeling solid for [`super::StepModel`] at the given geometric tolerance.
///
/// Contact and intersection curves become spatial splines (exact forms are retained when
/// available); rolling-ball surfaces become rational B-spline surfaces. Exact circular
/// quadratic splines become circles, and spherical octants align their parameter axes with
/// their circular boundaries. Fillet surfaces use normalized longitudinal parameters and
/// include the sampled extent of curved trims beyond their contact endpoints.
/// Eligible affine UV trims on clamped constant-weight spline boundaries retain the
/// stored boundary coefficients, with restricted domains and an outward-rounded bound
/// on the real curves defined by the stored coefficients. This accounts for knot mapping,
/// normalization and cut rounding; it does not bound floating evaluation or reader pcurves.
/// Other geometry stays exact. Topology, indices, face order and
/// orientation are preserved, and the input is not modified. Use this before exporting a blend:
/// the STEP formatter cannot directly represent [`Surface::Fillet`] or [`Curve::PCurve`].
///
/// `tol` is a positive finite distance in model units. Approximation is checked at sampled
/// parameters, not a certified global error bound. A curve and its supporting surface can each
/// deviate by `tol`, so allow up to `2 * tol` for their agreement in the exported model.
/// Fitted intersections of two extrusions with single-span polynomial profiles of degree at most three or affine conic profiles
/// additionally require whole-span spatial support agreement within `min(tol, 5e-8)`.
/// This reserves half of the target OCCT reader's default edge precision; it does not certify imported pcurves.
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
        if let Curve::BSplineCurve(leader) = intersection.leader().as_ref() {
            let (a, b) = curve.range_tuple();
            let agrees = |t| {
                intersection
                    .search_triple(t, 100)
                    .is_some_and(|(point, _, _)| point.distance(leader.subs(t)) <= tol * 0.5)
            };
            let (knots, _) = leader.knot_vec().to_single_multi();
            // A uniform grid can hit only exact guide nodes and miss chord sagitta.
            // Check each knot-span interior before substituting the guide.
            if (0..=64).all(|i| agrees(a + (b - a) * i as f64 / 64.))
                && knots.windows(2).all(|span| {
                    let lo = span[0].max(a);
                    let hi = span[1].min(b);
                    lo >= hi || (1..=3).all(|i| agrees(lo + (hi - lo) * i as f64 / 4.))
                })
                && supports_agree(leader, curve, tol)
            {
                return Some(Curve::BSplineCurve(leader.clone()));
            }
        }
    }
    if let Curve::PCurve(pcurve) = curve {
        if let Some(boundary) = affine_boundary(pcurve, tol) {
            return Some(boundary.into());
        }
        if let Some(arc) = extruded_circle_isocurve(pcurve) {
            return Some(arc);
        }
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
            Some(fit_procedural_curve(curve, tol)?.into())
        }
        _ => Some(curve.clone()),
    }
}

// A constant-v PCurve on an extruded round circle is an arc of that circle, translated.
// The UV spline stays in its control hull, so monotone u controls and v controls within
// TOLERANCE2 of the extrusion bound the arc exactly; a fitted cubic would leave the face.
fn extruded_circle_isocurve(pcurve: &PCurve<BSplineCurve<Point2>, Box<Surface>>) -> Option<Curve> {
    let Surface::Extruded(surface) = pcurve.surface().as_ref() else {
        return None;
    };
    let Curve::Conic(circle) = surface.entity_curve() else {
        return None;
    };
    let transform = circle.transform();
    let (x, y) = (transform[0].truncate(), transform[1].truncate());
    if !x.magnitude().near(&y.magnitude()) || !(x.dot(y) / x.magnitude2()).so_small() {
        return None;
    }
    let height = surface.extruding_vector().magnitude();
    let points = pcurve.curve().control_points();
    let v = points.first()?.y;
    if !height.is_finite()
        || points
            .iter()
            .any(|p| !p.x.is_finite() || !p.y.is_finite() || (p.y - v).abs() * height > TOLERANCE2)
    {
        return None;
    }
    let increasing = points.windows(2).all(|w| w[0].x < w[1].x);
    if !increasing && !points.windows(2).all(|w| w[0].x > w[1].x) {
        return None;
    }
    let (u0, u1) = (pcurve.curve().front().x, pcurve.curve().back().x);
    let (a, b) = circle.range_tuple();
    // Rounding may leave the chart by an ulp; the endpoint check below bounds the clamp.
    if u0.min(u1) < a - TOLERANCE2 || u0.max(u1) > b + TOLERANCE2 {
        return None;
    }
    let (lo, hi) = (u0.min(u1).max(a), u0.max(u1).min(b));
    if lo >= hi {
        return None;
    }
    let mut arc = circle.transformed(Matrix4::from_translation(surface.extruding_vector() * v));
    if hi < b {
        arc.cut(hi);
    }
    if lo > a {
        arc = arc.cut(lo);
    }
    if !increasing {
        arc.invert();
    }
    let arc = Curve::Conic(arc);
    (arc.front().near(&pcurve.front()) && arc.back().near(&pcurve.back())).then_some(arc)
}

// Restrict stored coefficients, rather than fitting another spatial curve. The
// narrow initial contract is a linear UV spline and a constant-weight boundary.
fn affine_boundary(
    pcurve: &PCurve<BSplineCurve<Point2>, Box<Surface>>,
    tol: f64,
) -> Option<BSplineCurve<Point3>> {
    let uv = pcurve.curve();
    let [p, q] = uv.control_points().as_slice() else {
        return None;
    };
    let (a, b) = uv.range_tuple();
    if uv.degree() != 1
        || uv.knot_vec().as_slice() != [a, a, b, b]
        || !a.is_finite()
        || !b.is_finite()
        || a >= b
        || ![p.x, p.y, q.x, q.y].iter().all(|x| x.is_finite())
    {
        return None;
    }
    let Surface::NurbsSurface(surface) = pcurve.surface().as_ref() else {
        return None;
    };
    let surface = surface.non_rationalized();
    let fixed = if p.x == q.x && p.y != q.y {
        0
    } else if p.y == q.y && p.x != q.x {
        1
    } else {
        return None;
    };
    let (fixed_knots, knots, fixed_degree, degree) = if fixed == 0 {
        (
            surface.uknot_vec(),
            surface.vknot_vec(),
            surface.udegree(),
            surface.vdegree(),
        )
    } else {
        (
            surface.vknot_vec(),
            surface.uknot_vec(),
            surface.vdegree(),
            surface.udegree(),
        )
    };
    if degree == 0
        || degree > 3
        || fixed_degree == 0
        || fixed_degree > 3
        || !fixed_knots
            .iter()
            .chain(knots.iter())
            .all(|x| x.is_finite())
    {
        return None;
    }
    let clamped = |k: &KnotVec, d: usize| {
        k.len() >= 2 * (d + 1)
            && k[0] < k[d + 1]
            && k[k.len() - d - 2] < k[k.len() - 1]
            && k[d + 1..k.len() - d - 1]
                .windows(d + 1)
                .all(|w| w[0] != w[d])
            && k[..=d].iter().all(|x| *x == k[0])
            && k[k.len() - d - 1..].iter().all(|x| *x == k[k.len() - 1])
            && k.windows(2).all(|w| w[0] <= w[1])
    };
    if !clamped(fixed_knots, fixed_degree) || !clamped(knots, degree) {
        return None;
    }
    let rows = surface.control_points();
    if rows
        .iter()
        .flatten()
        .any(|p| ![p.x, p.y, p.z, p.w].iter().all(|x| x.is_finite()) || p.w <= 0.)
    {
        return None;
    }
    let last = if fixed == 0 {
        rows.len() - 1
    } else {
        rows[0].len() - 1
    };
    let row = if p[fixed] == fixed_knots[0] {
        0
    } else if p[fixed] == fixed_knots[fixed_knots.len() - 1] {
        last
    } else {
        return None;
    };
    let poles: Vec<_> = if fixed == 0 {
        rows[row].clone()
    } else {
        rows.iter().map(|r| r[row]).collect()
    };
    let weight = poles[0].w;
    if !weight.is_normal()
        || weight < f64::MIN_POSITIVE.sqrt()
        || weight > f64::MAX.sqrt()
        || poles.iter().any(|p| {
            p.w != weight
                || ![p.x, p.y, p.z, p.w]
                    .iter()
                    .all(|x| x.abs() <= f64::MAX.sqrt())
        })
    {
        return None;
    }
    let varying = 1 - fixed;
    let slope = (q[varying] - p[varying]) / (b - a);
    let offset = p[varying] - slope * a;
    if !slope.is_normal() || !offset.is_finite() {
        return None;
    }
    let low = p[varying].min(q[varying]);
    let high = p[varying].max(q[varying]);
    if low < knots[0] || high > knots[knots.len() - 1] {
        return None;
    }
    let mut mapped: Vec<_> = knots.iter().map(|k| (k - offset) / slope).collect();
    if knots
        .windows(2)
        .zip(mapped.windows(2))
        .any(|(before, after)| before[0] != before[1] && after[0] == after[1])
    {
        return None;
    }
    let mut points: Vec<_> = poles
        .iter()
        .map(|p| {
            if weight == 1. {
                Point3::new(p.x, p.y, p.z)
            } else {
                p.to_point()
            }
        })
        .collect();
    if slope < 0. {
        mapped.reverse();
        points.reverse();
    }
    if !mapped.iter().all(|x| x.is_finite())
        || mapped.windows(2).any(|w| w[0] > w[1])
        || a < mapped[0]
        || b > mapped[mapped.len() - 1]
    {
        return None;
    }
    // Cuts introduce additional denominators. Check their positive spacings
    // before knot insertion; finite mapped knots alone do not make it safe.
    let safe_span = |width: f64| width.is_normal() && width > 0. && width.recip().is_finite();
    if !safe_span(b - a)
        || !safe_span(mapped[mapped.len() - 1] - mapped[0])
        || mapped.windows(2).any(|w| {
            (w[0] != w[1] && !safe_span(w[1] - w[0]))
                || [a, b]
                    .into_iter()
                    .any(|t| w[0] < t && t < w[1] && (!safe_span(t - w[0]) || !safe_span(w[1] - t)))
        })
    {
        return None;
    }
    let spacing = knots
        .windows(2)
        .filter(|w| w[0] < w[1])
        .map(|w| w[1] - w[0])
        .fold(f64::INFINITY, f64::min);
    let scale = knots
        .iter()
        .fold(low.abs().max(high.abs()), |m, x| m.max(x.abs()));
    let parameter_scale = slope.abs() * a.abs().max(b.abs()) + offset.abs() + scale;
    if (q[varying] - p[varying]).abs() <= 64. * f64::EPSILON * parameter_scale {
        return None;
    }
    if !spacing.is_normal() || !parameter_scale.is_finite() {
        return None;
    }
    // A rounded affine map must still reproduce both stored UV coefficients.
    let map_error = (slope * a + offset - p[varying])
        .abs()
        .max((slope * b + offset - q[varying]).abs());
    if map_error > 8. * f64::EPSILON * parameter_scale {
        return None;
    }
    let mut curve = BSplineCurve::try_new(mapped.into(), points).ok()?;
    let certificate =
        BoundaryCertificate::new(knots, &poles, &curve, (a, b), (p[varying], q[varying]))?;
    if a > curve.range_tuple().0 {
        curve = curve.cut(a);
    }
    if b < curve.range_tuple().1 {
        curve.cut(b);
    }
    if curve
        .control_points()
        .iter()
        .any(|p| ![p.x, p.y, p.z].into_iter().all(f64::is_finite))
    {
        return None;
    }
    let (knots, points) = curve.destruct();
    let curve = BSplineCurve::try_new(knots, points).ok()?;
    (certificate.error(&curve)? <= tol.min(SUPPORT_AGREEMENT)).then_some(curve)
}

fn fit_procedural_curve(curve: &Curve, tol: f64) -> Option<BSplineCurve<Point3>> {
    if let Curve::IntersectionCurve(intersection) = curve {
        if let Curve::BSplineCurve(leader) = intersection.leader().as_ref() {
            let (knots, _) = leader.knot_vec().to_single_multi();
            let (a, b) = curve.range_tuple();
            let mut remaining = curve.clone();
            let mut fitted: Option<BSplineCurve<Point3>> = None;
            // Cut the guide as well, so each span uses its own endpoint derivatives.
            for knot in knots.into_iter().filter(|&t| a < t && t < b).chain([b]) {
                let next = (knot < b).then(|| remaining.cut(knot));
                let piece = fit_procedural_span(&remaining, tol * 0.5)?;
                fitted = Some(match fitted {
                    None => piece,
                    Some(previous) => join_procedural_spans(&previous, piece, curve, tol)?,
                });
                if let Some(next) = next {
                    remaining = next;
                }
            }
            return fitted.filter(|spline| supports_agree(spline, curve, tol));
        }
    }
    fit_procedural_span(curve, tol)
}

fn join_procedural_spans(
    previous: &BSplineCurve<Point3>,
    mut piece: BSplineCurve<Point3>,
    curve: &Curve,
    tol: f64,
) -> Option<BSplineCurve<Point3>> {
    // Independent endpoint solves can differ. Charge the shared endpoint against
    // the support budget after overwriting it, as well as the fitting tolerance.
    let join = previous.back();
    if piece.front().distance(join) > tol * 0.5 {
        return None;
    }
    *piece.control_point_mut(0) = join;
    if !supports_agree(&piece, curve, tol) {
        return None;
    }
    previous.try_concat(&piece).ok()
}

// Half the target OCCT reader's default edge precision; parameter correspondence is separate.
const SUPPORT_AGREEMENT: f64 = 5e-8;

// A linear path in an extrusion's chart has curvature only in its profile.
// Restrict polynomial profiles to a single Bezier span: no derivative jump is hidden.
fn support_curvature(surface: &Surface) -> Option<f64> {
    let Surface::Extruded(surface) = surface else {
        return None;
    };
    match surface.entity_curve() {
        Curve::BSplineCurve(profile) => {
            if profile.degree() > 3 || profile.knot_vec().to_single_multi().0.len() != 2 {
                return None;
            }
            Some(
                profile
                    .derivation()
                    .derivation()
                    .control_points()
                    .iter()
                    .map(|v| v.magnitude())
                    .fold(0., f64::max),
            )
        }
        Curve::Conic(profile) => {
            let m = profile.transform();
            (m[0].w == 0. && m[1].w == 0. && m[2].w == 0. && m[3].w == 1.)
                .then(|| m[0].truncate().magnitude() + m[1].truncate().magnitude())
        }
        _ => None,
    }
}

fn supports_agree(spline: &BSplineCurve<Point3>, curve: &Curve, tol: f64) -> bool {
    let Curve::IntersectionCurve(intersection) = curve else {
        return true;
    };
    let supports = [
        intersection.surface0().as_ref(),
        intersection.surface1().as_ref(),
    ];
    let [Some(m0), Some(m1)] = supports.map(support_curvature) else {
        return true;
    };
    let budget = tol.min(SUPPORT_AGREEMENT);
    for (surface, curvature) in supports.into_iter().zip([m0, m1]) {
        let mut remaining = 131072usize;
        for span in spline.bezier_decomposition() {
            let source = span.control_points();
            if source.is_empty() || source.len() > 4 {
                return false;
            }
            let mut elevated = source.clone();
            while elevated.len() < 4 {
                let n = elevated.len();
                elevated = std::iter::once(elevated[0])
                    .chain((1..n).map(|i| {
                        elevated[i] + (elevated[i - 1] - elevated[i]) * (i as f64 / n as f64)
                    }))
                    .chain(std::iter::once(elevated[n - 1]))
                    .collect();
            }
            // At most three convex elevation passes. Charge their arithmetic
            // against the original stored guide, in addition to subdivision error.
            let scale = source
                .iter()
                .map(|p| p.to_vec().magnitude())
                .fold(1., f64::max);
            let ops = if source.len() < 4 {
                64. * f64::EPSILON
            } else {
                0.
            };
            let elevation_error = ops / (1. - ops) * scale;
            let Ok(points) = <[Point3; 4]>::try_from(elevated.as_slice()) else {
                return false;
            };
            if !elevation_error.is_finite()
                || elevation_error >= budget
                || !support_span(
                    points,
                    surface,
                    curvature,
                    budget - elevation_error,
                    0,
                    &mut remaining,
                )
            {
                return false;
            }
        }
    }
    true
}

fn support_span(
    p: [Point3; 4],
    surface: &Surface,
    curvature: f64,
    budget: f64,
    depth: usize,
    remaining: &mut usize,
) -> bool {
    if *remaining == 0 {
        return false;
    }
    *remaining -= 1;
    let Some(mut a) = surface.search_nearest_parameter(p[0], None, 100) else {
        return false;
    };
    let Some(mut b) = surface.search_nearest_parameter(p[3], Some(a), 100) else {
        return false;
    };
    let Surface::Extruded(extrusion) = surface else {
        return false;
    };
    let (lo, hi) = extrusion.entity_curve().range_tuple();
    if ![a.0, a.1, b.0, b.1, curvature]
        .into_iter()
        .all(f64::is_finite)
        || !lo.is_finite()
        || !hi.is_finite()
        || lo > hi
    {
        return false;
    }
    // Keep the witness path inside the retained chart. The chord hull charges
    // the full displacement for polynomial and conic profiles alike.
    a.0 = a.0.clamp(lo, hi);
    b.0 = b.0.clamp(lo, hi);
    let q = [surface.subs(a.0, a.1), surface.subs(b.0, b.1)];
    // Convexity bounds the stored cubic minus the endpoint chord. The chart
    // chord's interpolation remainder is M |du|²/8, independently of the solver:
    // any finite chart endpoints give a path on the represented support.
    let chord_error = (0..4)
        .map(|i| p[i].distance(q[0] + (q[1] - q[0]) * (i as f64 / 3.)))
        .fold(0., f64::max);
    let profile_scale = match extrusion.entity_curve() {
        Curve::BSplineCurve(profile) => profile
            .control_points()
            .iter()
            .map(|p| p.to_vec().magnitude())
            .sum::<f64>(),
        Curve::Conic(profile) => {
            if a.0.abs().max(b.0.abs()) > std::f64::consts::TAU {
                return false;
            }
            (0..4)
                .map(|i| profile.transform()[i].truncate().magnitude())
                .sum::<f64>()
        }
        _ => return false,
    };
    let scale = p
        .into_iter()
        .chain(q)
        .map(|x| x.to_vec().magnitude())
        .fold(1., f64::max);
    let scale =
        scale + profile_scale + extrusion.extruding_vector().magnitude() * a.1.abs().max(b.1.abs());
    // gamma(256) per subdivision level covers cubic de Casteljau/de Boor,
    // chart evaluation, chord arithmetic and the curvature remainder. Include
    // support coefficients before cancellation, not just the resulting points.
    let operations = 256. * (depth + 1) as f64;
    let roundoff = operations * f64::EPSILON / (1. - operations * f64::EPSILON) * scale;
    let bound = chord_error + curvature * (b.0 - a.0).powi(2) / 8. + roundoff;
    if bound.is_finite() && bound <= budget {
        return true;
    }
    if depth == 16 || !bound.is_finite() || roundoff >= budget {
        return false;
    }
    let a = p[0] + (p[1] - p[0]) * 0.5;
    let b = p[1] + (p[2] - p[1]) * 0.5;
    let c = p[2] + (p[3] - p[2]) * 0.5;
    let d = a + (b - a) * 0.5;
    let e = b + (c - b) * 0.5;
    let f = d + (e - d) * 0.5;
    support_span(
        [p[0], a, d, f],
        surface,
        curvature,
        budget,
        depth + 1,
        remaining,
    ) && support_span(
        [f, e, c, p[3]],
        surface,
        curvature,
        budget,
        depth + 1,
        remaining,
    )
}

fn fit_procedural_span(curve: &Curve, tol: f64) -> Option<BSplineCurve<Point3>> {
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
        if valid && supports_agree(&spline, curve, tol) {
            return Some(spline);
        }
    }
    None
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
        Surface::Extruded(extruded) => {
            let mut curve = prepare_curve(extruded.entity_curve(), tol)?;
            // Keep a partial profile's bounds; a complete ellipse retains its periodic chart.
            if let Curve::Conic(ellipse) = &curve {
                let (a, b) = ellipse.range_tuple();
                if surface.elementary().is_none() && !(b - a).near(&std::f64::consts::TAU) {
                    curve = Curve::NurbsCurve(NurbsCurve::try_from(&curve).ok()?);
                }
            }
            Some(Surface::Extruded(ExtrudedCurve::by_extrusion(
                curve,
                extruded.extruding_vector(),
            )))
        }
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
    fn stored_canal() -> Curve {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/i2-stored-canal.json"))
                .unwrap();
        assert_eq!(
            fixture["source_step_sha256"],
            "88e8492e7450fc7c80f0352e24f1432b7d622bd58521765b809aa82b1bc38d62"
        );
        serde_json::from_value(fixture["curve"].clone()).unwrap()
    }

    #[test]
    fn stored_canal_boundary_restriction() {
        for reverse in [false, true] {
            for swap in [false, true] {
                for moved in [false, true] {
                    for partial in [false, true] {
                        let Curve::PCurve(raw) = stored_canal() else {
                            unreachable!()
                        };
                        let (mut uv, mut surface) = raw.decompose();
                        if reverse {
                            uv.invert();
                        }
                        if swap {
                            surface.invert();
                            for p in uv.control_points_mut() {
                                std::mem::swap(&mut p.x, &mut p.y);
                            }
                        }
                        if moved {
                            surface.transform_by(
                                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                                    * Matrix4::from_axis_angle(
                                        Vector3::new(1., 2., 3.).normalize(),
                                        Rad(0.73),
                                    ),
                            );
                        }
                        if partial {
                            uv = uv.cut(-3.7);
                            uv.cut(-1.4);
                        }
                        let raw = Curve::PCurve(PCurve::new(uv, surface));
                        let before = serde_json::to_string(&raw).unwrap();
                        let prepared = prepare_curve(&raw, 0.00005).unwrap();
                        let Curve::BSplineCurve(ref spline) = prepared else {
                            panic!("boundary row")
                        };
                        assert!(
                            spline.control_points().len() > 100,
                            "must retain stored spans"
                        );
                        assert_eq!(prepared.range_tuple(), raw.range_tuple());
                        let (a, b) = raw.range_tuple();
                        for i in 0..=2048 {
                            let t = a + (b - a) * i as f64 / 2048.;
                            assert!(prepared.subs(t).distance(raw.subs(t)) < 5e-8);
                        }
                        assert_eq!(serde_json::to_string(&raw).unwrap(), before);
                    }
                }
            }
        }
    }

    #[test]
    fn boundary_restriction_upper_endpoint_and_scaled_parameter() {
        let surface = NurbsSurface::new(BSplineSurface::new(
            (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2)),
            vec![
                vec![
                    Vector4::new(0., 0., 0., 2.),
                    Vector4::new(0., 2., 0., 2.),
                    Vector4::new(0., 4., 0., 2.),
                ],
                vec![
                    Vector4::new(2., 0., 0., 2.),
                    Vector4::new(2., 2., 2., 2.),
                    Vector4::new(2., 4., 0., 2.),
                ],
            ],
        ));
        let uv = BSplineCurve::new(
            vec![2., 2., 3., 3.].into(),
            vec![Point2::new(1., 0.25), Point2::new(1., 0.75)],
        );
        let raw = PCurve::new(uv, Box::new(Surface::NurbsSurface(surface)));
        let result = affine_boundary(&raw, 0.00005).unwrap();
        assert_eq!(result.range_tuple(), (2., 3.));
        assert_eq!(result.degree(), 2);
        for i in 0..=100 {
            let t = 2. + i as f64 / 100.;
            assert!(raw.subs(t).distance(result.subs(t)) < 1e-12);
        }
        let invalid = BSplineCurve::new_unchecked(
            vec![2., 2., 2., 2.].into(),
            vec![Point2::new(1., 0.), Point2::new(1., 1.)],
        );
        assert!(affine_boundary(&PCurve::new(invalid, raw.surface().clone()), 0.00005).is_none());
    }

    #[test]
    fn boundary_restriction_rejects_unsafe_mapped_spans() {
        let row: Vec<_> = [0., 0.25, 0.5, 0.75, 1.]
            .into_iter()
            .map(|x| Vector4::new(x, 0., 0., 1.))
            .collect();
        let surface = Surface::NurbsSurface(NurbsSurface::new(BSplineSurface::new(
            (
                KnotVec::bezier_knot(1),
                vec![0., 0., 0.25, 0.5, 0.75, 1., 1.].into(),
            ),
            vec![row.clone(), row],
        )));
        // The incoming map passes its positional allowance, but cutting it
        // creates knot spans with unrepresentable derivative arithmetic.
        for (a, b, lo, hi) in [(0., 2e-308, 0.125, 0.875), (1e-308, 1., 1e-308, 1.)] {
            let uv = BSplineCurve::new(
                vec![a, a, b, b].into(),
                vec![Point2::new(0., lo), Point2::new(0., hi)],
            );
            let raw = PCurve::new(uv, Box::new(surface.clone()));
            let before = serde_json::to_string(&raw).unwrap();
            assert!(affine_boundary(&raw, 0.00005).is_none(), "domain {a}..{b}");
            assert_eq!(serde_json::to_string(&raw).unwrap(), before);
        }
    }

    #[test]
    fn boundary_restriction_rejects_ineligible_coefficients() {
        for kind in 0..6 {
            let Curve::PCurve(raw) = stored_canal() else {
                unreachable!()
            };
            let (mut uv, surface) = raw.decompose();
            match kind {
                0 => uv.control_point_mut(1).x = 0.1,
                1 => {
                    uv.control_point_mut(0).x = 0.1;
                    uv.control_point_mut(1).x = 0.1;
                }
                2 => *uv.control_point_mut(1) = *uv.control_point(0),
                3 => uv.control_point_mut(0).y = f64::NAN,
                4 => uv.control_point_mut(0).y = 6.,
                _ => uv.control_point_mut(1).y = uv.control_point(0).y - 1e-14,
            }
            let raw = PCurve::new(uv, surface);
            assert!(affine_boundary(&raw, 0.00005).is_none(), "case {kind}");
        }
    }

    #[test]
    fn retained_raw_support_preparation_regressions() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/step-support-regression-curves.json"
        ))
        .unwrap();
        let mut failures = Vec::new();
        for fixture in fixtures.as_array().unwrap() {
            let curve: Curve = serde_json::from_value(fixture["curve"].clone()).unwrap();
            let before = serde_json::to_string(&curve).unwrap();
            if let Some(prepared) = prepare_curve(&curve, 0.00005) {
                let Curve::BSplineCurve(spline) = &prepared else {
                    panic!("expected fitted intersection")
                };
                assert!(supports_agree(spline, &curve, 0.00005));
                let (a, b) = curve.range_tuple();
                for i in 0..=256 {
                    let t = a + (b - a) * i as f64 / 256.;
                    assert!(prepared.subs(t).distance(curve.subs(t)) <= 0.00005);
                }
            } else {
                failures.push(format!("{} {}", fixture["case"], fixture["edge"]));
            }
            assert_eq!(before, serde_json::to_string(&curve).unwrap());
        }
        assert!(failures.is_empty(), "failed raw curves: {failures:?}");
    }

    #[test]
    fn bounded_conic_chart_charges_endpoint_displacement() {
        let profile = Curve::Conic(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0., std::f64::consts::FRAC_PI_2)),
            Matrix4::identity(),
        ));
        let surface = Surface::Extruded(ExtrudedCurve::by_extrusion(profile, Vector3::unit_z()));
        for (angle, expected) in [(-1e-8, true), (-1e-6, false)] {
            let point = surface.subs(angle, 0.);
            let uv = surface.search_nearest_parameter(point, None, 100).unwrap();
            assert!(uv.0 < 0.);
            assert_eq!(
                support_span(
                    [point; 4],
                    &surface,
                    support_curvature(&surface).unwrap(),
                    SUPPORT_AGREEMENT,
                    0,
                    &mut 131072
                ),
                expected
            );
        }
    }

    #[test]
    fn bounded_chart_endpoint_witness() {
        let profile = BSplineCurve::new(
            KnotVec::bezier_knot(3),
            (0..4).map(|i| Point3::new(i as f64 / 3., 0., 0.)).collect(),
        );
        let surface = Surface::Extruded(ExtrudedCurve::by_extrusion(
            profile.into(),
            Vector3::unit_y(),
        ));
        for (offset, expected) in [(-1e-8, true), (-1e-6, false)] {
            let p = std::array::from_fn(|i| Point3::new(offset + i as f64 / 3., 0., 0.));
            let uv = surface.search_nearest_parameter(p[0], None, 100).unwrap();
            assert!(uv.0 < 0., "witness must exercise original chart rejection");
            assert_eq!(
                support_span(
                    p,
                    &surface,
                    support_curvature(&surface).unwrap(),
                    SUPPORT_AGREEMENT,
                    0,
                    &mut 131072
                ),
                expected
            );
        }
    }
    #[test]
    fn eligible_linear_and_quadratic_guide_certificate() {
        for degree in [1, 2] {
            for offset in [0., 1e-6] {
                let guide = BSplineCurve::new(
                    KnotVec::bezier_knot(degree),
                    (0..=degree)
                        .map(|i| Point3::new(i as f64 / degree as f64, 0., offset))
                        .collect(),
                );
                let curve = planar_extrusion_intersection(guide.clone());
                assert_eq!(supports_agree(&guide, &curve, 0.00005), offset == 0.);
            }
        }
    }

    fn planar_extrusion_intersection(guide: BSplineCurve<Point3>) -> Curve {
        let profile = BSplineCurve::new(
            KnotVec::bezier_knot(3),
            vec![
                Point3::new(-2., 0., 0.),
                Point3::new(-2. / 3., 0., 0.),
                Point3::new(2. / 3., 0., 0.),
                Point3::new(2., 0., 0.),
            ],
        );
        Curve::IntersectionCurve(IntersectionCurve::new(
            Box::new(Surface::Extruded(ExtrudedCurve::by_extrusion(
                profile.clone().into(),
                Vector3::unit_y(),
            ))),
            Box::new(Surface::Extruded(ExtrudedCurve::by_extrusion(
                profile.into(),
                Vector3::unit_z(),
            ))),
            Box::new(Curve::BSplineCurve(guide)),
        ))
    }

    #[test]
    fn intersection_guide_return_respects_support_budget() {
        let guide = BSplineCurve::new(
            KnotVec::bezier_knot(3),
            (0..4)
                .map(|i| Point3::new(i as f64 / 3., 0., 1e-6))
                .collect(),
        );
        let curve = planar_extrusion_intersection(guide);
        let before = serde_json::to_string(&curve).unwrap();
        let prepared = prepare_curve(&curve, 0.00005).expect("straight intersection fit");
        for i in 0..=128 {
            let point = prepared.subs(i as f64 / 128.);
            assert!(point.y.abs().max(point.z.abs()) <= SUPPORT_AGREEMENT);
        }
        assert_eq!(serde_json::to_string(&curve).unwrap(), before);
    }

    #[test]
    fn intersection_join_rechecks_overwritten_endpoint() {
        let piece = BSplineCurve::new(
            KnotVec::bezier_knot(3),
            [-4., 8., 0., 0.]
                .into_iter()
                .enumerate()
                .map(|(i, z)| Point3::new(i as f64 / 3., 0., z * 1e-8))
                .collect(),
        );
        let curve = planar_extrusion_intersection(piece.clone());
        let tol = 0.00005;
        assert!(supports_agree(&piece, &curve, tol));
        for height in [-4e-8, 4e-8] {
            let previous = BSplineCurve::new(
                vec![-1., -1., -1., -1., 0., 0., 0., 0.].into(),
                (0..4)
                    .map(|i| Point3::new(-1. + i as f64 / 3., 0., height))
                    .collect(),
            );
            assert!(supports_agree(&previous, &curve, tol));
            let joined = join_procedural_spans(&previous, piece.clone(), &curve, tol);
            if height > 0. {
                let mut unchecked = piece.clone();
                *unchecked.control_point_mut(0) = previous.back();
                assert!(unchecked.subs(0.2).z > SUPPORT_AGREEMENT);
                assert!(
                    joined.is_none(),
                    "joining must not invalidate the support bound"
                );
            } else {
                let joined = joined.expect("unchanged endpoint remains admissible");
                assert!(supports_agree(&joined, &curve, tol));
                for i in 0..=256 {
                    assert!(joined.subs(-1. + i as f64 / 128.).z.abs() <= SUPPORT_AGREEMENT);
                }
            }
        }
    }

    #[test]
    fn g8_preparation_preserves_incident_support_agreement() {
        let raw: Curve =
            serde_json::from_str(include_str!("../../tests/fixtures/g8-support-curve.json"))
                .unwrap();
        let before = serde_json::to_string(&raw).unwrap();
        let prepared = prepare_curve(&raw, 0.00005).expect("bounded support fit");
        let Curve::IntersectionCurve(intersection) = &raw else {
            unreachable!()
        };
        let cylinder_error = |p: Point3| {
            ((p.y - 24.099019513592765).hypot(p.z - 95.00000000000003) - 4.999999999999975).abs()
        };
        for t in (0..=2048)
            .map(|i| i as f64 / 2048.)
            .chain([0.031258539993374024])
        {
            let (p, uv0, uv1) = intersection.search_triple(t, 100).unwrap();
            assert!(cylinder_error(p) < 1e-9);
            assert!(p.distance(intersection.surface0().subs(uv0.x, uv0.y)) < 1e-9);
            assert!(p.distance(intersection.surface1().subs(uv1.x, uv1.y)) < 1e-9);
            let q = prepared.subs(t);
            assert!(cylinder_error(q) <= SUPPORT_AGREEMENT);
            assert!(p.distance(q) <= 0.00005);
            for surface in [intersection.surface0(), intersection.surface1()] {
                let uv = surface.search_nearest_parameter(q, None, 100).unwrap();
                assert!(q.distance(surface.subs(uv.0, uv.1)) <= SUPPORT_AGREEMENT);
            }
        }
        assert_eq!(before, serde_json::to_string(&raw).unwrap());
        let Curve::BSplineCurve(spline) = prepared else {
            unreachable!()
        };
        assert!(supports_agree(&spline, &raw, 0.00005));
    }

    #[test]
    fn intersection_export_does_not_substitute_its_polyline_guide() {
        let parabola = |t: f64| {
            let x = 2. * t - 1.;
            Point3::new(x, 0., x * x)
        };
        let profile = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(-1., 0., 1.),
                Point3::new(0., 0., -1.),
                Point3::new(1., 0., 1.),
            ],
        );
        let surface = Surface::Extruded(ExtrudedCurve::by_extrusion(
            profile.into(),
            Vector3::unit_y(),
        ));
        let plane = Surface::Plane(Plane::new(
            Point3::origin(),
            Point3::new(1., 0., 0.),
            Point3::new(0., 0., 1.),
        ));
        // All 65 old shortcut samples hit exact guide nodes, hiding the chord error.
        let guide = BSplineCurve::new(
            KnotVec::from_iter(
                std::iter::once(0.)
                    .chain((0..=64).map(|i| i as f64 / 64.))
                    .chain(std::iter::once(1.)),
            ),
            (0..=64).map(|i| parabola(i as f64 / 64.)).collect(),
        );
        let curve = Curve::IntersectionCurve(IntersectionCurve::new(
            Box::new(surface),
            Box::new(plane),
            Box::new(Curve::BSplineCurve(guide)),
        ));
        let tol = 0.00005;
        let prepared = prepare_curve(&curve, tol).unwrap();
        let Curve::BSplineCurve(spline) = &prepared else {
            panic!("expected a fitted spline");
        };
        let (_, multiplicities) = spline.knot_vec().to_single_multi();
        assert!(multiplicities[1..multiplicities.len() - 1]
            .iter()
            .all(|&m| m <= spline.degree()));
        for i in 0..=1024 {
            let t = i as f64 / 1024.;
            let raw = curve.subs(t);
            let point = prepared.subs(t);
            assert!(raw.y.abs() < 1e-8 && (raw.z - raw.x * raw.x).abs() < 1e-8);
            assert!(point.y.abs() <= tol && (point.z - point.x * point.x).abs() <= tol);
            assert!(point.distance(raw) <= tol);
        }
        assert!(prepared.front().distance(parabola(0.)) < 1e-10);
        assert!(prepared.back().distance(parabola(1.)) < 1e-10);
    }

    #[test]
    fn extruded_circle_isocurves_export_as_exact_arcs() {
        use std::f64::consts::FRAC_PI_2;
        for inverted in [false, true] {
            for decreasing in [false, true] {
                let matrix = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
                let mut profile = Processor::with_transform(
                    TrimmedCurve::new(UnitCircle::new(), (0., FRAC_PI_2)),
                    matrix * Matrix4::from_scale(12.),
                );
                if inverted {
                    profile.invert();
                }
                let (a, b) = profile.range_tuple();
                let surface = Surface::Extruded(ExtrudedCurve::by_extrusion(
                    Curve::Conic(profile),
                    matrix.transform_vector(Vector3::new(0., 0., 23.5)),
                ));
                // A chamfer contact stores its UV trace as a fitted cubic: u is not linear in t,
                // v carries ulp noise, and the first u leaves the chart by a rounding step.
                let mut controls: Vec<_> = (0..=12)
                    .map(|i| {
                        let s = i as f64 / 12.;
                        let u = a + (b - a) * (s + 0.02 * (std::f64::consts::PI * s).sin());
                        Point2::new(u, 0.9791666666666666 + f64::EPSILON * (i % 3) as f64)
                    })
                    .collect();
                controls[0].x = a - 4e-33;
                if decreasing {
                    controls.reverse();
                }
                let n = controls.len();
                let uv = BSplineCurve::new(raw_knots(n), controls.clone());
                let raw = Curve::PCurve(PCurve::new(uv, Box::new(surface.clone())));
                let prepared = prepare_curve(&raw, 0.00005).unwrap();
                let Curve::Conic(_) = prepared else {
                    panic!("isocurve must stay an exact arc: {inverted} {decreasing}")
                };
                assert!(prepared.front().near2(&raw.front()));
                assert!(prepared.back().near2(&raw.back()));
                let (s, t) = prepared.range_tuple();
                for i in 0..=256 {
                    let p = raw.subs(i as f64 / 256.);
                    let q = prepared.subs(s + (t - s) * i as f64 / 256.);
                    let on = |c: &Curve, p| {
                        let u = c.search_nearest_parameter(p, None, 100).unwrap();
                        c.subs(u).distance(p)
                    };
                    assert!(on(&prepared, p) < 1e-12);
                    let (u, v) = surface.search_nearest_parameter(q, None, 100).unwrap();
                    assert!(surface.subs(u, v).distance(q) < 1e-12);
                }
                for (index, offset) in [(5, Vector2::new(0., 1e-6)), (5, Vector2::new(-1., 0.))] {
                    let mut bent = controls.clone();
                    bent[index] += offset;
                    let uv = BSplineCurve::new(raw_knots(n), bent);
                    let bent = Curve::PCurve(PCurve::new(uv, Box::new(surface.clone())));
                    assert!(!matches!(
                        prepare_curve(&bent, 0.00005),
                        Some(Curve::Conic(_))
                    ));
                }
            }
        }
        fn raw_knots(n: usize) -> KnotVec {
            let mut knots = vec![0.; 4];
            knots.extend((1..n - 3).map(|i| i as f64 / (n - 3) as f64));
            knots.extend([1.; 4]);
            knots.into()
        }
    }

    #[test]
    fn extruded_ellipse_step_preserves_trim_and_normal_direction() {
        use crate::{
            out::*,
            r#in::{step_geometry::StepExtrudedCurve, SurfaceOfLinearExtrusionHolder, Table},
        };
        use ruststep::{ast::DataSection, tables::EntityTable};
        use std::{f64::consts::TAU, str::FromStr};
        for range in [(0., TAU), (0.3, 4.1)] {
            for reversed in [false, true] {
                for moved in [false, true] {
                    let matrix = if moved {
                        Matrix4::from_translation(Vector3::new(13., -7., 23.))
                            * Matrix4::from_axis_angle(
                                Vector3::new(1., 2., 3.).normalize(),
                                Rad(0.73),
                            )
                    } else {
                        Matrix4::identity()
                    };
                    let mut profile = Processor::with_transform(
                        TrimmedCurve::new(UnitCircle::new(), range),
                        matrix * Matrix4::from_nonuniform_scale(2., 1., 1.),
                    );
                    if reversed {
                        profile.invert();
                    }
                    let original = Surface::Extruded(ExtrudedCurve::by_extrusion(
                        Curve::Conic(profile),
                        matrix.transform_vector(Vector3::new(1., 0., 3.)),
                    ));
                    let prepared = prepare_surface(&original, &[], 1e-5).unwrap();
                    let data = format!("DATA;{}ENDSEC;", StepDataDisplay::new(&prepared, 1));
                    let table = Table::from_data_section(&DataSection::from_str(&data).unwrap());
                    let entity =
                        EntityTable::<SurfaceOfLinearExtrusionHolder>::get_owned(&table, 1)
                            .unwrap();
                    let read = StepExtrudedCurve::try_from(&entity).unwrap();
                    let (a, b) = read.entity_curve().range_tuple();
                    for v in [0., 0.5, 1.] {
                        assert!(read.subs(a, v).distance(original.subs(range.0, v)) < 1e-10);
                        assert!(read.subs(b, v).distance(original.subs(range.1, v)) < 1e-10);
                        for i in 0..=64 {
                            let u = range.0 + (range.1 - range.0) * i as f64 / 64.;
                            let p = original.subs(u, v);
                            let (s, t) = read.search_parameter(p, None, 100).unwrap();
                            assert!(read.subs(s, t).distance(p) < 1e-9);
                            let normal =
                                read.normal(s, t) * if prepared.same_sense() { 1. } else { -1. };
                            assert!(normal.dot(original.normal(u, v)) > 1. - 1e-10);
                        }
                    }
                }
            }
        }
    }

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
