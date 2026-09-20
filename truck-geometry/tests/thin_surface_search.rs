use truck_geometry::prelude::*;

#[test]
fn thin_rational_patch_inverts_boundary_and_interior_points() {
    let radius = 4.999999;
    let weight = std::f64::consts::FRAC_1_SQRT_2;
    let surface = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
        vec![
            vec![
                Vector4::new(0.0, 5.000001, radius, 1.0),
                Vector4::new(0.0, radius, radius, 1.0),
            ],
            vec![
                Vector4::new(0.0, 5.000001 * weight, 0.0, weight),
                Vector4::new(0.0, radius * weight, 0.0, weight),
            ],
            vec![
                Vector4::new(radius, 5.000001, 0.0, 1.0),
                Vector4::new(radius, radius, 0.0, 1.0),
            ],
        ],
    ));
    for v in [0.0, 0.37, 1.0] {
        let mut hint = None;
        for i in 0..=12 {
            let u = f64::from(i) / 12.0;
            let point = surface.subs(u, v);
            let found = surface
                .search_parameter(point, hint, 100)
                .expect("thin patch point");
            assert!(surface.subs(found.0, found.1).distance(point) < 1e-10);
            assert!((found.0 - u).abs() < 1e-6 && (found.1 - v).abs() < 1e-6);
            let outside = point + surface.normal(u, v) * 0.01;
            assert!(surface
                .search_parameter(outside, Some(found), 100)
                .is_none());
            hint = Some(found);
        }
    }
    assert!(surface.include(&surface.row_curve(0)));
    assert!(surface.include(&surface.row_curve(1)));
}
