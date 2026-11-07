use truck_shapeops::RobustSplitClosedEdgesAndFaces;
use truck_stepio::r#in::step_geometry::*;
use truck_topology::compress::CompressedShell;

#[test]
fn conical_thread_entry_reselects_the_discovered_boundary_crossing() {
    let mut shell: CompressedShell<Point3, Curve3D, Surface> =
        serde_json::from_str(include_str!("fixtures/thread-entry-cone.json")).unwrap();
    let surface = shell.faces[0].surface.clone();
    let original_vertices = shell.vertices.clone();
    shell.robust_split_closed_edges_and_faces(1e-6);
    assert_eq!(shell.faces.len(), 2);
    assert!(shell.faces.iter().all(|face| face.surface == surface));
    assert_eq!(
        &shell.vertices[..original_vertices.len()],
        &original_vertices
    );
    for face in &shell.faces {
        for wire in &face.boundaries {
            let vertices: std::collections::HashSet<_> = wire
                .iter()
                .map(|e| {
                    let (a, b) = shell.edges[e.index].vertices;
                    if e.orientation {
                        a
                    } else {
                        b
                    }
                })
                .collect();
            assert_eq!(vertices.len(), wire.len());
        }
    }
    let mapped = shell
        .try_mapped(
            |p| Some(*p),
            |c| truck_modeling::Curve::try_from(c).ok(),
            |s| truck_modeling::Surface::try_from(s).ok(),
        )
        .unwrap();
    let exact = truck_modeling::Shell::extract(mapped).unwrap();
    use truck_meshalgo::prelude::*;
    assert!(exact
        .robust_triangulation(0.01)
        .face_iter()
        .all(|face| face.surface().is_some()));
}
