use truck_modeling::*;

pub(super) fn projected_section(
    surface: &Surface,
    plane: Plane,
    from: Point3,
    to: Point3,
) -> Option<Curve> {
    let Surface::Extruded(extrusion) = surface else {
        return None;
    };
    let axis = extrusion.extruding_vector().normalize();
    let normal = plane.normal().normalize();
    let denominator = normal.dot(axis);
    if denominator.abs() <= TOLERANCE {
        return None;
    }
    let source = extrusion.entity_curve();
    let range = source.range_tuple();
    let origin = source.subs(range.0);
    if [0.25, 0.5, 0.75, 1.0].iter().any(|fraction| {
        axis.dot(source.subs(range.0 + fraction * (range.1 - range.0)) - origin)
            .abs()
            > TOLERANCE
    }) {
        return None;
    }
    let transverse = normal - axis * denominator;
    // Shearing the generator plane gives its exact projection without a singular
    // transform, so the resulting conic retains a usable inverse parameter map.
    let shear = Matrix4::from_cols(
        (Vector3::unit_x() - axis * transverse.x / denominator).extend(0.0),
        (Vector3::unit_y() - axis * transverse.y / denominator).extend(0.0),
        (Vector3::unit_z() - axis * transverse.z / denominator).extend(0.0),
        Vector4::unit_w(),
    );
    let projected = origin + axis * normal.dot(plane.origin() - origin) / denominator;
    let transform = Matrix4::from_translation(projected.to_vec())
        * shear
        * Matrix4::from_translation(-origin.to_vec());
    let mut curve = source.transformed(transform);
    if let Curve::Conic(conic) = &curve {
        // UnitCircle has zero Z: size its unused normal axis like its actual
        // axes so tessellation does not oversample a small circle as a unit one.
        let mut transform = *conic.transform();
        let scale = transform
            .x
            .truncate()
            .magnitude()
            .max(transform.y.truncate().magnitude());
        transform.z = (transform.z.truncate().normalize() * scale).extend(0.0);
        let inverse = transform.invert()?;
        let angle = |p: Point3| {
            let p = inverse.transform_point(p);
            p.y.atan2(p.x)
        };
        let (mut a, mut b) = (angle(from), angle(to));
        let (start, end) = conic.entity().range_tuple();
        let period = std::f64::consts::TAU;
        // A periodic support connects contacts locally across its seam; a
        // bounded source retains its explicit arc, including arcs over half a turn.
        if end - start >= period {
            b += period * ((a - b) / period).round();
        } else {
            let mid = start + (end - start) / 2.0;
            a += period * ((mid - a) / period).round();
            b += period * ((mid - b) / period).round();
            if a.min(b) < start - TOLERANCE || a.max(b) > end + TOLERANCE {
                return None;
            }
            a = a.clamp(start, end);
            b = b.clamp(start, end);
        }
        if a == b {
            return None;
        }
        let mut section = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (a.min(b), a.max(b))),
            transform,
        );
        if a > b {
            section.invert();
        }
        let curve = Curve::Conic(section);
        let (start, end) = curve.range_tuple();
        return (curve.subs(start).near(&from) && curve.subs(end).near(&to)).then_some(curve);
    }
    let a = curve.search_nearest_parameter(from, None, 100)?;
    let b = curve.search_nearest_parameter(to, None, 100)?;
    if a.min(b) < range.0 - TOLERANCE || a.max(b) > range.1 + TOLERANCE {
        return None;
    }
    curve = curve.cut(a.min(b).max(range.0));
    curve.cut(a.max(b).min(range.1));
    if a > b {
        curve.invert();
    }
    let range = curve.range_tuple();
    (curve.subs(range.0).near(&from) && curve.subs(range.1).near(&to)).then_some(curve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oblique_plane_section_preserves_exact_conic_at_different_scales() {
        for scale in [0.01, 1.0, 100.0] {
            let transform = Matrix4::from_translation(Vector3::new(13.0, -7.0, 23.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale);
            let circle = Curve::Conic(Processor::with_transform(
                TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, 1.2)),
                Matrix4::from_scale(2.0),
            ));
            let surface: Surface =
                ExtrudedCurve::by_extrusion(circle, Vector3::unit_z() * 7.0).into();
            let surface = surface.transformed(transform);
            let plane = Plane::new(
                Point3::new(0.0, 0.0, 4.0),
                Point3::new(3.0, 0.0, 3.0),
                Point3::new(0.0, 3.0, 2.0),
            )
            .transformed(transform);
            let expected = |t: f64| {
                let (x, y) = (2.0 * t.cos(), 2.0 * t.sin());
                transform.transform_point(Point3::new(x, y, 4.0 - (x + 2.0 * y) / 3.0))
            };
            for (a, b) in [(0.0, 1.2), (1.0, 0.2)] {
                let curve = projected_section(&surface, plane, expected(a), expected(b)).unwrap();
                assert!(matches!(curve, Curve::Conic(_)));
                let range = curve.range_tuple();
                for i in 0..=32 {
                    let fraction = i as f64 / 32.0;
                    let point = curve.subs(range.0 + fraction * (range.1 - range.0));
                    assert!(point.distance(expected(a + fraction * (b - a))) < 1.0e-9 * scale);
                    assert!(plane.search_parameter(point, None, 100).is_some());
                    assert!(surface.search_parameter(point, None, 100).is_some());
                }
            }
        }
    }
}

#[cfg(test)]
mod seam_tests {
    use super::*;
    fn check(full: bool, reversals: &[bool]) {
        for scale in [0.01, 1., 100.] {
            for &reversed in reversals {
                let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(scale);
                let mut circle = Processor::with_transform(
                    TrimmedCurve::new(
                        UnitCircle::<Point3>::new(),
                        (0., if full { std::f64::consts::TAU } else { 4.8 }),
                    ),
                    Matrix4::from_scale(2.),
                );
                if reversed {
                    circle.invert();
                }
                let surface: Surface =
                    ExtrudedCurve::by_extrusion(Curve::Conic(circle), Vector3::unit_z() * 7.)
                        .into();
                let surface = surface.transformed(transform);
                let plane = Plane::new(
                    Point3::new(0., 0., 4.),
                    Point3::new(3., 0., 3.),
                    Point3::new(0., 3., 2.),
                )
                .transformed(transform);
                let expected = |t: f64| {
                    let (x, y) = (2. * t.cos(), 2. * t.sin());
                    transform.transform_point(Point3::new(x, y, 4. - (x + 2. * y) / 3.))
                };
                let cases = if full {
                    vec![
                        (std::f64::consts::TAU - 1e-8, 0.04, 0.04000001),
                        (0.04, std::f64::consts::TAU - 1e-8, -0.04000001),
                        (std::f64::consts::TAU - 0.2, 0.15, 0.35),
                        (0., 0.04, 0.04),
                    ]
                } else {
                    vec![(0.2, 4.5, 4.3), (4.5, 0.2, -4.3)]
                };
                for (a, b, delta) in cases {
                    let curve=projected_section(&surface,plane,expected(a),expected(b)).unwrap_or_else(||panic!("no section scale={scale} reversed={reversed} full={full} a={a} b={b}"));
                    let range = curve.range_tuple();
                    for i in 0..=32 {
                        let t = i as f64 / 32.;
                        let p = curve.subs(range.0 + (range.1 - range.0) * t);
                        assert!(
                            p.distance(expected(a + delta * t)) < 1e-8 * scale,
                            "wrong branch at {a}->{b}, t{t}: {p:?}"
                        );
                        assert!(plane.search_parameter(p, None, 100).is_some());
                        assert!(surface.search_parameter(p, None, 100).is_some());
                    }
                }
            }
        }
    }
    #[test]
    fn full_period_sections_follow_the_local_arc_across_the_seam() { check(true, &[false, true]); }
    #[test]
    fn bounded_long_sections_preserve_their_explicit_arc() { check(false, &[false]); }
    #[test]
    fn reversed_bounded_sections_preserve_their_explicit_arc() { check(false, &[true]); }
}
