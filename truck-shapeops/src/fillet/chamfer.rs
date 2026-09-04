use super::*;
use truck_base::cgmath64::control_point::ControlPoint;

/// Number of samples along the edge is at least this, so a cubic interpolation is always possible.
const MIN_SAMPLES: usize = 4;

/// Samples the contact curves of a chamfer along `edge`.
///
/// At each sample the in-face direction on `surface_i` is the surface tangent perpendicular to the
/// edge, pointing into the face. The contact point is the point of `surface_i` nearest to the point
/// at distance `d_i` along that direction. Returns, for each surface, the samples in parameter
/// space and in space, together with the sample parameters.
#[allow(clippy::type_complexity)]
fn sample_contact_points<C, S>(
    edge: &C,
    surface0: &S,
    surface1: &S,
    (d0, d1): (f64, f64),
    tol: f64,
) -> Option<(Vec<f64>, [Vec<(Point2, Point3)>; 2])>
where
    C: FilletedCurve<S> + ParameterDivision1D<Point = Point3>,
    S: FilletedSurface<C> + SearchNearestParameter<D2, Point = Point3>,
{
    let (t0, t1) = edge.range_tuple();
    let n = edge
        .parameter_division((t0, t1), tol)
        .0
        .len()
        .max(MIN_SAMPLES);
    let params: Vec<f64> = (0..n)
        .map(|i| t0 + (t1 - t0) * i as f64 / (n - 1) as f64)
        .collect();

    let contact = |surface: &S, distance: f64, sign: f64, hint: &mut Option<(f64, f64)>| {
        let mut samples = Vec::with_capacity(n);
        for &t in &params {
            let p = edge.subs(t);
            let e = edge.der(t).normalize();
            let uv = surface.search_parameter(p, *hint, 100)?;
            let w = surface.normal(uv.0, uv.1).cross(e) * sign;
            let uv = surface.search_nearest_parameter(p + w * distance, Some(uv), 100)?;
            *hint = Some(uv);
            samples.push((Point2::from(uv), surface.subs(uv.0, uv.1)));
        }
        Some(samples)
    };
    let samples0 = contact(surface0, d0, 1.0, &mut None)?;
    let samples1 = contact(surface1, d1, -1.0, &mut None)?;
    Some((params, [samples0, samples1]))
}

fn interpolate<P>(params: &[f64], points: impl Iterator<Item = P>) -> BSplineCurve<P>
where P: ControlPoint<f64> + Copy {
    let (t0, t1) = (params[0], params[params.len() - 1]);
    let mut knot_vec = KnotVec::uniform_knot(3, params.len() - 3);
    knot_vec.transform(t1 - t0, t0);
    let parameter_points: Vec<_> = params.iter().copied().zip(points).collect();
    BSplineCurve::interpole(knot_vec, parameter_points)
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

    let (params, [samples0, samples1]) =
        sample_contact_points(&edge_curve, &surface0, &surface1, (d0, d1), tol)?;
    let pcurve0 = PCurve::new(
        interpolate(&params, samples0.iter().map(|s| s.0)),
        surface0.clone(),
    );
    let pcurve1 = PCurve::new(
        interpolate(&params, samples1.iter().map(|s| s.0)),
        surface1.clone(),
    );
    let curve0 = interpolate(&params, samples0.iter().map(|s| s.1));
    let curve1 = interpolate(&params, samples1.iter().map(|s| s.1));

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
