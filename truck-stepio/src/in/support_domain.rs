use super::*;

pub(super) fn trimmed_revolution(
    surface: &Surface,
    holder: &FaceSurfaceHolder,
    table: &Table,
) -> Option<Surface> {
    let Surface::SweptCurve(SweptCurve::RevolutedCurve(revolved)) = surface else {
        return None;
    };
    let Curve3D::Conic(Conic3D::Ellipse(ellipse)) = revolved.entity().entity_curve() else {
        return None;
    };
    if !ellipse.orientation() {
        return None;
    }
    let face = holder.clone().into_owned(table).ok()?;
    let support = Surface::try_from(&face.face_geometry).ok()?;
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut outer = false;
    for bound in &face.bounds {
        let mut points = Vec::new();
        for edge in &bound.bound.edge_list {
            let (edge, forward) = match edge {
                EdgeAny::EdgeCurve(edge) => (edge, true),
                EdgeAny::OrientedEdge(edge) => (&edge.edge_element, edge.orientation),
            };
            let CurveAny::SurfaceCurve(curve) = &edge.edge_geometry else {
                return None;
            };
            let pcurve = curve
                .associated_geometry
                .iter()
                .find_map(|geometry| match geometry {
                    PcurveOrSurface::Pcurve(pcurve)
                        if pcurve.basis_surface == face.face_geometry =>
                    {
                        Some(pcurve)
                    }
                    _ => None,
                })?;
            let pcurve = PCurve::try_from(pcurve.as_ref()).ok()?;
            let Curve2D::BSplineCurve(curve) = pcurve.curve().as_ref() else {
                return None;
            };
            let [a, b] = curve.control_points().as_slice() else {
                return None;
            };
            if curve.degree() != 1 || ![a.x, a.y, b.x, b.y].iter().all(|x| x.is_finite()) {
                return None;
            }
            let start = Point3::from(&edge.edge_start.vertex_geometry);
            let end = Point3::from(&edge.edge_end.vertex_geometry);
            let (start, end) = if edge.same_sense {
                (start, end)
            } else {
                (end, start)
            };
            // Only bounded pcurves that describe the complete edge can bound its support.
            if !pcurve.front().near(&start) || !pcurve.back().near(&end) {
                return None;
            }
            points.extend(if forward == edge.same_sense {
                [*a, *b]
            } else {
                [*b, *a]
            });
            lo = lo.min(a.y).min(b.y);
            hi = hi.max(a.y).max(b.y);
        }
        let origin = *points.first()?;
        let mut area = 0.;
        let mut scale = 0.;
        for i in 0..points.len() {
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            // Separate pcurve ends may be joined in UV only along a collapsed pole.
            if i % 2 == 1
                && !a.near(&b)
                && (!a.y.near(&b.y)
                    || !support.uder(a.x, a.y).so_small()
                    || !support.uder(b.x, b.y).so_small())
            {
                return None;
            }
            let (a, b) = (a - origin, b - origin);
            area += a.perp_dot(b);
            scale += (a.x * b.y).abs() + (a.y * b.x).abs();
        }
        let sign = if bound.orientation == face.same_sense {
            1.
        } else {
            -1.
        };
        outer |= sign * area > f64::EPSILON * (points.len() + 2) as f64 * scale;
    }
    if !outer || lo >= hi || !lo.is_finite() || !hi.is_finite() || hi - lo >= 2. * PI {
        return None;
    }
    let meridian = ellipse.map_ref(|curve| TrimmedCurve::new(*curve.curve(), (lo, hi)));
    Some(Surface::SweptCurve(SweptCurve::RevolutedCurve(
        revolved.map_ref(|entity| {
            RevolutedCurve::by_revolution(
                Curve3D::Conic(Conic3D::Ellipse(meridian)),
                entity.origin(),
                entity.axis(),
            )
        }),
    )))
}

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
    if let Surface::ElementarySurface(ElementarySurface::ConicalSurface(cone)) = surface {
        return fitted_cone(cone, &boundary_control_hull(boundaries, edges)?)
            .map(|cone| Surface::ElementarySurface(ElementarySurface::ConicalSurface(cone)));
    }
    let mut profile_controls = Vec::new();
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
                Curve3D::Conic(Conic3D::Ellipse(curve)) => Some(curve.transform()),
                Curve3D::Conic(Conic3D::Parabola(curve)) => Some(curve.transform()),
                Curve3D::Conic(Conic3D::Hyperbola(curve)) => Some(curve.transform()),
                Curve3D::BSplineCurve(_) | Curve3D::NurbsCurve(_) => {
                    profile_controls = positive_control_hull(extruded.entity_curve())?;
                    None
                }
                _ => return None,
            };
            let (origin, normal) = match transform {
                Some(transform) => (
                    transform.transform_point(Point3::origin()),
                    transform
                        .x
                        .truncate()
                        .cross(transform.y.truncate())
                        .normalize(),
                ),
                None => (
                    *profile_controls.first()?,
                    extruded.extruding_vector().normalize(),
                ),
            };
            (origin, normal, extruded.extruding_vector())
        }
        _ => return None,
    };
    let denominator = normal.dot(vector);
    if denominator == 0. || !denominator.is_finite() {
        return None;
    }
    let controls = boundary_control_hull(boundaries, edges)?;
    // The enclosing conic boxes and positive rational control hulls bound this
    // sweep coordinate over complete curves, including extrema between vertices.
    let parameter = |p: Point3| (p - origin).dot(normal) / denominator;
    let (mut base_lo, mut base_hi) = (0_f64, 0_f64);
    for p in profile_controls {
        let v = parameter(p);
        if !v.is_finite() {
            return None;
        }
        base_lo = base_lo.min(v);
        base_hi = base_hi.max(v);
    }
    let (mut lo, mut hi) = (0_f64, 1_f64);
    for &p in &controls {
        let v = parameter(p);
        if !v.is_finite() {
            return None;
        }
        // P = Q + v V: subtract the entire profile projection hull, including nonplanar profiles.
        lo = lo.min(v - base_hi);
        hi = hi.max(v - base_lo);
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

fn positive_control_hull(curve: &Curve3D) -> Option<Vec<Point3>> {
    truck_modeling::Curve::try_from(curve)
        .ok()?
        .try_lift_up()?
        .control_points()
        .iter()
        .map(|p| {
            (p.w > 0. && [p.x, p.y, p.z, p.w].iter().all(|x| x.is_finite()))
                .then(|| Point3::from_homogeneous(*p))
        })
        .collect()
}

fn boundary_control_hull(
    boundaries: &[Vec<CompressedEdgeIndex>],
    edges: &[CompressedEdge<Curve3D>],
) -> Option<Vec<Point3>> {
    let mut controls = Vec::new();
    for index in boundaries.iter().flatten() {
        let source = &edges.get(index.index)?.curve;
        if let Curve3D::Conic(Conic3D::Ellipse(ellipse)) = source {
            let matrix = ellipse.transform();
            if matrix.x.w == 0. && matrix.y.w == 0. && matrix.w.w != 0. {
                let center = Point3::from_homogeneous(matrix.w);
                let x = matrix.x.truncate() / matrix.w.w;
                let y = matrix.y.truncate() / matrix.w.w;
                // The affine circle lies inside this parallelogram, including full turns.
                for point in [
                    center + x + y,
                    center + x - y,
                    center - x + y,
                    center - x - y,
                ] {
                    if ![point.x, point.y, point.z].iter().all(|x| x.is_finite()) {
                        return None;
                    }
                    controls.push(point);
                }
                continue;
            }
        }
        controls.extend(positive_control_hull(source)?);
    }
    (!controls.is_empty()).then_some(controls)
}

fn fitted_cone(
    cone: &step_geometry::ConicalSurface,
    controls: &[Point3],
) -> Option<step_geometry::ConicalSurface> {
    let entity = cone.entity();
    let line = entity.entity_curve();
    let axis = entity.axis();
    let direction = line.1 - line.0;
    let axial = direction.dot(axis);
    let inverse = cone.transform().invert()?;
    if axial == 0. || !axial.is_finite() {
        return None;
    }
    let parameter = |p| (inverse.transform_point(p) - line.0).dot(axis) / axial;
    let (mut lower, mut upper) = (f64::INFINITY, f64::NEG_INFINITY);
    for &p in controls {
        let t = parameter(p);
        if !t.is_finite() {
            return None;
        }
        lower = lower.min(t);
        upper = upper.max(t);
    }
    let radial = line.0 - entity.origin();
    let radial = radial - axis * radial.dot(axis);
    let slope = direction - axis * axial;
    let pole = -radial.dot(slope) / slope.magnitude2();
    if !pole.is_finite() || !(radial + pole * slope).so_small() {
        return None;
    }
    let padding = TOLERANCE * inverse.transpose().transform_vector(axis).magnitude() / axial.abs();
    let (mut lo, mut hi) = (lower.min(0.) - padding, upper.max(1.) + padding);
    // Keep one cone nappe: padding through the apex would introduce an interior pole.
    if lower >= pole - padding {
        lo = lo.max(pole);
    } else if upper <= pole + padding {
        hi = hi.min(pole);
    } else {
        return None;
    }
    if !lo.is_finite() || !hi.is_finite() || !(hi - lo).is_finite() || lo >= hi {
        return None;
    }
    let point = |t| {
        let p = line.subs(t);
        if t == pole {
            entity.origin() + axis * (p - entity.origin()).dot(axis)
        } else {
            p
        }
    };
    Some(cone.map_ref(|entity| {
        RevolutedCurve::by_revolution(Line(point(lo), point(hi)), entity.origin(), entity.axis())
    }))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cone_support_bounds_keep_the_apex_at_a_generator_endpoint() {
        for slope in [0.5_f64, -0.5] {
            for negative_sheet in [false, true] {
                for reversed in [false, true] {
                    for transformed in [false, true] {
                        let line = Line(Point3::new(10., 0., 0.), Point3::new(10. + slope, 0., 1.));
                        let mut affine = Matrix4::from_nonuniform_scale(1.2, 0.8, 1.4);
                        affine.x.z = 0.2;
                        affine.z.x = 0.3;
                        let matrix = if transformed {
                            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                                * Matrix4::from_axis_angle(
                                    Vector3::new(1., 2., 3.).normalize(),
                                    Rad(0.73),
                                )
                                * affine
                        } else {
                            Matrix4::identity()
                        };
                        let mut cone = Processor::with_transform(
                            RevolutedCurve::by_revolution(
                                line,
                                Point3::origin(),
                                Vector3::unit_z(),
                            ),
                            matrix,
                        );
                        if reversed {
                            cone.invert();
                        }
                        let surface =
                            Surface::ElementarySurface(ElementarySurface::ConicalSurface(cone));
                        let pole = -10. / slope;
                        let sheet = if negative_sheet { -1. } else { 1. };
                        let sign = slope.signum() * sheet;
                        let curve = Curve3D::BSplineCurve(BSplineCurve::new(
                            KnotVec::bezier_knot(2),
                            [pole, pole + sign * 60., pole + sign * 12.]
                                .map(|t| matrix.transform_point(line.subs(t)))
                                .to_vec(),
                        ));
                        let edges = [CompressedEdge {
                            vertices: (0, 1),
                            curve,
                        }];
                        let boundaries = vec![vec![CompressedEdgeIndex {
                            index: 0,
                            orientation: true,
                        }]];
                        let fitted =
                            fitted(&surface, &boundaries, &edges).expect("complete cone domain");
                        let Surface::ElementarySurface(ElementarySurface::ConicalSurface(fitted)) =
                            fitted
                        else {
                            panic!("cone representation")
                        };
                        let generator = fitted.entity().entity_curve();
                        let at_pole = if sign > 0. { generator.0 } else { generator.1 };
                        assert!(at_pole.x.abs() < 1e-12);
                        assert!(sheet * generator.0.x >= -1e-12 && sheet * generator.1.x >= -1e-12);
                        let modeled = truck_modeling::Surface::try_from(
                            &Surface::ElementarySurface(ElementarySurface::ConicalSurface(fitted)),
                        )
                        .unwrap();
                        let curve = truck_modeling::Curve::try_from(&edges[0].curve).unwrap();
                        assert!(modeled.include(&curve));
                        let (u, v) = modeled.try_range_tuple();
                        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                        for i in 0..=128 {
                            let p = curve.subs(i as f64 / 128.);
                            let (u, v) = modeled.search_parameter(p, None, 100).unwrap();
                            assert!((a - 1e-9..=b + 1e-9).contains(&u));
                            assert!((c - 1e-9..=d + 1e-9).contains(&v));
                            assert!(modeled.subs(u, v).distance(p) < 1e-8);
                            let local = matrix.invert().unwrap().transform_point(p);
                            let t = local.z;
                            let normal = if reversed {
                                surface.normal(0., t)
                            } else {
                                surface.normal(t, 0.)
                            };
                            if i > 0 {
                                assert!(modeled.normal(u, v).dot(normal) > 1. - 1e-8);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn cone_support_bounds_do_not_clip_a_hull_across_the_apex() {
        let surface = Surface::ElementarySurface(ElementarySurface::ConicalSurface(
            Processor::new(RevolutedCurve::by_revolution(
                Line(Point3::new(1., 0., 0.), Point3::new(2., 0., 1.)),
                Point3::origin(),
                Vector3::unit_z(),
            )),
        ));
        let edges = [CompressedEdge {
            vertices: (0, 1),
            curve: Curve3D::Line(Line(Point3::new(-1., 0., -2.), Point3::new(2., 0., 1.))),
        }];
        let boundaries = vec![vec![CompressedEdgeIndex {
            index: 0,
            orientation: true,
        }]];
        assert!(fitted(&surface, &boundaries, &edges).is_none());
    }
}
