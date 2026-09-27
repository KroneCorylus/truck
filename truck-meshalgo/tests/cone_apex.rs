//! A cone revolved to its apex, as the app builds it: four quarter faces whose boundaries
//! collapse to the apex. Fans into the apex once came out inverted in parameter space, so
//! every winding refinement failed and the shell was meshed seven times, 128 times finer.
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_topology::shell::ShellCondition;

fn cone() -> Solid {
    let compressed = serde_json::from_str(include_str!("cone-apex.json")).unwrap();
    Solid::extract(compressed).unwrap()
}

#[test]
fn cone_apex_meshes_at_the_requested_tolerance() {
    let tolerance = 0.05;
    let meshed = cone().triangulation(tolerance);
    for face in meshed.face_iter() {
        let mesh = face.surface().unwrap();
        for triangle in mesh.tri_faces() {
            let [a, b, c] = triangle.map(|v| mesh.uv_coords()[v.uv.unwrap()]);
            assert!((b - a).perp_dot(c - a) >= 0.0, "inverted in parameter space");
        }
    }
    let mut mesh = meshed.to_polygon();
    assert!(mesh.tri_faces().len() < 150, "{} triangles", mesh.tri_faces().len());
    mesh.put_together_same_attrs(TOLERANCE).remove_degenerate_faces();
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    // radius 10, height 10; the chords lie inside the surface, by at most the tolerance
    let exact = std::f64::consts::PI * 1000.0 / 3.0;
    let area = std::f64::consts::PI * 10.0 * (10.0 + 200f64.sqrt());
    let volume = mesh.volume();
    assert!(volume <= exact && exact - volume < tolerance * area, "{volume}");
}
