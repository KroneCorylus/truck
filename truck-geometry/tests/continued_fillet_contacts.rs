use truck_geometry::prelude::*;

type Cylinder = ExtrudedCurve<Processor<UnitCircle<Point3>, Matrix4>, Vector3>;

fn supports() -> (Cylinder, Cylinder) {
    let post = Processor::with_transform(
        UnitCircle::new(),
        Matrix4::from_translation(Vector3::new(2.5, 2.5, 2.5)),
    );
    let roof = Processor::with_transform(
        UnitCircle::new(),
        Matrix4::from_cols(
            Vector4::new(0., 0., 1., 0.),
            Vector4::new(1., 0., 0., 0.),
            Vector4::new(0., 1., 0., 0.),
            Vector4::new(4., 0., 4., 1.),
        ),
    );
    (
        ExtrudedCurve::by_extrusion(post, Vector3::new(0., 0., 10.)),
        ExtrudedCurve::by_extrusion(roof, Vector3::new(0., 5., 0.)),
    )
}

#[test]
fn rolling_ball_continues_from_a_plane_onto_a_cylindrical_roof() {
    let limit = 0.75_f64.acos();
    for scale in [0.01, 1., 100.] {
        let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let inverse = transform.invert().unwrap();
        let (post, roof) = supports();
        let post = Processor::with_transform(post, transform);
        let roof = Processor::with_transform(roof, transform);
        let solve = |t: f64| {
            ContactCircle::try_from_contacts(
                (
                    transform.transform_point(Point3::new(2.5 + t.cos(), 2.5 + t.sin(), 5.)),
                    transform.transform_vector(Vector3::new(-t.sin(), t.cos(), 0.)),
                ),
                t,
                &post,
                &roof,
                scale,
                [Point2::new(t, 0.35), Point2::new(0., 0.5)],
            )
        };
        for i in 0..=32 {
            let t = -limit + 2. * limit * i as f64 / 32.;
            let circle = solve(t).expect("continued contacts must solve off the original edge");
            let center = inverse.transform_point(circle.center());
            let q = t.cos() - 0.75;
            let expected = Point3::new(
                2.5 + 2. * t.cos(),
                2.5 + 2. * t.sin(),
                4. + 2. * (1. - q * q).sqrt(),
            );
            assert!(center.distance(expected) < 1e-7);
            for (surface, contact) in [
                (&post, circle.contact_point0()),
                (&roof, circle.contact_point1()),
            ] {
                let normal = surface.normal(contact.uv.x, contact.uv.y);
                assert!((circle.center() - contact.point).distance(normal * scale) < 1e-7 * scale);
            }
        }
        let approx = ApproxFilletSurface::approx_contact_circles(
            &post,
            &roof,
            (-limit, limit),
            1e-5 * scale,
            solve,
        )
        .unwrap();
        for i in 0..=64 {
            let t = -limit + 2. * limit * i as f64 / 64.;
            let exact = solve(t).unwrap();
            let section: NurbsCurve<Vector4> = exact.to_same_geometry();
            for j in 0..=8 {
                let u = j as f64 / 8.;
                let point = approx.subs(u, t);
                let normal = approx.normal(u, t);
                assert!(point.distance(section.subs(u)) < 2e-5 * scale);
                assert!(((point - exact.center()).magnitude() - scale).abs() < 2e-5 * scale);
                assert!(normal.dot((exact.center() - point).normalize()) > 1. - 1e-5);
            }
        }
    }
}

#[test]
fn invalid_continuation_guesses_fail_without_panicking() {
    let (post, roof) = supports();
    let guide = (Point3::new(3.5, 2.5, 5.), Vector3::unit_y());
    for radius in [0., f64::NAN, f64::INFINITY] {
        assert!(ContactCircle::try_from_contacts(
            guide,
            0.,
            &post,
            &roof,
            radius,
            [Point2::new(0., 0.35), Point2::new(0., 0.5)]
        )
        .is_none());
    }
    assert!(ContactCircle::try_from_contacts(
        guide,
        0.,
        &post,
        &roof,
        1.,
        [Point2::new(f64::NAN, 0.35), Point2::new(0., 0.5)]
    )
    .is_none());
    assert!(ContactCircle::try_from_contacts(
        (guide.0, Vector3::zero()),
        0.,
        &post,
        &roof,
        1.,
        [Point2::new(0., 0.35), Point2::new(0., 0.5)]
    )
    .is_none());
}
