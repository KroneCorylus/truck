mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::{out::*, r#in::Table};

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/spherical-meridian/input.step")).unwrap();
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

fn expected_volume(radius: f64, round: f64) -> f64 {
    let height = radius * 3_f64.sqrt() / 2.;
    let center = (radius * radius - 2. * radius * round).sqrt();
    let cap_area = |rho: f64| {
        height * (rho * rho - height * height).sqrt() + rho * rho * (height / rho).asin()
    };
    let limit = (-round / (radius - round)).acos();
    let n = 4096;
    let step = limit / n as f64;
    // Integrate removed half-disc strips in planes parallel to the selected support.
    let removed = (0..=n)
        .map(|i| {
            let angle = i as f64 * step;
            let y = round * (1. - angle.cos());
            let sphere = (radius * radius - y * y).sqrt();
            let fillet = center + round * angle.sin();
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * (cap_area(sphere) - cap_area(fillet)) * round * angle.sin()
        })
        .sum::<f64>()
        * step
        / 3.;
    PI / 2. * (radius * radius * height - height.powi(3) / 3.) - removed
}

#[test]
fn spherical_meridian_fillet_exports_with_its_curved_cap_trims() {
    for transformed in [false, true] {
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        };
        let solid = builder::transformed(&input(), matrix);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let inverse = matrix.invert().unwrap();
        assert!(solid.is_geometric_consistent());
        let edge = solid
            .edge_iter()
            .find(|edge| {
                let curve = edge.curve();
                let (a, b) = curve.range_tuple();
                inverse
                    .transform_point(curve.subs((a + b) / 2.))
                    .distance(Point3::new(-150., 0., 0.))
                    < 1e-6
            })
            .unwrap();
        let edge = if transformed { edge.inverse() } else { edge };
        let result = try_fillet_solid_along_wire(&solid, &wire![edge], 10., TOL).unwrap();
        assert!(result.solid.is_geometric_consistent());
        let center = (150_f64.powi(2) - 2. * 150. * 10.).sqrt();
        for face in &result.generated_faces {
            let surface = face.oriented_surface();
            if let Surface::Fillet(processor) = &surface {
                assert_smooth_internal_knots(processor.entity());
            }
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            for i in 0..=8 {
                for j in 0..=16 {
                    let u = a + (b - a) * i as f64 / 8.;
                    let v = c + (d - c) * j as f64 / 16.;
                    let p = inverse.transform_point(surface.subs(u, v));
                    let rho = p.x.hypot(p.z);
                    assert!(((rho - center).hypot(p.y - 10.) - 10.).abs() < TOL);
                    let expected = Vector3::new(
                        (rho - center) * p.x / rho,
                        p.y - 10.,
                        (rho - center) * p.z / rho,
                    )
                    .normalize();
                    let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(normal.dot(expected) > 1. - 1e-5);
                }
            }
        }
        let prepared = prepare_for_step(&result.solid.compress(), TOL / 20.)
            .expect("spherical-meridian fillet must export at the requested tolerance");
        assert_eq!(prepared.boundaries.len(), 1);
        let expected = expected_volume(150., 10.);
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}

fn assert_smooth_internal_knots(surface: &ApproxFilletSurface<Box<Surface>, Box<Surface>>) {
    let (knots, _) = surface.vknot_vec().to_single_multi();
    for knots in knots.windows(3) {
        let v = knots[1];
        let h = (v - knots[0]).min(knots[2] - v) * 1e-5;
        for u in [0.2, 0.5, 0.8] {
            for order in [1, 2] {
                let left = 2. * surface.der_mn(0, order, u, v - h)
                    - surface.der_mn(0, order, u, v - 2. * h);
                let right = 2. * surface.der_mn(0, order, u, v + h)
                    - surface.der_mn(0, order, u, v + 2. * h);
                let scale = left.magnitude().max(right.magnitude()).max(1.);
                assert!(
                    left.distance(right) < 1e-6 * scale,
                    "derivative {order} jumps at knot {v}: {:?} versus {:?}",
                    left,
                    right
                );
            }
        }
    }
}
