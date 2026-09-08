//! Original R9 shaft fixture: fine polar caps and dense groups of equal normals.
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_topology::shell::ShellCondition;

fn shaft() -> Solid {
    let compressed = serde_json::from_str(include_str!("R9-revolve-shaft.json")).unwrap();
    Solid::extract(compressed).unwrap()
}

fn check_shaft(robust: bool) {
    let solid = shaft();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let start = Instant::now();
    let meshed = if robust {
        solid.robust_triangulation(0.001)
    } else {
        solid.triangulation(0.001)
    };
    assert!(meshed.face_iter().all(|face| face.surface().is_some()));
    let mut mesh = meshed.to_polygon();
    eprintln!(
        "robust={robust}: triangulation {:?}, {} vertices, {} triangles",
        start.elapsed(),
        mesh.positions().len(),
        mesh.faces().triangle_iter().count()
    );
    assert!(mesh.positions().len() < 2_000, "runaway mesh refinement");
    eprintln!("starting put_together_same_attrs");
    mesh.put_together_same_attrs(1e-7);
    eprintln!("finished put_together_same_attrs: {:?}", start.elapsed());
    mesh.remove_degenerate_faces();
    assert!(!mesh.normals().is_empty());
    assert!(mesh.faces().face_iter().flatten().all(|v| v.nor.is_some()));
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    let volume = mesh.volume();
    assert!(
        (volume - 250.0 * std::f64::consts::PI).abs() < 0.3,
        "volume {volume}"
    );
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    eprintln!(
        "including mesh cleanup: {:?}, volume {volume}",
        start.elapsed()
    );
}

#[test]
fn fine_shaft_triangulation_is_bounded_and_closed() { check_shaft(false); }

#[test]
fn fine_shaft_robust_triangulation_is_bounded_and_closed() { check_shaft(true); }

#[test]
fn fine_shaft_faces_are_bounded() {
    for (index, face) in shaft().face_iter().enumerate() {
        let shell: Shell = vec![face.clone()].into();
        let start = Instant::now();
        eprintln!("starting face {index}");
        let mesh = shell.triangulation(0.001).to_polygon();
        eprintln!(
            "face {index}: {:?}, {} vertices",
            start.elapsed(),
            mesh.positions().len()
        );
        assert!(!mesh.positions().is_empty());
        assert!(
            mesh.positions().len() < 200,
            "face {index}: runaway mesh refinement"
        );
    }
}

// Diagnostic only: dropping normals is not the requested production fix.
#[test]
fn fine_shaft_welding_without_normals_control() {
    let mut mesh = shaft().triangulation(0.001).to_polygon();
    {
        let editor = mesh.debug_editor();
        editor.attributes.normals.clear();
        for vertex in editor.faces.face_iter_mut().flatten() {
            vertex.nor = None;
        }
    }
    let start = Instant::now();
    mesh.put_together_same_attrs(1e-7).remove_degenerate_faces();
    eprintln!("welding without normals: {:?}", start.elapsed());
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    assert!((mesh.volume() - 250.0 * std::f64::consts::PI).abs() < 0.3);
}
