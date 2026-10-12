mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/rounded-roof-post/input.step")).unwrap();
    let source = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table.to_compressed_solid(source).unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(TOL / 100.);
    Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap()
}

fn root(solid: &Solid, inverse: Matrix4) -> Wire {
    solid
        .face_iter()
        .find_map(|face| {
            face.boundaries().into_iter().find(|wire| {
                wire.iter().all(|edge| {
                    let curve = edge.curve();
                    let (a, b) = curve.range_tuple();
                    [a, (a + b) / 2., b].iter().all(|&t| {
                        let point = inverse.transform_point(curve.subs(t));
                        (point.z - 5.).abs() < 1e-6
                            && ((point.x - 2.5).hypot(point.y - 2.5) - 1.).abs() < 1e-6
                    })
                })
            })
        })
        .unwrap()
}

fn input_volume() -> f64 { 125. - 5. * (1. - PI / 4.) + 7.5 * PI }

#[test]
fn rounded_roof_post_import_preserves_its_material() {
    let solid = input();
    assert!(solid.is_geometric_consistent());
    common::assert_solid(&solid, input_volume(), &[0], TOL);
}

#[test]
fn post_fillet_continues_across_the_rounded_roof_boundary() {
    let expected = continued_volume(512);
    assert!(
        (expected - continued_volume(1024)).abs() < 1e-7,
        "quadrature: {expected} / {}",
        continued_volume(1024)
    );
    for (scale, transformed) in [(1_f64, false), (0.25, true), (10., true)] {
        let matrix = (if transformed {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        }) * Matrix4::from_scale(scale);
        let solid = builder::transformed(&input(), matrix);
        let mut wire = root(&solid, matrix.invert().unwrap());
        if transformed {
            wire.invert();
        }
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let result = try_fillet_solid_along_wire(&solid, &wire, scale, TOL * scale);
        let result = result.expect("the rolling ball must continue onto the rounded roof");
        assert!(result.solid.is_geometric_consistent());
        assert_contacts(&result, matrix.invert().unwrap());
        common::assert_solid(
            &result.solid,
            expected * scale.powi(3),
            &[0],
            TOL * scale / 10.,
        );
        common::blend::assert_step(&result.solid, expected * scale.powi(3), TOL * scale / 10.);
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}

#[test]
fn post_fillet_that_fits_the_roof_remains_supported() {
    let solid = input();
    let radius = 0.2_f64;
    let result =
        try_fillet_solid_along_wire(&solid, &root(&solid, Matrix4::identity()), radius, TOL)
            .unwrap();
    let added = PI
        * (2. * radius.powi(2) + 5. * radius.powi(3) / 3.
            - PI / 2. * (1. + radius) * radius.powi(2));
    common::assert_solid(&result.solid, input_volume() + added, &[0], TOL);
    common::blend::assert_step(&result.solid, input_volume() + added, TOL);
}

fn assert_contacts(result: &BlendResult, inverse: Matrix4) {
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for j in 0..=16 {
            let v = c + (d - c) * j as f64 / 16.;
            let endpoints = [a, b].map(|u| inverse.transform_point(surface.subs(u, v)));
            let post = endpoints
                .iter()
                .find(|p| ((p.x - 2.5).hypot(p.y - 2.5) - 1.).abs() < 1e-6)
                .unwrap();
            let theta = (post.y - 2.5).atan2(post.x - 2.5);
            let q = (theta.cos() - 0.75).max(0.);
            let center = Point3::new(
                2.5 + 2. * theta.cos(),
                2.5 + 2. * theta.sin(),
                4. + 2. * (1. - q * q).sqrt(),
            );
            for i in 0..=8 {
                let u = a + (b - a) * i as f64 / 8.;
                let point = inverse.transform_point(surface.subs(u, v));
                let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                assert!((point.distance(center) - 1.).abs() < TOL);
                assert!(normal.dot((center - point).normalize()) > 1. - 1e-5);
            }
        }
    }
    for expected in [
        Point3::new(3.5, 2.5, 4. + 3.75_f64.sqrt()),
        Point3::new(4.25, 2.5, 4. + 0.9375_f64.sqrt()),
    ] {
        assert!(result
            .solid
            .vertex_iter()
            .any(|v| inverse.transform_point(v.point()).distance(expected) < 1e-6));
    }
}

fn simpson(n: usize, a: f64, b: f64, f: impl Fn(f64) -> f64) -> f64 {
    let h = (b - a) / n as f64;
    (0..=n)
        .map(|i| {
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * f(a + h * i as f64)
        })
        .sum::<f64>()
        * h
        / 3.
}

fn continued_volume(n: usize) -> f64 {
    let limit = 0.75_f64.acos();
    let addition = [(-limit, limit, true), (limit, 2. * PI - limit, false)]
        .into_iter()
        .map(|(a, b, curved)| {
            simpson(n, a, b, |theta| {
                let (s, c) = theta.sin_cos();
                let q = if curved { c - 0.75 } else { 0. };
                let h = (1. - q * q).sqrt();
                let dq = if curved { -s } else { 0. };
                let dh = -q * dq / h;
                let center = Vector3::new(2.5 + 2. * c, 2.5 + 2. * s, 4. + 2. * h);
                let dc = Vector3::new(-2. * s, 2. * c, 2. * dh);
                let n0 = Vector3::new(c, s, 0.);
                let dn0 = Vector3::new(-s, c, 0.);
                let n1 = Vector3::new(q, 0., h);
                let dn1 = Vector3::new(dq, 0., dh);
                let alpha = (c * q).acos();
                let sa = alpha.sin();
                let da = -(-s * q + c * dq) / sa;
                let fillet = simpson(n, 0., 1., |u| {
                    let a = ((1. - u) * alpha).sin() / sa;
                    let b = (u * alpha).sin() / sa;
                    let ap = ((1. - u) * ((1. - u) * alpha).cos() - a * alpha.cos()) * da / sa;
                    let bp = (u * (u * alpha).cos() - b * alpha.cos()) * da / sa;
                    let p = center - a * n0 - b * n1;
                    let pu = alpha / sa * (((1. - u) * alpha).cos() * n0 - (u * alpha).cos() * n1);
                    let pv = dc - ap * n0 - a * dn0 - bp * n1 - b * dn1;
                    p.z * (pu.x * pv.y - pu.y * pv.x)
                });
                let x = (center - n1).x;
                let primitive = if x <= 4. {
                    5. * x
                } else {
                    let d = x - 4.;
                    20. + 4. * d + 0.5 * (d * (1. - d * d).sqrt() + d.asin())
                };
                fillet - primitive * (dc - dn1).y
            })
        })
        .sum::<f64>();
    // Integrate z dx dy over the fillet, subtract the removed roof, retain the post disc.
    input_volume() + addition + 5. * PI
}
