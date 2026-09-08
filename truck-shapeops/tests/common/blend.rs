//! Blend regressions use the public modeling representations directly.

use truck_geometry::prelude::*;
#[allow(unused_imports)]
pub use truck_modeling::{shell, wire, Curve, Edge, Face, Shell, Solid, Surface, Vertex, Wire};

/// Keeps the modeling solid and its topology IDs.
pub fn from_modeling(solid: &truck_modeling::Solid) -> Solid { solid.clone() }

/// Index of the face of `shell` whose surface passes through `point`.
pub fn face_through(shell: &Shell, point: Point3) -> usize {
    shell
        .face_iter()
        .position(|face| {
            let surface = face.surface();
            surface
                .search_parameter(point, None, 10)
                .is_some_and(|(u, v)| surface.subs(u, v).near(&point))
        })
        .expect("no face through the point")
}

/// The edge of `shell` that passes through `point`, within its parameter range.
pub fn edge_through(shell: &Shell, point: Point3) -> Edge {
    shell
        .edge_iter()
        .find(|edge| {
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            curve
                .search_nearest_parameter(point, None, 10)
                .is_some_and(|t| t0 <= t && t <= t1 && curve.subs(t).near(&point))
        })
        .expect("no edge through the point")
}

#[cfg(feature = "step-test")]
pub fn assert_step(solid: &Solid, expected: f64, tol: f64) {
    use truck_stepio::{out::*, r#in::Table};
    let compressed = solid.compress();
    let prepared = prepare_for_step(&compressed, tol / 20.0).expect("STEP preparation");
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
    super::assert_topology(&read, &[0]);
    let mut mesh = read.robust_triangulation(tol).to_polygon();
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
    let allowed = area * (tol + 2.0 * tol / 20.0);
    assert!(
        (mesh.volume() - expected).abs() <= allowed,
        "STEP volume {}, expected {expected}, allowance {allowed}",
        mesh.volume()
    );
}

#[cfg(not(feature = "step-test"))]
pub fn assert_step(_: &Solid, _: f64, _: f64) {}
