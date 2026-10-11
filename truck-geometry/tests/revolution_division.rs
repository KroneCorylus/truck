use std::f64::consts::FRAC_PI_2;
use truck_geometry::prelude::*;

#[test]
fn revolution_chords_use_world_radius_and_share_the_tolerance_between_directions() {
    let tol = 0.001;
    for radius in [0.01, 1., 150., 900.] {
        let cylinder = RevolutedCurve::by_revolution(
            Line(Point3::new(radius, 0., -1.), Point3::new(radius, 0., 1.)),
            Point3::origin(),
            Vector3::unit_z(),
        );
        check(&cylinder, ((0., 1.), (0., FRAC_PI_2)), tol);
        let meridian = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (-0.7, 0.7)),
            Matrix4::from_cols(
                Vector4::new(radius, 0., 0., 0.),
                Vector4::new(0., 0., radius, 0.),
                Vector4::new(0., 1., 0., 0.),
                Vector4::unit_w(),
            ),
        );
        let sphere = RevolutedCurve::by_revolution(meridian, Point3::origin(), Vector3::unit_z());
        check(&sphere, ((-0.7, 0.7), (0., FRAC_PI_2)), tol);
    }
}

fn check<S: ParametricSurface3D + ParameterDivision2D>(
    surface: &S,
    range: ((f64, f64), (f64, f64)),
    tol: f64,
) {
    let (us, vs) = surface.parameter_division(range, tol);
    assert_eq!((us[0], *us.last().unwrap()), range.0);
    assert_eq!((vs[0], *vs.last().unwrap()), range.1);
    for u in us.windows(2) {
        for v in vs.windows(2) {
            for reverse in [false, true] {
                let (v0, v1) = if reverse { (v[1], v[0]) } else { (v[0], v[1]) };
                let a = surface.subs(u[0], v0);
                let b = surface.subs(u[1], v1);
                for t in [0.25, 0.5, 0.75] {
                    let uv = (u[0] + (u[1] - u[0]) * t, v0 + (v1 - v0) * t);
                    let error = surface.subs(uv.0, uv.1).distance(a + (b - a) * t);
                    assert!(
                        error <= tol * (1. + 1e-8),
                        "chord error {error} exceeds {tol}"
                    );
                }
            }
        }
    }
}
