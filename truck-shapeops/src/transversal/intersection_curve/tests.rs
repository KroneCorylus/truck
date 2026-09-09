use super::*;
use std::f64::consts::PI;

#[test]
fn intersection_curve_sphere_case() {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
    const M: usize = 5;
    let polyline = (0..=M)
        .map(|i| {
            let t = 2.0 * PI * i as f64 / M as f64;
            Point3::new(0.8 * f64::cos(t), 0.8 * f64::sin(t), 0.0)
        })
        .collect::<PolylineCurve<_>>();
    let curve: IntersectionCurve<_, _, _> =
        IntersectionCurveWithParameters::try_new(sphere0, sphere1, polyline, 0.3)
            .unwrap()
            .into();

    const N: usize = 100;
    let mut sum = 0.0;
    let (t0, t1) = curve.range_tuple();
    for i in 0..=N {
        let t = t0 + (t1 - t0) * i as f64 / N as f64;
        let pt = curve.subs(t);
        assert_near!(pt.distance(Point3::origin()), 1.0);
        let vec = curve.der(t);
        assert!(pt.dot(vec).so_small(), "{i} {t} {vec:?}");
        assert!(vec[2].so_small());
        let denom = if matches!(i, 0 | N) { 2.0 } else { 1.0 };
        sum += vec.magnitude() / denom * (t1 - t0) / N as f64;
    }
    assert!(
        f64::abs(sum - 2.0 * PI) < 0.01,
        "res: {}\nans: {}",
        sum,
        2.0 * PI
    );

    let theta = 2.0 * PI * rand::random::<f64>();
    let pt = Point3::new(f64::cos(theta), f64::sin(theta), 0.0);
    let t = curve.search_parameter(pt, None, 10).unwrap();
    assert_near!(curve.subs(t), pt);
    let pt = Point3::new(1.1 * f64::cos(theta), 1.1 * f64::sin(theta), 0.0);
    assert!(curve.search_parameter(pt, None, 10).is_none());
    let t = curve.search_nearest_parameter(pt, None, 10).unwrap();
    assert_near!(curve.subs(t).distance(pt), 0.1);

    let mut curve0 = curve.clone();
    let curve1 = curve0.cut(2.5);
    assert_near!(curve0.front(), curve.front());
    assert_near!(curve0.back(), curve.subs(2.5));
    assert_near!(curve1.front(), curve.subs(2.5));
    assert_near!(curve1.back(), curve.back());
    let mut curve0 = curve.clone();
    let curve1 = curve0.cut(2.0);
    assert_near!(curve0.front(), curve.front());
    assert_near!(curve0.back(), curve.subs(2.0));
    assert_near!(curve1.front(), curve.subs(2.0));
    assert_near!(curve1.back(), curve.back());
}

#[test]
fn collide_parabola() {
    const TOL: f64 = 0.05;

    // define surfaces
    #[rustfmt::skip]
	let ctrl0 = vec![
		vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
		vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
		vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
	];
    #[rustfmt::skip]
	let ctrl1 = vec![
		vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
		vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
		vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
	];
    let surface0 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0);
    let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1);

    // meshing surface
    let instant = std::time::Instant::now();
    let polygon0 = StructuredMesh::from_surface(&surface0, surface0.range_tuple(), TOL).destruct();
    let polygon1 = StructuredMesh::from_surface(&surface1, surface1.range_tuple(), TOL).destruct();
    println!("Meshing Surfaces: {}s", instant.elapsed().as_secs_f64());
    // extract intersection curves
    let instant = std::time::Instant::now();
    let curves = intersection_curves(surface0, &polygon0, surface1, &polygon1, TOL).unwrap();
    println!(
        "Extracting Intersection: {}s",
        instant.elapsed().as_secs_f64()
    );
    assert_eq!(curves.len(), 1);
    let curve = curves[0].1.clone();
    const N: usize = 100;
    for i in 0..N {
        let t1 = curve.range_tuple().1;
        let t = t1 * i as f64 / N as f64;
        let pt = curve.subs(t);
        assert_near!(pt.distance(Point3::origin()) * 0.5, f64::sqrt(0.5) * 0.5);
    }
}

/// Quadratic height graph a x^2 + 2 b xy + c y^2, with unequal parameter speeds.
fn quadratic_graph(a: f64, b: f64, c: f64) -> BSplineSurface<Point3> {
    let linear = [-1.0, 0.0, 1.0];
    let square = [1.0, -1.0, 1.0];
    let points = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| {
                    Point3::new(
                        2.0 * linear[i],
                        3.0 * linear[j],
                        4.0 * a * square[i]
                            + 12.0 * b * linear[i] * linear[j]
                            + 9.0 * c * square[j],
                    )
                })
                .collect()
        })
        .collect();
    BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), points)
}

#[test]
fn tangent_curvature_classification() {
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    for (a, b, c, crossing) in [
        (1.0, 0.0, -1.0, true),
        (0.0, 1.0, 0.0, true),
        (1.0, 0.0, 1.0, false),
        (-1.0, 0.0, -1.0, false),
        (1.0, 0.0, 0.0, false),
        (0.0, 0.0, 0.0, false),
    ] {
        let graph = quadratic_graph(a, b, c);
        for graph in [graph.clone(), graph.inverse()] {
            for plane in [plane, plane.inverse()] {
                assert_eq!(
                    tangent_crossing_at(&graph, &plane, Point3::origin()),
                    Some(crossing)
                );
                assert_eq!(
                    tangent_crossing_at(&plane, &graph, Point3::origin()),
                    Some(crossing)
                );
            }
        }
    }
    let upper = Sphere::new(Point3::new(1.0, 0.0, 0.0), 1.0);
    let lower = Sphere::new(Point3::new(-1.0, 0.0, 0.0), 1.0);
    assert_eq!(
        tangent_crossing_at(&upper, &lower, Point3::origin()),
        Some(false)
    );
    let offset = Plane::new(
        Point3::new(0.0, 0.0, 0.1),
        Point3::new(1.0, 0.0, 0.1),
        Point3::new(0.0, 1.0, 0.1),
    );
    assert_eq!(
        tangent_crossing_at(&quadratic_graph(1.0, 0.0, -1.0), &offset, Point3::origin()),
        Some(false)
    );
}

#[test]
fn tangent_contact_refines_a_mesh_seed() {
    let graph = quadratic_graph(1.0, 0.0, -1.0);
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let contact = tangent_contact(&graph, &plane, Point3::new(0.01, -0.02, 0.003)).unwrap();
    assert_near!(contact, Point3::origin());
    assert_eq!(tangent_crossing_at(&graph, &plane, contact), Some(true));
}

/// Seeding each sample with the previous one must not change what the samples mean. A seeded
/// solve can stall on a singular Jacobian and report a step it never took, and near a periodic
/// seam it can continue past the end of the range; the point still looks right in both cases,
/// so only the parameters expose it. Face division cuts with these parameters, so each one has
/// to name the point the curve reports. The seam sweep below crosses the sphere seam at four
/// different offsets.
#[test]
fn samples_crossing_a_periodic_seam_still_name_their_points() {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
    const M: usize = 24;
    for offset in [0.0, -0.5, 0.5, 2.0] {
        let polyline = (0..=M)
            .map(|i| {
                let t = offset + 2.0 * PI * i as f64 / M as f64;
                Point3::new(0.8 * f64::cos(t), 0.8 * f64::sin(t), 0.0)
            })
            .collect::<PolylineCurve<_>>();
        let curve = IntersectionCurveWithParameters::try_new(sphere0, sphere1, polyline, 0.3)
            .unwrap_or_else(|| panic!("offset {offset}"));
        let leader = curve.ic.leader();
        for (i, p) in curve.params0.iter().enumerate() {
            assert_near!(sphere0.subs(p.x, p.y), leader[i]);
        }
        for (i, p) in curve.params1.iter().enumerate() {
            assert_near!(sphere1.subs(p.x, p.y), leader[i]);
        }
        // Every sample lies on the circle the two spheres actually meet in.
        leader
            .iter()
            .for_each(|p| assert_near!(p.to_vec().magnitude(), 1.0));
    }
}
