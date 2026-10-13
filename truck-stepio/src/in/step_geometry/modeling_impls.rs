//! Conversions into the curve and surface enums of `truck-modeling`.
//!
//! Circles, ellipses and elementary surfaces keep their exact form: a cylinder becomes a circle
//! swept along its axis, a cone a revolved line, a sphere and a torus a revolved circle.
//! Finite parabolas and hyperbolas become exact quadratic splines on `[0, 1]`.
//! Linear pcurves on planes and cylindrical generators become exact spatial lines.
//! Isoparametric pcurves on toroidal surfaces become exact spatial circles.
//! Isoparametric pcurves on spline surfaces become exact spatial splines.
//! Other linear pcurves on cylinders, cones and tori retain their exact surface coordinates.
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
                if ![
                    line.0.x,
                    line.0.y,
                    line.1.x,
                    line.1.y,
                    line.1.x - line.0.x,
                    line.1.y - line.0.y,
                ]
                .iter()
                .all(|x| x.is_finite())
                {
                    return Err("non-finite imported pcurve".into());
                }
                if line.0 == line.1 {
                    return Err("degenerate imported pcurve".into());
                }
                match pcurve.surface().as_ref() {
                    Surface::ElementarySurface(ElementarySurface::ToroidalSurface(surface)) => {
                        return toroidal_pcurve(surface, line);
                    }
                    Surface::BSplineSurface(surface) => {
                        return Ok(Self::BSplineCurve(spline_isoparameter(surface, line)?));
                    }
                    Surface::NurbsSurface(surface) => {
                        return Ok(Self::NurbsCurve(NurbsCurve::new(spline_isoparameter(
                            surface.non_rationalized(),
                            line,
                        )?)));
                    }
                    _ => {}
                }
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
                    return match pcurve.surface().as_ref() {
                        Surface::ElementarySurface(surface) => elementary_pcurve(surface, line),
                        _ => Err("unsupported imported pcurve".into()),
                    };
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

fn elementary_pcurve(
    surface: &ElementarySurface,
    line: &Line<Point2>,
) -> std::result::Result<ModelingCurve, StepConvertingError> {
    let revolved = match surface {
        ElementarySurface::CylindricalSurface(surface)
        | ElementarySurface::ConicalSurface(surface) => surface.map_ref(|revolved| {
            RevolutedCurve::by_revolution(
                ModelingCurve::Line(*revolved.entity_curve()),
                revolved.origin(),
                revolved.axis(),
            )
        }),
        ElementarySurface::ToroidalSurface(surface) => {
            let mut revolved = surface.map_ref(|torus| {
                let center = torus.center();
                let meridian = arc(
                    center + torus.large_radius() * Vector3::unit_x(),
                    torus.small_radius() * Vector3::unit_x(),
                    -Vector3::unit_y(),
                    (0., 2. * PI),
                );
                RevolutedCurve::by_revolution(meridian, center, Vector3::unit_z())
            });
            // Torus coordinates are (major, minor); revolution coordinates are (meridian, major).
            revolved.invert();
            revolved
        }
        _ => return Err("unsupported imported pcurve".into()),
    };
    let transform = revolved.transform();
    if (0..4).any(|i| (0..4).any(|j| !transform[i][j].is_finite())) || transform.invert().is_none()
    {
        return Err("invalid imported pcurve transform".into());
    }
    let curve = ModelingCurve::PCurve(truck_modeling::PCurve::new(
        BSplineCurve::from(*line),
        Box::new(ModelingSurface::RevolutedCurve(revolved)),
    ));
    let mut regular = false;
    for t in [0., 0.5, 1.] {
        regular |= curve.der(t) != Vector3::zero();
        if [curve.subs(t).to_vec(), curve.der(t), curve.der2(t)]
            .iter()
            .any(|v| ![v.x, v.y, v.z].iter().all(|x| x.is_finite()))
        {
            return Err("non-finite imported pcurve geometry".into());
        }
    }
    if !regular {
        return Err("degenerate imported pcurve geometry".into());
    }
    Ok(curve)
}

fn toroidal_pcurve(
    surface: &ToroidalSurface,
    source_line: &Line<Point2>,
) -> std::result::Result<ModelingCurve, StepConvertingError> {
    let mut line = *source_line;
    if !surface.orientation() {
        line = Line(
            Point2::new(line.0.y, line.0.x),
            Point2::new(line.1.y, line.1.x),
        );
    }
    let entity = surface.entity();
    let (major, minor) = (entity.large_radius(), entity.small_radius());
    let transform = surface.transform();
    let scale = [transform.x, transform.y, transform.z]
        .into_iter()
        .map(|v| v.truncate().magnitude2())
        .sum::<f64>()
        .sqrt();
    // Bound the spatial error from flattening seam-search roundoff after the affine transform.
    let constant = |a: f64, b: f64, radius: f64| {
        let delta = (b - a).abs();
        delta <= 32. * f64::EPSILON * a.abs().max(b.abs()).max(1.)
            && delta * radius * scale / 2. <= TOLERANCE
    };
    let (center, radial, axis, start, end) = if constant(line.0.x, line.1.x, major + minor) {
        let u = line.0.x + (line.1.x - line.0.x) / 2.;
        let radial = Vector3::new(u.cos(), u.sin(), 0.);
        (
            entity.center() + major * radial,
            minor * radial,
            radial.cross(Vector3::unit_z()),
            line.0.y,
            line.1.y,
        )
    } else if constant(line.0.y, line.1.y, minor) {
        let v = line.0.y + (line.1.y - line.0.y) / 2.;
        (
            entity.center() + minor * v.sin() * Vector3::unit_z(),
            (major + minor * v.cos()) * Vector3::unit_x(),
            Vector3::unit_z(),
            line.0.x,
            line.1.x,
        )
    } else {
        return elementary_pcurve(&ElementarySurface::ToroidalSurface(*surface), source_line);
    };
    if start == end || !(end - start).is_finite() || radial.magnitude2() == 0. {
        return Err("degenerate toroidal pcurve".into());
    }
    let mut curve =
        arc(center, radial, axis, (start.min(end), start.max(end))).transformed(*transform);
    if start > end {
        curve.invert();
    }
    if [curve.front(), curve.back()]
        .iter()
        .any(|p| ![p.x, p.y, p.z].iter().all(|x| x.is_finite()))
    {
        return Err("non-finite toroidal pcurve geometry".into());
    }
    Ok(curve)
}

fn spline_isoparameter<P: truck_base::cgmath64::control_point::ControlPoint<f64> + Tolerance>(
    surface: &BSplineSurface<P>,
    line: &Line<Point2>,
) -> std::result::Result<BSplineCurve<P>, StepConvertingError> {
    let (uk, vk) = surface.knot_vecs();
    if [uk, vk].iter().any(|knots| {
        let (a, b) = (knots[0], knots[knots.len() - 1]);
        !a.is_finite() || !b.is_finite() || a >= b || !(b - a).is_finite()
    }) {
        return Err("spline pcurve requires a finite support domain".into());
    }
    let roundoff =
        |a: f64, b: f64| (a - b).abs() <= 32. * f64::EPSILON * a.abs().max(b.abs()).max(1.);
    let mut line = *line;
    // Seam searches can leave endpoints a few ulps outside the support or off a constant parameter.
    for p in [&mut line.0, &mut line.1] {
        for (axis, knots) in [uk, vk].iter().enumerate() {
            let clamped = p[axis].clamp(knots[0], knots[knots.len() - 1]);
            if !p[axis].is_finite() || !roundoff(p[axis], clamped) {
                return Err("spline pcurve is outside its finite support domain".into());
            }
            p[axis] = clamped;
        }
    }
    let (knots, points, start, end) = if roundoff(line.0.x, line.1.x) {
        let u = line.0.x + (line.1.x - line.0.x) / 2.;
        let points = (0..surface.control_points()[0].len())
            .map(|j| surface.row_curve(j).subs(u))
            .collect();
        (vk.clone(), points, line.0.y, line.1.y)
    } else if roundoff(line.0.y, line.1.y) {
        let v = line.0.y + (line.1.y - line.0.y) / 2.;
        let points = (0..surface.control_points().len())
            .map(|i| surface.column_curve(i).subs(v))
            .collect();
        (uk.clone(), points, line.0.x, line.1.x)
    } else {
        return Err("unsupported non-isoparametric spline pcurve".into());
    };
    if start == end {
        return Err("degenerate spline pcurve".into());
    }
    let mut curve = BSplineCurve::new(knots, points);
    let (a, b) = (start.min(end), start.max(end));
    if a > curve.range_tuple().0 {
        curve = curve.cut(a);
    }
    if b < curve.range_tuple().1 {
        curve.cut(b);
    }
    if start > end {
        curve.invert();
    }
    let (knots, points) = curve.destruct();
    let (a, b) = (knots[0], knots[knots.len() - 1]);
    if a >= b || !(b - a).is_finite() {
        return Err("degenerate spline pcurve trim".into());
    }
    let knots = knots.iter().map(|t| (t - a) / (b - a)).collect();
    Ok(BSplineCurve::try_new(knots, points)?)
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
