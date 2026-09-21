mod common;
use common::{assert_solid, modeling::cuboid};
use truck_modeling::*;
#[test]
fn blind_hole_at_exact_box_corner_retains_closed_quarter_bore() {
    let plate = cuboid(Point3::origin(), Point3::new(60.0, 40.0, 10.0));
    let origin = Point3::new(0.0, 0.0, 10.0);
    let axis = -Vector3::unit_z();
    let vertex = builder::vertex(origin + Vector3::unit_x() * 2.0);
    let circle: Wire = builder::rsweep(&vertex, origin, axis, Rad(std::f64::consts::TAU), 4);
    let disk: Face = builder::try_attach_plane(&[circle]).unwrap();
    let cylinder: Solid = builder::tsweep(&disk, axis * 5.0);
    assert_solid(&cylinder, 20.0 * std::f64::consts::PI, &[0], 0.001);
    let result = truck_shapeops::try_subtract(&plate, &cylinder, 0.01).unwrap();
    assert_solid(&result, 24000.0 - 5.0 * std::f64::consts::PI, &[0], 0.001);
}

#[test]
fn through_holes_and_counterbores_cross_cylinder_seams_at_plate_boundaries() {
    use std::f64::consts::PI;
    let plate = cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., 10.));
    for (x, y, fraction) in [(10., 0., 0.5), (10., 10., 0.25)] {
        let cylinder = |radius, depth| {
            let origin = Point3::new(x, y, 10.);
            let vertex = builder::vertex(origin + Vector3::unit_x() * radius);
            let wire: Wire = builder::rsweep(&vertex, origin, -Vector3::unit_z(), Rad(2. * PI), 4);
            builder::tsweep(
                &builder::try_attach_plane(&[wire]).unwrap(),
                -Vector3::unit_z() * depth,
            )
        };
        let through = truck_shapeops::try_subtract(&plate, &cylinder(2., 12.), 0.01).unwrap();
        assert!(through.is_geometric_consistent());
        assert_solid(&through, 4000. - fraction * PI * 4. * 10., &[0], 0.001);
        let blind = truck_shapeops::try_subtract(&plate, &cylinder(2., 6.), 0.01).unwrap();
        let counterbore = truck_shapeops::try_subtract(&blind, &cylinder(4., 2.), 0.01).unwrap();
        assert!(counterbore.is_geometric_consistent());
        assert_solid(
            &counterbore,
            4000. - fraction * PI * (4. * 6. + 12. * 2.),
            &[0],
            0.001,
        );
    }
}
