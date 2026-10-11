mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

fn quarter_cone(radius: f64, height: f64) -> Solid {
    let o = builder::vertex(Point3::origin());
    let x = builder::vertex(Point3::new(radius, 0., 0.));
    let y = builder::vertex(Point3::new(0., radius, 0.));
    let p = builder::vertex(Point3::new(0., 0., height));
    let ox = builder::line(&o, &x);
    let oy = builder::line(&o, &y);
    let op = builder::line(&o, &p);
    let xp = builder::line(&x, &p);
    let yp = builder::line(&y, &p);
    let arc = builder::circle_arc(
        &x,
        &y,
        Point3::new(radius / 2_f64.sqrt(), radius / 2_f64.sqrt(), 0.),
    );
    let base = builder::try_attach_plane(&[wire![ox.clone(), arc.clone(), oy.inverse()]])
        .unwrap()
        .inverse();
    let side0 = builder::try_attach_plane(&[wire![ox, xp.clone(), op.inverse()]]).unwrap();
    let side1 = builder::try_attach_plane(&[wire![op, yp.inverse(), oy.inverse()]]).unwrap();
    let mut cone = Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
        Curve::Line(Line(x.point(), p.point())),
        Point3::origin(),
        Vector3::unit_z(),
    )));
    cone.invert();
    let curved = Face::new(vec![wire![arc, yp, xp.inverse()]], cone);
    Solid::new(vec![vec![base, side0, side1, curved].into()])
}

fn volume(radius: f64, height: f64, round: f64) -> f64 {
    // Integrate the exact conical height over the removed corner in polar coordinates.
    // theta=(pi/4)(1-cos(phi)) smooths the square-root endpoints of the radial bound.
    let n = 2048;
    let h = PI / n as f64;
    let moment = (0..=n)
        .map(|i| {
            let phi = i as f64 * h;
            let theta = PI / 4. * (1. - phi.cos());
            let a = theta.cos() + theta.sin();
            let rho = round / (a + (a * a - 1.).max(0.).sqrt());
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * rho.powi(3) / 3. * PI / 4. * phi.sin()
        })
        .sum::<f64>()
        * h
        / 3.;
    PI * radius * radius * height / 12. - height * round * round * (1. - PI / 4.)
        + height / radius * moment
}

#[test]
fn curved_end_trims_fit_inside_the_exported_fillet_surface_domain() {
    for round in [0.25, 1.] {
        for reversed in [false, true] {
            let transform = if reversed {
                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            } else {
                Matrix4::identity()
            };
            check_case(5., 12., round, reversed, transform);
        }
    }
}

fn check_case(radius: f64, height: f64, round: f64, reversed: bool, transform: Matrix4) {
    let input = builder::transformed(&quarter_cone(radius, height), transform);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let inverse = transform.invert().unwrap();
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, PI * radius * radius * height / 12., &[0], TOL);
    let edge = input
        .edge_iter()
        .find(|e| {
            [e.front(), e.back()].iter().all(|v| {
                let p = inverse.transform_point(v.point());
                p.x.hypot(p.y) < TOLERANCE
            })
        })
        .unwrap();
    let edge = if reversed { edge.inverse() } else { edge };
    let result = try_fillet_solid_along_wire(&input, &wire![edge], round, TOL).unwrap();
    let expected = volume(radius, height, round);
    assert!(result.solid.is_geometric_consistent());
    assert_eq!(result.generated_faces.len(), 1);
    let surface = result.generated_faces[0].oriented_surface();
    let (ur, vr) = surface.try_range_tuple();
    let ((a, b), (c, d)) = (ur.unwrap(), vr.unwrap());
    for i in 0..=8 {
        for j in 0..=8 {
            let u = a + (b - a) * i as f64 / 8.;
            let v = c + (d - c) * j as f64 / 8.;
            let p = inverse.transform_point(surface.subs(u, v));
            assert!(((p.x - round).hypot(p.y - round) - round).abs() <= TOL);
            let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
            let expected = Vector3::new(p.x - round, p.y - round, 0.).normalize();
            assert!(normal.dot(expected) > 1. - 1e-5);
        }
    }
    common::assert_solid(&result.solid, expected, &[0], TOL);
    let prepared =
        truck_stepio::out::prepare_for_step(&result.solid.compress(), TOL / 20.).unwrap();
    for (source, fitted) in result.solid.face_iter().zip(&prepared.boundaries[0].faces) {
        if !matches!(source.surface(), Surface::Fillet(_)) {
            continue;
        }
        let (u_range, v_range) = fitted.surface.try_range_tuple();
        let ((u0, u1), (v0, v1)) = (u_range.unwrap(), v_range.unwrap());
        for edge in source.boundary_iters().into_iter().flatten() {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            for i in 0..=173 {
                let point = curve.subs(a + (b - a) * i as f64 / 173.);
                let (u, v) = fitted
                    .surface
                    .search_nearest_parameter(point, None, 100)
                    .unwrap();
                let bounded = fitted.surface.subs(u.clamp(u0, u1), v.clamp(v0, v1));
                assert!(
                    bounded.distance(point) <= TOL / 10.,
                    "trim leaves prepared surface: {}",
                    bounded.distance(point)
                );
            }
        }
    }
    common::blend::assert_step(&result.solid, expected, TOL);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
