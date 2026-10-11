mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

const TOL: f64 = 0.001;

#[test]
fn planar_end_faces_trim_a_fillet_wider_than_its_spine() {
    for height in [0.2, 1., 10.] {
        for radius in [0.5, 10.] {
            for skew in [-0.4, 0., 0.4] {
                let mut shear = Matrix4::identity();
                shear.x.z = skew;
                shear.y.z = -skew / 2.;
                for transform in [
                    Matrix4::identity(),
                    Matrix4::from_translation(Vector3::new(13., -7., 23.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
                ] {
                    let input = builder::transformed(
                        &common::modeling::cuboid(
                            Point3::origin(),
                            Point3::new(100., 100., height),
                        ),
                        transform * shear,
                    );
                    assert!(input.is_geometric_consistent());
                    common::assert_solid(&input, 10000. * height, &[0], TOL);
                    let before = serde_json::to_string(&input.compress()).unwrap();
                    let selected = common::blend::edge_through(
                        &input.boundaries()[0],
                        transform.transform_point(Point3::new(0., 0., height / 2.)),
                    )
                    .id();
                    let result = try_fillet_solid_edges(&input, &[selected], radius, TOL)
                        .unwrap_or_else(|e| {
                            panic!("height={height}, radius={radius}, skew={skew}: {e:?}")
                        });
                    assert!(result.solid.is_geometric_consistent());
                    let volume = height * (10000. - (1. - PI / 4.) * radius * radius);
                    common::assert_solid(&result.solid, volume, &[0], TOL);
                    common::blend::assert_step(&result.solid, volume, TOL);
                    assert_eq!(result.modified_faces.len(), 6);
                    assert_eq!(result.generated_faces.len(), 1);
                    let inverse = transform.invert().unwrap();
                    let surface = result.generated_faces[0].oriented_surface();
                    let (u, v) = surface.try_range_tuple();
                    let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                    for i in 0..=16 {
                        for j in 0..=8 {
                            let (u, v) =
                                (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 8.);
                            let p = inverse.transform_point(surface.subs(u, v));
                            assert!(((p.x - radius).hypot(p.y - radius) - radius).abs() < 1e-6);
                            let z = p.z - skew * p.x + skew / 2. * p.y;
                            assert!(z >= -1e-6 && z <= height + 1e-6);
                            let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                            assert!(
                                normal
                                    .dot(Vector3::new(p.x - radius, p.y - radius, 0.).normalize())
                                    > 1. - 1e-8
                            );
                        }
                    }
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                    let error = try_fillet_solid_edges(&input, &[selected], 101., TOL).unwrap_err();
                    assert_eq!(error.code, truck_base::diagnostics::Code::OutsideNeighbour);
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}
