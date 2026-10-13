use truck_geometry::prelude::*;

fn check<C>(curve: C)
where C: ParametricCurve3D + BoundedCurve + Clone + Invertible + Transformed<Matrix4> {
    for scale in [0.01, 1., 100.] {
        let matrix = Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_nonuniform_scale(scale, 0.7 * scale, 1.3 * scale);
        for curve in [curve.clone(), curve.inverse()] {
            let surface =
                ExtrudedCurve::by_extrusion(curve, Vector3::new(2., -1., 10.)).transformed(matrix);
            let (urange, vrange) = surface.range_tuple();
            for fraction in [0., 1e-7, 1e-5, 0.0005, 0.003, 0.02, 0.37, 0.5] {
                for fraction in [fraction, 1. - fraction] {
                    let u = urange.0 + (urange.1 - urange.0) * fraction;
                    for v in [0., 0.37, 1.] {
                        let p = surface.subs(u, v);
                        for hint in [SPHint2D::None, SPHint2D::Range(urange, vrange)] {
                            let uv = surface.search_parameter(p, hint, 100).unwrap_or_else(|| {
                                panic!("exact scale={scale}, uv=({u},{v}), hint={hint:?}")
                            });
                            assert!(surface.subs(uv.0, uv.1).distance(p) < 1e-7);
                            if fraction > 1e-6 && fraction < 1. - 1e-6 {
                                assert!((uv.0 - u).abs() < 1e-7, "{uv:?} != ({u},{v})");
                            }
                            if (1e-5..=1. - 1e-5).contains(&fraction) {
                                let q = p + surface.normal(u, v) * (1e-5 * scale);
                                let uv = surface.search_nearest_parameter(q, hint, 100).unwrap();
                                assert!(surface.subs(uv.0, uv.1).distance(p) < 1e-7);
                                let off_surface = p + surface.normal(u, v) * (1e-3 * scale);
                                assert!(surface.search_parameter(off_surface, hint, 100).is_none());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cubic() -> BSplineCurve<Point3> {
    BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Point3::origin(),
            Point3::new(10., 0., 0.),
            Point3::new(0., 10., 0.),
            Point3::origin(),
        ],
    )
}

#[test]
fn closed_cubic_extrusion_searches_both_sides_of_the_cusp() { check(cubic()); }

#[test]
fn closed_quartic_extrusion_searches_both_sides_of_the_cusp() {
    check(BSplineCurve::new(
        KnotVec::bezier_knot(4),
        vec![
            Point3::origin(),
            Point3::new(0., -10., 0.),
            Point3::new(50., 50., 0.),
            Point3::new(-10., 0., 0.),
            Point3::origin(),
        ],
    ));
}

#[test]
fn closed_rational_extrusion_searches_both_sides_of_the_cusp() {
    check(NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(3),
        cubic()
            .control_points()
            .iter()
            .zip([1., 0.7, 1.4, 1.])
            .map(|(p, w)| p.to_vec().extend(1.) * w)
            .collect(),
    )));
}

#[test]
fn explicit_extrusion_seeds_keep_the_requested_branch_and_trial_budget() {
    let surface = ExtrudedCurve::by_extrusion(cubic(), Vector3::new(0., 0., 10.));
    let point = surface.subs(0., 0.3);
    for u in [0., 1.] {
        let hint = (u, 0.3);
        assert_eq!(surface.search_parameter(point, hint, 100), Some(hint));
        assert_eq!(
            surface.search_nearest_parameter(point, hint, 100),
            Some(hint)
        );
        let point = surface.subs(0.4, 0.7);
        assert_eq!(
            surface.search_parameter(point, hint, 0),
            algo::surface::search_parameter(&surface, point, hint, 0)
        );
        assert_eq!(
            surface.search_nearest_parameter(point, hint, 0),
            algo::surface::search_nearest_parameter(&surface, point, hint, 0)
        );
    }
}

#[test]
fn closed_rational_ellipse_extrusion_searches_both_sides_of_the_seam() {
    let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., std::f64::consts::TAU));
    let curve: NurbsCurve<Vector4> = circle.to_same_geometry();
    check(curve.transformed(Matrix4::from_nonuniform_scale(10., 5., 1.)));
}

#[test]
fn nearest_extrusion_search_does_not_choose_the_other_cusp_branch() {
    let surface = ExtrudedCurve::by_extrusion(cubic(), Vector3::new(0., 0., 10.));
    for u in [0.0005, 0.9995] {
        let point = surface.subs(u, 0.37);
        let query = point + surface.normal(u, 0.37) * 0.0001;
        let uv = surface.search_nearest_parameter(query, None, 100).unwrap();
        assert!(
            surface.subs(uv.0, uv.1).distance(point) < 1e-8,
            "{uv:?} != ({u},0.37)"
        );
    }
}
