mod common;
use common::assert_solid;
use truck_modeling::*;
#[test]
fn app_revolved_sphere_in_equal_radius_bore() {
    let a: CompressedSolid =
        serde_json::from_str(include_str!("contact-bored-block.json")).unwrap();
    let b: CompressedSolid =
        serde_json::from_str(include_str!("contact-revolved-sphere.json")).unwrap();
    let a = Solid::extract(a).unwrap();
    let b = Solid::extract(b).unwrap();
    let result = truck_shapeops::try_subtract(&a, &b, 0.01).unwrap();
    assert_solid(
        &result,
        64.0 - 2.5 * std::f64::consts::PI - 5.0 * std::f64::consts::PI / 24.0,
        &[0],
        0.001,
    );
}
