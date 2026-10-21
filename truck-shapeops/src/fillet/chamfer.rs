use super::*;
use std::result::Result;
use truck_base::cgmath64::control_point::ControlPoint;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};

/// Number of samples along the edge is at least this, so a cubic interpolation is always possible.
const MIN_SAMPLES: usize = 4;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum DistanceMode {
    TangentProjection,
    NormalSection,
    NormalSectionAngle { reference_side: bool, radians: f64 },
}

fn contact_parameter<S>(
    surface: &S,
    point: Point3,
    tangent: Vector3,
    distance: f64,
    sign: f64,
    hint: Option<(f64, f64)>,
    mode: DistanceMode,
) -> Option<(f64, f64)>
where
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    let origin = surface.search_parameter(point, hint, 100)?;
    let normal = surface.normal(origin.0, origin.1);
    let inward = normal.cross(tangent) * sign;
    let mut uv = surface.search_nearest_parameter(point + inward * distance, Some(origin), 100)?;
    if mode == DistanceMode::TangentProjection {
        return Some(uv);
    }
    // Intersect the support with the edge's normal plane and the requested distance sphere.
    let residual = |uv: (f64, f64)| {
        let delta = surface.subs(uv.0, uv.1) - point;
        Vector2::new(
            delta.dot(tangent),
            (delta.magnitude2() - distance * distance) / (2. * distance),
        )
    };
    let tolerance = (distance * 1e-10).max(1e-12);
    for _ in 0..32 {
        let delta = surface.subs(uv.0, uv.1) - point;
        let value = residual(uv);
        if !value.x.is_finite() || !value.y.is_finite() || delta.dot(inward) <= 0. {
            return None;
        }
        if value.magnitude2() <= tolerance * tolerance {
            return (surface.normal(uv.0, uv.1).dot(normal) > 0.).then_some(uv);
        }
        let du = surface.uder(uv.0, uv.1);
        let dv = surface.vder(uv.0, uv.1);
        let a = Vector2::new(du.dot(tangent), du.dot(delta) / distance);
        let b = Vector2::new(dv.dot(tangent), dv.dot(delta) / distance);
        let det = a.x * b.y - b.x * a.y;
        if !det.is_finite() || det.abs() <= 1e-12 * a.magnitude() * b.magnitude() {
            return None;
        }
        let step = Vector2::new(
            (b.y * value.x - b.x * value.y) / det,
            (a.x * value.y - a.y * value.x) / det,
        );
        uv = (0..8).find_map(|i| {
            let step = step / 2_f64.powi(i);
            let next = (uv.0 - step.x, uv.1 - step.y);
            (next.0.is_finite()
                && next.1.is_finite()
                && residual(next).magnitude2() < value.magnitude2())
            .then_some(next)
        })?;
    }
    None
}

fn angle_contact_parameter<S>(
    surface: &S,
    guide: Point3,
    tangent: Vector3,
    (reference, reference_normal): (Point3, Vector3),
    reference_sign: f64,
    angle: f64,
    hint: Option<(f64, f64)>,
) -> Option<(f64, f64)>
where
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    let origin = surface.search_parameter(guide, hint, 100)?;
    let normal = surface.normal(origin.0, origin.1);
    let inward = normal.cross(tangent) * -reference_sign;
    let section = reference_normal.cross(tangent).normalize() * reference_sign;
    let direction =
        -section * angle.cos() - tangent.cross(section) * (reference_sign * angle.sin());
    let perpendicular = tangent.cross(direction);
    let length = (guide - reference).dot(normal) / direction.dot(normal);
    if !length.is_finite() || length <= 0. {
        return None;
    }
    let mut uv =
        surface.search_nearest_parameter(reference + direction * length, Some(origin), 100)?;
    // The bevel ray is the intersection of two planes through the reference contact.
    let residual = |uv: (f64, f64)| {
        let delta = surface.subs(uv.0, uv.1) - reference;
        Vector2::new(delta.dot(tangent), delta.dot(perpendicular))
    };
    let tolerance = (guide.distance(reference) * 1e-10).max(1e-12);
    for _ in 0..32 {
        let point = surface.subs(uv.0, uv.1);
        let value = residual(uv);
        if !value.x.is_finite() || !value.y.is_finite() || (point - guide).dot(inward) <= 0. {
            return None;
        }
        if value.magnitude2() <= tolerance * tolerance {
            return ((point - reference).dot(direction) > 0.
                && surface.normal(uv.0, uv.1).dot(normal) > 0.)
                .then_some(uv);
        }
        let du = surface.uder(uv.0, uv.1);
        let dv = surface.vder(uv.0, uv.1);
        let a = Vector2::new(du.dot(tangent), du.dot(perpendicular));
        let b = Vector2::new(dv.dot(tangent), dv.dot(perpendicular));
        let det = a.x * b.y - b.x * a.y;
        if !det.is_finite() || det.abs() <= 1e-12 * a.magnitude() * b.magnitude() {
            return None;
        }
        let step = Vector2::new(
            (b.y * value.x - b.x * value.y) / det,
            (a.x * value.y - a.y * value.x) / det,
        );
        uv = (0..8).find_map(|i| {
            let step = step / 2_f64.powi(i);
            let next = (uv.0 - step.x, uv.1 - step.y);
            (next.0.is_finite()
                && next.1.is_finite()
                && residual(next).magnitude2() < value.magnitude2())
            .then_some(next)
        })?;
    }
    None
}

fn unwrap_contact<S: ParametricSurface3D>(
    surface: &S,
    uv: &mut (f64, f64),
    hint: Option<(f64, f64)>,
) {
    if let Some((u, v)) = hint {
        if let Some(period) = surface.u_period() {
            uv.0 += ((u - uv.0) / period).round() * period;
        }
        if let Some(period) = surface.v_period() {
            uv.1 += ((v - uv.1) / period).round() * period;
        }
    }
}

fn contact_pair<S>(
    surface0: &S,
    surface1: &S,
    point: Point3,
    tangent: Vector3,
    (d0, d1): (f64, f64),
    hints: &mut [Option<(f64, f64)>; 2],
    mode: DistanceMode,
) -> Option<[(Point2, Point3); 2]>
where
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    let surfaces = [surface0, surface1];
    let mut parameters = if let DistanceMode::NormalSectionAngle {
        reference_side,
        radians,
    } = mode
    {
        let side = usize::from(reference_side);
        let sign = if reference_side { -1. } else { 1. };
        let reference = surfaces[side];
        let uv = contact_parameter(reference, point, tangent, d0, sign, hints[side], mode)?;
        let other = angle_contact_parameter(
            surfaces[1 - side],
            point,
            tangent,
            (reference.subs(uv.0, uv.1), reference.normal(uv.0, uv.1)),
            sign,
            radians,
            hints[1 - side],
        )?;
        if reference_side {
            [other, uv]
        } else {
            [uv, other]
        }
    } else {
        [
            contact_parameter(surface0, point, tangent, d0, 1., hints[0], mode)?,
            contact_parameter(surface1, point, tangent, d1, -1., hints[1], mode)?,
        ]
    };
    for side in 0..2 {
        unwrap_contact(surfaces[side], &mut parameters[side], hints[side]);
        hints[side] = Some(parameters[side]);
    }
    Some(std::array::from_fn(|side| {
        let uv = parameters[side];
        (Point2::from(uv), surfaces[side].subs(uv.0, uv.1))
    }))
}

/// Samples the contact curves of a chamfer along `edge` using the requested section constraints.
#[allow(clippy::type_complexity)]
fn sample_contact_points<C, S>(
    edge: &C,
    surface0: &S,
    surface1: &S,
    (d0, d1): (f64, f64),
    params: &[f64],
    mode: DistanceMode,
) -> Option<[Vec<(Point2, Point3)>; 2]>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
{
    let mut samples = [
        Vec::with_capacity(params.len()),
        Vec::with_capacity(params.len()),
    ];
    let mut hints = [None; 2];
    for &t in params {
        let pair = contact_pair(
            surface0,
            surface1,
            edge.subs(t),
            edge.der(t).normalize(),
            (d0, d1),
            &mut hints,
            mode,
        )?;
        for (samples, contact) in samples.iter_mut().zip(pair) {
            samples.push(contact);
        }
    }
    Some(samples)
}

fn interpolate<P>(params: &[f64], points: impl Iterator<Item = P>) -> BSplineCurve<P>
where P: ControlPoint<f64> + Copy {
    let (t0, t1) = (params[0], params[params.len() - 1]);
    let averaged_knots = params[1..params.len() - 1]
        .windows(3)
        .map(|p| p[0] + (p[1] - p[0]) / 3. + (p[2] - p[0]) / 3.);
    let knot_vec = KnotVec::from(
        [t0; 4]
            .into_iter()
            .chain(averaged_knots)
            .chain([t1; 4])
            .collect::<Vec<_>>(),
    );
    let parameter_points: Vec<_> = params.iter().copied().zip(points).collect();
    BSplineCurve::interpole(knot_vec, parameter_points)
}

#[allow(clippy::type_complexity)]
fn fit_contact_curves<C, S>(
    edge: &C,
    surface0: &S,
    surface1: &S,
    distances: (f64, f64),
    tol: f64,
    range: Option<(f64, f64)>,
    mode: DistanceMode,
) -> Option<(
    Vec<f64>,
    [PCurve<BSplineCurve<Point2>, S>; 2],
    [BSplineCurve<Point3>; 2],
)>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
{
    let (t0, t1) = range.unwrap_or_else(|| edge.range_tuple());
    let count = edge
        .parameter_division((t0, t1), tol)
        .0
        .len()
        .max(MIN_SAMPLES);
    let budget = count.max(257);
    let fit_tolerance2 = tol.min(TOLERANCE * 0.1).powi(2);
    let uniform = |count: usize| {
        (0..count)
            .map(|i| t0 + (t1 - t0) * i as f64 / (count - 1) as f64)
            .collect::<Vec<_>>()
    };
    let mut params = uniform(count);
    let mut local_refinements = 0;
    loop {
        let samples = sample_contact_points(edge, surface0, surface1, distances, &params, mode)?;
        let pcurves = std::array::from_fn(|i| {
            PCurve::new(
                interpolate(&params, samples[i].iter().map(|s| s.0)),
                if i == 0 {
                    surface0.clone()
                } else {
                    surface1.clone()
                },
            )
        });
        let curves = std::array::from_fn(|i| interpolate(&params, samples[i].iter().map(|s| s.1)));
        // The ruled surface must contain its support pcurves at kernel consistency tolerance.
        let fits = |pair: &[f64]| {
            (0..=4).all(|i| {
                let t = pair[0] + (pair[1] - pair[0]) * i as f64 / 4.;
                (0..2).all(|side| {
                    let point = pcurves[side].subs(t);
                    point.distance2(curves[side].subs(t)) <= fit_tolerance2
                        || curves[side]
                            .search_nearest_parameter(point, Some(t), 100)
                            .filter(|u| u.is_finite())
                            .is_some_and(|u| {
                                point.distance2(curves[side].subs(u.clamp(t0, t1)))
                                    <= fit_tolerance2
                            })
                })
            })
        };
        if params.windows(2).all(fits) {
            return Some((params, pcurves, curves));
        }
        let count = params.len().checked_mul(2)?.checked_sub(1)?;
        if count <= budget {
            params = uniform(count);
        } else {
            if local_refinements == 8 {
                return None;
            }
            local_refinements += 1;
            let refinement: Option<Vec<_>> = params
                .windows(2)
                .filter(|pair| !fits(pair))
                .map(|pair| {
                    let midpoint = pair[0] + (pair[1] - pair[0]) / 2.;
                    (pair[0] < midpoint && midpoint < pair[1]).then_some(midpoint)
                })
                .collect();
            let refinement = refinement?;
            if refinement.is_empty() || params.len() + refinement.len() > budget {
                return None;
            }
            params.extend(refinement);
            params.sort_by(f64::total_cmp);
        }
    }
}

fn constrain_contact<C, S>(
    source: &PCurve<BSplineCurve<Point2>, S>,
    edge: &Edge<Point3, C>,
    tol: f64,
) -> Option<C>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
{
    let old = edge.oriented_curve();
    let (a, b) = old.range_tuple();
    if old.subs(a).near(&edge.front().point()) && old.subs(b).near(&edge.back().point()) {
        return Some(old);
    }
    // Inversion followed by trimming can shift the parameter interval.
    let original_curve: C = source.to_same_geometry();
    let a = original_curve.search_nearest_parameter(old.subs(a), None, 100)?;
    let b = original_curve.search_nearest_parameter(old.subs(b), None, 100)?;
    if !a.is_finite() || !b.is_finite() || a >= b {
        return None;
    }
    let (mut curve, surface) = source.clone().decompose();
    curve = curve.cut(a);
    curve.cut(b);
    let original = PCurve::new(curve.clone(), surface.clone());
    let endpoints = [(a, edge.front().point()), (b, edge.back().point())];
    let mut shifts = Vec::new();
    for (t, p) in endpoints {
        if original.subs(t).distance(p) > tol {
            return None;
        }
        let hint = curve.subs(t);
        let (mut u, mut v) = surface.search_parameter(p, Some((hint.x, hint.y)), 100)?;
        if let Some(period) = surface.u_period() {
            u += ((hint.x - u) / period).round() * period;
        }
        if let Some(period) = surface.v_period() {
            v += ((hint.y - v) / period).round() * period;
        }
        shifts.push(Point2::new(u, v) - hint);
    }
    // Constrain the endpoint and its tangent on the original support, within caller tolerance.
    let count = curve.control_points().len();
    for (i, p) in curve.control_points_mut().enumerate() {
        if i < 2 {
            *p += shifts[0];
        }
        if i + 2 >= count {
            *p += shifts[1];
        }
    }
    let corrected = PCurve::new(curve, surface);
    if (0..=32).any(|i| {
        let t = a + (b - a) * i as f64 / 32.;
        corrected.subs(t).distance(original.subs(t)) > tol
            || corrected.der(t).dot(original.der(t)) <= 0.
    }) {
        return None;
    }
    Some(corrected.to_same_geometry())
}

fn fit_trimmed_contacts<C: ParametricCurve3D + BoundedCurve>(
    curves: &[C; 2],
    tol: f64,
) -> Option<[BSplineCurve<Point3>; 2]> {
    let ranges = curves.each_ref().map(|c| c.range_tuple());
    let point = |side: usize, t: f64| {
        let (a, b) = ranges[side];
        curves[side].subs(a + (b - a) * t)
    };
    let mut count = 5;
    while count <= 257 {
        let params: Vec<_> = (0..count).map(|i| i as f64 / (count - 1) as f64).collect();
        let fit: [BSplineCurve<Point3>; 2] = std::array::from_fn(|side| {
            interpolate(&params, params.iter().map(|&t| point(side, t)))
        });
        let good = params.windows(2).all(|w| {
            (0..=4).all(|i| {
                let t = w[0] + (w[1] - w[0]) * i as f64 / 4.;
                (0..2).all(|side| {
                    point(side, t).distance(fit[side].subs(t)) <= tol.min(TOLERANCE * 0.1)
                })
            })
        });
        if good {
            return Some(fit);
        }
        count = 2 * count - 1;
    }
    None
}

fn ruling_edge<C, S>(
    (v0, uv0): (&Vertex<Point3>, Point2),
    (v1, uv1): (&Vertex<Point3>, Point2),
    surface: S,
) -> Edge<Point3, C>
where
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
{
    let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![uv0, uv1]);
    Edge::new(v0, v1, PCurve::new(bsp, surface).to_same_geometry())
}

/// Creates a chamfer along the edge `chamfered_edge_id` shared by `face0` and `face1`.
///
/// The chamfer surface is the ruled surface between the contact curve on `face0`, at distance
/// `d0` from the edge, and the contact curve on `face1`, at distance `d1`. Distances are measured
/// along the surface tangent perpendicular to the edge and projected back onto the surface, which
/// is exact for planar faces.
///
/// The returned faces have the same layout as [`simple_fillet`].
pub fn simple_chamfer<C, S>(
    face0: &Face<Point3, C, S>,
    face1: &Face<Point3, C, S>,
    chamfered_edge_id: EdgeID<C>,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Option<SimpleFillet<C, S>>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    BSplineSurface<Point3>: ToSameGeometry<S>,
{
    let chamfered_edge = face0
        .edge_iter()
        .find(|edge| edge.id() == chamfered_edge_id)?;
    let surface0 = face0.oriented_surface();
    let surface1 = face1.oriented_surface();
    let edge_curve = chamfered_edge.oriented_curve();

    let (params, [pcurve0, pcurve1], [curve0, curve1]) = fit_contact_curves(
        &edge_curve,
        &surface0,
        &surface1,
        (d0, d1),
        tol,
        None,
        DistanceMode::TangentProjection,
    )?;

    let (new_face0, chamfer_edge0, _) =
        cut_face_by_curve(face0, pcurve0.to_same_geometry(), chamfered_edge_id)?;
    let (new_face1, chamfer_edge1, _) = cut_face_by_curve(
        face1,
        pcurve1.to_same_geometry().inverse(),
        chamfered_edge_id,
    )?;

    // Orient the ruled surface outward: its normal must agree with the sum of the face normals.
    let outward = {
        let t = (params[0] + params[params.len() - 1]) / 2.0;
        let p = edge_curve.subs(t);
        let normal = |surface: &S| {
            let (u, v) = surface.search_parameter(p, None, 100)?;
            Some(surface.normal(u, v))
        };
        normal(&surface0)? + normal(&surface1)?
    };
    let mut chamfer_surface = BSplineSurface::homotopy(curve0.clone(), curve1.clone());
    let ((u0, u1), (v0, v1)) = chamfer_surface.range_tuple();
    let mid_normal = chamfer_surface.normal((u0 + u1) / 2.0, (v0 + v1) / 2.0);
    let (v_side0, v_side1) = if mid_normal.dot(outward) >= 0.0 {
        (v0, v1)
    } else {
        chamfer_surface = BSplineSurface::homotopy(curve1, curve0);
        (v1, v0)
    };
    let surface = chamfer_surface.to_same_geometry();
    let ((vertex0, vertex1), (vertex2, vertex3)) = (chamfer_edge0.ends(), chamfer_edge1.ends());
    let edge0 = ruling_edge(
        (vertex0, Point2::new(u0, v_side0)),
        (vertex3, Point2::new(u0, v_side1)),
        surface.clone(),
    );
    let edge1 = ruling_edge(
        (vertex2, Point2::new(u1, v_side1)),
        (vertex1, Point2::new(u1, v_side0)),
        surface.clone(),
    );
    let boundary = [
        chamfer_edge0.inverse(),
        edge0,
        chamfer_edge1.inverse(),
        edge1,
    ];
    let chamfer = Face::new(vec![boundary.into()], surface);

    Some(SimpleFillet {
        face0: new_face0,
        face1: new_face1,
        fillet: chamfer,
    })
}

/// Creates a chamfer and trims the faces at its two ends. See [`fillet_with_side`].
#[allow(clippy::too_many_arguments)]
pub fn chamfer_with_side<C, S>(
    face0: &Face<Point3, C, S>,
    face1: &Face<Point3, C, S>,
    chamfered_edge_id: EdgeID<C>,
    side0: Option<&Face<Point3, C, S>>,
    side1: Option<&Face<Point3, C, S>>,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Option<FilletWithSide<C, S>>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
    BSplineSurface<Point3>: ToSameGeometry<S>,
{
    let simple = simple_chamfer(face0, face1, chamfered_edge_id, d0, d1, tol)?;
    attach_sides(face0, chamfered_edge_id, side0, side1, simple)
}

/// Chamfers a tangent-continuous wire, with distances on the faces oriented along
/// (`d0`) and against (`d1`) the wire. Each junction must have matching contact points;
/// seams between consecutive supporting faces must meet those points without extending the
/// contact curves. This includes the rim of a prism with filleted vertical edges.
///
/// Each open end must terminate on a single face at the contact endpoints; both ends may
/// share that face. Clipped contacts keep their support geometry, with intersection curves
/// closing the chamfer against terminal faces. The chamfer must fit
/// inside its supporting faces and avoid distant faces. Branching selections and non-tangent
/// corners are unsupported. Original faces retain their
/// order, followed by one ruled chamfer face per wire edge. Returns `None` on failed construction.
pub fn chamfer_along_wire<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Option<Shell<Point3, C, S>>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    BSplineSurface<Point3>: ToSameGeometry<S>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
{
    try_chamfer_along_wire(shell, wire, d0, d1, tol).ok()
}

/// Tangent-chain chamfers with validation, contact and trimming diagnostics.
pub fn try_chamfer_along_wire<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    d0: f64,
    d1: f64,
    tol: f64,
) -> Result<Shell<Point3, C, S>, Diagnostic>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    BSplineSurface<Point3>: ToSameGeometry<S>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
{
    try_chamfer_along_wire_with_trim(
        shell,
        wire,
        (d0, d1),
        tol,
        along_wire::trim_contact_edge,
        false,
        DistanceMode::TangentProjection,
    )
}

pub(super) fn try_chamfer_along_wire_with_trim<C, S>(
    shell: &Shell<Point3, C, S>,
    wire: &Wire<Point3, C>,
    (d0, d1): (f64, f64),
    tol: f64,
    trim: impl Fn(&mut C, &Edge<Point3, C>, bool) -> Option<along_wire::TrimmedContact<C>>,
    continue_contacts: bool,
    mode: DistanceMode,
) -> Result<Shell<Point3, C, S>, Diagnostic>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    BSplineSurface<Point3>: ToSameGeometry<S>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
{
    let operation = "chamfer_along_wire";
    validate_tolerance(tol, operation)?;
    let failed = || Diagnostic::new(Code::BlendConstructionFailed, operation, "construct_blend");
    for (name, value) in [("d0", d0), ("d1", d1)] {
        if !value.is_finite() || value <= 0.0 {
            return Err(
                Diagnostic::new(Code::InvalidParameter, operation, "validate_input")
                    .parameter(name, value),
            );
        }
    }
    if wire.is_empty() {
        return Err(Diagnostic::new(
            Code::EmptySelection,
            operation,
            "validate_input",
        ));
    }
    if !wire.is_continuous() || !wire.is_simple() {
        return Err(Diagnostic::new(
            Code::UnsupportedTopology,
            operation,
            "validate_wire",
        ));
    }
    let closed = wire.is_cyclic();
    if !along_wire::is_tangent_continuous(wire, closed) {
        let mut error = Diagnostic::new(Code::UnsupportedGeometry, operation, "validate_wire");
        error.message = "The selected wire is not tangent continuous.".into();
        return Err(error);
    }
    let mut contacts = Vec::new();
    let mut support_contacts = Vec::new();
    let mut surfaces = Vec::new();
    let mut sides = Vec::new();
    for (index, edge) in wire.iter().enumerate() {
        let mut adjacent = [None, None];
        let mut cap_side = None;
        for face in shell {
            if let Some(found) = face.edge_iter().find(|e| e.id() == edge.id()) {
                let side = usize::from(found.front() != edge.front());
                if continue_contacts
                    && wire
                        .iter()
                        .all(|e| face.edge_iter().any(|q| q.id() == e.id()))
                {
                    cap_side = Some(side);
                }
                if adjacent[side].replace(face.oriented_surface()).is_some() {
                    return Err(Diagnostic::new(
                        Code::UnsupportedTopology,
                        operation,
                        "validate_wire",
                    )
                    .selection(index));
                }
            }
        }
        let [Some(s0), Some(s1)] = adjacent else {
            return Err(
                Diagnostic::new(Code::UnsupportedTopology, operation, "validate_wire")
                    .selection(index),
            );
        };
        let curve = edge.oriented_curve();
        let (a, b) = curve.range_tuple();
        let range = if continue_contacts {
            let speeds = [curve.der(a).magnitude(), curve.der(b).magnitude()];
            if speeds.iter().any(|s| !s.is_finite() || *s <= 0.) {
                return Err(failed().stage("continue_contacts").selection(index));
            }
            let range = (
                a - 2. * d0.max(d1) / speeds[0],
                b + 2. * d0.max(d1) / speeds[1],
            );
            if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
                return Err(failed().stage("continue_contacts").selection(index));
            }
            Some(range)
        } else {
            None
        };
        let (params, pcurves, [c0, c1]) =
            fit_contact_curves(&curve, &s0, &s1, (d0, d1), tol, range, mode)
                .ok_or_else(|| failed().stage("find_contact").selection(index))?;
        let mut contact: [C; 2] = pcurves.each_ref().map(|curve| curve.to_same_geometry());
        // Reject collapsed/reversed offsets and fits that separate the contact edge from
        // either its supporting face or the ruled chamfer by more than the fitting tolerance.
        for pair in params.windows(2) {
            for i in 0..=4 {
                let t = pair[0] + (pair[1] - pair[0]) * i as f64 / 4.0;
                let p = curve.subs(t);
                let tangent = curve.der(t).normalize();
                let expected = contact_pair(&s0, &s1, p, tangent, (d0, d1), &mut [None; 2], mode)
                    .ok_or_else(failed)?;
                for (side, spline) in [(0, &c0), (1, &c1)] {
                    let q = contact[side].subs(t);
                    let advance = contact[side].der(t).dot(tangent);
                    if !advance.is_finite()
                        || advance <= 0.0
                        || q.distance(expected[side].1) > tol
                        || q.distance(spline.subs(t)) > tol
                    {
                        return Err(failed().stage("validate_contact").selection(index));
                    }
                }
            }
        }
        if continue_contacts {
            let side = cap_side.ok_or_else(failed)?;
            contact[side] = contact[side].cut(a);
            contact[side].cut(b);
        }
        if continue_contacts {
            support_contacts.push(pcurves);
        }
        contacts.push(contact);
        let t = (params[0] + params[params.len() - 1]) / 2.0;
        let p = curve.subs(t);
        let uv0 = s0.search_parameter(p, None, 100).ok_or_else(failed)?;
        let uv1 = s1.search_parameter(p, None, 100).ok_or_else(failed)?;
        let outward = s0.normal(uv0.0, uv0.1) + s1.normal(uv1.0, uv1.1);
        let mut surface = BSplineSurface::homotopy(c0.clone(), c1.clone());
        let ((u0, u1), (v0, v1)) = surface.range_tuple();
        let swap = surface.normal((u0 + u1) / 2., (v0 + v1) / 2.).dot(outward) < 0.0;
        if swap {
            surface = BSplineSurface::homotopy(c1, c0);
        }
        sides.push(if swap { (1.0, 0.0) } else { (0.0, 1.0) });
        surfaces.push(surface);
    }
    let mut trimmed = along_wire::trim_closed_chain(shell, wire, &contacts, trim)
        .ok_or_else(|| failed().stage("trim_faces"))?;
    if continue_contacts {
        for k in 0..wire.len() {
            for (source, edge) in support_contacts[k].iter().zip(&trimmed.contacts[k]) {
                let curve = constrain_contact(source, edge, tol)
                    .ok_or_else(|| failed().stage("constrain_contact").selection(k))?;
                edge.set_curve(if edge.orientation() {
                    curve
                } else {
                    curve.inverse()
                });
            }
            contacts[k] = std::array::from_fn(|side| trimmed.contacts[k][side].oriented_curve());
            let [c0, c1] = fit_trimmed_contacts(&contacts[k], tol)
                .ok_or_else(|| failed().stage("fit_trimmed_contacts"))?;
            surfaces[k] = if sides[k].0 == 0. {
                BSplineSurface::homotopy(c0, c1)
            } else {
                BSplineSurface::homotopy(c1, c0)
            };
        }
    }
    if !continue_contacts {
        for (contact, edges) in contacts.iter_mut().zip(&trimmed.contacts) {
            *contact = edges.each_ref().map(Edge::oriented_curve);
        }
    }
    let n = wire.len();
    let nv = if closed { n } else { n + 1 };
    let cross_parameters = |k: usize, j: usize, at_end: bool| -> Option<[f64; 2]> {
        let (a, b) = surfaces[k].range_tuple().0;
        let hint = if at_end { b } else { a };
        let mut parameters = [hint; 2];
        for (side, parameter) in parameters.iter_mut().enumerate() {
            let column = if side == 0 { sides[k].0 } else { sides[k].1 };
            let point = trimmed.vertices[j][side].point();
            if !surfaces[k].subs(hint, column).near(&point) {
                let curve = surfaces[k].row_curve(column as usize);
                *parameter = curve
                    .search_nearest_parameter(point, Some(hint), 100)?
                    .clamp(a, b);
                if !curve.subs(*parameter).near(&point) {
                    return None;
                }
            }
        }
        Some(parameters)
    };
    let mut cross = Vec::new();
    for k in 0..n {
        for (side, curve) in contacts[k].iter().enumerate() {
            let (t0, t1) = curve.range_tuple();
            if !curve.subs(t0).near(&trimmed.vertices[k][side].point())
                || !curve
                    .subs(t1)
                    .near(&trimmed.vertices[(k + 1) % nv][side].point())
            {
                return Err(failed().stage("trim_faces").selection(k));
            }
        }
        let parameters =
            cross_parameters(k, k, false).ok_or_else(|| failed().stage("trim_faces"))?;
        cross.push(ruling_edge(
            (
                &trimmed.vertices[k][0],
                Point2::new(parameters[0], sides[k].0),
            ),
            (
                &trimmed.vertices[k][1],
                Point2::new(parameters[1], sides[k].1),
            ),
            surfaces[k].to_same_geometry(),
        ));
    }
    if !closed {
        let k = n - 1;
        let parameters =
            cross_parameters(k, n, true).ok_or_else(|| failed().stage("trim_faces"))?;
        cross.push(ruling_edge(
            (
                &trimmed.vertices[n][0],
                Point2::new(parameters[0], sides[k].0),
            ),
            (
                &trimmed.vertices[n][1],
                Point2::new(parameters[1], sides[k].1),
            ),
            surfaces[k].to_same_geometry(),
        ));
        let mut ends: Vec<(usize, Vec<usize>)> = Vec::new();
        for j in [0, n] {
            let vertex = if j == 0 {
                wire.front_vertex().unwrap()
            } else {
                wire.back_vertex().unwrap()
            };
            let candidates: Vec<_> = shell
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    f.vertex_iter().any(|v| v.id() == vertex.id())
                        && !f.edge_iter().any(|e| wire.iter().any(|w| w.id() == e.id()))
                })
                .collect();
            let [(index, _)] = candidates.as_slice() else {
                return Err(failed().stage("trim_end_faces"));
            };
            let surface = shell[*index].oriented_surface();
            let guide = cross[j].curve();
            let (a, b) = guide.range_tuple();
            if (0..=16).any(|i| {
                let point = guide.subs(a + (b - a) * i as f64 / 16.);
                surface.search_parameter(point, None, 100).is_none()
            }) {
                let k = if j == 0 { 0 } else { n - 1 };
                let intersection =
                    IntersectionCurve::new(surface, surfaces[k].to_same_geometry(), guide);
                for i in 0..=32 {
                    let t = a + (b - a) * i as f64 / 32.;
                    let (point, uv0, uv1) = intersection
                        .search_triple(t, 100)
                        .ok_or_else(|| failed().stage("trim_end_faces"))?;
                    let normal0 = intersection.surface0().normal(uv0.x, uv0.y);
                    let normal1 = intersection.surface1().normal(uv1.x, uv1.y);
                    let transverse = normal0.cross(normal1).magnitude2();
                    if !transverse.is_finite()
                        || transverse <= TOLERANCE2
                        || (i == 0 && !point.near(&cross[j].front().point()))
                        || (i == 32 && !point.near(&cross[j].back().point()))
                    {
                        return Err(failed().stage("trim_end_faces"));
                    }
                }
                cross[j].set_curve(intersection.to_same_geometry());
            }
            if let Some((_, junctions)) = ends.iter_mut().find(|(i, _)| i == index) {
                junctions.push(j);
            } else {
                ends.push((*index, vec![j]));
            }
        }
        // Apply both end bridges together when they trim the same terminal face.
        for (index, junctions) in ends {
            let face = &shell[index];
            let mut used = vec![false; junctions.len()];
            let mut loops = Vec::new();
            for boundary in face.boundaries() {
                let mut pieces = Vec::new();
                for edge in boundary {
                    pieces.extend(trimmed.cuts.piece(&edge).unwrap_or(Some(edge)));
                }
                let mut boundary = Wire::new();
                for i in 0..pieces.len() {
                    boundary.push_back(pieces[i].clone());
                    let (a, b) = (pieces[i].back(), pieces[(i + 1) % pieces.len()].front());
                    if a != b {
                        let (bridge, edge) = junctions
                            .iter()
                            .enumerate()
                            .find_map(|(i, &j)| {
                                if used[i] {
                                    None
                                } else if cross[j].front() == a && cross[j].back() == b {
                                    Some((i, cross[j].clone()))
                                } else if cross[j].back() == a && cross[j].front() == b {
                                    Some((i, cross[j].inverse()))
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| failed().stage("trim_end_faces"))?;
                        boundary.push_back(edge);
                        used[bridge] = true;
                    }
                }
                loops.push(boundary);
            }
            if used.iter().any(|used| !used) {
                return Err(failed().stage("trim_end_faces"));
            }
            trimmed.faces[index] = Face::try_new(loops, face.oriented_surface())
                .map_err(|_| failed().stage("trim_end_faces"))?;
        }
    }
    let mut faces = trimmed.faces;
    for k in 0..n {
        let boundary = [
            trimmed.contacts[k][0].inverse(),
            cross[k].clone(),
            trimmed.contacts[k][1].clone(),
            cross[(k + 1) % nv].inverse(),
        ];
        faces.push(
            Face::try_new(vec![boundary.into()], surfaces[k].to_same_geometry()).map_err(|e| {
                Diagnostic::new(Code::InvalidOutputTopology, operation, "validate_output")
                    .selection(k)
                    .with_coded_source(e)
            })?,
        );
    }
    Ok(faces.into())
}

#[cfg(test)]
mod contact_interpolation_tests {
    use super::*;

    #[cfg(feature = "step-test")]
    fn check_curved_cap_contacts(step: &str, task: &str) {
        use crate::SplitClosedEdgesAndFaces;
        use std::collections::HashSet;
        use truck_modeling as model;
        use truck_stepio::r#in::Table;

        let table = Table::from_step(step).unwrap();
        assert_eq!(table.manifold_solid_brep.len(), 1);
        let entity = table.manifold_solid_brep.values().next().unwrap();
        let (mut compressed, skipped) = table.to_compressed_solid(entity).unwrap();
        assert!(skipped.is_empty());
        compressed.split_closed_edges_and_faces(1e-5);
        let compressed = compressed
            .try_mapped(
                |p| Some(*p),
                |c| model::Curve::try_from(c).ok(),
                |s| model::Surface::try_from(s).ok(),
            )
            .unwrap();
        let solid = model::Solid::extract(compressed).unwrap();
        assert!(solid.is_geometric_consistent());
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let task: serde_json::Value = serde_json::from_str(task).unwrap();
        let points = task["selection"][0]["points"].as_array().unwrap();
        let point = |p: &serde_json::Value| {
            Point3::new(
                p[0].as_f64().unwrap(),
                p[1].as_f64().unwrap(),
                p[2].as_f64().unwrap(),
            )
        };
        let (a, b) = (point(&points[0]), point(points.last().unwrap()));
        let mut seen = HashSet::new();
        let selected: Vec<_> = solid
            .edge_iter()
            .filter(|e| seen.insert(e.id()))
            .filter_map(|e| {
                if e.front().point().near(&a) && e.back().point().near(&b) {
                    Some(e.clone())
                } else if e.front().point().near(&b) && e.back().point().near(&a) {
                    Some(e.inverse())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(selected.len(), 1);
        for edge in [selected[0].clone(), selected[0].inverse()] {
            let mut supports = [None, None];
            for face in solid.face_iter() {
                if let Some(found) = face.edge_iter().find(|e| e.id() == edge.id()) {
                    let side = usize::from(found.front() != edge.front());
                    assert!(supports[side].replace(face.oriented_surface()).is_none());
                }
            }
            let [surface0, surface1] = supports.map(Option::unwrap);
            let curve = edge.oriented_curve();
            let range = curve.range_tuple();
            let budget = curve.parameter_division(range, 0.001).0.len().max(257);
            for distances in [(0.01, 0.01), (0.01, 0.006), (0.006, 0.01)] {
                let (params, contacts, fits) = fit_contact_curves(
                    &curve,
                    &surface0,
                    &surface1,
                    distances,
                    0.001,
                    None,
                    DistanceMode::TangentProjection,
                )
                .expect("localized contact error fits within the existing sample budget");
                assert!(params.len() <= budget);
                assert_eq!((params[0], *params.last().unwrap()), range);
                assert!(params.windows(2).all(|w| w[0] < w[1]));
                for i in 0..=4096 {
                    let t = range.0 + (range.1 - range.0) * i as f64 / 4096.;
                    for side in 0..2 {
                        let point = contacts[side].subs(t);
                        let u = fits[side]
                            .search_nearest_parameter(point, Some(t), 100)
                            .unwrap()
                            .clamp(range.0, range.1);
                        assert!(point.distance(fits[side].subs(u)) < TOLERANCE);
                        assert!(contacts[side].der(t).dot(curve.der(t)) > 0.);
                    }
                }
                assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
            }
        }
    }

    #[cfg(feature = "step-test")]
    #[test]
    fn curved_cap_contacts_fit_within_the_existing_sample_budget() {
        check_curved_cap_contacts(
            include_str!("../../tests/fixtures/adaptive-chamfer-contacts/001.input.step"),
            include_str!("../../tests/fixtures/adaptive-chamfer-contacts/001.input.json"),
        );
    }

    #[cfg(feature = "step-test")]
    #[test]
    fn previously_trimmed_cap_contacts_fit_within_the_existing_sample_budget() {
        check_curved_cap_contacts(
            include_str!("../../tests/fixtures/adaptive-chamfer-contacts/003.input.step"),
            include_str!("../../tests/fixtures/adaptive-chamfer-contacts/003.input.json"),
        );
    }

    #[test]
    fn dense_affine_contact_samples_preserve_points_and_derivatives() {
        for (a, b) in [(0., 1.), (2., 5.)] {
            for count in [4, 17, 129, 257] {
                let params: Vec<_> = (0..count)
                    .map(|i| a + (b - a) * i as f64 / (count - 1) as f64)
                    .collect();
                let point = |t: f64| Point3::new(1. + 2. * t, -3. + 0.5 * t, 5. - 4. * t);
                let curve = interpolate(&params, params.iter().map(|&t| point(t)));
                assert_eq!(curve.range_tuple(), (a, b));
                for i in 0..=1024 {
                    let t = a + (b - a) * i as f64 / 1024.;
                    assert!(
                        curve.subs(t).distance(point(t)) < 1e-9,
                        "{count} samples, t={t}"
                    );
                    assert!(curve.der(t).distance(Vector3::new(2., 0.5, -4.)) < 1e-7);
                    assert!(curve.der2(t).magnitude() < 1e-6);
                }
            }
        }
    }
}
