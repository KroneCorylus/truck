use truck_geometry::prelude::*;

fn bezier(values: &[Vector4], t: f64) -> Vector4 {
    let n = values.len() - 1;
    values
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let binomial = match (n, i) {
                (2, 1) => 2.,
                (3, 1 | 2) => 3.,
                _ => 1.,
            };
            *p * (binomial * t.powi(i as i32) * (1. - t).powi((n - i) as i32))
        })
        .sum()
}

fn rational_reference(controls: &[Vector4], t: f64) -> (Point3, Vector3, Vector3) {
    let degree = controls.len() - 1;
    let first: Vec<_> = controls
        .windows(2)
        .map(|p| (p[1] - p[0]) * degree as f64)
        .collect();
    let second: Vec<_> = first
        .windows(2)
        .map(|p| (p[1] - p[0]) * (degree - 1) as f64)
        .collect();
    let (h, d, dd) = (bezier(controls, t), bezier(&first, t), bezier(&second, t));
    let position = h.truncate() / h.w;
    let derivative = (d.truncate() - position * d.w) / h.w;
    let second = (dd.truncate() - position * dd.w - 2. * derivative * d.w) / h.w;
    (Point3::from_vec(position), derivative, second)
}

#[test]
fn curved_polynomial_and_rational_derivatives_survive_chart_changes_and_knot_edits() {
    for degree in [2, 3] {
        for rational in [false, true] {
            for scale in [1., 1e-8] {
                let offset = 0.3 * scale;
                let points = [
                    Point3::new(0., 0., 0.),
                    Point3::new(1., 2., 1.),
                    Point3::new(3., -1., 2.),
                    Point3::new(-1., 4., 3.),
                ][..=degree]
                    .to_vec();
                let weights = [1., 0.7, 1.3, 0.9][..=degree]
                    .iter()
                    .map(|w| if rational { *w } else { 1. })
                    .collect::<Vec<_>>();
                let homogeneous: Vec<_> = points
                    .iter()
                    .zip(&weights)
                    .map(|(p, w)| p.to_homogeneous() * *w)
                    .collect();
                let knots = [vec![offset; degree + 1], vec![offset + scale; degree + 1]].concat();
                if !rational {
                    let polynomial =
                        BSplineCurve::new(KnotVec::from(knots.clone()), points.clone());
                    for x in [0.1, 0.3, 0.8] {
                        let (position, first, second) = rational_reference(&homogeneous, x);
                        assert!(polynomial.subs(offset + scale * x).distance(position) < 1e-8);
                        assert!(
                            (polynomial.der(offset + scale * x) * scale - first).magnitude() < 1e-7
                        );
                        assert!(
                            (polynomial.der2(offset + scale * x) * scale.powi(2) - second)
                                .magnitude()
                                < 1e-6
                        );
                    }
                }
                let mut curve: NurbsCurve<Vector4> = NurbsCurve::try_from_bspline_and_weights(
                    BSplineCurve::new(KnotVec::from(knots), points),
                    weights,
                )
                .unwrap();
                for edited in [false, true] {
                    if edited {
                        curve
                            .add_knot(offset + scale * 0.3)
                            .add_knot(offset + scale * 0.3000001);
                    }
                    for reverse in [false, true] {
                        let mut c = curve.clone();
                        if reverse {
                            c.invert();
                        }
                        for x in [0., 0.1, 0.30000005, 0.5, 0.9, 1.] {
                            let t = offset + scale * x;
                            let parameter = if reverse {
                                1. - (t - offset) / scale
                            } else {
                                (t - offset) / scale
                            };
                            let (position, first, second) =
                                rational_reference(&homogeneous, parameter);
                            assert!(c.subs(t).distance(position) < 1e-8);
                            assert!(
                                (c.der(t) * scale - first * if reverse { -1. } else { 1. })
                                    .magnitude()
                                    < 1e-7
                            );
                            assert!((c.der2(t) * scale.powi(2) - second).magnitude() < 1e-6);
                        }
                    }
                }
                for x in [0.3000001, 0.3] {
                    let knot = offset + scale * x;
                    let index = curve.knot_vec().iter().position(|t| *t == knot).unwrap();
                    curve.try_remove_knot(index).unwrap();
                }
                for x in [0.1, 0.3, 0.8] {
                    let (position, first, second) = rational_reference(&homogeneous, x);
                    assert!(curve.subs(offset + scale * x).distance(position) < 1e-8);
                    assert!((curve.der(offset + scale * x) * scale - first).magnitude() < 1e-7);
                    assert!(
                        (curve.der2(offset + scale * x) * scale.powi(2) - second).magnitude()
                            < 1e-6
                    );
                }
            }
        }
    }
}

#[test]
fn surface_first_second_and_mixed_derivatives_keep_their_polynomial_values() {
    for scale in [1., 1e-8] {
        let knots = KnotVec::from(vec![0., 0., 0., scale, scale, scale]);
        let square = [0., 0., 1.];
        let controls = (0..3)
            .map(|i| {
                (0..3)
                    .map(|j| {
                        let (u, v) = (i as f64 / 2., j as f64 / 2.);
                        Point3::new(square[i] + u * v, square[j] + 2. * u * v, u + v)
                    })
                    .collect()
            })
            .collect();
        let mut s = BSplineSurface::new((knots.clone(), knots), controls);
        s.add_uknot(0.3 * scale).add_uknot(0.3000001 * scale);
        s.add_vknot(0.6 * scale).add_vknot(0.6000001 * scale);
        for u in [0.1, 0.30000005, 0.8] {
            for v in [0.2, 0.60000005, 0.9] {
                let (a, b) = (u * scale, v * scale);
                assert!(
                    s.subs(a, b)
                        .distance(Point3::new(u * u + u * v, v * v + 2. * u * v, u + v))
                        < 1e-8
                );
                assert!(
                    (s.uder(a, b) * scale - Vector3::new(2. * u + v, 2. * v, 1.)).magnitude()
                        < 1e-7
                );
                assert!(
                    (s.vder(a, b) * scale - Vector3::new(u, 2. * v + 2. * u, 1.)).magnitude()
                        < 1e-7
                );
                assert!(
                    (s.uuder(a, b) * scale.powi(2) - Vector3::new(2., 0., 0.)).magnitude() < 1e-6
                );
                assert!(
                    (s.uvder(a, b) * scale.powi(2) - Vector3::new(1., 2., 0.)).magnitude() < 1e-6
                );
                assert!(
                    (s.vvder(a, b) * scale.powi(2) - Vector3::new(0., 2., 0.)).magnitude() < 1e-6
                );
            }
        }
    }
}
