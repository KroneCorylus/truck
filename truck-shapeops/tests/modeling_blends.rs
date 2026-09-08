mod common;

use common::{assert_solid, blend::edge_through, fillet_removed_volume, modeling::cuboid};
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

fn ids(solid: &Solid) -> Vec<EdgeID> {
    let mut seen = std::collections::HashSet::new();
    solid
        .edge_iter()
        .map(|e| e.id())
        .filter(|id| seen.insert(*id))
        .collect()
}

fn snapshot(solid: &Solid) -> String { serde_json::to_string(&solid.compress()).unwrap() }

fn assert_history(input: &Solid, result: &BlendResult, generated: usize) {
    assert_eq!(result.generated_faces.len(), generated);
    for (old, new) in &result.modified_faces {
        assert!(input.face_iter().any(|f| f.id() == *old));
        assert!(result.solid.face_iter().any(|f| f.id() == new.id()));
        assert_ne!(*old, new.id());
        assert!(new.edge_iter().count() > 0);
    }
    for face in &result.generated_faces {
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        for edge in face.edge_iter() {
            assert_eq!(
                result
                    .solid
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                    .count(),
                2
            );
        }
    }
    assert_eq!(
        result.modified_faces.len()
            + result.generated_faces.len()
            + input
                .face_iter()
                .filter(|f| result.solid.face_iter().any(|g| f.id() == g.id()))
                .count(),
        result.solid.face_iter().count()
    );
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

fn check(
    input: &Solid,
    result: &BlendResult,
    volume: f64,
    generated: usize,
    cutter: Solid,
    cut_volume: f64,
) {
    assert_history(input, result, generated);
    assert_solid(&result.solid, volume, &[0], TOL);
    assert_step(&result.solid, volume);
    let before = snapshot(&result.solid);
    let cut: Solid = truck_shapeops::and(&result.solid, &cutter, TOL).expect("boolean after blend");
    assert_solid(&cut, cut_volume, &[0], TOL);
    assert_step(&cut, cut_volume);
    assert_eq!(before, snapshot(&result.solid));
}

#[test]
fn box_chamfer_and_chain_fillet_remain_modeling_solids() {
    let input = cuboid(Point3::origin(), Point3::new(2.0, 3.0, 4.0));
    let before = snapshot(&input);
    let edge = edge_through(&input.boundaries()[0], Point3::new(0.0, 0.0, 2.0));
    let cutter = cuboid(Point3::new(-1.0, -1.0, -1.0), Point3::new(3.0, 4.0, 2.0));
    let chamfer = chamfer_solid_edge(&input, edge.id(), 0.2, 0.3, TOL).unwrap();
    check(
        &input,
        &chamfer,
        24.0 - 4.0 * 0.2 * 0.3 / 2.0,
        1,
        cutter.clone(),
        12.0 - 2.0 * 0.2 * 0.3 / 2.0,
    );
    let fillet = fillet_solid_along_wire(&input, &vec![edge].into(), 0.2, TOL).unwrap();
    check(
        &input,
        &fillet,
        24.0 - fillet_removed_volume(0.2, 4.0),
        1,
        cutter,
        12.0 - fillet_removed_volume(0.2, 2.0),
    );
    assert_eq!(before, snapshot(&input));
}

#[test]
fn twelve_edges_with_spherical_corners_remain_modeling_solids() {
    let input = cuboid(Point3::origin(), Point3::new(2.0, 3.0, 4.0));
    let before = snapshot(&input);
    let r: f64 = 0.2;
    let result = fillet_solid_edges(&input, &ids(&input), r, TOL).unwrap();
    assert_eq!(
        result
            .generated_faces
            .iter()
            .filter(|f| matches!(f.surface(), Surface::Sphere(_)))
            .count(),
        8
    );
    let (x, y, z) = (2.0 - 2.0 * r, 3.0 - 2.0 * r, 4.0 - 2.0 * r);
    let volume = x * y * z
        + 2.0 * r * (x * y + y * z + z * x)
        + PI * r * r * (x + y + z)
        + 4.0 * PI * r.powi(3) / 3.0;
    check(
        &input,
        &result,
        volume,
        20,
        cuboid(Point3::new(-1.0, -1.0, -1.0), Point3::new(3.0, 4.0, 2.0)),
        volume / 2.0,
    );
    assert_eq!(before, snapshot(&input));
}

fn split_box() -> (Solid, Wire) {
    let vertices = [(0.0, 0.0), (2.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)));
    let boundary: Wire = (0..5)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 5]))
        .collect();
    let face = builder::try_attach_plane(&[boundary]).unwrap();
    let solid = builder::tsweep(&face, Vector3::unit_z() * 1.5);
    let edges = [1.0, 3.0].map(|x| {
        let edge = edge_through(&solid.boundaries()[0], Point3::new(x, 0.0, 1.5));
        if edge.front().point().x < edge.back().point().x {
            edge
        } else {
            edge.inverse()
        }
    });
    (solid, edges.into_iter().collect())
}

#[derive(Clone, Copy, Debug)]
struct Radius {
    start: f64,
    slope: f64,
}
impl ScalarFunctionD1 for Radius {
    fn der_n(&self, n: usize, t: f64) -> f64 {
        match n {
            0 => self.start + self.slope * t,
            1 => self.slope,
            _ => 0.0,
        }
    }
}

#[test]
fn constant_and_variable_radius_tangent_chains() {
    let (input, wire) = split_box();
    let before = snapshot(&input);
    for slope in [0.0, 0.15] {
        let radius = Radius { start: 0.2, slope };
        let result = fillet_solid_along_wire(&input, &wire, radius, TOL).unwrap();
        // Integral of R(x)^2 over the retained length; each wire edge spans two model units.
        let removed = |length: f64| {
            let (a, b) = (radius.start, radius.slope / 2.0);
            (1.0 - PI / 4.0)
                * (a * a * length + a * b * length.powi(2) + b * b * length.powi(3) / 3.0)
        };
        check(
            &input,
            &result,
            12.0 - removed(4.0),
            2,
            cuboid(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.25, 3.0, 2.0)),
            1.25 * 2.0 * 1.5 - removed(1.25),
        );
    }
    assert_eq!(before, snapshot(&input));
}

#[test]
fn unsupported_operations_leave_geometry_and_ids_unchanged() {
    let (input, wire) = split_box();
    let before = snapshot(&input);
    let before_ids = ids(&input);
    for bad in [0.0, -0.1, f64::NAN, f64::INFINITY] {
        assert!(fillet_solid_along_wire(&input, &wire, bad, TOL).is_none());
        assert!(fillet_solid_along_wire(&input, &wire, 0.2, bad).is_none());
        assert!(fillet_solid_edges(&input, &before_ids, bad, TOL).is_none());
        assert!(chamfer_solid_edge(&input, wire[0].id(), bad, 0.2, TOL).is_none());
    }
    let other = cuboid(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
    assert!(fillet_solid_edges(&input, &ids(&other), 0.2, TOL).is_none());
    assert!(chamfer_solid_edge(&input, ids(&other)[0], 0.2, 0.2, TOL).is_none());
    assert!(fillet_solid_along_wire(&input, &Wire::new(), 0.2, TOL).is_none());
    assert!(fillet_solid_along_wire(
        &input,
        &wire,
        Radius {
            start: 0.2,
            slope: -0.2
        },
        TOL
    )
    .is_none());
    assert!(fillet_solid_edges(&input, &before_ids, 10.0, TOL).is_none());
    assert!(chamfer_solid_edge(&input, wire[0].id(), 10.0, 10.0, TOL).is_none());
    assert_eq!(before, snapshot(&input));
    assert_eq!(before_ids, ids(&input));
}

#[test]
fn curved_chain_survives_transform_serialization_and_step() {
    let input = common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 1.0, 1.0);
    let cap = input
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| v.point().z.near(&1.0)))
        .unwrap();
    let wire = cap.boundaries()[0].clone();
    let result = fillet_solid_along_wire(&input, &wire, 0.2, TOL).unwrap();
    let r: f64 = 0.2;
    let area = r * r * (1.0 - PI / 4.0);
    let centroid = r * (10.0 - 3.0 * PI) / (12.0 - 3.0 * PI);
    let expected = PI - 2.0 * PI * (1.0 - centroid) * area;
    assert_solid(&result.solid, expected, &[0], TOL);
    assert_step(&result.solid, expected);
    let transform = Matrix4::from_translation(Vector3::new(1.0, -2.0, 0.5))
        * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
        * Matrix4::from_scale(1.5);
    let transformed = builder::transformed(&result.solid, transform);
    let serialized = snapshot(&transformed);
    let restored = Solid::extract(serde_json::from_str(&serialized).unwrap()).unwrap();
    assert_solid(&restored, expected * 1.5_f64.powi(3), &[0], TOL);
    assert_step(&restored, expected * 1.5_f64.powi(3));
    for face in restored.face_iter() {
        let surface = face.surface();
        if let Surface::Fillet(_) = &surface {
            let (urange, vrange) = surface.try_range_tuple();
            let (a, b) = urange.unwrap();
            let (c, d) = vrange.unwrap();
            let (u, v) = ((a + b) / 2.0, (c + d) / 2.0);
            let inverse = surface.inverse();
            assert_near!(surface.subs(u, v), inverse.subs(v, u));
            assert_near!(surface.normal(u, v), -inverse.normal(v, u));
        }
    }
}
