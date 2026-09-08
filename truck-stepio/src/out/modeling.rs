//! Preparing procedural modeling geometry for STEP.

use truck_geometry::prelude::*;
use truck_modeling::{Curve, Surface};
use truck_topology::compress::CompressedSolid;

/// Prepares a modeling solid for [`super::StepModel`] at the given geometric tolerance.
///
/// Contact and intersection curves become spatial splines (exact forms are retained when
/// available); rolling-ball surfaces become rational B-spline surfaces. Other geometry stays
/// exact. Topology, indices, face order and
/// orientation are preserved, and the input is not modified. Use this before exporting a blend:
/// the STEP formatter cannot directly represent [`Surface::Fillet`] or [`Curve::PCurve`].
///
/// `tol` is a positive finite distance in model units. Approximation is checked at sampled
/// parameters, not a certified global error bound. A curve and its supporting surface can each
/// deviate by `tol`, so allow up to `2 * tol` for their agreement in the exported model.
/// Returns `None` for invalid tolerances or if approximation fails within the subdivision limit.
/// Keep the original solid for further modeling and topology naming. Use
/// `truck_meshalgo::tessellation::RobustMeshableShape::robust_triangulation` when tessellating a
/// STEP round trip: the fitted boundary curves need not lie exactly on their fitted surfaces.
pub fn prepare_for_step(
    solid: &CompressedSolid<Point3, Curve, Surface>,
    tol: f64,
) -> Option<CompressedSolid<Point3, Curve, Surface>> {
    if !tol.is_finite() || tol <= 0.0 {
        return None;
    }
    solid.try_mapped(
        |p| Some(*p),
        |c| prepare_curve(c, tol),
        |s| prepare_surface(s, tol),
    )
}

fn prepare_curve(curve: &Curve, tol: f64) -> Option<Curve> {
    if let Curve::IntersectionCurve(intersection) = curve {
        let leader = intersection.leader();
        let (a, b) = curve.range_tuple();
        if (0..=64).all(|i| {
            let t = a + (b - a) * i as f64 / 64.0;
            curve.subs(t).distance(leader.subs(t)) <= tol * 0.5
        }) {
            return prepare_curve(leader, tol * 0.5);
        }
    }
    if let Curve::PCurve(pcurve) = curve {
        if let Surface::Plane(plane) = pcurve.surface().as_ref() {
            return Some(
                BSplineCurve::new(
                    pcurve.curve().knot_vec().clone(),
                    pcurve
                        .curve()
                        .control_points()
                        .iter()
                        .map(|p| plane.subs(p.x, p.y))
                        .collect(),
                )
                .into(),
            );
        }
    }
    match curve {
        Curve::PCurve(_) | Curve::IntersectionCurve(_) => {
            let spline = BSplineCurve::cubic_approximation(
                curve,
                curve.range_tuple(),
                tol * 0.25,
                tol.sqrt(),
                16,
            )?;
            let (a, b) = curve.range_tuple();
            for i in 0..=256 {
                let t = a + (b - a) * i as f64 / 256.0;
                let error = spline.subs(t).distance(curve.subs(t));
                if !error.is_finite() || error > tol {
                    return None;
                }
            }
            Some(spline.into())
        }
        _ => Some(curve.clone()),
    }
}

fn prepare_surface(surface: &Surface, tol: f64) -> Option<Surface> {
    match surface {
        Surface::Fillet(processor) => {
            let transform = *processor.transform();
            let scale = (0..3)
                .map(|i| transform[i].truncate().magnitude2())
                .sum::<f64>()
                .sqrt();
            if !scale.is_finite() || scale <= 0.0 {
                return None;
            }
            let mut nurbs = fillet_to_nurbs(processor.entity(), tol / scale)?;
            nurbs.transform_by(transform);
            if !processor.orientation() {
                nurbs.invert();
            }
            Some(nurbs.into())
        }
        Surface::RevolutedCurve(processor) => {
            let entity = processor.entity();
            let curve = prepare_curve(entity.entity_curve(), tol)?;
            let mut prepared = Processor::with_transform(
                RevolutedCurve::by_revolution(curve, entity.origin(), entity.axis()),
                *processor.transform(),
            );
            if !processor.orientation() {
                prepared.invert();
            }
            Some(Surface::RevolutedCurve(prepared))
        }
        Surface::Extruded(extruded) => Some(Surface::Extruded(ExtrudedCurve::by_extrusion(
            prepare_curve(extruded.entity_curve(), tol)?,
            extruded.extruding_vector(),
        ))),
        _ => Some(surface.clone()),
    }
}

fn fillet_to_nurbs(
    surface: &ApproxFilletSurface<Box<Surface>, Box<Surface>>,
    tol: f64,
) -> Option<NurbsSurface<Vector4>> {
    let (_, (a, b)) = surface.range_tuple();
    // Interpolate the four homogeneous control points of each exact cross-section. Only the
    // longitudinal direction is approximated; the rational cubic across the fillet is retained.
    for level in 0..7 {
        let n = 3 * (1 << level) + 1;
        let mut knots = KnotVec::uniform_knot(3, n - 3);
        knots.transform(b - a, a);
        // Greville stations keep the clamped cubic interpolation system well-conditioned.
        let params: Vec<_> = (0..n)
            .map(|i| (knots[i + 1] + knots[i + 2] + knots[i + 3]) / 3.0)
            .collect();
        let sections: Vec<_> = params.iter().map(|&v| surface.fillet_bezier(v)).collect();
        let rows = (0..4)
            .map(|row| {
                Some(
                    BSplineCurve::try_interpole(
                        knots.clone(),
                        params
                            .iter()
                            .copied()
                            .zip(
                                sections
                                    .iter()
                                    .map(|c| c.non_rationalized().control_points()[row]),
                            )
                            .collect::<Vec<_>>(),
                    )
                    .ok()?
                    .control_points()
                    .to_vec(),
                )
            })
            .collect::<Option<Vec<_>>>()?;
        let nurbs: NurbsSurface<Vector4> =
            NurbsSurface::new(BSplineSurface::new((KnotVec::bezier_knot(3), knots), rows));
        let valid = (0..=(n - 1) * 4).all(|j| {
            let v = a + (b - a) * j as f64 / ((n - 1) * 4) as f64;
            (0..=8).all(|i| {
                let u = i as f64 / 8.0;
                let error = nurbs.subs(u, v).distance(surface.subs(u, v));
                error.is_finite() && error <= tol
            })
        });
        if valid {
            return Some(nurbs);
        }
    }
    None
}
