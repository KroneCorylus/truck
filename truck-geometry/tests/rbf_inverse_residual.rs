use truck_geometry::prelude::*;

#[test]
fn singular_support_inverse_fails_without_panicking() {
    let fillet = RbfSurface::new(
        Line(Point3::origin(), Point3::new(1., 0., 0.)),
        Plane::xy(),
        Plane::xy(),
        0.5,
    );
    assert!(fillet.contact_circle(0.5).is_none());
    assert!(fillet
        .search_parameter(Point3::new(0.5, 0.25, 0.1), None, 10)
        .is_none());
}

#[test]
fn rolling_surface_inverse_uses_the_evaluated_contact_circle() {
    let mut first = BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Point3::origin(),
            Point3::new(10., 0., 0.),
            Point3::new(0., 10., 0.),
            Point3::origin(),
        ],
    );
    let second = first.cut(0.5);
    for scale in [0.01, 1., 100.] {
        let matrix = Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let vector = matrix.transform_vector(Vector3::new(0., 0., 10.));
        let surface0 = ExtrudedCurve::by_extrusion(first.transformed(matrix), vector);
        let surface1 = ExtrudedCurve::by_extrusion(second.transformed(matrix), vector);
        let guide = Line(
            matrix.transform_point(Point3::new(0., 0., 10.)),
            matrix.transform_point(Point3::origin()),
        );
        let fillet = RbfSurface::new(guide, surface0, surface1, 0.5 * scale);
        for v in [0., 0.2, 0.5, 0.8, 1.] {
            for u in [0.2, 0.5, 0.8] {
                let point = fillet.subs(u, v);
                let (a, b) = fillet
                    .search_parameter(point, None, 100)
                    .unwrap_or_else(|| panic!("scale={scale} uv=({u},{v})"));
                assert!(
                    fillet.subs(a, b).distance(point) < 1e-7 * scale,
                    "scale={scale} uv=({u},{v}) found=({a},{b})"
                );
            }
        }
    }
}
