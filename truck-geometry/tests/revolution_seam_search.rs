use std::f64::consts::{PI, TAU};
use truck_geometry::prelude::*;

#[test]
fn revolution_search_resolves_small_angles_at_the_seam_and_antipode() {
    for radius in [1., 150., 1000.] {
        for azimuth in [0., 0.1, 0.37, 0.73, 1.1, 2.3] {
            let radial = Vector3::new(f64::cos(azimuth), f64::sin(azimuth), 0.) * radius;
            let line = Line(
                Point3::from_vec(radial),
                Point3::from_vec(radial + Vector3::unit_z() * 100.),
            );
            let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_z());
            for angle in [0., 1e-10, 1e-9, 1e-8, TAU - 1e-9, PI - 1e-9, PI, PI + 1e-9] {
                let point = surface.subs(0.4, angle);
                let uv = surface
                    .search_parameter(point, None, 100)
                    .unwrap_or_else(|| panic!("radius={radius}, azimuth={azimuth}, angle={angle}"));
                assert!(surface.subs(uv.0, uv.1).distance(point) < 1e-9);
                let nearest = surface.search_nearest_parameter(point, None, 100).unwrap();
                assert!(surface.subs(nearest.0, nearest.1).distance(point) < 1e-9);
            }
        }
    }
}
