mod common;
use common::assert_solid;
use common::blend::assert_step;
use truck_modeling::*;
use truck_shapeops::local::try_draft;

fn circle(radius: f64) -> Wire {
    let vertices: Vec<_> = (0..4)
        .map(|i| {
            let t = std::f64::consts::FRAC_PI_2 * i as f64;
            builder::vertex(Point3::new(radius * t.cos(), radius * t.sin(), 0.))
        })
        .collect();
    (0..4)
        .map(|i| {
            let start = std::f64::consts::FRAC_PI_2 * i as f64;
            Edge::new(
                &vertices[i],
                &vertices[(i + 1) % 4],
                Curve::Conic(Processor::with_transform(
                    TrimmedCurve::new(
                        UnitCircle::new(),
                        (start, start + std::f64::consts::FRAC_PI_2),
                    ),
                    Matrix4::from_nonuniform_scale(radius, radius, 1.),
                )),
            )
        })
        .collect()
}
#[test]
fn inner_and_outer_cylindrical_walls_draft_together() {
    for angle_sign in [-1., 1.] {
        for placed in [false, true] {
            for sign in [-1., 1.] {
                for second in [0., 2.] {
                    let face =
                        builder::try_attach_plane(&[circle(10.), circle(5.).inverse()]).unwrap();
                    let face = builder::translated(&face, Vector3::new(0., 0., -sign * second));
                    let mut solid: Solid =
                        builder::tsweep(&face, Vector3::new(0., 0., sign * (4. + second)));
                    if sign < 0. {
                        solid.not();
                    }
                    let transform = if placed {
                        Matrix4::from_translation(Vector3::new(17., -21., 32.))
                            * Matrix4::from_axis_angle(
                                Vector3::new(1., 2., 3.).normalize(),
                                Rad(0.7),
                            )
                    } else {
                        Matrix4::identity()
                    };
                    let solid = builder::transformed(&solid, transform);
                    let before = serde_json::to_string(&solid.compress()).unwrap();
                    let ids = solid
                        .face_iter()
                        .filter(|f| {
                            matches!(
                                f.surface().elementary(),
                                Some((Elementary::Cylinder { .. }, _))
                            )
                        })
                        .map(|f| f.id())
                        .collect::<Vec<_>>();
                    let plane = Plane::new(
                        Point3::origin(),
                        Point3::new(1., 0., 0.),
                        Point3::new(0., 1., 0.),
                    );
                    let plane = plane.transformed(transform);
                    let result = try_draft(
                        &solid,
                        &ids,
                        &plane,
                        transform.transform_vector(Vector3::unit_z() * sign),
                        Rad(angle_sign * 3f64.to_radians()),
                    )
                    .unwrap();
                    assert!(result.is_geometric_consistent());
                    let expected = std::f64::consts::PI
                        * (75. * (4. + second)
                            + 15. * angle_sign * 3f64.to_radians().tan() * (16. - second * second));
                    assert_solid(&result, expected, &[1], 0.001);
                    assert_step(&result, expected, 0.001);
                    assert_eq!(serde_json::to_string(&solid.compress()).unwrap(), before);
                }
            }
        }
    }
}
