use truck_geometry::prelude::*;

fn clustered(scale: f64, offset: f64) -> BSplineCurve<Point3> {
    let knots = [0., 0., 0., 0.5, 0.5, 0.5000001, 0.5000001, 1., 1., 1.]
        .into_iter()
        .map(|t| offset + scale * t)
        .collect();
    BSplineCurve::new(
        knots,
        (0..7).map(|j| Point3::new(0.3, j as f64, 2.)).collect(),
    )
}

#[test]
fn nearby_distinct_knots_are_not_repetitions() {
    for scale in [1., 1e-8, 1e4] {
        let c = clustered(scale, 0.3 * scale);
        let k = c.knot_vec();
        assert_eq!(k.multiplicity(3), 2);
        assert_eq!(k.multiplicity(5), 2);
        let (_, counts) = k.to_single_multi();
        assert_eq!(counts, vec![3, 2, 2, 3]);
    }
}

fn check_piecewise_line(c: &BSplineCurve<Point3>, scale: f64, offset: f64, a: f64, b: f64) {
    let breaks = [
        offset,
        offset + scale * 0.5,
        offset + scale * 0.5000001,
        offset + scale,
    ];
    for segment in 0..3 {
        let lo = breaks[segment];
        let hi = breaks[segment + 1];
        for fraction in [0.125, 0.25, 0.5, 0.75, 0.875] {
            let t = lo + fraction * (hi - lo);
            if t < a || t > b {
                continue;
            }
            let local = (t - lo) / (hi - lo);
            let expected = Point3::new(0.3, 2. * (segment as f64 + local), 2.);
            assert!(c.subs(t).distance(expected) < 1e-8);
            let speed = 2. / (hi - lo);
            assert!((c.der(t) / speed - Vector3::unit_y()).magnitude() < 1e-8);
            assert!((c.der2(t) * (hi - lo).powi(2)).magnitude() < 1e-8);
        }
    }
}

#[test]
fn clustered_spans_keep_geometry_and_derivatives_across_affine_charts() {
    for scale in [1., 1e-8, 1e4] {
        let offset = 0.3 * scale;
        let c = clustered(scale, offset);
        check_piecewise_line(&c, scale, offset, offset, offset + scale);
        assert!(c.front().distance(Point3::new(0.3, 0., 2.)) < 1e-12);
        assert!(c.back().distance(Point3::new(0.3, 6., 2.)) < 1e-12);
    }
}

#[test]
fn exact_cut_preserves_nearby_knot_spans_and_trim_endpoints() {
    for scale in [1., 1e-8, 1e4] {
        let offset = 0.3 * scale;
        for end in [0.50000005, 0.5000001, 0.50000015] {
            let mut c = clustered(scale, offset);
            let (a, b) = (offset + scale * 0.1, offset + scale * end);
            let expected_front = c.subs(a);
            let expected_back = c.subs(b);
            c = c.cut(a);
            c.cut(b);
            assert_eq!(c.range_tuple(), (a, b));
            assert_eq!(c.degree(), 2);
            assert_eq!(c.knot_vec().multiplicity(0), 3);
            assert_eq!(c.knot_vec().multiplicity(c.knot_vec().len() - 1), 3);
            assert!(c.front().distance(expected_front) < 1e-8);
            assert!(c.back().distance(expected_back) < 1e-8);
            check_piecewise_line(&c, scale, offset, a, b);
        }
    }
}

#[test]
fn normalization_preserves_distinct_knots_and_exact_multiplicities() {
    for scale in [1., 1e-8, 1e4] {
        let mut knots = clustered(scale, 0.3 * scale).knot_vec().clone();
        knots.try_normalize().unwrap();
        let (single, counts) = knots.to_single_multi();
        assert_eq!(counts, vec![3, 2, 2, 3]);
        assert_eq!(single.len(), 4);
        assert!(single[1] < single[2]);
        assert!((single[1] - 0.5).abs() < 1e-14);
        assert!((single[2] - 0.5000001).abs() < 1e-14);
    }
}

#[test]
fn both_surface_cut_directions_preserve_degree_and_clustered_spans() {
    for swap in [false, true] {
        let curve = clustered(1., 0.);
        let points = (0..7)
            .map(|i| vec![Point3::new(i as f64, 0., 2.), Point3::new(i as f64, 1., 2.)])
            .collect();
        let mut original =
            BSplineSurface::new((curve.knot_vec().clone(), KnotVec::bezier_knot(1)), points);
        if swap {
            original.swap_axes();
        }
        for cut in [0.50000005, 0.5000001, 0.50000015] {
            let mut left = original.clone();
            let right = if swap { left.vcut(cut) } else { left.ucut(cut) };
            for part in [&left, &right] {
                assert_eq!(
                    (part.udegree(), part.vdegree()),
                    if swap { (1, 2) } else { (2, 1) }
                );
                let knots = if swap {
                    part.vknot_vec()
                } else {
                    part.uknot_vec()
                };
                assert_eq!(knots.multiplicity(0), 3);
                assert_eq!(knots.multiplicity(knots.len() - 1), 3);
                let (a, b) = (knots[0], knots[knots.len() - 1]);
                assert!(a == cut || b == cut);
                for t in [a, a + (b - a) * 0.25, a + (b - a) * 0.75, b] {
                    let (u, v) = if swap { (0.3, t) } else { (t, 0.3) };
                    assert!(part.subs(u, v).distance(original.subs(u, v)) < 1e-8);
                }
            }
        }
    }
}
