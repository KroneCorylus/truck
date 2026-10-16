use super::*;
use std::result::Result;
use truck_base::cgmath64::control_point::ControlPoint;
use truck_base::diagnostics::{validate_tolerance, Code, Diagnostic};

/// Number of samples along the edge is at least this, so a cubic interpolation is always possible.
const MIN_SAMPLES: usize = 4;

/// Samples the contact curves of a chamfer along `edge`.
///
/// At each sample the in-face direction on `surface_i` is the surface tangent perpendicular to the
/// edge, pointing into the face. The contact point is the point of `surface_i` nearest to the point
/// at distance `d_i` along that direction. Returns, for each surface, the samples in parameter
/// space and in space at the supplied parameters.
#[allow(clippy::type_complexity)]
fn sample_contact_points<C, S>(
    edge: &C,
    surface0: &S,
    surface1: &S,
    (d0, d1): (f64, f64),
    params: &[f64],
) -> Option<[Vec<(Point2, Point3)>; 2]>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
{
    let contact = |surface: &S, distance: f64, sign: f64, hint: &mut Option<(f64, f64)>| {
        let mut samples = Vec::with_capacity(params.len());
        for &t in params {
            let p = edge.subs(t);
            let e = edge.der(t).normalize();
            let uv = surface.search_parameter(p, *hint, 100)?;
            let w = surface.normal(uv.0, uv.1).cross(e) * sign;
            let mut uv = surface.search_nearest_parameter(p + w * distance, Some(uv), 100)?;
            if let Some((u, v)) = *hint {
                if let Some(period) = surface.u_period() {
                    uv.0 += ((u - uv.0) / period).round() * period;
                }
                if let Some(period) = surface.v_period() {
                    uv.1 += ((v - uv.1) / period).round() * period;
                }
            }
            *hint = Some(uv);
            samples.push((Point2::from(uv), surface.subs(uv.0, uv.1)));
        }
        Some(samples)
    };
    let samples0 = contact(surface0, d0, 1.0, &mut None)?;
    let samples1 = contact(surface1, d1, -1.0, &mut None)?;
    Some([samples0, samples1])
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
) -> Option<(
    Vec<f64>,
    [PCurve<BSplineCurve<Point2>, S>; 2],
    [BSplineCurve<Point3>; 2],
)>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
{
    let (t0, t1) = edge.range_tuple();
    let mut count = edge
        .parameter_division((t0, t1), tol)
        .0
        .len()
        .max(MIN_SAMPLES);
    let budget = count.max(257);
    let fit_tolerance2 = tol.min(TOLERANCE * 0.1).powi(2);
    loop {
        let params: Vec<_> = (0..count)
            .map(|i| t0 + (t1 - t0) * i as f64 / (count - 1) as f64)
            .collect();
        let samples = sample_contact_points(edge, surface0, surface1, distances, &params)?;
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
        let fits = params.windows(2).all(|pair| {
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
        });
        if fits {
            return Some((params, pcurves, curves));
        }
        count = count.checked_mul(2)?.checked_sub(1)?;
        if count > budget {
            return None;
        }
    }
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

    let (params, [pcurve0, pcurve1], [curve0, curve1]) =
        fit_contact_curves(&edge_curve, &surface0, &surface1, (d0, d1), tol)?;

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
/// share that face. The chamfer must fit
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
    let mut surfaces = Vec::new();
    let mut sides = Vec::new();
    for (index, edge) in wire.iter().enumerate() {
        let mut adjacent = [None, None];
        for face in shell {
            if let Some(found) = face.edge_iter().find(|e| e.id() == edge.id()) {
                let side = usize::from(found.front() != edge.front());
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
        let (params, pcurves, [c0, c1]) = fit_contact_curves(&curve, &s0, &s1, (d0, d1), tol)
            .ok_or_else(|| failed().stage("find_contact").selection(index))?;
        let contact: [C; 2] = pcurves.map(|curve| curve.to_same_geometry());
        // Reject collapsed/reversed offsets and fits that separate the contact edge from
        // either its supporting face or the ruled chamfer by more than the fitting tolerance.
        for pair in params.windows(2) {
            for i in 0..=4 {
                let t = pair[0] + (pair[1] - pair[0]) * i as f64 / 4.0;
                let p = curve.subs(t);
                let tangent = curve.der(t).normalize();
                for (side, support, spline, distance, sign) in
                    [(0, &s0, &c0, d0, 1.0), (1, &s1, &c1, d1, -1.0)]
                {
                    let uv = support.search_parameter(p, None, 100).ok_or_else(failed)?;
                    let target = p + support.normal(uv.0, uv.1).cross(tangent) * (distance * sign);
                    let uv = support
                        .search_nearest_parameter(target, Some(uv), 100)
                        .ok_or_else(failed)?;
                    let q = contact[side].subs(t);
                    let advance = contact[side].der(t).dot(tangent);
                    if !advance.is_finite()
                        || advance <= 0.0
                        || q.distance(support.subs(uv.0, uv.1)) > tol
                        || q.distance(spline.subs(t)) > tol
                    {
                        return Err(failed().stage("validate_contact").selection(index));
                    }
                }
            }
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
    let mut trimmed = along_wire::trim_closed_chain(shell, wire, &contacts)
        .ok_or_else(|| failed().stage("trim_faces"))?;
    let n = wire.len();
    let nv = if closed { n } else { n + 1 };
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
        let t = surfaces[k].range_tuple().0 .0;
        cross.push(ruling_edge(
            (&trimmed.vertices[k][0], Point2::new(t, sides[k].0)),
            (&trimmed.vertices[k][1], Point2::new(t, sides[k].1)),
            surfaces[k].to_same_geometry(),
        ));
    }
    if !closed {
        let k = n - 1;
        let t = surfaces[k].range_tuple().0 .1;
        cross.push(ruling_edge(
            (&trimmed.vertices[n][0], Point2::new(t, sides[k].0)),
            (&trimmed.vertices[n][1], Point2::new(t, sides[k].1)),
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
