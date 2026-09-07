//! Concrete curve and surface types for fillet and chamfer tests.
//!
//! `truck-modeling` types cannot hold fillet or chamfer surfaces, so tests use these enums. Solids
//! built with `truck-modeling` are absorbed whole through the `Modeling` variants.

use derive_more::From;
use truck_geometry::prelude::*;

#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Cut,
    SearchNearestParameterD1,
    ParameterDivision1D,
    Invertible,
    From,
)]
pub enum Curve {
    Modeling(truck_modeling::Curve),
    Line(Line<Point3>),
    Nurbs(NurbsCurve<Vector4>),
    Parametric(PCurve<BSplineCurve<Point2>, Box<Surface>>),
    Intersection(IntersectionCurve<Box<Self>, Box<Surface>, Box<Surface>>),
}

impl ToSameGeometry<Curve> for IntersectionCurve<Curve, Surface, Surface> {
    fn to_same_geometry(&self) -> Curve {
        let (surface0, surface1, leader) = self.clone().destruct();
        Curve::Intersection(IntersectionCurve::new(
            Box::new(surface0),
            Box::new(surface1),
            Box::new(leader),
        ))
    }
}

impl ToSameGeometry<Curve> for PCurve<BSplineCurve<Point2>, Surface> {
    fn to_same_geometry(&self) -> Curve {
        let (curve, surface) = self.clone().decompose();
        Curve::Parametric(PCurve::new(curve, Box::new(surface)))
    }
}

#[derive(
    Clone,
    Debug,
    ParametricSurface3D,
    SearchParameterD2,
    SearchNearestParameterD2,
    ParameterDivision2D,
    From,
)]
pub enum Surface {
    Modeling(truck_modeling::Surface),
    Nurbs(NurbsSurface<Vector4>),
    Round(Processor<Sphere, Matrix4>),
    Fillet(ApproxFilletSurface<Box<Self>, Box<Self>>),
    Processor(Processor<Box<Self>, Matrix4>),
}

impl ToSameGeometry<Surface> for ApproxFilletSurface<Surface, Surface> {
    fn to_same_geometry(&self) -> Surface { Surface::Fillet(self.clone().into()) }
}

impl ToSameGeometry<Surface> for BSplineSurface<Point3> {
    fn to_same_geometry(&self) -> Surface { Surface::Nurbs(self.clone().into()) }
}

impl Invertible for Surface {
    fn invert(&mut self) {
        match self {
            Self::Modeling(surface) => surface.invert(),
            Self::Nurbs(surface) => surface.invert(),
            Self::Round(surface) => surface.invert(),
            Self::Fillet(_) => {
                let mut processor = Processor::new(Box::new(self.clone()));
                processor.invert();
                *self = Self::Processor(processor);
            }
            Self::Processor(processor) => processor.invert(),
        }
    }
}

impl ToSameGeometry<Curve> for NurbsCurve<Vector4> {
    fn to_same_geometry(&self) -> Curve { Curve::Nurbs(self.clone()) }
}

impl ToSameGeometry<Curve> for Line<Point3> {
    fn to_same_geometry(&self) -> Curve { Curve::Line(*self) }
}

impl ToSameGeometry<Surface> for Plane {
    fn to_same_geometry(&self) -> Surface { Surface::Nurbs(BSplineSurface::from(*self).into()) }
}

truck_topology::prelude!(Point3, Curve, Surface, pub);

/// Converts a `truck-modeling` solid into these types.
pub fn from_modeling(solid: &truck_modeling::Solid) -> Solid {
    solid.mapped(
        |p| *p,
        |c| Curve::Modeling(c.clone()),
        |s| Surface::Modeling(s.clone()),
    )
}

/// Index of the face of `shell` whose surface passes through `point`.
pub fn face_through(shell: &Shell, point: Point3) -> usize {
    shell
        .face_iter()
        .position(|face| {
            let surface = face.surface();
            surface
                .search_parameter(point, None, 10)
                .is_some_and(|(u, v)| surface.subs(u, v).near(&point))
        })
        .expect("no face through the point")
}

/// The edge of `shell` that passes through `point`, within its parameter range.
pub fn edge_through(shell: &Shell, point: Point3) -> Edge {
    shell
        .edge_iter()
        .find(|edge| {
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            curve
                .search_nearest_parameter(point, None, 10)
                .is_some_and(|t| t0 <= t && t <= t1 && curve.subs(t).near(&point))
        })
        .expect("no edge through the point")
}

impl ToSameGeometry<Surface> for NurbsSurface<Vector4> {
    fn to_same_geometry(&self) -> Surface { Surface::Nurbs(self.clone()) }
}

impl ToSameGeometry<Surface> for Processor<Sphere, Matrix4> {
    fn to_same_geometry(&self) -> Surface { Surface::Round(*self) }
}
