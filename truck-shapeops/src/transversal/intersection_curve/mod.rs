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
        .flat_map(|polyline| split_at_tangencies(&surface0, &surface1, polyline))
        .filter(|polyline| !surfaces_graze_along(&surface0, &surface1, polyline))
        .map(|polyline| {
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

/// Refines a mesh seed to coincidence with parallel normals. The six residuals constrain
/// four parameters; a small diagonal regularization leaves a contact curve's free direction
/// close to its seed instead of requiring an invertible coincidence-only Newton system.
fn tangent_contact<S0, S1>(surface0: &S0, surface1: &S1, point: Point3) -> Option<Point3>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    let (u, v) = surface0.search_nearest_parameter(point, None, 100)?;
    let (s, t) = surface1.search_nearest_parameter(point, None, 100)?;
    let mut parameters = Vector4::new(u, v, s, t);
    let normal0 = surface0.normal(u, v);
    let normal1 = surface1.normal(s, t);
    if normal0.cross(normal1).magnitude2() > 0.01 {
        return None;
    }
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
        let difference = p0 - p1;
        let normals = surface0.normal(u, v) - sign * surface1.normal(s, t);
        if difference.magnitude2() < TOLERANCE2 * TOLERANCE2
            && normals.magnitude2() < TOLERANCE2 * TOLERANCE2
        {
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
        for i in 0..4 {
            matrix[i][i] += scale * 1.0e-12;
        }
        parameters -= matrix.invert()? * rhs;
    }
    None
}

fn split_at_tangencies<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    mut polyline: PolylineCurve<Point3>,
) -> Vec<PolylineCurve<Point3>>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    for p in polyline.iter_mut() {
        if let Some(contact) = tangent_contact(surface0, surface1, *p) {
            if tangent_crossing_at(surface0, surface1, contact) == Some(true)
                && contact.distance(*p) < TOLERANCE
            {
                *p = contact;
            }
        }
    }
    let singular = |p| tangent_crossing_at(surface0, surface1, p) == Some(true);
    if polyline.front().near(&polyline.back()) {
        if let Some(i) = polyline.iter().position(|&p| singular(p)) {
            polyline.pop();
            polyline.rotate_left(i);
            let front = polyline[0];
            polyline.push(front);
        }
    }
    let mut pieces = Vec::new();
    let mut start = 0;
    for i in 1..polyline.len() {
        if singular(polyline[i]) || i == polyline.len() - 1 {
            pieces.push(PolylineCurve(polyline[start..=i].to_vec()));
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
    let uv0 = surface0.search_nearest_parameter(point, None, 100)?;
    let uv1 = surface1.search_nearest_parameter(point, None, 100)?;
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
    let tangent = surface0.uder(uv0.0, uv0.1).normalize();
    let bitangent = normal.cross(tangent);
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

fn curvature_form<S: ParametricSurface3D>(
    surface: &S,
    (u, v): (f64, f64),
    normal: Vector3,
    tangent: Vector3,
    bitangent: Vector3,
) -> Option<[f64; 3]> {
    let derivatives = surface.ders(2, u, v);
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

/// Whether the mesh interference refines entirely to contact without crossing branches.
/// Different chord errors can make touching surfaces appear to cross in small loops;
/// coincidence plus parallel normals places those seeds back on the contact curve.
fn surfaces_graze_along<S0, S1>(
    surface0: &S0,
    surface1: &S1,
    polyline: &PolylineCurve<Point3>,
) -> bool
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    polyline.iter().all(|&p| {
        tangent_contact(surface0, surface1, p)
            .is_some_and(|contact| tangent_crossing_at(surface0, surface1, contact) == Some(false))
    })
}

#[cfg(test)]
mod tests;
