use truck_geometry::prelude::*;

#[test]
fn clustered_surface_insertions_can_be_removed_in_both_directions() {
    for scale in [1., 1e-8] {
        for u_direction in [true, false] {
            let knots = KnotVec::from(vec![0., 0., 0., scale, scale, scale]);
            let square = [0., 0., 1.];
            let controls = (0..3)
                .map(|i| {
                    (0..3)
                        .map(|j| {
                            let (u, v) = (i as f64 / 2., j as f64 / 2.);
                            Point3::new(square[i] + u * v, square[j], u + v)
                        })
                        .collect()
                })
                .collect();
            let mut surface = BSplineSurface::new((knots.clone(), knots), controls);
            for x in [0.3, 0.3000001] {
                if u_direction {
                    surface.add_uknot(x * scale);
                } else {
                    surface.add_vknot(x * scale);
                }
            }
            for x in [0.3000001, 0.3] {
                let knots = if u_direction {
                    surface.uknot_vec()
                } else {
                    surface.vknot_vec()
                };
                let index = knots.iter().position(|t| *t == x * scale).unwrap();
                if u_direction {
                    surface.try_remove_uknot(index).unwrap();
                } else {
                    surface.try_remove_vknot(index).unwrap();
                }
            }
            for u in [0.1, 0.30000005, 0.8] {
                for v in [0.2, 0.30000005, 0.9] {
                    let (a, b) = (u * scale, v * scale);
                    assert!(
                        surface
                            .subs(a, b)
                            .distance(Point3::new(u * u + u * v, v * v, u + v))
                            < 1e-8
                    );
                    assert!(
                        (surface.uder(a, b) * scale - Vector3::new(2. * u + v, 0., 1.)).magnitude()
                            < 1e-7
                    );
                    assert!(
                        (surface.vder(a, b) * scale - Vector3::new(u, 2. * v, 1.)).magnitude()
                            < 1e-7
                    );
                    assert!(
                        (surface.uuder(a, b) * scale.powi(2) - Vector3::new(2., 0., 0.))
                            .magnitude()
                            < 1e-6
                    );
                    assert!(
                        (surface.vvder(a, b) * scale.powi(2) - Vector3::new(0., 2., 0.))
                            .magnitude()
                            < 1e-6
                    );
                    assert!(
                        (surface.uvder(a, b) * scale.powi(2) - Vector3::new(1., 0., 0.))
                            .magnitude()
                            < 1e-6
                    );
                }
            }
        }
    }
}
