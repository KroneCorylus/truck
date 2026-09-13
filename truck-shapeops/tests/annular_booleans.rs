mod common;
use common::{assert_solid, blend::assert_step};
use truck_modeling::*;
fn circle(radius: f64, z: f64) -> Wire {
    let vertices: Vec<_> = (0..4)
        .map(|i| {
            let t = std::f64::consts::FRAC_PI_2 * i as f64;
            builder::vertex(Point3::new(radius * t.cos(), radius * t.sin(), z))
        })
        .collect();
    (0..4)
        .map(|i| {
            let a = std::f64::consts::FRAC_PI_2 * i as f64;
            Edge::new(
                &vertices[i],
                &vertices[(i + 1) % 4],
                Curve::Conic(Processor::with_transform(
                    TrimmedCurve::new(UnitCircle::new(), (a, a + std::f64::consts::FRAC_PI_2)),
                    Matrix4::from_translation(Vector3::new(0., 0., z))
                        * Matrix4::from_nonuniform_scale(radius, radius, 1.),
                )),
            )
        })
        .collect()
}
fn tube(outer: f64, inner: f64, z: f64, height: f64) -> Solid {
    let mut wires = vec![circle(outer, z)];
    if inner > 0. {
        wires.push(circle(inner, z).inverse());
    }
    let face = builder::try_attach_plane(wires).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * height)
}
#[test]
fn annular_cuts_preserve_the_inner_web_and_counterbores_preserve_the_through_hole() {
    for placed in [false, true] {
        for (target, tool, volume, genus) in [
            (
                tube(5., 0., 0., 10.),
                tube(6., 4., 3., 4.),
                214. * std::f64::consts::PI,
                0,
            ),
            (
                tube(10., 3., 0., 10.),
                tube(4., 0., 0., 5.),
                875. * std::f64::consts::PI,
                1,
            ),
        ] {
            let transform = if placed {
                Matrix4::from_translation(Vector3::new(17., -21., 32.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
            } else {
                Matrix4::identity()
            };
            let target = builder::transformed(&target, transform);
            let tool = builder::transformed(&tool, transform);
            let before = [&target, &tool].map(|s| serde_json::to_string(&s.compress()).unwrap());
            let mut negative = tool.clone();
            negative.not();
            let result = truck_shapeops::try_and(&target, &negative, 0.001).unwrap();
            assert!(result.is_geometric_consistent());
            assert_solid(&result, volume, &[genus], 0.001);
            assert_step(&result, volume, 0.001);
            assert_eq!(
                [&target, &tool].map(|s| serde_json::to_string(&s.compress()).unwrap()),
                before
            );
        }
    }
}
