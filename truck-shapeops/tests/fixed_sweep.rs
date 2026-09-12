mod common;
use common::{blend::from_modeling, *};
use truck_modeling::*;
fn profile() -> Face {
    let points = [
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
    ]
    .map(builder::vertex);
    let wire: Wire = (0..4)
        .map(|i| builder::line(&points[i], &points[(i + 1) % 4]))
        .collect();
    builder::try_attach_plane(vec![wire]).unwrap()
}
#[test]
fn fixed_normal_spline_sweep_keeps_sections_and_exact_volume() {
    let a = builder::vertex(Point3::origin());
    let b = builder::vertex(Point3::new(2.0, 1.0, 8.0));
    let curve = Curve::BSplineCurve(BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            a.point(),
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(4.0, 1.0, 6.0),
            b.point(),
        ],
    ));
    let path: Wire = vec![truck_modeling::Edge::new(&a, &b, curve.clone())].into();
    let face = profile();
    let solid = builder::sweep_along_wire_fixed(&face, &path, 0.001).unwrap();
    assert_solid(&from_modeling(&solid), 32.0, &[0], 0.001);
    for surface in solid.boundaries()[0].iter().take(4).map(|f| f.surface()) {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let p = surface.subs(0.5, t) - curve.subs(t).to_vec();
            assert!(p.z.abs() < 1e-9);
            assert!((p.x.abs() - 1.0).abs() < 1e-9 || (p.y.abs() - 1.0).abs() < 1e-9);
        }
    }
}
#[test]
fn fixed_sweep_refuses_tangent_folds_and_invalid_tolerance_without_mutating() {
    let face = profile();
    let before = serde_json::to_string(&Shell::from(vec![face.clone()]).compress()).unwrap();
    let a = builder::vertex(Point3::origin());
    let b = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let path: Wire = vec![builder::line(&a, &b)].into();
    assert!(builder::sweep_along_wire_fixed(&face, &path, 0.001).is_err());
    for tol in [0.0, -1.0, f64::NAN] {
        assert!(builder::sweep_along_wire_fixed(&face, &path, tol).is_err());
    }
    assert_eq!(
        serde_json::to_string(&Shell::from(vec![face]).compress()).unwrap(),
        before
    );
}
