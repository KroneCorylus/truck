use truck_meshalgo::prelude::*;
use truck_modeling::*;

#[test]
fn revolved_poles_keep_boundary_parameters_on_their_spatial_points() {
    let compressed = serde_json::from_str(include_str!(
        "../../truck-shapeops/tests/contact-revolved-sphere.json"
    ))
    .unwrap();
    let solid = Solid::extract(compressed).unwrap();
    assert!(solid.is_geometric_consistent());
    for tolerance in [0.05, 0.01, 0.001] {
        let tessellation = solid.robust_triangulation(tolerance);
        for (face, tessellated) in solid.face_iter().zip(tessellation.face_iter()) {
            let mesh = tessellated.surface().unwrap();
            let surface = face.surface();
            for triangle in mesh.tri_faces() {
                for vertex in triangle {
                    let uv = mesh.uv_coords()[vertex.uv.unwrap()];
                    assert!(mesh.positions()[vertex.pos].near(&surface.subs(uv.x, uv.y)));
                }
            }
        }
        let mut mesh = tessellation.to_polygon();
        mesh.put_together_same_attrs(TOLERANCE)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
        assert!(
            (mesh.volume().abs() - 4. * std::f64::consts::PI / 3.).abs()
                < 4. * std::f64::consts::PI * tolerance
        );
    }
}
