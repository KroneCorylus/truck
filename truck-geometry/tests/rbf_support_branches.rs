use truck_geometry::prelude::*;

type Support = ExtrudedCurve<BSplineCurve<Point3>, Vector3>;
type Blend = RbfSurface<Line<Point3>, Support, Support, f64>;

fn blend(matrix: Matrix4, scale: f64) -> Blend {
    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Point3::origin(),
            Point3::new(10., 0., 0.),
            Point3::new(0., 10., 0.),
            Point3::origin(),
        ],
    )
    .transformed(matrix);
    let surface =
        ExtrudedCurve::by_extrusion(curve, matrix.transform_vector(Vector3::new(0., 0., 10.)));
    let guide = Line(
        matrix.transform_point(Point3::new(0., 0., 10.)),
        matrix.transform_point(Point3::origin()),
    );
    RbfSurface::new(guide, surface.clone(), surface, 0.5 * scale)
        .try_with_support_hints([(0., 0.5), (1., 0.5)])
        .unwrap()
}

#[test]
fn support_branches_preserve_contacts_derivatives_and_inverse_searches() {
    for scale in [0.01, 1., 100.] {
        let matrix = Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let inverse = matrix.invert().unwrap();
        let fillet = blend(matrix, scale);
        for t in [0., 0.2, 0.5, 0.8, 1.] {
            let circle = fillet.contact_circle(t).unwrap();
            assert!(
                inverse
                    .transform_point(circle.center())
                    .distance(Point3::new(
                        0.5094782647758623,
                        0.5094782647758623,
                        10. * (1. - t)
                    ))
                    < 1e-8
            );
            for (support, contact) in [
                (fillet.surface0(), circle.contact_point0()),
                (fillet.surface1(), circle.contact_point1()),
            ] {
                assert!(
                    support
                        .subs(contact.uv.x, contact.uv.y)
                        .distance(contact.point)
                        < 1e-8 * scale
                );
                let radial = contact.point - circle.center();
                assert!((radial.magnitude() - 0.5 * scale).abs() < 1e-8 * scale);
                assert!(
                    radial
                        .normalize()
                        .dot(support.normal(contact.uv.x, contact.uv.y))
                        > 1. - 1e-9
                );
            }
            for curve in [fillet.contact_curve0(), fillet.contact_curve1()] {
                let point = curve.subs(t);
                let parameter = curve.search_parameter(point, None, 100).unwrap();
                assert!((parameter - t).abs() < 1e-8);
                let reversed = curve.inverse();
                assert!(reversed.subs(1. - t).distance(point) < 1e-8 * scale);
                assert!(
                    (reversed.search_parameter(point, None, 100).unwrap() - (1. - t)).abs() < 1e-8
                );
                let nearest = curve.search_nearest_parameter(point, None, 100).unwrap();
                assert!((nearest - t).abs() < 1e-8);
            }
            for u in [0., 0.2, 0.5, 0.8, 1.] {
                let point = fillet.subs(u, t);
                let uv = fillet.search_parameter(point, None, 100).unwrap();
                assert!(
                    fillet.subs(uv.0, uv.1).distance(point) < 1e-7 * scale,
                    "scale={scale} expected=({u},{t}) got={uv:?} error={}",
                    fillet.subs(uv.0, uv.1).distance(point)
                );
                let eps = 1e-5;
                assert!(
                    ((fillet.subs(u + eps, t) - fillet.subs(u - eps, t)) / (2. * eps)
                        - fillet.uder(u, t))
                    .magnitude()
                        < 1e-7 * scale
                );
                assert!(
                    ((fillet.subs(u, t + eps) - fillet.subs(u, t - eps)) / (2. * eps)
                        - fillet.vder(u, t))
                    .magnitude()
                        < 1e-7 * scale
                );
                assert!(
                    ((fillet.uder(u + eps, t) - fillet.uder(u - eps, t)) / (2. * eps)
                        - fillet.uuder(u, t))
                    .magnitude()
                        < 1e-7 * scale
                );
                assert!(fillet.uvder(u, t).magnitude() < 1e-8 * scale);
                assert!(fillet.vvder(u, t).magnitude() < 1e-8 * scale);
            }
        }
    }
}

#[test]
fn support_branches_survive_cut_reversal_and_boxing() {
    let fillet = blend(Matrix4::identity(), 1.);
    let boxed: RbfSurface<Box<_>, Box<_>, Box<_>, _> = fillet.clone().into();
    assert_eq!(boxed.support_hints(), fillet.support_hints());
    for mut curve in [fillet.contact_curve0(), fillet.contact_curve1()] {
        let original = curve.clone();
        let latter = curve.cut(0.4);
        assert_eq!(
            curve.fillet_surface().support_hints(),
            fillet.support_hints()
        );
        assert_eq!(
            latter.fillet_surface().support_hints(),
            fillet.support_hints()
        );
        for segment in [curve, latter] {
            let mut reversed = segment.clone();
            reversed.invert();
            let boxed: RbfContactCurve<Box<_>, Box<_>, Box<_>, _> = segment.clone().into();
            assert_eq!(
                boxed.fillet_surface().support_hints(),
                fillet.support_hints()
            );
            let (a, b) = segment.range_tuple();
            for i in 0..=8 {
                let t = a + (b - a) * i as f64 / 8.;
                // Cutting a Line renormalizes each guide segment to [0, 1].
                let original_t = 1. - segment.fillet_surface().edge_curve().subs(t).z / 10.;
                let expected = original.subs(original_t);
                assert!(segment.subs(t).distance(expected) < 1e-8);
                assert!(boxed.subs(t).distance(expected) < 1e-8);
                assert!(reversed.subs(a + b - t).distance(expected) < 1e-8);
            }
        }
    }
    assert!(boxed.subs(0.3, 0.7).distance(fillet.subs(0.3, 0.7)) < 1e-8);
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for side in 0..2 {
            for coordinate in 0..2 {
                let mut hints = [(0., 0.5), (1., 0.5)];
                if coordinate == 0 {
                    hints[side].0 = invalid;
                } else {
                    hints[side].1 = invalid;
                }
                assert!(fillet.clone().try_with_support_hints(hints).is_none());
            }
        }
    }
}

#[test]
fn support_branches_reach_both_adjacent_edges() {
    let fillet = blend(Matrix4::identity(), 1.);
    for t in [0., 1.] {
        let profile = fillet
            .surface0()
            .entity_curve()
            .transformed(Matrix4::from_translation(Vector3::new(
                0.,
                0.,
                10. * (1. - t),
            )));
        for side in 0..2 {
            let result = if side == 0 {
                fillet.search_contact_curve0_cross_point_with_adjacent_edge(t, &profile, 0., 100)
            } else {
                fillet.search_contact_curve1_cross_point_with_adjacent_edge(t, &profile, 1., 100)
            }
            .unwrap();
            let contact = if side == 0 { result.0 } else { result.1 };
            assert!((result.2 - t).abs() < 1e-8);
            assert!(profile.subs(result.3).distance(contact.point) < 1e-8);
            assert!(fillet.subs(side as f64, t).distance(contact.point) < 1e-8);
        }
    }
}
