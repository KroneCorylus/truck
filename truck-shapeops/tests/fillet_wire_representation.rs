mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::{try_fillet_solid_along_wire, BlendResult};

const TOL: f64 = 0.001;

fn cube(representation: usize) -> Solid {
    let solid = common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.));
    solid.mapped(
        |point| *point,
        |curve| {
            if representation == 0 {
                return curve.clone();
            }
            let Curve::Line(line) = curve else {
                panic!("straight fixture")
            };
            let spline = BSplineCurve::new(
                KnotVec::bezier_knot(3),
                [0., 0.2, 0.7, 1.].map(|t| line.subs(t)).to_vec(),
            );
            if representation == 1 {
                Curve::BSplineCurve(spline)
            } else {
                Curve::NurbsCurve(NurbsCurve::new(BSplineCurve::new(
                    spline.knot_vec().clone(),
                    spline
                        .control_points()
                        .iter()
                        .zip([1., 0.7, 1.4, 1.])
                        .map(|(p, weight)| p.to_vec().extend(1.) * weight)
                        .collect(),
                )))
            }
        },
        Clone::clone,
    )
}

fn fillet(input: &Solid, probe: Point3, radius: f64, tol: f64, reverse: bool) -> BlendResult {
    let edge = common::blend::edge_through(&input.boundaries()[0], probe);
    let wire = Wire::from(vec![if reverse { edge.inverse() } else { edge }]);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let ids: Vec<_> = input.edge_iter().map(|edge| edge.id()).collect();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        try_fillet_solid_along_wire(input, &wire, radius, tol)
    }));
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert_eq!(
        ids,
        input.edge_iter().map(|edge| edge.id()).collect::<Vec<_>>()
    );
    result
        .expect("fillet must not panic")
        .expect("valid fillet")
}

#[test]
fn a_straight_spline_wire_fillets_without_panicking() {
    for representation in 0..3 {
        let input = cube(representation);
        assert!(input.is_geometric_consistent());
        let result = fillet(&input, Point3::new(5., 0., 10.), 0.6, TOL, false);
        assert!(result.solid.is_geometric_consistent());
        common::assert_solid(
            &result.solid,
            1000. - 10. * 0.6_f64.powi(2) * (1. - PI / 4.),
            &[0],
            TOL,
        );
    }
}

fn assert_radius(face: &Face, inverse: Matrix4, radius: f64, first: bool) {
    let surface = face.surface();
    let (urange, vrange) = surface.try_range_tuple();
    let ((a, b), (c, d)) = (urange.unwrap(), vrange.unwrap());
    for i in 0..=8 {
        let u = a + (b - a) * i as f64 / 8.;
        let point = inverse.transform_point(surface.subs(u, (c + d) / 2.));
        let horizontal = if first { point.y } else { point.x };
        let distance = ((horizontal - radius).powi(2) + (point.z - (10. - radius)).powi(2)).sqrt();
        assert_near!(distance, radius);
        let radial = if first {
            Vector3::new(0., point.y - radius, point.z - (10. - radius))
        } else {
            Vector3::new(point.x - radius, 0., point.z - (10. - radius))
        };
        let normal = inverse
            .transform_vector(surface.normal(u, (c + d) / 2.))
            .normalize();
        assert_near!(normal.dot(radial.normalize()).abs(), 1.);
    }
}

fn check_adjacent_fillets(representation: usize) {
    for (radius, scale, reverse) in [(0.6_f64, 1_f64, false), (0.3, 1.7, true)] {
        let transform = if reverse {
            Matrix4::from_translation(Vector3::new(3., -2., 1.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let input = builder::transformed(&cube(representation), transform);
        let tol = TOL * scale;
        let first = fillet(
            &input,
            transform.transform_point(Point3::new(5., 0., 10.)),
            radius * scale,
            tol,
            reverse,
        );
        assert_eq!(first.generated_faces.len(), 1);
        assert_radius(
            &first.generated_faces[0],
            transform.invert().unwrap(),
            radius,
            true,
        );
        let result = fillet(
            &first.solid,
            transform.transform_point(Point3::new(0., 5., 10.)),
            radius * scale,
            tol,
            reverse,
        );
        assert_eq!(result.generated_faces.len(), 1);
        assert_radius(
            &result.generated_faces[0],
            transform.invert().unwrap(),
            radius,
            false,
        );
        assert!(result.solid.is_geometric_consistent());
        // Two quarter-cylinder removals minus their overlap at the miter.
        let expected = (1000. - 20. * radius.powi(2) * (1. - PI / 4.)
            + radius.powi(3) * (5. / 3. - PI / 2.))
            * scale.powi(3);
        common::assert_solid(&result.solid, expected, &[0], tol);
        common::blend::assert_step(&result.solid, expected, tol);
    }
}

#[test]
fn adjacent_analytic_fillets_meet_at_a_valid_miter() { check_adjacent_fillets(0); }

#[test]
fn adjacent_bspline_fillets_meet_at_a_valid_miter_without_panicking() { check_adjacent_fillets(1); }

#[test]
fn adjacent_nurbs_fillets_meet_at_a_valid_miter() { check_adjacent_fillets(2); }
