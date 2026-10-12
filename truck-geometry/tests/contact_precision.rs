use truck_geometry::prelude::*;

#[test]
fn small_spherical_contact_is_solved_before_fitting_the_blend() {
    let scale = 0.01;
    let sphere = Sphere::new(Point3::origin(), 5. * scale);
    let plane = Plane::new(
        Point3::new(0., 0., 3. * scale),
        Point3::new(scale, 0., 3. * scale),
        Point3::new(0., -scale, 3. * scale),
    );
    let guide = Processor::with_transform(
        UnitCircle::<Point3>::new(),
        Matrix4::from_translation(Vector3::new(0., 0., 3. * scale))
            * Matrix4::from_scale(4. * scale),
    );
    let fillet = RbfSurface::new(guide, sphere, plane, 0.2 * scale);
    for t in [0., 0.3, 1., 2., 3.] {
        let cc = fillet.contact_circle(t).unwrap();
        let height = 3.2 * scale;
        let radial = ((4.8 * scale).powi(2) - height * height).sqrt();
        let expected = Point3::new(radial * t.cos(), radial * t.sin(), height);
        assert!(cc.center().distance(expected) < 1e-7 * scale);
        for (point, normal) in [
            (
                cc.contact_point0().point,
                cc.contact_point0().point.to_vec().normalize(),
            ),
            (cc.contact_point1().point, -Vector3::unit_z()),
        ] {
            assert!((point - cc.center()).distance(normal * 0.2 * scale) < 1e-7 * scale);
        }
    }
}
