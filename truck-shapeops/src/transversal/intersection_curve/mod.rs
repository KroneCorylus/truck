use std::ops::RangeBounds;
use truck_base::cgmath64::*;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;

/// polyline base ntersection curve with parameter
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct IntersectionCurveWithParameters<S0, S1> {
    #[deref]
    #[deref_mut]
    ic: IntersectionCurve<PolylineCurve<Point3>, S0, S1>,
    params0: PolylineCurve<Point2>,
    params1: PolylineCurve<Point2>,
}

impl<S0, S1> From<IntersectionCurveWithParameters<S0, S1>>
    for IntersectionCurve<PolylineCurve<Point3>, S0, S1>
{
    fn from(a: IntersectionCurveWithParameters<S0, S1>) -> Self { a.ic }
}

impl<S0, S1> IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    pub fn try_new(
        surface0: S0,
        surface1: S1,
        poly: PolylineCurve<Point3>,
        tol: f64,
    ) -> Option<Self> {
        let ic = IntersectionCurve::new(&surface0, &surface1, poly);
        let poly = ic.leader();
        let len = poly.len();
        let mut polyline = PolylineCurve(Vec::new());
        let mut params0 = PolylineCurve(Vec::new());
        let mut params1 = PolylineCurve(Vec::new());
        let mut hints: Option<(Point2, Point2)> = None;
        // Seeding each sample with the previous one skips the presearch grid, but the seeded
        // solve has to be checked before it is trusted. It can stall on a singular Jacobian and
        // report a step it never took, and on a periodic surface it can continue straight past
        // the seam, which gives the right point at a parameter a full period outside the range.
        // Neither is visible in the point alone, so the parameters are checked too and anything
        // suspect falls back to the unseeded search.
        let ranges = (surface0.parameter_range(), surface1.parameter_range());
        let accept = |&(point, p0, p1): &(Point3, Point2, Point2), t: f64| {
            let in_range = |p: Point2, (ur, vr): (ParameterRange, ParameterRange)| {
                ur.contains(&p.x) && vr.contains(&p.y)
            };
            point.distance(poly.subs(t)) <= tol
                && in_range(p0, ranges.0)
                && in_range(p1, ranges.1)
                && surface0.subs(p0.x, p0.y).distance(point) <= tol
                && surface1.subs(p1.x, p1.y).distance(point) <= tol
        };
        let search = |t, hints: Option<(Point2, Point2)>| {
            hints
                .and_then(|(h0, h1)| {
                    ic.search_triple_with_hints(t, Some((h0.x, h0.y)), Some((h1.x, h1.y)), 100)
                        .filter(|triple| accept(triple, t))
                })
                .or_else(|| ic.search_triple(t, 100))
        };
        for i in 0..len - 1 {
            let t = i as f64;
            let (q, p0, p1) = search(t, hints)?;
            polyline.push(q);
            params0.push(p0);
            params1.push(p1);
            hints = Some((p0, p1));
        }
        let (q, p0, p1) = match poly[0].near(&poly[len - 1]) {
            true => (polyline[0], params0[0], params1[0]),
            false => search((len - 1) as f64, hints)?,
        };
        polyline.push(q);
        params0.push(p0);
        params1.push(p1);
        Some(Self {
            ic: IntersectionCurve::new(surface0, surface1, polyline),
            params0,
            params1,
        })
    }
}

impl<S0, S1> ParametricCurve for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.ic.der_n(n, t) }
    #[inline(always)]
    fn subs(&self, t: f64) -> Point3 { self.ic.subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Vector3 { self.ic.der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Vector3 { self.ic.der2(t) }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.ic.parameter_range() }
}

impl<S0, S1> BoundedCurve for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
}

impl<S0, S1> ParameterDivision1D for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    #[inline(always)]
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        self.ic.parameter_division(range, tol)
    }
}

impl<S0, S1> Cut for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self {
        Self {
            ic: self.ic.cut(t),
            params0: self.params0.cut(t),
            params1: self.params1.cut(t),
        }
    }
}

impl<S0: Clone, S1: Clone> Invertible for IntersectionCurveWithParameters<S0, S1> {
    fn invert(&mut self) {
        self.ic.invert();
        self.params0.invert();
        self.params1.invert();
    }
}

impl<S0, S1> SearchParameter<D1> for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.ic.search_parameter(point, hint, trials)
    }
}

impl<S0, S1> SearchNearestParameter<D1> for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.ic.search_nearest_parameter(point, hint, trials)
    }
}

type IntersectionTuple<S0, S1> = (
    PolylineCurve<Point3>,
    IntersectionCurveWithParameters<S0, S1>,
);
/// `tol` is the chord error of `polygon0` and `polygon1`, which bounds how far the polylines
/// they intersect in may sit from the true intersection, and so how far a projected sample may
/// sit from its polyline vertex before it is treated as a bad solve.
pub fn intersection_curves<S0, S1>(
    surface0: S0,
    polygon0: &PolygonMesh,
    surface1: S1,
    polygon1: &PolygonMesh,
    tol: f64,
) -> Option<Vec<IntersectionTuple<S0, S1>>>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let interferences = polygon0.extract_interference(polygon1);
    let polylines = super::polyline_construction::construct_polylines(&interferences);
    polylines
        .into_iter()
        .flat_map(|polyline| split_at_tangencies(&surface0, &surface1, polyline, tol))
        .filter(|(_, grazes)| !grazes)
        .map(|(polyline, _)| {
            Some((
                polyline.clone(),
                IntersectionCurveWithParameters::try_new(
                    surface0.clone(),
                    surface1.clone(),
                    polyline,
                    tol,
                )?,
            ))
        })
        .collect()
}

/// The parameters of the points of `surface0` and `surface1` nearest to `point`.
fn nearest_parameters<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    point: Point3,
) -> Option<[(f64, f64); 2]>
where
    S0: SearchNearestParameter<D2, Point = Point3>,
    S1: SearchNearestParameter<D2, Point = Point3>,
{
    Some([
        surface0.search_nearest_parameter(point, None, 100)?,
        surface1.search_nearest_parameter(point, None, 100)?,
    ])
}

/// Refines a mesh seed to coincidence with parallel normals. The six residuals constrain
/// four parameters; a small diagonal regularization leaves a contact curve's free direction
/// close to its seed instead of requiring an invertible coincidence-only Newton system.
fn tangent_contact<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    point: Point3,
    tol: f64,
) -> Option<Point3>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    contact_from(
        surface0,
        surface1,
        nearest_parameters(surface0, surface1, point)?,
        tol,
    )
}

/// [`tangent_contact`] from the parameters nearest to the seed.
fn contact_from<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    [(u, v), (s, t)]: [(f64, f64); 2],
    tol: f64,
) -> Option<Point3>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    let mut parameters = Vector4::new(u, v, s, t);
    let normal0 = surface0.normal(u, v);
    let normal1 = surface1.normal(s, t);
    let start0 = surface0.subs(u, v);
    let start1 = surface1.subs(s, t);
    let curvature = curvature_bound(surface0, (u, v))? + curvature_bound(surface1, (s, t))?;
    // Chord error grows quadratically with distance from a tangency. Scale the
    // angular gate and the refinement neighborhood by curvature and mesh error.
    let angular_budget = 8. * tol * curvature;
    if normal0.cross(normal1).magnitude2() > angular_budget.max(TOLERANCE2) {
        return None;
    }
    let distance_budget2 = if curvature > 0. {
        8. * tol / curvature
    } else {
        tol * tol
    };
    let sign = if normal0.dot(normal1) > 0.0 {
        1.0
    } else {
        -1.0
    };
    let length = surface0
        .uder(u, v)
        .magnitude()
        .max(surface0.vder(u, v).magnitude())
        .max(surface1.uder(s, t).magnitude())
        .max(surface1.vder(s, t).magnitude());
    for _ in 0..30 {
        let Vector4 {
            x: u,
            y: v,
            z: s,
            w: t,
        } = parameters;
        let p0 = surface0.subs(u, v);
        let p1 = surface1.subs(s, t);
        if p0.distance2(start0) > distance_budget2 || p1.distance2(start1) > distance_budget2 {
            return None;
        }
        let difference = p0 - p1;
        let normals = surface0.normal(u, v) - sign * surface1.normal(s, t);
        if difference.magnitude2() < TOLERANCE2 * 1e-4 && normals.magnitude2() < TOLERANCE2 * 1e-4 {
            return Some(p0.midpoint(p1));
        }
        let columns = [
            [
                surface0.uder(u, v) / length,
                surface0.vder(u, v) / length,
                -surface1.uder(s, t) / length,
                -surface1.vder(s, t) / length,
            ],
            [
                surface0.normal_uder(u, v),
                surface0.normal_vder(u, v),
                -sign * surface1.normal_uder(s, t),
                -sign * surface1.normal_vder(s, t),
            ],
        ];
        let mut matrix = Matrix4::zero();
        let mut rhs = Vector4::zero();
        for (columns, residual) in columns.into_iter().zip([difference / length, normals]) {
            for i in 0..3 {
                let row = Vector4::new(columns[0][i], columns[1][i], columns[2][i], columns[3][i]);
                matrix += Matrix4::from_cols(row * row.x, row * row.y, row * row.z, row * row.w);
                rhs += row * residual[i];
            }
        }
        let scale = (0..4).map(|i| matrix[i][i]).fold(0.0_f64, f64::max);
        // Scale damping per parameter so a free contact direction does not drift
        // toward a pole merely because its angular derivative is small.
        for i in 0..4 {
            matrix[i][i] += 1e-3 * matrix[i][i] + scale * 1e-14;
        }
        parameters -= matrix.invert()? * rhs;
    }
    None
}

/// A vertex of an interference polyline, moved onto a tangent crossing if one lies within
/// `TOLERANCE`.
#[derive(Clone, Copy)]
struct Sample {
    point: Point3,
    /// Whether the surfaces cross tangentially at `point`.
    singular: bool,
    /// [`grazes_at`] at `point`, if already known.
    grazes: Option<bool>,
}

/// The tangency tests of one polyline vertex. They start from the same nearest parameters,
/// which are searched once.
fn sample<S0, S1>(surface0: &S0, surface1: &S1, point: Point3, tol: f64) -> Sample
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    let parameters = nearest_parameters(surface0, surface1, point);
    let contact =
        parameters.and_then(|parameters| contact_from(surface0, surface1, parameters, tol));
    let crossing = contact.map(|contact| tangent_crossing_at(surface0, surface1, contact));
    match contact {
        Some(contact) if crossing == Some(Some(true)) && contact.distance(point) < TOLERANCE => {
            Sample {
                point: contact,
                singular: true,
                grazes: None,
            }
        }
        _ => Sample {
            point,
            singular: parameters
                .and_then(|parameters| crossing_from(surface0, surface1, parameters))
                == Some(true),
            grazes: Some(crossing == Some(Some(false))),
        },
    }
}

/// Whether the mesh interference at `point` refines to contact without crossing branches.
fn grazes_at<S0, S1>(surface0: &S0, surface1: &S1, point: Point3, tol: f64) -> bool
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    tangent_contact(surface0, surface1, point, tol)
        .is_some_and(|contact| tangent_crossing_at(surface0, surface1, contact) == Some(false))
}

/// Splits `polyline` at its tangent crossings, each piece with whether the surfaces graze
/// along all of it. Different chord errors can make touching surfaces appear to cross in small
/// loops; coincidence plus parallel normals places those seeds back on the contact curve.
fn split_at_tangencies<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    polyline: PolylineCurve<Point3>,
    tol: f64,
) -> Vec<(PolylineCurve<Point3>, bool)>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let mut samples: Vec<Sample> = polyline
        .iter()
        .map(|&point| sample(surface0, surface1, point, tol))
        .collect();
    if samples[0].point.near(&samples[samples.len() - 1].point) {
        if let Some(i) = samples.iter().position(|sample| sample.singular) {
            samples.pop();
            samples.rotate_left(i);
            samples.push(samples[0]);
        }
    }
    let mut pieces = Vec::new();
    let mut start = 0;
    for i in 1..samples.len() {
        if samples[i].singular || i == samples.len() - 1 {
            let piece = &samples[start..=i];
            let grazes = piece.iter().all(|sample| {
                sample
                    .grazes
                    .unwrap_or_else(|| grazes_at(surface0, surface1, sample.point, tol))
            });
            let points = piece.iter().map(|sample| sample.point).collect();
            pieces.push((PolylineCurve(points), grazes));
            start = i;
        }
    }
    pieces
}

/// Whether the difference of the normal-curvature forms at a contact is indefinite.
/// Both forms use the same unit normal and orthonormal tangent basis: opposite signs of
/// the height difference in two directions imply four crossing branches. Higher-order
/// contacts with a semidefinite or zero difference are outside this classification.
fn tangent_crossing_at<S0, S1>(surface0: &S0, surface1: &S1, point: Point3) -> Option<bool>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    crossing_from(
        surface0,
        surface1,
        nearest_parameters(surface0, surface1, point)?,
    )
}

/// [`tangent_crossing_at`] from the parameters nearest to the point.
fn crossing_from<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    [uv0, uv1]: [(f64, f64); 2],
) -> Option<bool>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    let n0 = surface0.normal(uv0.0, uv0.1);
    let n1 = surface1.normal(uv1.0, uv1.1);
    if !n0.magnitude2().is_finite()
        || !n1.magnitude2().is_finite()
        || n0.magnitude2() == 0.0
        || n1.magnitude2() == 0.0
    {
        return None;
    }
    let normal = n0.normalize();
    if !normal.cross(n1.normalize()).so_small()
        || !surface0
            .subs(uv0.0, uv0.1)
            .near(&surface1.subs(uv1.0, uv1.1))
    {
        return Some(false);
    }
    let (tangent, bitangent) = tangent_basis(surface0, uv0, normal);
    let curvature0 = curvature_form(surface0, uv0, normal, tangent, bitangent)?;
    let curvature1 = curvature_form(surface1, uv1, normal, tangent, bitangent)?;
    let [a, b, c] = std::array::from_fn(|i| curvature0[i] - curvature1[i]);
    let scale = a.abs().max(b.abs()).max(c.abs());
    if !scale.is_finite() || scale == 0.0 {
        return Some(false);
    }
    let (a, b, c) = (a / scale, b / scale, c / scale);
    Some(a * c - b * b < -TOLERANCE)
}

fn tangent_basis<S: ParametricSurface3D>(
    surface: &S,
    (u, v): (f64, f64),
    normal: Vector3,
) -> (Vector3, Vector3) {
    let (du, dv) = (surface.uder(u, v), surface.vder(u, v));
    let tangent = if du.magnitude2() >= dv.magnitude2() {
        du
    } else {
        dv
    };
    let tangent = (tangent - normal * tangent.dot(normal)).normalize();
    (tangent, normal.cross(tangent))
}

fn curvature_bound<S: ParametricSurface3D>(surface: &S, uv: (f64, f64)) -> Option<f64> {
    let normal = surface.normal(uv.0, uv.1).normalize();
    let (tangent, bitangent) = tangent_basis(surface, uv, normal);
    let [a, b, c] = curvature_form(surface, uv, normal, tangent, bitangent)?;
    let bound = (a * a + 2. * b * b + c * c).sqrt();
    bound.is_finite().then_some(bound)
}

fn curvature_form<S: ParametricSurface3D>(
    surface: &S,
    (u, v): (f64, f64),
    normal: Vector3,
    tangent: Vector3,
    bitangent: Vector3,
) -> Option<[f64; 3]> {
    let derivatives = surface.ders(2, u, v);
    let du2 = derivatives[1][0].magnitude2();
    let dv2 = derivatives[0][1].magnitude2();
    if du2.min(dv2) <= 1e-24 * du2.max(dv2) {
        if let Some(form) =
            polar_curvature_form(surface, (u, v), normal, tangent, bitangent, du2 >= dv2)
        {
            return Some(form);
        }
    }
    let inverse = Matrix3::from_cols(derivatives[1][0], derivatives[0][1], normal).invert()?;
    let x = inverse * tangent;
    let y = inverse * bitangent;
    let (uu, uv, vv) = (
        normal.dot(derivatives[2][0]),
        normal.dot(derivatives[1][1]),
        normal.dot(derivatives[0][2]),
    );
    let form =
        |a: Vector3, b: Vector3| uu * a.x * b.x + uv * (a.x * b.y + a.y * b.x) + vv * a.y * b.y;
    Some([form(x, x), form(x, y), form(y, y)])
}

// At a smooth pole, the collapsed parameter line names one point but its meridians
// supply different tangent directions. Their exact normal accelerations determine
// the quadratic form without inverting the singular surface Jacobian.
fn polar_curvature_form<S: ParametricSurface3D>(
    surface: &S,
    (u, v): (f64, f64),
    normal: Vector3,
    tangent: Vector3,
    bitangent: Vector3,
    collapse_v: bool,
) -> Option<[f64; 3]> {
    use std::ops::Bound::{Excluded, Included};
    let ranges = surface.parameter_range();
    let range = if collapse_v { ranges.1 } else { ranges.0 };
    let (Included(start) | Excluded(start), Included(end) | Excluded(end)) = range else {
        return None;
    };
    let point = surface.subs(u, v);
    let mut equations = Vec::new();
    for i in 0..8 {
        let parameter = start + (end - start) * i as f64 / 8.;
        let (u, v) = if collapse_v {
            (u, parameter)
        } else {
            (parameter, v)
        };
        let n = surface.normal(u, v);
        if !surface.subs(u, v).near(&point) || !n.cross(normal).so_small() {
            return None;
        }
        let (direction, acceleration) = if collapse_v {
            (surface.uder(u, v), surface.uuder(u, v))
        } else {
            (surface.vder(u, v), surface.vvder(u, v))
        };
        let speed2 = direction.magnitude2();
        if !speed2.is_finite()
            || speed2 == 0.
            || direction.dot(normal).abs() > TOLERANCE * speed2.sqrt()
        {
            return None;
        }
        let direction = direction / speed2.sqrt();
        let (x, y) = (direction.dot(tangent), direction.dot(bitangent));
        equations.push((
            Vector3::new(x * x, 2. * x * y, y * y),
            normal.dot(acceleration) / speed2,
        ));
    }
    let mut matrix = Matrix3::zero();
    let mut rhs = Vector3::zero();
    for &(row, curvature) in &equations {
        matrix += Matrix3::from_cols(row * row.x, row * row.y, row * row.z);
        rhs += row * curvature;
    }
    let form = matrix.invert()? * rhs;
    let scale = form.x.abs().max(form.y.abs()).max(form.z.abs());
    if !scale.is_finite()
        || equations.iter().any(|(row, curvature)| {
            (row.dot(form) - curvature).abs() > TOLERANCE * scale.max(TOLERANCE)
        })
    {
        return None;
    }
    Some([form.x, form.y, form.z])
}

#[cfg(test)]
mod tests;
