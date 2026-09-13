mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_along_wire;
const TOL: f64 = 0.001;
fn rim(input: &Solid, z: f64) -> Wire {
    input
        .face_iter()
        .find(|f| {
            matches!(f.surface(), Surface::Plane(_))
                && f.vertex_iter().all(|v| (v.point().z - z).abs() < 1e-6)
        })
        .unwrap()
        .boundaries()
        .into_iter()
        .find(|w| w.iter().all(|e| !matches!(e.curve(), Curve::Line(_))))
        .unwrap()
}
#[test]
fn boolean_hole_rims_are_closed_consistent_and_have_the_expected_volume() {
    let block = common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., 10.));
    for blind in [false, true] {
        let cutter = common::modeling::cylinder(
            Point3::new(0., 0., if blind { 5. } else { -1. }),
            Vector3::unit_z(),
            3.,
            12.,
        );
        let input = truck_shapeops::subtract(&block, &cutter, TOL).unwrap();
        assert!(input.is_geometric_consistent());
        let before = serde_json::to_string(&input.compress()).unwrap();
        for z in [10., if blind { 5. } else { 0. }] {
            for reverse in [false, true] {
                let wire = rim(&input, z);
                let wire = if reverse { wire.inverse() } else { wire };
                let result = try_fillet_solid_along_wire(&input, &wire, 0.5, TOL).unwrap();
                assert!(
                    result.solid.is_geometric_consistent(),
                    "blind={blind} z={z} reverse={reverse}"
                );
                let pi = std::f64::consts::PI;
                // Revolve the square-minus-quarter-circle section and its radial moment.
                let a = 3. * 0.5_f64.powi(2) * (1. - pi / 4.);
                let b = 0.5_f64.powi(3) * (5. / 6. - pi / 4.);
                let base = 4000. - 9. * pi * if blind { 5. } else { 10. };
                let expected = base
                    + if blind && z == 5. {
                        2. * pi * (a - b)
                    } else {
                        -2. * pi * (a + b)
                    };
                common::assert_solid(&result.solid, expected, &[usize::from(!blind)], TOL);
                common::blend::assert_step(&result.solid, expected, TOL);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}
