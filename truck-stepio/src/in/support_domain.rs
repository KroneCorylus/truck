use super::*;

pub(super) fn fit(
    surface: &mut Surface,
    boundaries: &[Vec<CompressedEdgeIndex>],
    edges: &[CompressedEdge<Curve3D>],
) {
    if let Some(fitted) = fitted(surface, boundaries, edges) {
        *surface = fitted;
    }
}

fn fitted(
    surface: &Surface,
    boundaries: &[Vec<CompressedEdgeIndex>],
    edges: &[CompressedEdge<Curve3D>],
) -> Option<Surface> {
    let (origin, normal, vector) = match surface {
        Surface::ElementarySurface(ElementarySurface::CylindricalSurface(cylinder)) => {
            let entity = cylinder.entity();
            let transform = cylinder.transform();
            (
                transform.transform_point(entity.entity_curve().0),
                transform.transform_vector(entity.axis()).normalize(),
                transform.transform_vector(entity.entity_curve().1 - entity.entity_curve().0),
            )
        }
        Surface::SweptCurve(SweptCurve::ExtrudedCurve(extruded)) => {
            let transform = match extruded.entity_curve() {
                Curve3D::Conic(Conic3D::Ellipse(curve)) => curve.transform(),
                Curve3D::Conic(Conic3D::Parabola(curve)) => curve.transform(),
                Curve3D::Conic(Conic3D::Hyperbola(curve)) => curve.transform(),
                _ => return None,
            };
            (
                transform.transform_point(Point3::origin()),
                transform
                    .x
                    .truncate()
                    .cross(transform.y.truncate())
                    .normalize(),
                extruded.extruding_vector(),
            )
        }
        _ => return None,
    };
    let denominator = normal.dot(vector);
    if denominator == 0. || !denominator.is_finite() {
        return None;
    }
    let mut controls = Vec::new();
    for index in boundaries.iter().flatten() {
        let curve = truck_modeling::Curve::try_from(&edges.get(index.index)?.curve).ok()?;
        let lifted = curve.try_lift_up()?;
        for p in lifted.control_points() {
            if p.w <= 0. || ![p.x, p.y, p.z, p.w].iter().all(|x| x.is_finite()) {
                return None;
            }
            controls.push(Point3::from_homogeneous(*p));
        }
    }
    if controls.is_empty() {
        return None;
    }
    // Positive rational control hulls bound this linear sweep coordinate over
    // the complete boundary curves, including extrema between vertices.
    let parameter = |p: Point3| (p - origin).dot(normal) / denominator;
    let (mut lo, mut hi) = (0_f64, 1_f64);
    for &p in &controls {
        let v = parameter(p);
        if !v.is_finite() {
            return None;
        }
        lo = lo.min(v);
        hi = hi.max(v);
    }
    let padding = TOLERANCE / denominator.abs();
    lo -= padding;
    hi += padding;
    if !lo.is_finite() || !hi.is_finite() || !(hi - lo).is_finite() {
        return None;
    }
    Some(match surface {
        Surface::ElementarySurface(ElementarySurface::CylindricalSurface(cylinder)) => {
            let fitted = cylinder.map_ref(|entity| {
                let line = entity.entity_curve();
                let direction = line.1 - line.0;
                RevolutedCurve::by_revolution(
                    Line(line.0 + lo * direction, line.0 + hi * direction),
                    entity.origin(),
                    entity.axis(),
                )
            });
            Surface::ElementarySurface(ElementarySurface::CylindricalSurface(fitted))
        }
        Surface::SweptCurve(SweptCurve::ExtrudedCurve(extruded)) => {
            let mut curve = extruded.entity_curve().clone();
            match &mut curve {
                Curve3D::Conic(Conic3D::Parabola(conic)) => {
                    let inverse = conic.transform().invert()?;
                    let (mut a, mut b) = conic.range_tuple();
                    for &p in &controls {
                        let t = inverse.transform_point(p - vector * parameter(p)).y / 2.;
                        if !t.is_finite() {
                            return None;
                        }
                        a = a.min(t);
                        b = b.max(t);
                    }
                    *conic = conic.map_ref(|c| TrimmedCurve::new(*c.curve(), (a, b)));
                }
                Curve3D::Conic(Conic3D::Hyperbola(conic)) => {
                    let inverse = conic.transform().invert()?;
                    let (mut a, mut b) = conic.range_tuple();
                    for &p in &controls {
                        let t = inverse.transform_point(p - vector * parameter(p)).y.asinh();
                        if !t.is_finite() {
                            return None;
                        }
                        a = a.min(t);
                        b = b.max(t);
                    }
                    *conic = conic.map_ref(|c| TrimmedCurve::new(*c.curve(), (a, b)));
                }
                _ => {}
            }
            curve.transform_by(Matrix4::from_translation(vector * lo));
            Surface::SweptCurve(SweptCurve::ExtrudedCurve(ExtrudedCurve::by_extrusion(
                curve,
                vector * (hi - lo),
            )))
        }
        _ => unreachable!(),
    })
}
