use truck_modeling::*;
fn cylinder() -> Surface {
    let w = 0.5_f64.sqrt();
    let rows = [
        Vector4::new(2., 0., 0., 1.),
        Vector4::new(2. * w, 2. * w, 0., w),
        Vector4::new(0., 2., 0., 1.),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, p)| {
        vec![
            p + Vector4::new(0., 0., 0.3 * i as f64 * p.w, 0.),
            p + Vector4::new(0., 0., (10. + 0.7 * i as f64) * p.w, 0.),
        ]
    })
    .collect();
    Surface::NurbsSurface(NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
        rows,
    )))
}
#[test]
fn rational_ruled_cylinders_have_exact_typed_offsets() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        for inverse in [false, true] {
            let mut input = cylinder().transformed(transform);
            if inverse {
                input.invert();
            }
            let Some((geometry::Elementary::Cylinder { radius, .. }, outward)) = input.elementary()
            else {
                panic!("exact rational cylinder must be recognized")
            };
            assert!((radius - 2.).abs() < 1e-8);
            assert_eq!(outward, !inverse);
            for distance in [-0.5, 0., 0.5] {
                let offset = input.offset(distance).unwrap();
                let Some((geometry::Elementary::Cylinder { radius, .. }, orientation)) =
                    offset.elementary()
                else {
                    panic!("cylinder offset")
                };
                assert!((radius - (2. + if inverse { -distance } else { distance })).abs() < 1e-8);
                assert_eq!(orientation, outward);
                for (u, v) in [(0.1, 0.1), (0.5, 0.5), (0.9, 0.9)] {
                    let p = input.subs(u, v) + distance * input.normal(u, v).normalize();
                    let (s, t) = offset.search_parameter(p, None, 100).unwrap();
                    assert!(offset.subs(s, t).distance(p) < 1e-6);
                }
            }
        }
    }
}
#[test]
fn elliptic_or_nonruled_rational_surfaces_are_not_cylinders() {
    let mut ellipse = cylinder();
    ellipse.transform_by(Matrix4::from_nonuniform_scale(2., 1., 1.));
    assert!(ellipse.elementary().is_none());
    let Surface::NurbsSurface(mut patch) = cylinder() else {
        unreachable!()
    };
    patch.control_point_mut(1, 1).x += 0.1;
    assert!(Surface::NurbsSurface(patch).elementary().is_none());
}
#[test]
fn reversed_native_cylinder_offsets_preserve_orientation() {
    let native = cylinder().offset(0.).unwrap();
    for input in [native.clone(), native.inverse()] {
        let (_, outward) = input.elementary().unwrap();
        for distance in [-0.5, 0.5] {
            let offset = input.offset(distance).unwrap();
            let Some((geometry::Elementary::Cylinder { radius, .. }, orientation)) =
                offset.elementary()
            else {
                panic!("cylinder")
            };
            assert!((radius - (2. + if outward { distance } else { -distance })).abs() < 1e-8);
            assert_eq!(orientation, outward);
        }
    }
}
#[test]
fn non_bezier_knot_spans_are_not_assumed_to_be_circular() {
    let Surface::NurbsSurface(patch) = cylinder() else {
        unreachable!()
    };
    let patch = NurbsSurface::new(BSplineSurface::new(
        (
            KnotVec::from(vec![0., 0., 0., 1., 2., 3.]),
            KnotVec::bezier_knot(1),
        ),
        patch.control_points().clone(),
    ));
    assert!(Surface::NurbsSurface(patch).elementary().is_none());
}
