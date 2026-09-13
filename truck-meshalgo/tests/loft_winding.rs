use truck_meshalgo::prelude::*;
use truck_modeling::*;

#[test]
fn thin_curved_loft_keeps_closed_mesh_at_display_tolerances() {
    let wires = [(2., 0.), (2., 10.), (4., 20.)].map(|(x, z)| {
        let center = Point3::new(x, 0., z);
        let radial = Vector3::new(0.2 * 0.1_f64.cos(), 0.2 * 0.1_f64.sin(), 0.);
        builder::rsweep(
            &builder::vertex(center + radial),
            center,
            Vector3::unit_z(),
            Rad(2. * std::f64::consts::PI),
            4,
        )
    });
    let solid: Solid = builder::try_loft(&builder::align_sections(&wires)).unwrap();
    assert!(solid.is_geometric_consistent());
    for face in solid.face_iter() {
        let surface = face.surface();
        if matches!(surface, Surface::Plane(_)) {
            continue;
        }
        let (u_range, v_range) = surface.try_range_tuple();
        let ((u0, u1), (v0, v1)) = (u_range.unwrap(), v_range.unwrap());
        for u in [0.25, 0.5, 0.75] {
            for v in [0.25, 0.5, 0.75] {
                let point = surface.subs(u0 + u * (u1 - u0), v0 + v * (v1 - v0));
                assert!(radial_error(point) < 1.0e-8);
            }
        }
    }
    let before = serde_json::to_string(&solid.compress()).unwrap();
    for threads in [1, 4] {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                for tolerance in [0.05, 0.01, 0.001] {
                    check(
                        solid.robust_triangulation(tolerance).to_polygon(),
                        tolerance,
                    );
                    check(
                        solid
                            .compress()
                            .robust_triangulation(tolerance)
                            .to_polygon(),
                        tolerance,
                    );
                }
            });
    }
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}

fn check(mut mesh: PolygonMesh, tolerance: f64) {
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(
        mesh.shell_condition(),
        ShellCondition::Closed,
        "tol={tolerance}"
    );
    assert!(
        (mesh.volume() - 0.8 * std::f64::consts::PI).abs() < 0.03,
        "{}",
        mesh.volume()
    );
    if tolerance != 0.05 {
        return;
    }
    for triangle in mesh.tri_faces() {
        let points = triangle.map(|v| mesh.positions()[v.pos]);
        let [x, y, z] = points;
        for point in [
            x + (y - x) / 2.,
            y + (z - y) / 2.,
            z + (x - z) / 2.,
            x + (y + (z - y) / 2. - x) * (2. / 3.),
        ] {
            if point.z <= TOLERANCE || point.z >= 20. - TOLERANCE {
                continue;
            }
            assert!(
                radial_error(point) <= tolerance,
                "tol={tolerance}, error={}",
                radial_error(point)
            );
        }
    }
}

// The three equally sized circles interpolate a quadratic centerline with chord parameters.
// A point at the same z and polar angle on that circle gives a distance upper bound.
fn radial_error(point: Point3) -> f64 {
    let p = 10. / (10. + 104_f64.sqrt());
    let a = (20. * p - 10.) / (p - p * p);
    let b = 20. - a;
    let t = 2. * point.z / (b + (b * b + 4. * a * point.z).sqrt());
    let center = 2. + 2. * t * (t - p) / (1. - p);
    (((point.x - center).powi(2) + point.y.powi(2)).sqrt() - 0.2).abs()
}
