mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
const TOL: f64 = 0.001;
fn prism(skew: f64) -> Solid {
    let v = builder::vertices([
        Point3::origin(),
        Point3::new(20., 0., 0.),
        Point3::new(20. + skew, 10., 0.),
        Point3::new(skew, 10., 0.),
    ]);
    let wire: Wire = (0..4)
        .map(|i| builder::line(&v[i], &v[(i + 1) % 4]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}
fn expected(skew: f64, radius: f64, selected: usize) -> f64 {
    let angle = 10_f64.atan2(skew);
    let corner = 1. / (angle / 2.).tan() - (PI - angle) / 2.;
    let horizontal = (100. + skew * skew).sqrt();
    let k = 1. - PI / 4.;
    match selected {
        0 => 2000. - 10. * corner * radius * radius,
        1 => 2000. - 20. * k * radius * radius,
        2 => 2000. - horizontal * k * radius * radius,
        4 => {
            2000. - (20. + horizontal) * k * radius * radius
                + (5. / 3. - PI / 2.) / angle.sin() * radius.powi(3)
        }
        // Integrate horizontal sections of the rounded corner: inset r-sqrt(r²-z²),
        // overlap inset²/sin(angle), and the remaining circular wedge.
        _ => {
            2000. - (10. * corner + (20. + horizontal) * k) * radius * radius
                + (corner / 3. + (5. / 3. - PI / 2.) / angle.sin()) * radius.powi(3)
        }
    }
}
#[test]
fn oblique_planar_terminations_and_spherical_corners_remain_exact() {
    for skew in [-5., 5.] {
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
        ] {
            let input = builder::transformed(&prism(skew), transform);
            assert!(input.is_geometric_consistent());
            let before = serde_json::to_string(&input.compress()).unwrap();
            let ids = [
                Point3::new(0., 0., 5.),
                Point3::new(10., 0., 10.),
                Point3::new(skew / 2., 5., 10.),
            ]
            .map(|p| {
                common::blend::edge_through(&input.boundaries()[0], transform.transform_point(p))
                    .id()
            });
            for radius in [0.2, 1., 2.] {
                for selected in 0..5 {
                    let mut edges = match selected {
                        3 => ids.to_vec(),
                        4 => ids[1..].to_vec(),
                        _ => vec![ids[selected]],
                    };
                    let result = try_fillet_solid_edges(&input, &edges, radius, TOL).unwrap();
                    assert!(
                        result.solid.is_geometric_consistent(),
                        "skew={skew} radius={radius} selection={selected}"
                    );
                    let volume = expected(skew, radius, selected);
                    common::assert_solid(&result.solid, volume, &[0], TOL);
                    if radius == 1. {
                        common::blend::assert_step(&result.solid, volume, TOL);
                    }
                    edges.reverse();
                    let reversed = try_fillet_solid_edges(&input, &edges, radius, TOL).unwrap();
                    assert_eq!(
                        serde_json::to_string(&result.solid.compress()).unwrap(),
                        serde_json::to_string(&reversed.solid.compress()).unwrap()
                    );
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}
