use std::f64::consts::FRAC_PI_2;
use truck_geometry::prelude::*;

fn check<S>(surface: &S)
where S: ParametricSurface3D
        + BoundedSurface
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3> {
    let (ur, vr) = surface.range_tuple();
    for (a, b) in [
        (0.0, 0.0),
        (1.0, 1.0),
        (1e-7, 0.9),
        (0.4, 1e-7),
        (0.371, 0.613),
    ] {
        let u = ur.0 + a * (ur.1 - ur.0);
        let v = vr.0 + b * (vr.1 - vr.0);
        let p = surface.subs(u, v);
        for hint in [
            SPHint2D::None,
            SPHint2D::Range(ur, vr),
            SPHint2D::Parameter(u, v),
            SPHint2D::Range(
                (u.max(ur.0), (u + 0.01).min(ur.1)),
                (v.max(vr.0), (v + 0.01).min(vr.1)),
            ),
        ] {
            let (x, y) = surface.search_parameter(p, hint, 100).unwrap();
            assert_near!(surface.subs(x, y), p);
            let q = p + surface.normal(u, v) * 0.001;
            let (x, y) = surface.search_nearest_parameter(q, hint, 100).unwrap();
            assert_near!(surface.subs(x, y), p);
            assert!(surface.search_parameter(q, hint, 100).is_none());
        }
    }
    let p = surface.subs((ur.0 + ur.1) / 2.0, (vr.0 + vr.1) / 2.0);
    let hint = (ur.0, vr.0);
    assert_eq!(
        surface.search_parameter(p, hint, 0),
        algo::surface::search_parameter(surface, p, hint, 0)
    );
    assert_eq!(
        surface.search_nearest_parameter(p, hint, 0),
        algo::surface::search_nearest_parameter(surface, p, hint, 0)
    );
}

fn rational_extrusion() -> NurbsSurface<Vector4> {
    let w = 0.5_f64.sqrt();
    let points = [
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(w, w, 0.0, w),
        Vector4::new(0.0, 1.0, 0.0, 1.0),
    ];
    let vector = Vector4::new(0.4, -0.2, 3.0, 0.0);
    NurbsSurface::new(BSplineSurface::new(
        (
            KnotVec::bezier_knot(2),
            KnotVec::from(vec![2.0, 2.0, 5.0, 5.0]),
        ),
        points
            .into_iter()
            .map(|p| vec![p, p + vector * p.w])
            .collect(),
    ))
}

#[test]
fn rational_circles_ellipses_and_swapped_parameters() {
    let circle = rational_extrusion();
    let before = circle.clone();
    check(&circle);
    check(&circle.inverse());
    let ellipse = circle.transformed(Matrix4::from_nonuniform_scale(2.0, 0.7, 1.0));
    check(&ellipse);
    check(&ellipse.inverse());
    assert_eq!(circle, before);
}

#[test]
fn native_conics_with_oblique_positive_and_negative_extrusion() {
    for scale in [1.0, 2.0] {
        let arc = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, FRAC_PI_2)),
            Matrix4::from_nonuniform_scale(scale, 1.0, 1.0),
        );
        for direction in [-1.0, 1.0] {
            let s = ExtrudedCurve::by_extrusion(arc, Vector3::new(0.4, -0.2, 3.0) * direction);
            check(&s);
            check(&s.inverse());
        }
    }
}

#[test]
fn bspline_ruled_and_general_surfaces() {
    let rational = rational_extrusion();
    let base = rational.non_rationalized();
    let s = BSplineSurface::new(
        base.knot_vecs().clone(),
        base.control_points()
            .iter()
            .map(|row| row.iter().map(|p| p.to_point()).collect())
            .collect(),
    );
    check(&s);
    check(&s.inverse());
    let general = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        (0..3)
            .map(|i| {
                (0..3)
                    .map(|j| Point3::new(i as f64, j as f64, (i * j) as f64 * 0.1))
                    .collect()
            })
            .collect(),
    );
    check(&general);
}

#[test]
fn unequal_ruling_weights_use_grid_seed() {
    let s = rational_extrusion();
    let mut points = s.non_rationalized().control_points().clone();
    for row in &mut points {
        row[1] *= 2.0;
    }
    let s = NurbsSurface::new(BSplineSurface::new(
        s.non_rationalized().knot_vecs().clone(),
        points,
    ));
    check(&s);
    check(&s.inverse());
    let p = s.subs(0.37, 3.9) + Vector3::new(0.1, 0.0, 0.0);
    let seed = algo::surface::presearch(&s, p, s.range_tuple(), 50);
    assert_eq!(
        s.search_nearest_parameter(p, None, 3),
        algo::surface::search_nearest_parameter(&s, p, seed, 3)
    );
}

#[test]
fn degenerate_rulings_and_ranges_fall_back() {
    let s = ExtrudedCurve::by_extrusion(
        Line(Point3::origin(), Point3::new(1.0, 0.0, 0.0)),
        Vector3::zero(),
    );
    let p = Point3::new(0.3, 0.1, 0.0);
    for range in [
        s.range_tuple(),
        ((0.0, 1.0), (0.5, 0.5)),
        ((0.0, f64::INFINITY), (0.0, 1.0)),
    ] {
        assert_eq!(
            algo::surface::presearch_linear(&s, p, range, 50, algo::surface::LinearAxis::V),
            algo::surface::presearch(&s, p, range, 50)
        );
    }
}

#[test]
fn nonpositive_rational_weights_use_grid_seed() {
    let base = rational_extrusion();
    for weight in [-1.0, 0.0] {
        let mut points = base.non_rationalized().control_points().clone();
        points[1][0] *= weight;
        points[1][1] *= weight;
        let s = NurbsSurface::new(BSplineSurface::new(
            base.non_rationalized().knot_vecs().clone(),
            points,
        ));
        let p = s.subs(0.13, 3.9);
        let seed = algo::surface::presearch(&s, p, s.range_tuple(), 50);
        assert_eq!(
            s.search_parameter(p, None, 3),
            algo::surface::search_parameter(&s, p, seed, 3)
        );
        assert_eq!(
            s.search_nearest_parameter(p, None, 3),
            algo::surface::search_nearest_parameter(&s, p, seed, 3)
        );
    }
}
