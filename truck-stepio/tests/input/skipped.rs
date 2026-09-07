//! Faces, edges and vertices the reader cannot convert are left out and reported, not lost.

use truck_stepio::r#in::*;
use truck_topology::shell::ShellCondition;

const CUBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/step/occt-cube.step"
));

fn convert(step: &str) -> (Table, Vec<convert::Skipped>, usize, usize, usize) {
    let table = Table::try_from_step(step).unwrap();
    let step_shell = table.shell.values().next().unwrap();
    let (cshell, skipped) = table.to_compressed_shell(step_shell).unwrap();
    let counts = (
        cshell.vertices.len(),
        cshell.edges.len(),
        cshell.faces.len(),
    );
    (table, skipped, counts.0, counts.1, counts.2)
}

#[test]
fn face_on_an_unsupported_surface_is_skipped_and_named() {
    let step = CUBE.replace("#32 = PLANE('',#33);", "#32 = HOGE_SURFACE('',#33);");
    let (table, skipped, vertices, edges, faces) = convert(&step);
    assert_eq!((vertices, edges, faces), (8, 12, 5));
    assert_eq!(skipped.len(), 1, "{skipped:?}");
    assert_eq!((skipped[0].id, skipped[0].entity), (17, "FACE_SURFACE"));
    let reason = skipped[0].reason.to_string();
    assert!(
        reason.contains("#32") && reason.contains("HOGE_SURFACE"),
        "{reason}"
    );
    assert_eq!(table.unsupported()["HOGE_SURFACE"], 1);
}

#[test]
fn unreadable_vertex_takes_its_edges_and_faces_with_it() {
    let step = CUBE.replace(
        "#23 = CARTESIAN_POINT('',(0.,0.,0.));",
        "#23 = CARTESIAN_POINT('',(0.,0.,'x'));",
    );
    assert_ne!(step, CUBE);
    let (table, skipped, vertices, edges, faces) = convert(&step);
    assert_eq!(table.errors.len(), 1);
    assert_eq!((vertices, edges, faces), (7, 9, 3));
    let kinds = |entity| skipped.iter().filter(|s| s.entity == entity).count();
    assert_eq!(
        (
            kinds("VERTEX_POINT"),
            kinds("EDGE_CURVE"),
            kinds("FACE_SURFACE")
        ),
        (1, 3, 3)
    );
    let vertex = skipped.iter().find(|s| s.entity == "VERTEX_POINT").unwrap();
    assert_eq!(vertex.id, 22);
    assert!(
        vertex.reason.to_string().contains("#23 could not be read"),
        "{vertex}"
    );
    for edge in skipped.iter().filter(|s| s.entity == "EDGE_CURVE") {
        assert_eq!(edge.reason.to_string(), "vertex #22 was skipped");
    }
    for face in skipped.iter().filter(|s| s.entity == "FACE_SURFACE") {
        assert!(face.reason.to_string().ends_with("was skipped"), "{face}");
    }

    let step_shell = table.shell.values().next().unwrap();
    let (cshell, _) = table.to_compressed_shell(step_shell).unwrap();
    let shell = truck_topology::Shell::extract(cshell).unwrap();
    assert_eq!(shell.shell_condition(), ShellCondition::Oriented);
    assert!(shell.iter().all(|face| face.boundaries()[0].len() == 4));
}
