//! Conversions into the curve and surface enums of `truck-modeling`.
//!
//! Circles, ellipses and elementary surfaces keep their exact form: a cylinder becomes a circle
//! swept along its axis, a cone a revolved line, a sphere and a torus a revolved circle.
//! Finite parabolas and hyperbolas become exact quadratic splines on `[0, 1]`.
//! Linear pcurves on planes and cylindrical generators become exact spatial lines.
//! Other pcurves are rejected.
//!
//! ```
//! use truck_modeling::{Curve, Point3, Surface};
//! use truck_stepio::r#in::Table;
//! use truck_topology::compress::CompressedSolid;
//! let step_string = include_str!(concat!(
//!     env!("CARGO_MANIFEST_DIR"),
//!     "/../resources/step/occt-cylinder.step",
//! ));
//! let table = Table::from_step(step_string).unwrap();
//! let step_solid = table.manifold_solid_brep.values().next().unwrap();
//! let (solid, skipped) = table.to_compressed_solid(step_solid).unwrap();
//! assert!(skipped.is_empty());
//! let solid: CompressedSolid<Point3, Curve, Surface> = solid
//!     .try_mapped(|p| Some(*p), |c| c.try_into().ok(), |s| s.try_into().ok())
//!     .unwrap();
//! assert!(solid.boundaries[0]
//!     .faces
//!     .iter()
//!     .any(|face| matches!(face.surface, Surface::Extruded(_))));
//! ```

use super::*;
use std::f64::consts::PI;
use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};

impl TryFrom<&Curve3D> for ModelingCurve {
    type Error = StepConvertingError;
    fn try_from(curve: &Curve3D) -> std::result::Result<Self, Self::Error> {
        Ok(match curve {
            Curve3D::Line(line) => Self::Line(*line),
            Curve3D::Polyline(polyline) => {
                if polyline.0.len() < 2 {
                    return Err("a polyline needs at least two points".into());
                }
                let knots = KnotVec::uniform_knot(1, polyline.0.len() - 1);
                Self::BSplineCurve(BSplineCurve::new(knots, polyline.0.clone()))
            }
            Curve3D::Conic(Conic3D::Ellipse(ellipse)) => Self::Conic(*ellipse),
            Curve3D::Conic(Conic3D::Parabola(curve)) => {
                let (a, b) = finite_conic_range(curve)?;
                let p = curve.subs(a);
                let points = vec![p, p + curve.der(a) * ((b - a) / 2.), curve.subs(b)];
                if points
                    .iter()
                    .any(|p| ![p.x, p.y, p.z].iter().all(|x| x.is_finite()))
                {
                    return Err("non-finite parabola geometry".into());
                }
                Self::BSplineCurve(BSplineCurve::new(KnotVec::bezier_knot(2), points))
            }
            Curve3D::Conic(Conic3D::Hyperbola(curve)) => {
                Self::NurbsCurve(hyperbola_to_nurbs(curve)?)
            }
            Curve3D::BSplineCurve(curve) => Self::BSplineCurve(curve.clone()),
            Curve3D::NurbsCurve(curve) => Self::NurbsCurve(curve.clone()),
            Curve3D::PCurve(pcurve) => {
                let Curve2D::Line(line) = pcurve.curve().as_ref() else {
                    return Err("unsupported imported pcurve".into());
                };
                let linear = match pcurve.surface().as_ref() {
                    Surface::ElementarySurface(ElementarySurface::Plane(_)) => true,
                    Surface::ElementarySurface(ElementarySurface::CylindricalSurface(surface)) => {
                        let (a, b, curvature) = if surface.orientation() {
                            (
                                line.0.y,
                                line.1.y,
                                surface.vvder(line.0.x, line.0.y).magnitude()
                                    + surface
                                        .vvder(line.0.x, line.0.y + std::f64::consts::FRAC_PI_2)
                                        .magnitude(),
                            )
                        } else {
                            (
                                line.0.x,
                                line.1.x,
                                surface.uuder(line.0.x, line.0.y).magnitude()
                                    + surface
                                        .uuder(line.0.x + std::f64::consts::FRAC_PI_2, line.0.y)
                                        .magnitude(),
                            )
                        };
                        // Allow seam-search roundoff only, with a bound on the chord error.
                        let delta = (b - a).abs();
                        delta <= 32. * f64::EPSILON * a.abs().max(b.abs()).max(1.)
                            && delta <= 1e-6
                            && curvature * delta * delta / 8. <= TOLERANCE
                    }
                    _ => false,
                };
                if !linear {
                    return Err("unsupported imported pcurve".into());
                }
                let (front, back) = (pcurve.front(), pcurve.back());
                if ![front.x, front.y, front.z, back.x, back.y, back.z]
                    .iter()
                    .all(|x| x.is_finite())
                {
                    return Err("non-finite imported pcurve".into());
                }
                Self::Line(Line(front, back))
            }
        })
    }
}

fn finite_conic_range(
    curve: &impl BoundedCurve,
) -> std::result::Result<(f64, f64), StepConvertingError> {
    let (a, b) = curve.range_tuple();
    if !a.is_finite() || !b.is_finite() || a >= b || !(b - a).is_finite() {
        return Err("conic requires a finite increasing trim range".into());
    }
    Ok((a, b))
}

fn hyperbola_to_nurbs(
    curve: &Hyperbola<Point3, Matrix4>,
) -> std::result::Result<NurbsCurve<Vector4>, StepConvertingError> {
    let (a, b) = finite_conic_range(curve)?;
    if !a.cosh().is_finite() || !b.cosh().is_finite() {
        return Err("non-finite hyperbola geometry".into());
    }
    let count = (((b - a) / 2.).ceil() as usize).max(1);
    let mut knots = vec![0., 0., 0.];
    let point = |t: f64| Vector4::new(t.cosh(), t.sinh(), 0., 1.);
    let mut points = vec![point(a)];
    for i in 0..count {
        let start = a + (b - a) * i as f64 / count as f64;
        let end = if i + 1 == count {
            b
        } else {
            a + (b - a) * (i + 1) as f64 / count as f64
        };
        let middle = start + (end - start) / 2.;
        // The tangent intersection has weight cosh(half-span); its weighted
        // coordinates are the hyperbola's midpoint, giving an exact conic arc.
        points.push(Vector4::new(
            middle.cosh(),
            middle.sinh(),
            0.,
            ((end - start) / 2.).cosh(),
        ));
        points.push(point(end));
        knots.extend([(i + 1) as f64 / count as f64; 2]);
    }
    knots.push(1.);
    for p in &mut points {
        *p = curve.transform() * *p;
    }
    if points
        .iter()
        .any(|p| ![p.x, p.y, p.z, p.w].iter().all(|x| x.is_finite()) || p.w <= 0.)
    {
        return Err("non-finite hyperbola control geometry".into());
    }
    let mut result = NurbsCurve::new(BSplineCurve::new(KnotVec::from(knots), points));
    if !curve.orientation() {
        result.invert();
    }
    Ok(result)
}

impl TryFrom<&Surface> for ModelingSurface {
    type Error = StepConvertingError;
    fn try_from(surface: &Surface) -> std::result::Result<Self, Self::Error> {
        Ok(match surface {
            Surface::ElementarySurface(elementary) => elementary.into(),
            Surface::SweptCurve(SweptCurve::ExtrudedCurve(extruded)) => {
                let curve: ModelingCurve = extruded.entity_curve().try_into()?;
                Self::Extruded(ExtrudedCurve::by_extrusion(
                    curve,
                    extruded.extruding_vector(),
                ))
            }
            Surface::SweptCurve(SweptCurve::RevolutedCurve(revolved)) => {
                let curve: ModelingCurve = revolved.entity().entity_curve().try_into()?;
                Self::RevolutedCurve(revolved.map_ref(|revolved| {
                    RevolutedCurve::by_revolution(curve, revolved.origin(), revolved.axis())
                }))
            }
            Surface::BSplineSurface(surface) => Self::BSplineSurface(surface.clone()),
            Surface::NurbsSurface(surface) => Self::NurbsSurface(surface.clone()),
        })
    }
}

impl From<&ElementarySurface> for ModelingSurface {
    fn from(surface: &ElementarySurface) -> Self {
        match surface {
            ElementarySurface::Plane(plane) => Self::Plane(*plane),
            ElementarySurface::CylindricalSurface(cylinder) => {
                let (transform, revolved) = (cylinder.transform(), cylinder.entity());
                let origin = transform.transform_point(revolved.origin());
                let axis = transform.transform_vector(revolved.axis()).normalize();
                let point = transform.transform_point(revolved.entity_curve().0);
                let foot = origin + axis * (point - origin).dot(axis);
                let circle = arc(foot, point - foot, axis, (0.0, 2.0 * PI));
                let vector = transform
                    .transform_vector(revolved.entity_curve().1 - revolved.entity_curve().0);
                let extruded = Self::Extruded(ExtrudedCurve::by_extrusion(circle, vector));
                oriented_like(extruded, cylinder, |p| {
                    p - (origin + axis * (p - origin).dot(axis))
                })
            }
            ElementarySurface::ConicalSurface(cone) => {
                Self::RevolutedCurve(cone.map_ref(|revolved| {
                    let line = ModelingCurve::Line(*revolved.entity_curve());
                    RevolutedCurve::by_revolution(line, revolved.origin(), revolved.axis())
                }))
            }
            ElementarySurface::Sphere(sphere) => {
                let (transform, entity) = (sphere.transform(), sphere.entity().0);
                let center = transform.transform_point(entity.center());
                let [_, y, z] = axes(transform);
                let meridian = arc(center, -entity.radius() * z, -y, (0.0, PI));
                let revolved = RevolutedCurve::by_revolution(meridian, center, z);
                let surface = Self::RevolutedCurve(Processor::new(revolved));
                oriented_like(surface, sphere, |p| p - center)
            }
            ElementarySurface::ToroidalSurface(torus) => {
                let (transform, entity) = (torus.transform(), torus.entity());
                let center = transform.transform_point(entity.center());
                let (major, minor) = (entity.large_radius(), entity.small_radius());
                let [x, y, z] = axes(transform);
                let tube = arc(center + major * x, minor * x, -y, (0.0, 2.0 * PI));
                let revolved = RevolutedCurve::by_revolution(tube, center, z);
                let surface = Self::RevolutedCurve(Processor::new(revolved));
                oriented_like(surface, torus, |p| {
                    let radial = p - center;
                    let radial = radial - z * radial.dot(z);
                    p - (center + major * radial.normalize())
                })
            }
        }
    }
}

/// The circle around `axis` through `center + radial`, with the angle `range` measured from
/// `radial`.
fn arc(center: Point3, radial: Vector3, axis: Vector3, range: (f64, f64)) -> ModelingCurve {
    let transform = Matrix4::from_cols(
        radial.extend(0.0),
        axis.cross(radial).extend(0.0),
        axis.extend(0.0),
        center.to_homogeneous(),
    );
    let unit_arc = TrimmedCurve::new(UnitCircle::new(), range);
    ModelingCurve::Conic(Processor::with_transform(unit_arc, transform))
}

/// unit axes of a rigid `transform`
fn axes(transform: &Matrix4) -> [Vector3; 3] {
    [0, 1, 2].map(|i| transform[i].truncate().normalize())
}

/// Whether the normal of `surface` at a sample point points along `outward` there.
fn faces_outward(surface: &impl ParametricSurface3D, outward: impl Fn(Point3) -> Vector3) -> bool {
    let (u, v) = (0.3, 0.4);
    surface.normal(u, v).dot(outward(surface.subs(u, v))) > 0.0
}

/// `surface` flipped if it faces the other way than `reference`. `outward` gives a direction
/// crossing the surface, the same for both, at any of their points.
fn oriented_like<S: ParametricSurface3D + Invertible>(
    mut surface: S,
    reference: &impl ParametricSurface3D,
    outward: impl Fn(Point3) -> Vector3,
) -> S {
    if faces_outward(&surface, &outward) != faces_outward(reference, &outward) {
        surface.invert();
    }
    surface
}
