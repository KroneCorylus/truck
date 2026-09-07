//! Hidden-line projection of solids built with `truck-modeling`, checked with the harness.

#[path = "../../truck-shapeops/tests/common/mod.rs"]
mod common;

use common::{
    modeling::{cuboid, cylinder},
    *,
};
use truck_drafting::Curve as Curve2;
use truck_hlr::{project, ProjectedView, Visibility};
use truck_modeling::{
    builder, BoundedCurve, EuclideanSpace, InnerSpace, MetricSpace, ParametricCurve, Plane, Point2,
    Point3, Solid, Vector3, Wire,
};

const TOL: f64 = 0.01;

/// A plane through the origin perpendicular to `direction`, with the viewer on its normal side.
fn view_plane(direction: Vector3) -> Plane {
    let d = direction.normalize();
    let helper = if d.x.abs() < 0.9 {
        Vector3::unit_x()
    } else {
        Vector3::unit_y()
    };
    let u = helper.cross(d).normalize();
    let v = u.cross(d);
    let plane = Plane::new(Point3::origin(), Point3::from_vec(u), Point3::from_vec(v));
    assert!(plane.normal().dot(d) < 0.0);
    plane
}

fn count(view: &ProjectedView, visibility: Visibility) -> usize {
    view.curves.iter().filter(|(_, v)| *v == visibility).count()
}

fn line_ends(curve: &Curve2) -> (Point2, Point2) {
    let Curve2::Line(line) = curve else {
        panic!("projected edge of a box is not a line: {curve:?}");
    };
    (line.0, line.1)
}

/// L-shaped plate: a leg along `y` and an arm along `x`, both of width 1, thickness 1.
fn l_plate() -> Solid {
    let v: Vec<_> = [
        (0.0, 0.0),
        (3.0, 0.0),
        (3.0, 1.0),
        (1.0, 1.0),
        (1.0, 3.0),
        (0.0, 3.0),
    ]
    .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
    .into();
    let wire: Wire = (0..6)
        .map(|i| builder::line(&v[i], &v[(i + 1) % 6]))
        .collect();
    let face = builder::try_attach_plane(&[wire]).unwrap();
    builder::tsweep(&face, Vector3::unit_z())
}

#[test]
fn cube_isometric() {
    let cube = cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
    assert_solid(&cube, 1.0, &[0], TOL);
    let direction = -Vector3::new(1.0, 1.0, 1.0).normalize();
    let view = project(&cube, &view_plane(direction), direction, TOL);
    assert_eq!(view.curves.len(), 12);
    assert_eq!(count(&view, Visibility::Visible), 9);
    assert_eq!(count(&view, Visibility::Hidden), 3);
    for (curve, _) in &view.curves {
        let (p, q) = line_ends(curve);
        assert!((p.distance(q) - (2.0f64 / 3.0).sqrt()).abs() < 1e-10);
    }
}

#[test]
fn cube_along_face_normal() {
    let cube = cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
    assert_solid(&cube, 1.0, &[0], TOL);
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let view = project(&cube, &plane, -Vector3::unit_z(), TOL);
    assert_eq!(
        view.curves.len(),
        8,
        "the four edges along the view are dropped"
    );
    assert_eq!(count(&view, Visibility::Visible), 4);
    assert_eq!(count(&view, Visibility::Hidden), 4);
    for (curve, _) in &view.curves {
        let (p, q) = line_ends(curve);
        assert!(
            (p.distance(q) - 1.0).abs() < 1e-10,
            "edge projected exactly"
        );
        assert!(
            p.x.abs() < 1e-10
                || (p.x - 1.0).abs() < 1e-10
                || p.y.abs() < 1e-10
                || (p.y - 1.0).abs() < 1e-10
        );
    }
}

/// Seen from below the arm, the top edge of the inner face of the leg passes behind the arm for
/// its first half and is split where the arm's bottom edge crosses it.
#[test]
fn l_plate_edge_split_at_occlusion() {
    let plate = l_plate();
    assert_solid(&plate, 5.0, &[0], TOL);
    assert_counts(&plate, 12, 18, 8);
    let direction = -Vector3::new(1.0, -1.0, -1.0).normalize();
    let plane = view_plane(direction);
    let view = project(&plate, &plane, direction, TOL);

    let projected = |x, y, z| {
        let param = plane.get_parameter(Point3::new(x, y, z));
        Point2::new(param.x, param.y)
    };
    let inner_start = projected(1.0, 1.0, 1.0);
    let crossing = projected(1.0, 2.0, 1.0);
    let inner_end = projected(1.0, 3.0, 1.0);
    let piece = |from: Point2, to: Point2| {
        view.curves.iter().find(|(curve, _)| {
            let (p, q) = line_ends(curve);
            (p.distance(from) < TOL && q.distance(to) < TOL)
                || (p.distance(to) < TOL && q.distance(from) < TOL)
        })
    };
    let (_, hidden) = piece(inner_start, crossing).expect("hidden piece of the inner edge");
    let (_, visible) = piece(crossing, inner_end).expect("visible piece of the inner edge");
    assert_eq!(*hidden, Visibility::Hidden);
    assert_eq!(*visible, Visibility::Visible);
    assert!(
        piece(inner_start, inner_end).is_none(),
        "the inner edge is split"
    );
}

/// Along its axis the seam lines of a cylinder project to points and its end circles onto one
/// another, exactly. The far circle lies on the silhouette of the wall; whether it is hidden is
/// the tie rule of the silhouette step, not asserted here.
#[test]
fn cylinder_along_axis_projects_exact_arcs() {
    let cylinder = cylinder(Point3::origin(), Vector3::unit_z(), 1.0, 2.0);
    assert_solid(&cylinder, cylinder_volume(1.0, 2.0), &[0], TOL);
    assert_counts(&cylinder, 4, 6, 4);
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let view = project(&cylinder, &plane, -Vector3::unit_z(), TOL);
    assert_eq!(view.curves.len(), 4, "four half circles, two seams dropped");
    assert!(
        count(&view, Visibility::Visible) >= 2,
        "the near circle is visible"
    );
    for (curve, _) in &view.curves {
        let Curve2::CircleArc(arc) = curve else {
            panic!("projected circle is not an arc: {curve:?}");
        };
        let (t0, t1) = arc.range_tuple();
        assert!((t1 - t0 - std::f64::consts::PI).abs() < 1e-10);
        for i in 0..=10 {
            let t = t0 + (t1 - t0) * i as f64 / 10.0;
            assert!((arc.subs(t).to_vec().magnitude() - 1.0).abs() < 1e-10);
        }
    }
}

/// The edges a boolean union makes are intersection curves, projected by sampling and fitting.
/// Between boxes they are straight, so their projections must be straight within `tol`.
#[test]
fn intersection_curves_fitted_within_tol() {
    let a = cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
    let b = cuboid(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5));
    let union = truck_shapeops::or(&a, &b, TOL).unwrap();
    assert_solid(&union, 1.875, &[0], TOL);
    let direction = -Vector3::new(1.0, 1.0, 1.0).normalize();
    let view = project(&union, &view_plane(direction), direction, TOL);
    let mut fitted = 0;
    for (curve, _) in &view.curves {
        let Curve2::BSplineCurve(spline) = curve else {
            assert!(
                matches!(curve, Curve2::Line(_)),
                "unexpected kind: {curve:?}"
            );
            continue;
        };
        fitted += 1;
        let (p, q) = (spline.front(), spline.back());
        let (t0, t1) = spline.range_tuple();
        assert!(p.distance(q) > TOL, "degenerate piece");
        for i in 0..=20 {
            let t = t0 + (t1 - t0) * i as f64 / 20.0;
            let sag = (spline.subs(t) - p).perp_dot((q - p).normalize()).abs();
            assert!(sag < TOL, "projected intersection curve bends by {sag}");
        }
    }
    assert!(fitted > 0, "the union has intersection-curve edges");
}

#[test]
fn deterministic() {
    let plate = l_plate();
    let direction = -Vector3::new(1.0, -1.0, -1.0).normalize();
    let plane = view_plane(direction);
    let first = project(&plate, &plane, direction, TOL);
    let second = project(&plate, &plane, direction, TOL);
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
}
