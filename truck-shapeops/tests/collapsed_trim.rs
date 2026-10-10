mod common;

use truck_meshalgo::prelude::*;
use truck_modeling::*;

#[test]
fn thin_circular_segment_solid_keeps_shared_edges_when_refined() {
    for angle in [std::f64::consts::TAU / 64.0, std::f64::consts::TAU / 4096.0] {
        let a = builder::vertex(Point3::new(12.0, 0.0, 0.0));
        let b = builder::vertex(Point3::new(12.0 * angle.cos(), 12.0 * angle.sin(), 0.0));
        let arc = builder::circle_arc(&a, &b, Vector3::unit_y());
        let chord = builder::line(&b, &a);
        let origin = Point3::new(11.975386691676, 0.588313022933, 0.0);
        let plane = Plane::new(
            origin,
            origin + Vector3::unit_x(),
            origin + Vector3::unit_y(),
        );
        let face: Face = Face::new(vec![vec![arc, chord].into()], plane.into());
        let solid: Solid = builder::tsweep(&face, Vector3::unit_z() * 2.0);
        let exact_volume = 144.0 * (angle - angle.sin());
        for tolerance in [0.05, 100.0] {
            common::assert_solid(&solid, exact_volume, &[0], tolerance);
            for mut mesh in [
                solid.triangulation(tolerance).to_polygon(),
                solid.compress().triangulation(tolerance).to_polygon(),
            ] {
                assert!(mesh.volume() > exact_volume / 2.0 && mesh.volume() <= exact_volume);
                assert!(mesh
                    .positions()
                    .iter()
                    .all(|p| p.x.hypot(p.y) <= 12.0 + 1e-8));
                mesh.put_together_same_attrs(1e-8).remove_degenerate_faces();
                assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
            }
        }
    }
}
