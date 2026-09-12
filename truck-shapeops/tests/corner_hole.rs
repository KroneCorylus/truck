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
