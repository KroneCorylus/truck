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
        self.try_lift_up()
            .expect("intersection curve cannot connect by homotopy")
    }
    /// The non-rationalized 4-dimensional B-spline form, `None` for an intersection curve.
    pub fn try_lift_up(&self) -> Option<BSplineCurve<Vector4>> {
        Some(match self {
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
            Curve::IntersectionCurve(_) => return None,
        })
    }
}

impl TryFrom<&Curve> for NurbsCurve<Vector4> {
    type Error = errors::Error;
    fn try_from(curve: &Curve) -> Result<Self> {
        curve
            .try_lift_up()
            .map(NurbsCurve::new)
            .ok_or(errors::Error::NoNurbsForm)
    }
}

impl ToSameGeometry<Curve> for NurbsCurve<Vector4> {
    fn to_same_geometry(&self) -> Curve { Curve::NurbsCurve(self.clone()) }
}

impl ToSameGeometry<Surface> for NurbsSurface<Vector4> {
    fn to_same_geometry(&self) -> Surface { Surface::NurbsSurface(self.clone()) }
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

/// An elementary surface recognised by [`Surface::elementary`], with its parameters.
///
/// Each variant has a canonical normal: the plane's own normal, away from the axis for the
/// cylinder and the cone, away from the centre for the sphere, away from the tube centre for
/// the torus. These are the normals of the corresponding STEP entities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elementary {
    /// plane
    Plane(Plane),
    /// cylinder of radius `radius` around the line through `origin` along `axis`
    Cylinder {
        /// a point on the axis
        origin: Point3,
        /// unit direction of the axis
        axis: Vector3,
        /// radius
        radius: f64,
    },
    /// the nappe of a cone opening from `apex` along `axis`
    Cone {
        /// apex
        apex: Point3,
        /// unit direction from the apex into the nappe the surface lies on
        axis: Vector3,
        /// angle between the axis and the generator, in `(0, π/2)`
        half_angle: Rad<f64>,
    },
    /// sphere
    Sphere {
        /// centre
        center: Point3,
        /// radius
        radius: f64,
    },
    /// torus around the line through `center` along `axis`
    Torus {
        /// centre
        center: Point3,
        /// unit direction of the axis
        axis: Vector3,
        /// distance from the centre to the tube centre
        major: f64,
        /// radius of the tube
        minor: f64,
    },
}

impl Surface {
    /// The elementary surface this is, and whether the surface normal points the canonical
    /// way of [`Elementary`]. `None` for anything else.
    ///
    /// Recognised: a plane; an extruded round circle along its own axis; a revolved line
    /// parallel to the axis (cylinder), perpendicular to it (plane) or meeting it (cone); a
    /// revolved round circle in a plane through the axis, centred on the axis (sphere) or off
    /// it (torus).
    pub fn elementary(&self) -> Option<(Elementary, bool)> {
        let elementary = match self {
            Surface::Plane(plane) => return Some((Elementary::Plane(*plane), true)),
            Surface::Extruded(surface) => extruded_elementary(surface)?,
            Surface::RevolutedCurve(surface) => {
                revolved_elementary(surface.entity())?.transformed(*surface.transform())?
            }
            _ => return None,
        };
        let (urange, vrange) = self.try_range_tuple();
        let mid = |range: Option<(f64, f64)>| range.map_or(0.5, |(a, b)| (a + b) / 2.0);
        let (u, v) = (mid(urange), mid(vrange));
        let outward = self
            .normal(u, v)
            .dot(elementary.canonical_direction(self.subs(u, v)));
        Some((elementary, outward > 0.0))
    }
}

impl Elementary {
    /// A direction along which the canonical normal points at the surface point `p`.
    fn canonical_direction(&self, p: Point3) -> Vector3 {
        match *self {
            Elementary::Plane(plane) => plane.normal(),
            Elementary::Cylinder { origin, axis, .. } => radial(p, origin, axis),
            Elementary::Cone { apex, axis, .. } => radial(p, apex, axis),
            Elementary::Sphere { center, .. } => p - center,
            Elementary::Torus {
                center,
                axis,
                major,
                ..
            } => p - (center + major * radial(p, center, axis).normalize()),
        }
    }

    /// The image under `transform`, `None` if the transform is not a similarity.
    fn transformed(self, transform: Matrix4) -> Option<Self> {
        let cols = [0, 1, 2].map(|i| transform[i].truncate());
        let scale = cols[0].magnitude();
        let similarity = cols.iter().all(|c| c.magnitude().near(&scale))
            && [(0, 1), (1, 2), (2, 0)]
                .iter()
                .all(|&(i, j)| (cols[i].dot(cols[j]) / (scale * scale)).so_small());
        if !similarity {
            return None;
        }
        let point = |p: Point3| transform.transform_point(p);
        let direction = |v: Vector3| transform.transform_vector(v).normalize();
        Some(match self {
            Elementary::Plane(plane) => Elementary::Plane(plane.transformed(transform)),
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            } => Elementary::Cylinder {
                origin: point(origin),
                axis: direction(axis),
                radius: radius * scale,
            },
            Elementary::Cone {
                apex,
                axis,
                half_angle,
            } => Elementary::Cone {
                apex: point(apex),
                axis: direction(axis),
                half_angle,
            },
            Elementary::Sphere { center, radius } => Elementary::Sphere {
                center: point(center),
                radius: radius * scale,
            },
            Elementary::Torus {
                center,
                axis,
                major,
                minor,
            } => Elementary::Torus {
                center: point(center),
                axis: direction(axis),
                major: major * scale,
                minor: minor * scale,
            },
        })
    }
}

/// The component of `p - origin` perpendicular to the unit vector `axis`.
fn radial(p: Point3, origin: Point3, axis: Vector3) -> Vector3 {
    let r = p - origin;
    r - axis * r.dot(axis)
}

/// The centre, radius and unit normal of a round conic, `None` for an ellipse.
fn round_circle(
    conic: &Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>,
) -> Option<(Point3, f64, Vector3)> {
    let transform = *conic.transform();
    let (x, y) = (transform[0].truncate(), transform[1].truncate());
    let (rx, ry) = (x.magnitude(), y.magnitude());
    let round = rx.near(&ry) && (x.dot(y) / (rx * ry)).so_small();
    round.then(|| (transform[3].to_point(), rx, x.cross(y).normalize()))
}

fn extruded_elementary(surface: &ExtrudedCurve<Curve, Vector3>) -> Option<Elementary> {
    let Curve::Conic(circle) = surface.entity_curve() else {
        return None;
    };
    let (center, radius, axis) = round_circle(circle)?;
    let vector = surface.extruding_vector();
    let along_axis = (vector.cross(axis) / vector.magnitude()).so_small();
    along_axis.then_some(Elementary::Cylinder {
        origin: center,
        axis,
        radius,
    })
}

fn revolved_elementary(surface: &RevolutedCurve<Curve>) -> Option<Elementary> {
    let (origin, axis) = (surface.origin(), surface.axis());
    match surface.entity_curve() {
        Curve::Line(line) => revolved_line(*line, origin, axis),
        Curve::Conic(conic) => revolved_circle(conic, origin, axis),
        _ => None,
    }
}

fn revolved_line(Line(p0, p1): Line<Point3>, origin: Point3, axis: Vector3) -> Option<Elementary> {
    let dir = (p1 - p0).normalize();
    let r0 = radial(p0, origin, axis);
    if dir.cross(axis).so_small() {
        return (!r0.so_small()).then_some(Elementary::Cylinder {
            origin: p0 - r0,
            axis,
            radius: r0.magnitude(),
        });
    }
    if dir.dot(axis).so_small() {
        return Some(Elementary::Plane(Plane::new(
            p0,
            p0 + dir,
            p0 + axis.cross(dir),
        )));
    }
    // a line neither parallel nor perpendicular gives a cone only if it meets the axis
    let dir_radial = radial(p0 + dir, p0, axis);
    if !(r0.cross(dir_radial) / dir_radial.magnitude()).so_small() {
        return None;
    }
    let apex = p0 - dir * (r0.dot(dir_radial) / dir_radial.magnitude2());
    let half_angle = Rad(dir.dot(axis).abs().acos());
    let into_nappe = (p0.midpoint(p1) - apex).dot(axis).signum();
    Some(Elementary::Cone {
        apex,
        axis: axis * into_nappe,
        half_angle,
    })
}

fn revolved_circle(
    conic: &Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>,
    origin: Point3,
    axis: Vector3,
) -> Option<Elementary> {
    let (center, radius, normal) = round_circle(conic)?;
    let axis_in_plane = normal.dot(axis).so_small() && (origin - center).dot(normal).so_small();
    if !axis_in_plane {
        return None;
    }
    let r = radial(center, origin, axis);
    Some(match r.so_small() {
        true => Elementary::Sphere { center, radius },
        false => Elementary::Torus {
            center: center - r,
            axis,
            major: r.magnitude(),
            minor: radius,
        },
    })
}
