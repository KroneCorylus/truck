use truck_geometry::prelude::*;

fn profiles() -> Vec<BSplineCurve<Point3>> {
    [
        vec![
            Point3::origin(),
            Point3::new(0., 10., 0.),
            Point3::new(10., 0., 0.),
            Point3::origin(),
        ],
        vec![
            Point3::origin(),
            Point3::new(0., -10., 0.),
            Point3::new(50., 50., 0.),
            Point3::new(-10., 0., 0.),
            Point3::origin(),
        ],
    ]
    .into_iter()
    .map(|points| BSplineCurve::new(KnotVec::bezier_knot(points.len() - 1), points))
    .collect()
}

fn lift(curve: &BSplineCurve<Point3>) -> BSplineCurve<Vector4> {
    BSplineCurve::new(
        curve.knot_vec().clone(),
        curve
            .control_points()
            .iter()
            .map(|p| p.to_vec().extend(1.))
            .collect(),
    )
}

#[test]
fn closed_nurbs_surfaces_include_boundary_curves_starting_on_either_cusp_branch() {
    for profile in profiles() {
        for scale in [0.01, 1., 100.] {
            let matrix = Matrix4::from_translation(Vector3::new(2., -3., 5.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale);
            let base = lift(&profile);
            let top =
                lift(&profile.transformed(Matrix4::from_translation(Vector3::new(2., -1., 10.))));
            let surface =
                NurbsSurface::new(BSplineSurface::homotopy(base, top)).transformed(matrix);
            let mut first = profile.clone();
            let second = first.cut(0.5);
            for profile in [first, second] {
                for profile in [profile.clone(), profile.inverse()] {
                    let curve = profile.transformed(matrix);
                    let rational = NurbsCurve::new(lift(&curve));
                    for surface in [surface.clone(), surface.inverse()] {
                        assert!(surface.include(&curve));
                        assert!(surface.include(&rational));
                        for offset in [-0.01, 1.01] {
                            let outside = curve.transformed(Matrix4::from_translation(
                                matrix.transform_vector(Vector3::new(2., -1., 10.) * offset),
                            ));
                            assert!(!surface.include(&outside));
                            assert!(!surface.include(&NurbsCurve::new(lift(&outside))));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn closed_nurbs_inclusion_checks_interior_samples_and_each_knot_span() {
    let profile = profiles().remove(0);
    let base = lift(&profile);
    let surface = NurbsSurface::new(BSplineSurface::homotopy(
        base.clone(),
        lift(&profile.transformed(Matrix4::from_translation(Vector3::new(0., 0., 10.)))),
    ));
    let mut profile = profile;
    let latter = profile.cut(0.5);
    let mut points = latter.control_points().clone();
    points[1].z = -0.1;
    let outside = BSplineCurve::new(latter.knot_vec().clone(), points);
    assert!(!surface.include(&outside));
    assert!(!surface.include(&NurbsCurve::new(lift(&outside))));
    let joined = profile.try_concat(&outside).unwrap();
    assert!(!surface.include(&joined));
    assert!(!surface.include(&NurbsCurve::new(lift(&joined))));
}

#[test]
fn rational_closed_surface_inclusion_follows_each_cusp_branch() {
    let base = profiles().remove(0);
    let mut curve = NurbsCurve::new(BSplineCurve::new(
        base.knot_vec().clone(),
        base.control_points()
            .iter()
            .zip([1., 0.7, 1.4, 1.])
            .map(|(p, w)| p.to_vec().extend(1.) * w)
            .collect(),
    ));
    let top = curve.transformed(Matrix4::from_translation(Vector3::new(2., -1., 10.)));
    let surface = NurbsSurface::new(BSplineSurface::homotopy(
        curve.non_rationalized().clone(),
        top.into_non_rationalized(),
    ));
    let latter = curve.cut(0.5);
    for curve in [curve, latter] {
        for curve in [curve.clone(), curve.inverse()] {
            for surface in [surface.clone(), surface.inverse()] {
                assert!(surface.include(&curve));
                let outside = curve.transformed(Matrix4::from_translation(
                    Vector3::new(0.02, -0.01, 0.1) * -1.,
                ));
                assert!(!surface.include(&outside));
            }
        }
    }
}
