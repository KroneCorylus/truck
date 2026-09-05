use super::*;
use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
#[doc(hidden)]
pub use truck_geometry::prelude::{algo, inv_or_zero};
pub use truck_geometry::{decorators::*, nurbs::*, specifieds::*};

/// 3-dimensional curve
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    From,
    TryInto,
    ParametricCurve,
    BoundedCurve,
    ParameterDivision1D,
    Cut,
    Invertible,
    SearchNearestParameterD1,
    SearchParameterD1,
)]
pub enum Curve {
    /// line
    Line(Line<Point3>),
    /// 3-dimensional B-spline curve
    BSplineCurve(BSplineCurve<Point3>),
    /// 3-dimensional NURBS curve
    NurbsCurve(NurbsCurve<Vector4>),
    /// circle, circular arc, ellipse or elliptical arc, kept exact
    Conic(Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>),
    /// intersection curve
    IntersectionCurve(IntersectionCurve<Box<Curve>, Box<Surface>, Box<Surface>>),
}

macro_rules! derive_curve_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::Line(got) => $method(got, $($ver), *),
            Curve::BSplineCurve(got) => $method(got, $($ver), *),
            Curve::NurbsCurve(got) => $method(got, $($ver), *),
            Curve::Conic(got) => $method(got, $($ver), *),
            Curve::IntersectionCurve(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_curve_self_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::Line(got) => Curve::Line($method(got, $($ver), *)),
            Curve::BSplineCurve(got) => Curve::BSplineCurve($method(got, $($ver), *)),
            Curve::NurbsCurve(got) => Curve::NurbsCurve($method(got, $($ver), *)),
            Curve::Conic(got) => Curve::Conic($method(got, $($ver), *)),
            Curve::IntersectionCurve(got) => Curve::IntersectionCurve($method(got, $($ver), *)),
        }
    };
}

impl Transformed<Matrix4> for Curve {
    fn transform_by(&mut self, trans: Matrix4) {
        derive_curve_method!(self, Transformed::transform_by, trans);
    }
    fn transformed(&self, trans: Matrix4) -> Self {
        derive_curve_self_method!(self, Transformed::transformed, trans)
    }
}

impl From<IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>> for Curve {
    fn from(c: IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>) -> Curve {
        let (surface0, surface1, leader) = c.destruct();
        Curve::IntersectionCurve(IntersectionCurve::new(
            Box::new(surface0),
            Box::new(surface1),
            Box::new(leader.into()),
        ))
    }
}

impl ToSameGeometry<Curve> for Line<Point3> {
    #[inline]
    fn to_same_geometry(&self) -> Curve { Curve::from(*self) }
}

impl ToSameGeometry<Curve> for Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    #[inline]
    fn to_same_geometry(&self) -> Curve { Curve::Conic(*self) }
}

impl ToSameGeometry<Curve> for BSplineCurve<Point3> {
    #[inline]
    fn to_same_geometry(&self) -> Curve { Curve::from(self.clone()) }
}

impl Curve {
    /// Into non-ratinalized 4-dimensional B-spline curve
    pub fn lift_up(&self) -> BSplineCurve<Vector4> {
        match self {
            Curve::Line(curve) => Curve::BSplineCurve((*curve).into()).lift_up(),
            Curve::BSplineCurve(curve) => BSplineCurve::new(
                curve.knot_vec().clone(),
                curve
                    .control_points()
                    .iter()
                    .map(|pt| pt.to_vec().extend(1.0))
                    .collect(),
            ),
            Curve::NurbsCurve(curve) => curve.non_rationalized().clone(),
            Curve::Conic(curve) => conic_to_nurbs(curve).into_non_rationalized(),
            Curve::IntersectionCurve(_) => {
                unimplemented!("intersection curve cannot connect by homotopy")
            }
        }
    }
}

/// 3-dimensional surfaces
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    From,
    TryInto,
    ParametricSurface,
    ParameterDivision2D,
    Invertible,
    SearchParameterD2,
)]
pub enum Surface {
    /// Plane
    Plane(Plane),
    /// 3-dimensional B-spline surface
    BSplineSurface(BSplineSurface<Point3>),
    /// 3-dimensional NURBS Surface
    NurbsSurface(NurbsSurface<Vector4>),
    /// revoluted curve
    RevolutedCurve(Processor<RevolutedCurve<Curve>, Matrix4>),
    /// curve swept along a vector, kept exact
    Extruded(ExtrudedCurve<Curve, Vector3>),
}

macro_rules! derive_surface_method {
    ($surface: expr, $method: expr, $($ver: ident),*) => {
        match $surface {
            Self::Plane(got) => $method(got, $($ver), *),
            Self::BSplineSurface(got) => $method(got, $($ver), *),
            Self::NurbsSurface(got) => $method(got, $($ver), *),
            Self::RevolutedCurve(got) => $method(got, $($ver), *),
            Self::Extruded(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_surface_self_method {
    ($surface: expr, $method: expr, $($ver: ident),*) => {
        match $surface {
            Self::Plane(got) => Self::Plane($method(got, $($ver), *)),
            Self::BSplineSurface(got) => Self::BSplineSurface($method(got, $($ver), *)),
            Self::NurbsSurface(got) => Self::NurbsSurface($method(got, $($ver), *)),
            Self::RevolutedCurve(got) => Self::RevolutedCurve($method(got, $($ver), *)),
            Self::Extruded(got) => Self::Extruded($method(got, $($ver), *)),
        }
    };
}

impl ParametricSurface3D for Surface {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface3D::normal, u, v)
    }
}

impl Transformed<Matrix4> for Surface {
    fn transform_by(&mut self, trans: Matrix4) {
        derive_surface_method!(self, Transformed::transform_by, trans);
    }
    fn transformed(&self, trans: Matrix4) -> Self {
        derive_surface_self_method!(self, Transformed::transformed, trans)
    }
}

impl IncludeCurve<Curve> for Surface {
    #[inline(always)]
    fn include(&self, curve: &Curve) -> bool {
        match self {
            Surface::Plane(surface) => include_curve(surface, curve),
            Surface::BSplineSurface(surface) => include_curve(surface, curve),
            Surface::NurbsSurface(surface) => include_curve(surface, curve),
            Surface::RevolutedCurve(surface) => {
                let (origin, axis) = (surface.origin(), surface.axis());
                match surface.entity_curve() {
                    &Curve::Line(entity) => {
                        let entity = BSplineCurve::from(entity);
                        include_curve(&RevolutedCurve::by_revolution(&entity, origin, axis), curve)
                    }
                    Curve::BSplineCurve(entity) => {
                        include_curve(&RevolutedCurve::by_revolution(entity, origin, axis), curve)
                    }
                    Curve::NurbsCurve(entity) => {
                        include_curve(&RevolutedCurve::by_revolution(entity, origin, axis), curve)
                    }
                    Curve::Conic(entity) => {
                        let entity = conic_to_nurbs(entity);
                        include_curve(&RevolutedCurve::by_revolution(&entity, origin, axis), curve)
                    }
                    Curve::IntersectionCurve(_) => unimplemented!(),
                }
            }
            Surface::Extruded(surface) => include_curve(&extruded_to_nurbs(surface), curve),
        }
    }
}

fn include_curve<S>(surface: &S, curve: &Curve) -> bool
where S: IncludeCurve<BSplineCurve<Point3>> + IncludeCurve<NurbsCurve<Vector4>> {
    match curve {
        &Curve::Line(curve) => surface.include(&BSplineCurve::from(curve)),
        Curve::BSplineCurve(curve) => surface.include(curve),
        Curve::NurbsCurve(curve) => surface.include(curve),
        Curve::Conic(curve) => surface.include(&conic_to_nurbs(curve)),
        Curve::IntersectionCurve(_) => unimplemented!(),
    }
}

/// The extruded curve as a NURBS surface, for the curve inclusion test.
fn extruded_to_nurbs(surface: &ExtrudedCurve<Curve, Vector3>) -> NurbsSurface<Vector4> {
    let curve0 = surface.entity_curve();
    let curve1 = curve0.transformed(Matrix4::from_translation(surface.extruding_vector()));
    NurbsSurface::new(BSplineSurface::homotopy(curve0.lift_up(), curve1.lift_up()))
}

#[inline(always)]
fn conic_to_nurbs(
    curve: &Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>,
) -> NurbsCurve<Vector4> {
    curve.to_same_geometry()
}

impl IncludeCurve<Curve> for Plane {
    fn include(&self, curve: &Curve) -> bool {
        curve.lift_up().control_points().iter().all(|v| {
            let p = v.to_point();
            self.search_parameter(p, None, 1).is_some()
        })
    }
}

impl ToSameGeometry<Surface> for Plane {
    fn to_same_geometry(&self) -> Surface { (*self).into() }
}

impl ToSameGeometry<Surface> for RevolutedCurve<Curve> {
    fn to_same_geometry(&self) -> Surface { Surface::RevolutedCurve(Processor::new(self.clone())) }
}

impl SearchNearestParameter<D2> for Surface {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        match self {
            Surface::Plane(plane) => plane.search_nearest_parameter(point, hint, trials),
            Surface::BSplineSurface(bspsurface) => {
                bspsurface.search_nearest_parameter(point, hint, trials)
            }
            Surface::NurbsSurface(surface) => surface.search_nearest_parameter(point, hint, trials),
            Surface::RevolutedCurve(rotted) => {
                let hint = match hint.into() {
                    SPHint2D::Parameter(hint0, hint1) => (hint0, hint1),
                    SPHint2D::Range(x, y) => algo::surface::presearch(rotted, point, (x, y), 100),
                    SPHint2D::None => {
                        algo::surface::presearch(rotted, point, rotted.range_tuple(), 100)
                    }
                };
                algo::surface::search_nearest_parameter(rotted, point, hint, trials)
            }
            Surface::Extruded(surface) => surface.search_nearest_parameter(point, hint, trials),
        }
    }
}

impl ToSameGeometry<Surface> for HomotopySurface<Curve, Curve> {
    fn to_same_geometry(&self) -> Surface {
        let curve0 = self.curve0().clone().lift_up();
        let curve1 = self.curve1().clone().lift_up();
        NurbsSurface::new(BSplineSurface::homotopy(curve0, curve1)).into()
    }
}

impl ToSameGeometry<Surface> for ExtrudedCurve<Curve, Vector3> {
    fn to_same_geometry(&self) -> Surface {
        let (curve0, vector) = (self.entity_curve(), self.extruding_vector());
        let trsl = Matrix4::from_translation(vector);
        let curve1 = self.entity_curve().transformed(trsl);
        match (curve0, &curve1) {
            (Curve::Line(line), Curve::Line(_)) => {
                Plane::new(line.0, line.1, line.0 + vector).into()
            }
            (Curve::BSplineCurve(curve0), Curve::BSplineCurve(curve1)) => {
                BSplineSurface::homotopy(curve0.clone(), curve1.clone()).into()
            }
            (Curve::NurbsCurve(curve0), Curve::NurbsCurve(curve1)) => {
                NurbsSurface::new(BSplineSurface::homotopy(
                    curve0.non_rationalized().clone(),
                    curve1.non_rationalized().clone(),
                ))
                .into()
            }
            (Curve::Conic(_), Curve::Conic(_)) => Surface::Extruded(self.clone()),
            (Curve::IntersectionCurve(_), Curve::IntersectionCurve(_)) => unimplemented!(),
            _ => unreachable!(),
        }
    }
}
