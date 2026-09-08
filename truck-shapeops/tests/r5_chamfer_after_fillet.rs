mod common;

use std::collections::HashSet;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::{chamfer_solid_along_wire, chamfer_solid_edge, fillet_solid_edges};

const TOL: f64 = 0.001;

fn cube() -> Solid {
    let vertices = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)));
    let wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let face: Face = builder::try_attach_plane(vec![wire]).unwrap();
    builder::tsweep(&face, Vector3::new(0.0, 0.0, 10.0))
}

fn rounded() -> Solid {
    let solid = cube();
    let mut seen = HashSet::new();
    let edges: Vec<_> = solid
        .edge_iter()
        .filter(|edge| {
            let direction = edge.back().point() - edge.front().point();
            seen.insert(edge.id()) && direction.x.abs() < 1.0e-6 && direction.y.abs() < 1.0e-6
        })
        .map(|edge| edge.id())
        .collect();
    assert_eq!(edges.len(), 4);
    fillet_solid_edges(&solid, &edges, 1.0, TOL)
        .expect("four vertical fillets")
        .solid
}

fn midpoint(edge: &Edge) -> Point3 {
    let curve = edge.curve();
    let (a, b) = curve.range_tuple();
    curve.subs((a + b) / 2.0)
}

fn top_edges(solid: &Solid) -> Vec<Edge> {
    let mut seen = HashSet::new();
    solid
        .edge_iter()
        .filter(|edge| {
            seen.insert(edge.id())
                && (edge.front().point().z - 10.0).abs() < 1.0e-6
                && (edge.back().point().z - 10.0).abs() < 1.0e-6
        })
        .collect()
}

fn straight(edge: &Edge) -> bool {
    midpoint(edge).distance(edge.front().point().midpoint(edge.back().point())) < 1.0e-6
}

#[test]
fn plain_top_chamfer_control() {
    let solid = cube();
    for edge in top_edges(&solid) {
        assert!(chamfer_solid_edge(&solid, edge.id(), 0.2, 0.2, TOL).is_some());
    }
}

#[test]
fn four_vertical_fillets_control() {
    let solid = rounded();
    assert!(Solid::try_new(solid.boundaries().clone()).is_ok());
    let mesh = solid.triangulation(TOL).to_polygon();
    let expected = 1000.0 - 40.0 + 10.0 * std::f64::consts::PI;
    assert!(
        (mesh.volume() - expected).abs() < 0.1,
        "{} != {expected}",
        mesh.volume()
    );
    assert_eq!(top_edges(&solid).len(), 8);
}

fn top_wire(solid: &Solid) -> Wire {
    solid
        .face_iter()
        .flat_map(|f| f.boundaries())
        .find(|w| w.len() == 8 && w.vertex_iter().all(|v| (v.point().z - 10.0).abs() < 1.0e-6))
        .unwrap()
}

#[test]
fn single_top_edges_require_explicit_tangent_wire() {
    let solid = rounded();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let edges = top_edges(&solid);
    assert_eq!(edges.iter().filter(|e| straight(e)).count(), 4);
    assert_eq!(edges.iter().filter(|e| !straight(e)).count(), 4);
    for edge in edges {
        assert!(chamfer_solid_edge(&solid, edge.id(), 0.2, 0.2, TOL).is_none());
    }
    assert_eq!(serde_json::to_string(&solid.compress()).unwrap(), before);
}

fn expected_volume(inset: f64, depth: f64) -> f64 {
    let pi = std::f64::consts::PI;
    // Integrate A(s) = A(0) - perimeter * s + pi * s^2 over the chamfer height.
    1000.0 - 40.0 + 10.0 * pi - (32.0 + 2.0 * pi) * inset * depth / 2.0
        + pi * inset * inset * depth / 3.0
}

#[test]
fn complete_top_wire_after_four_vertical_fillets() {
    let solid = rounded();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let wire = top_wire(&solid);
    let result =
        chamfer_solid_along_wire(&solid, &wire, 0.2, 0.2, TOL).expect("closed tangent top chamfer");
    let expected = expected_volume(0.2, 0.2);
    assert_eq!(result.generated_faces.len(), 8);
    assert_eq!(result.modified_faces.len(), 9);
    for (id, face) in &result.modified_faces {
        assert!(solid.face_iter().any(|f| f.id() == *id));
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
    }
    for face in &result.generated_faces {
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
    }
    let untouched: Vec<_> = solid
        .face_iter()
        .filter(|f| result.solid.face_iter().any(|g| f.id() == g.id()))
        .collect();
    assert_eq!(untouched.len(), 1);
    assert!(untouched[0]
        .vertex_iter()
        .all(|v| v.point().z.abs() < 1.0e-6));
    common::assert_solid(&result.solid, expected, &[0], TOL);
    assert_step(&result.solid, expected);
    assert_eq!(serde_json::to_string(&solid.compress()).unwrap(), before);
}

#[test]
fn asymmetric_distances_follow_wire_orientation() {
    let solid = rounded();
    let wire = top_wire(&solid);
    for (wire, d0, d1) in [(wire.clone(), 0.3, 0.15), (wire.inverse(), 0.15, 0.3)] {
        let result = chamfer_solid_along_wire(&solid, &wire, d0, d1, TOL).unwrap();
        common::assert_solid(&result.solid, expected_volume(0.3, 0.15), &[0], TOL);
    }
}

#[test]
fn invalid_wires_and_distances_leave_input_unchanged() {
    let solid = rounded();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let wire = top_wire(&solid);
    for (d0, d1, tol) in [
        (0.0, 0.2, TOL),
        (-0.2, 0.2, TOL),
        (f64::NAN, 0.2, TOL),
        (0.2, f64::INFINITY, TOL),
        (0.2, 0.2, 0.0),
        (0.2, 0.2, f64::NAN),
        (1.0, 0.2, TOL),
        (1.1, 0.2, TOL),
        (0.2, 10.0, TOL),
    ] {
        assert!(
            chamfer_solid_along_wire(&solid, &wire, d0, d1, tol).is_none(),
            "accepted distances {d0}, {d1}, tolerance {tol}"
        );
    }
    let mut scrambled = wire.clone();
    scrambled.swap(0, 2);
    let repeated = wire.iter().chain(wire.iter()).cloned().collect();
    let foreign = top_wire(&rounded());
    let sharp = cube().face_iter().next().unwrap().boundaries().remove(0);
    for selection in [
        Wire::new(),
        wire.iter().take(2).cloned().collect(),
        scrambled,
        repeated,
        foreign,
        sharp,
    ] {
        assert!(chamfer_solid_along_wire(&solid, &selection, 0.2, 0.2, TOL).is_none());
    }
    assert_eq!(serde_json::to_string(&solid.compress()).unwrap(), before);
}

#[cfg(feature = "step-test")]
fn assert_step(solid: &Solid, expected: f64) {
    use truck_stepio::{out::*, r#in::Table};
    let compressed = solid.compress();
    let prepared = prepare_for_step(&compressed, TOL / 20.0).expect("STEP preparation");
    assert_eq!(compressed.boundaries.len(), prepared.boundaries.len());
    for (a, b) in compressed.boundaries.iter().zip(&prepared.boundaries) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.edges.len(), b.edges.len());
        assert_eq!(a.faces.len(), b.faces.len());
        for (a, b) in a.edges.iter().zip(&b.edges) {
            assert_eq!(a.vertices, b.vertices);
        }
        for (a, b) in a.faces.iter().zip(&b.faces) {
            assert_eq!(a.boundaries, b.boundaries);
            assert_eq!(a.orientation, b.orientation);
        }
    }
    let design = StepDesign::from_model(StepModel::from(&prepared));
    let step = StepDisplay::new(Default::default(), design).to_string();
    let table = Table::from_step(&step).unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (read, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    let read = read
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap();
    let read = Solid::extract(read).unwrap();
    use truck_meshalgo::prelude::*;
    common::assert_topology(&read, &[0]);
    let mut mesh = read.robust_triangulation(TOL).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(
        mesh.shell_condition(),
        truck_topology::shell::ShellCondition::Closed
    );
    let area: f64 = mesh
        .faces()
        .triangle_iter()
        .map(|tri| {
            let [p, q, r] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos]);
            (q - p).cross(r - p).magnitude() / 2.0
        })
        .sum();
    // STEP approximation adds at most two sampled fitting tolerances to the chord allowance.
    let allowed = area * (TOL + 2.0 * TOL / 20.0);
    assert!(
        (mesh.volume() - expected).abs() <= allowed,
        "STEP volume {}, expected {expected}, allowance {allowed}",
        mesh.volume()
    );
}

#[cfg(not(feature = "step-test"))]
fn assert_step(_: &Solid, _: f64) {}
