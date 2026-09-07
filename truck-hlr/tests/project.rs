//! Hidden-line projection of solids built with `truck-modeling`, checked with the harness.

#[path = "../../truck-shapeops/tests/common/mod.rs"]
mod common;

use common::{
    modeling::{cuboid, cylinder, sphere},
    *,
};
use std::f64::consts::PI;
use truck_drafting::Curve as Curve2;
use truck_hlr::{project, ProjectedView, Visibility};
use truck_modeling::{
    builder, BoundedCurve, EuclideanSpace, InnerSpace, MetricSpace, ParametricCurve, Plane, Point2,
    Point3, Rad, Solid, Vector2, Vector3, Wire,
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
/// another, exactly. The wall has no silhouette, its normal being perpendicular to the view
/// everywhere, and the far circle, lying on that wall, is hidden behind the near one.
/// At `tol = 0.03` the old mesh-only test missed the far circle because of chord sag.
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
    assert_eq!(count(&view, Visibility::Visible), 2, "the near circle");
    assert_eq!(count(&view, Visibility::Hidden), 2, "the far circle");
    for tol in [0.001, 0.03] {
        let view = project(&cylinder, &plane, -Vector3::unit_z(), tol);
        assert_eq!(view.curves.len(), 4);
        assert_eq!(count(&view, Visibility::Visible), 2, "near circle at {tol}");
        assert_eq!(count(&view, Visibility::Hidden), 2, "far circle at {tol}");
    }
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

/// Across its axis a cylinder shows its two rulings as silhouette lines and both end circles
/// edge-on, all visible; only the seam on the far side is hidden.
#[test]
fn cylinder_across_axis_silhouette_lines() {
    let cylinder = cylinder(Point3::origin(), Vector3::unit_z(), 1.0, 2.0);
    assert_solid(&cylinder, cylinder_volume(1.0, 2.0), &[0], TOL);
    let direction = -Vector3::unit_y();
    let plane = view_plane(direction);
    let view = project(&cylinder, &plane, direction, TOL);
    let projected = |p: Point3| {
        let param = plane.get_parameter(p);
        Point2::new(param.x, param.y)
    };
    assert_eq!(
        view.curves.len(),
        8,
        "four half circles, two seams, two rulings"
    );
    assert_eq!(count(&view, Visibility::Hidden), 1, "the far seam");
    for x in [-1.0, 1.0] {
        let from = projected(Point3::new(x, 0.0, 0.0));
        let to = projected(Point3::new(x, 0.0, 2.0));
        let ruling = view.curves.iter().find(|(curve, _)| {
            let Curve2::Line(line) = curve else {
                return false;
            };
            (line.0.distance(from) < TOL && line.1.distance(to) < TOL)
                || (line.0.distance(to) < TOL && line.1.distance(from) < TOL)
        });
        let (_, visibility) = ruling.expect("silhouette ruling");
        assert_eq!(*visibility, Visibility::Visible);
    }
    let ends: Vec<(Point2, Vector2)> = [0.0, 2.0]
        .map(|z| {
            let a = projected(Point3::new(-1.0, 0.0, z));
            (a, (projected(Point3::new(1.0, 0.0, z)) - a).normalize())
        })
        .into();
    let mut arcs = 0;
    for (curve, visibility) in &view.curves {
        let Curve2::CircleArc(arc) = curve else {
            continue;
        };
        arcs += 1;
        assert_eq!(*visibility, Visibility::Visible, "edge-on end circle");
        let (t0, t1) = arc.range_tuple();
        for i in 0..=10 {
            let p = arc.subs(t0 + (t1 - t0) * i as f64 / 10.0);
            let off = ends.iter().map(|(a, dir)| (p - a).perp_dot(*dir).abs());
            assert!(off.fold(f64::MAX, f64::min) < 1e-9, "arc off the end plane");
        }
    }
    assert_eq!(arcs, 4);
}

/// The two outer silhouette rulings of a bored cone are exact lines, trimmed to its end caps.
#[test]
fn cone_silhouette_rulings() {
    let vertices = [(0.25, 0.0), (0.25, 2.0), (0.5, 2.0), (1.0, 0.0)]
        .map(|(r, z)| builder::vertex(Point3::new(0.0, r, z)));
    let wire: Wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let profile = builder::try_attach_plane(&[wire]).unwrap();
    let cone = builder::rsweep(&profile, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    assert_solid(&cone, PI * 2.0 * (1.75 / 3.0 - 0.25 * 0.25), &[1], TOL);
    let direction = -Vector3::unit_y();
    let plane = view_plane(direction);
    let view = project(&cone, &plane, direction, TOL);
    for sign in [-1.0, 1.0] {
        let from = Point2::from_vec(plane.get_parameter(Point3::new(sign, 0.0, 0.0)).truncate());
        let to = Point2::from_vec(
            plane
                .get_parameter(Point3::new(sign * 0.5, 0.0, 2.0))
                .truncate(),
        );
        let rulings: Vec<_> = view
            .curves
            .iter()
            .filter(|(curve, _)| {
                let Curve2::Line(line) = curve else {
                    return false;
                };
                (line.0.distance(from) < 1e-9 && line.1.distance(to) < 1e-9)
                    || (line.0.distance(to) < 1e-9 && line.1.distance(from) < 1e-9)
            })
            .collect();
        assert_eq!(rulings.len(), 1, "one exact outer ruling per side");
        assert_eq!(rulings[0].1, Visibility::Visible);
    }
}

/// Seen along its axis the equators of a torus are silhouettes, not edges: traced on each face
/// they cross and fitted, the pieces must lie on the equators, close up end to end, and be
/// visible, the inner one being the outline of the hole.
#[test]
fn torus_along_axis_contour_closed() {
    let (major, minor) = (2.0, 0.5);
    let v = builder::vertex(Point3::new(major, 0.0, minor));
    let profile: Wire = builder::rsweep(
        &v,
        Point3::new(major, 0.0, 0.0),
        Vector3::unit_y(),
        Rad(7.0),
        2,
    );
    let shell = builder::rsweep(&profile, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    let torus = Solid::new(vec![shell]);
    assert_solid(&torus, 2.0 * PI * PI * major * minor * minor, &[1], TOL);
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let view = project(&torus, &plane, -Vector3::unit_z(), TOL);
    let repeated = project(&torus, &plane, -Vector3::unit_z(), TOL);
    assert_eq!(format!("{view:?}"), format!("{repeated:?}"));
    let contours: Vec<_> = view
        .curves
        .iter()
        .filter_map(|(curve, visibility)| match curve {
            Curve2::BSplineCurve(spline) => Some((spline, *visibility)),
            _ => None,
        })
        .collect();
    assert_eq!(contours.len(), 4, "two halves of each equator");
    let mut ends = Vec::new();
    let mut outer = 0;
    for (spline, visibility) in &contours {
        assert_eq!(*visibility, Visibility::Visible);
        let radius = spline.front().to_vec().magnitude();
        assert!(
            (radius - (major + minor)).abs() < TOL || (radius - (major - minor)).abs() < TOL,
            "contour off the equators: radius {radius}"
        );
        outer += usize::from((radius - (major + minor)).abs() < TOL);
        let (t0, t1) = spline.range_tuple();
        for i in 0..=20 {
            let p = spline.subs(t0 + (t1 - t0) * i as f64 / 20.0);
            assert!((p.to_vec().magnitude() - radius).abs() < TOL);
        }
        ends.push(spline.front());
        ends.push(spline.back());
    }
    assert_eq!(outer, 2, "two outer halves and two inner halves");
    for end in &ends {
        let shared = ends
            .iter()
            .filter(|other| other.distance(*end) < TOL)
            .count();
        assert_eq!(shared, 2, "each end is shared by exactly two pieces");
    }
}

/// The silhouette of a sphere is a great circle, exact, in one arc per face it crosses, and
/// the arcs cover the circle once.
#[test]
fn sphere_silhouette_exact_arcs() {
    let sphere = sphere(Point3::origin(), 1.0);
    assert_solid(&sphere, sphere_volume(1.0), &[0], TOL);
    let direction = -Vector3::new(1.0, 0.0, 1.0).normalize();
    let view = project(&sphere, &view_plane(direction), direction, TOL);
    let (mut arcs, mut angle) = (0, 0.0);
    for (curve, visibility) in &view.curves {
        let Curve2::CircleArc(arc) = curve else {
            continue;
        };
        let (t0, t1) = arc.range_tuple();
        let on_outline = (0..=10).all(|i| {
            let p = arc.subs(t0 + (t1 - t0) * i as f64 / 10.0);
            (p.to_vec().magnitude() - 1.0).abs() < 1e-6
        });
        if on_outline {
            arcs += 1;
            angle += t1 - t0;
            assert_eq!(*visibility, Visibility::Visible);
        }
    }
    assert_eq!(arcs, 6, "three faces on each side");
    assert!(
        (angle - 2.0 * PI).abs() < 1e-5,
        "outline covered once, got {angle}"
    );
    let scaled = project(&sphere, &view_plane(direction), direction / 4096.0, TOL);
    assert_eq!(
        view.curves.iter().map(|(_, v)| v).collect::<Vec<_>>(),
        scaled.curves.iter().map(|(_, v)| v).collect::<Vec<_>>(),
        "visibility depends on the direction, not its magnitude"
    );
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
