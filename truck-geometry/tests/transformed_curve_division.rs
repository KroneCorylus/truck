use std::f64::consts::TAU;
use truck_geometry::prelude::*;

fn verify<C, P>(curve: &C, tol: f64)
where
    C: ParametricCurve<Point = P> + BoundedCurve + ParameterDivision1D<Point = P>,
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + Copy,
    P::Diff: InnerSpace<Scalar = f64>, {
    let (a, b) = curve.range_tuple();
    let (parameters, points) = curve.parameter_division((a, b), tol);
    assert_eq!(parameters.len(), points.len());
    assert_eq!(parameters.first().copied(), Some(a));
    assert_eq!(parameters.last().copied(), Some(b));
    for (span, chord) in parameters.windows(2).zip(points.windows(2)) {
        assert!(span[0] < span[1]);
        let delta = chord[1] - chord[0];
        for i in 0..=16 {
            let p = curve.subs(span[0] + (span[1] - span[0]) * i as f64 / 16.);
            let u = if delta.magnitude2() > 0. {
                ((p - chord[0]).dot(delta) / delta.magnitude2()).clamp(0., 1.)
            } else {
                0.
            };
            assert!(
                p.distance(chord[0] + delta * u) <= tol * 1.001,
                "world chord error {} exceeds {tol}",
                p.distance(chord[0] + delta * u)
            );
        }
    }
}

#[test]
fn scaled_parabola_division_keeps_world_tolerance_without_panicking() {
    for reversed in [false, true] {
        let mut curve = Processor::with_transform(
            TrimmedCurve::new(UnitParabola::<Point3>::new(), (-1., 1.)),
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(1000.),
        );
        if reversed {
            curve.invert();
        }
        verify(&curve, 0.00001);
    }
}

#[test]
fn sheared_and_flattened_circles_keep_world_chord_accuracy() {
    let mut shear = Matrix4::identity();
    shear.y.x = 40.;
    for transform in [shear, Matrix4::from_nonuniform_scale(2., 3., 0.)] {
        for reversed in [false, true] {
            let mut curve = Processor::with_transform(
                TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., TAU)),
                transform,
            );
            if reversed {
                curve.invert();
            }
            verify(&curve, 0.001);
        }
    }
}

#[test]
fn two_dimensional_curves_preserve_scaled_and_sheared_accuracy() {
    for reversed in [false, true] {
        let mut parabola = Processor::with_transform(
            TrimmedCurve::new(UnitParabola::<Point2>::new(), (-1., 1.)),
            Matrix3::from_angle_z(Rad(0.73)) * Matrix3::from_nonuniform_scale(1000., 1000.),
        );
        if reversed {
            parabola.invert();
        }
        verify(&parabola, 0.00001);
        let mut shear = Matrix3::identity();
        shear.y.x = 40.;
        for transform in [shear, Matrix3::from_nonuniform_scale(0., 2.)] {
            let mut circle = Processor::with_transform(
                TrimmedCurve::new(UnitCircle::<Point2>::new(), (0., TAU)),
                transform,
            );
            if reversed {
                circle.invert();
            }
            verify(&circle, 0.001);
        }
    }
}

#[test]
fn hyperbolic_and_polynomial_curves_accept_small_local_tolerances() {
    let transform = Matrix4::from_scale(250.);
    let hyperbola = Processor::with_transform(
        TrimmedCurve::new(UnitHyperbola::<Point3>::new(), (-1., 1.)),
        transform,
    );
    verify(&hyperbola, 0.00001);
    let polynomial = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(0., 0., 0.),
            Point3::new(0.5, 1., 0.),
            Point3::new(1., 0., 0.),
        ],
    );
    verify(&Processor::with_transform(polynomial, transform), 0.00001);
    let line = Line(Point3::origin(), Point3::new(1., 0., 0.));
    let (parameters, points) = algo::curve::parameter_division(&line, (0., 1.), 1e-200);
    assert_eq!(parameters, [0., 1.]);
    assert_eq!(points, [line.0, line.1]);
}
