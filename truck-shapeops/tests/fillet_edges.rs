mod common;

use common::{blend::*, *};
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_shapeops::fillet::fillet_edges;

const TOL: f64 = 0.001;

fn edge_ids(shell: &Shell) -> Vec<truck_topology::EdgeID<Curve>> {
    let mut ids: Vec<_> = shell.edge_iter().map(|e| e.id()).collect();
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| seen.insert(*id));
    ids
}

/// Checks contact lines and spherical corner arcs; terminal arcs on end planes stay sharp.
fn assert_tangent(shell: &Shell) {
    for (i, face) in shell.iter().enumerate() {
        for edge in face.edge_iter() {
            let Some(other) = shell
                .iter()
                .skip(i + 1)
                .find(|f| f.edge_iter().any(|e| e.id() == edge.id()))
            else {
                continue;
            };
            if matches!(face.surface(), Surface::Plane(_))
                && matches!(other.surface(), Surface::Plane(_))
            {
                continue;
            }
            let curve = edge.curve();
            if matches!(curve, Curve::NurbsCurve(_))
                && (matches!(face.surface(), Surface::Plane(_))
                    || matches!(other.surface(), Surface::Plane(_)))
            {
                continue;
            }
            let (a, b) = curve.range_tuple();
            for k in 1..8 {
                let p = curve.subs(a + (b - a) * k as f64 / 8.0);
                let normals = [face, other].map(|f| {
                    let surface = f.oriented_surface();
                    let (u, v) = surface.search_parameter(p, None, 100).unwrap();
                    surface.normal(u, v).normalize()
                });
                let angle = normals[0]
                    .cross(normals[1])
                    .magnitude()
                    .atan2(normals[0].dot(normals[1]));
                assert!(angle < 1.0e-6, "normal mismatch {angle} at {p:?}");
            }
        }
    }
}

/// Steiner's formula for the radius-r parallel body of the inset box. The allowed volume
/// error is TOL times curved area (the harness chord-error bound); all patches here are exact.
#[test]
fn all_twelve_box_edges() {
    let (a, b, c, r) = (2.0, 3.0, 4.0, 0.2);
    let original = from_modeling(&modeling::cuboid(Point3::origin(), Point3::new(a, b, c)));
    assert_solid(&original, a * b * c, &[0], TOL);
    let shell = &original.boundaries()[0];
    let result = fillet_edges(shell, &edge_ids(shell), r, TOL).unwrap();
    assert_eq!(result.len(), 26);
    assert_tangent(&result);
    let (x, y, z) = (a - 2.0 * r, b - 2.0 * r, c - 2.0 * r);
    let volume = x * y * z
        + 2.0 * r * (x * y + y * z + z * x)
        + PI * r * r * (x + y + z)
        + 4.0 * PI * r.powi(3) / 3.0;
    assert_solid(&Solid::new(vec![result]), volume, &[0], TOL);
    assert_solid(&original, a * b * c, &[0], TOL);
}

#[test]
fn unblendable_four_edge_vertex() {
    use truck_modeling::{builder, Edge as ModelingEdge, Wire as ModelingWire};
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
        Point3::new(1.0, 1.0, 2.0),
    ];
    let vertices = points.map(builder::vertex);
    let mut edges = std::collections::HashMap::new();
    let mut wire = |indices: &[usize]| -> ModelingWire {
        (0..indices.len())
            .map(|i| {
                let (a, b) = (indices[i], indices[(i + 1) % indices.len()]);
                let e: &ModelingEdge = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| builder::line(&vertices[a.min(b)], &vertices[a.max(b)]));
                if a < b {
                    e.clone()
                } else {
                    e.inverse()
                }
            })
            .collect()
    };
    let faces = [
        vec![3, 2, 1, 0],
        vec![0, 1, 4],
        vec![1, 2, 4],
        vec![2, 3, 4],
        vec![3, 0, 4],
    ]
    .map(|indices| builder::try_attach_plane(&[wire(&indices)]).unwrap());
    let original = from_modeling(&truck_modeling::Solid::new(vec![faces
        .into_iter()
        .collect()]));
    assert_solid(&original, 8.0 / 3.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let before: Vec<_> = shell.vertex_iter().map(|v| v.point()).collect();
    assert!(fillet_edges(shell, &edge_ids(shell), 0.1, TOL).is_none());
    assert_eq!(
        before,
        shell.vertex_iter().map(|v| v.point()).collect::<Vec<_>>()
    );
    assert_solid(&original, 8.0 / 3.0, &[0], TOL);
}

#[test]
fn one_edge_ends_on_planes() {
    let original = from_modeling(&modeling::cuboid(
        Point3::origin(),
        Point3::new(2.0, 3.0, 4.0),
    ));
    assert_solid(&original, 24.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let edge = edge_through(shell, Point3::new(1.0, 0.0, 0.0));
    let result = fillet_edges(shell, &[edge.id()], 0.2, TOL).unwrap();
    assert_eq!(result.len(), 7);
    assert_tangent(&result);
    assert_solid(
        &Solid::new(vec![result]),
        24.0 - fillet_removed_volume(0.2, 2.0),
        &[0],
        TOL,
    );
}

#[test]
fn three_edges_meet_at_one_corner() {
    let original = from_modeling(&modeling::cuboid(
        Point3::origin(),
        Point3::new(2.0, 3.0, 4.0),
    ));
    assert_solid(&original, 24.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let ids: Vec<_> = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.5, 0.0),
        Point3::new(0.0, 0.0, 2.0),
    ]
    .map(|p| edge_through(shell, p).id())
    .into();
    let r: f64 = 0.3;
    let result = fillet_edges(shell, &ids, r, TOL).unwrap();
    assert_eq!(result.len(), 10);
    assert_tangent(&result);
    let volume = 24.0 - fillet_removed_volume(r, 9.0) + (2.0 - 7.0 * PI / 12.0) * r.powi(3);
    assert_solid(&Solid::new(vec![result]), volume, &[0], TOL);
}

#[test]
fn rejects_unsupported_selections_and_radius_without_changes() {
    let original = from_modeling(&modeling::cuboid(
        Point3::origin(),
        Point3::new(2.0, 3.0, 4.0),
    ));
    assert_solid(&original, 24.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let ids = edge_ids(shell);
    let points: Vec<_> = shell.vertex_iter().map(|v| v.point()).collect();
    for r in [0.0, -0.1, 1.0, f64::NAN, f64::INFINITY] {
        assert!(fillet_edges(shell, &ids, r, TOL).is_none());
    }
    let pair = [
        edge_through(shell, Point3::new(1.0, 0.0, 0.0)).id(),
        edge_through(shell, Point3::new(0.0, 1.5, 0.0)).id(),
    ];
    assert!(fillet_edges(shell, &pair, 0.2, TOL).is_some());
    assert!(fillet_edges(shell, &[ids[0], ids[0]], 0.2, TOL).is_none());
    let cylinder = from_modeling(&modeling::cylinder(
        Point3::origin(),
        Vector3::unit_z(),
        1.0,
        2.0,
    ));
    assert_solid(&cylinder, 2.0 * PI, &[0], TOL);
    let curved = &cylinder.boundaries()[0];
    assert!(fillet_edges(curved, &edge_ids(curved), 0.2, TOL).is_none());
    assert!(fillet_edges(shell, &edge_ids(curved), 0.2, TOL).is_none());
    assert_eq!(
        points,
        shell.vertex_iter().map(|v| v.point()).collect::<Vec<_>>()
    );
    assert_eq!(ids, edge_ids(shell));
    assert_solid(&original, 24.0, &[0], TOL);
    assert_solid(&cylinder, 2.0 * PI, &[0], TOL);
}

#[test]
fn rounded_tetrahedron() {
    use truck_modeling::{builder, Edge as ModelingEdge, Wire as ModelingWire};
    let points = [
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(1.0, -1.0, -1.0),
        Point3::new(-1.0, 1.0, -1.0),
        Point3::new(-1.0, -1.0, 1.0),
    ];
    let vertices = points.map(builder::vertex);
    let mut edges = std::collections::HashMap::new();
    let faces = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]].map(|mut face| {
        let [a, b, c] = face;
        if (points[b] - points[a])
            .cross(points[c] - points[a])
            .dot(points[a].to_vec())
            < 0.0
        {
            face.swap(1, 2);
        }
        let wire: ModelingWire = (0..3)
            .map(|i| {
                let (a, b) = (face[i], face[(i + 1) % 3]);
                let e: &ModelingEdge = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| builder::line(&vertices[a.min(b)], &vertices[a.max(b)]));
                if a < b {
                    e.clone()
                } else {
                    e.inverse()
                }
            })
            .collect();
        builder::try_attach_plane(&[wire]).unwrap()
    });
    let original = from_modeling(&truck_modeling::Solid::new(vec![faces
        .into_iter()
        .collect()]));
    assert_solid(&original, 8.0 / 3.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let r: f64 = 0.1;
    let result = fillet_edges(shell, &edge_ids(shell), r, TOL).unwrap();
    assert_eq!(result.len(), 14);
    assert_tangent(&result);
    let q = 1.0 - r * 3.0_f64.sqrt();
    let volume = 8.0 / 3.0 * q.powi(3)
        + r * 8.0 * 3.0_f64.sqrt() * q * q
        + r * r * 6.0 * 2.0_f64.sqrt() * q * (-1.0_f64 / 3.0).acos()
        + 4.0 * PI * r.powi(3) / 3.0;
    assert_solid(&Solid::new(vec![result]), volume, &[0], TOL);
}

#[test]
fn tangent_chain_subdivision_is_kept() {
    let original = from_modeling(&modeling::cuboid(
        Point3::origin(),
        Point3::new(2.0, 2.0, 2.0),
    ));
    assert_solid(&original, 8.0, &[0], TOL);
    let shell = &original.boundaries()[0];
    let split = edge_through(shell, Point3::new(1.0, 0.0, 0.0)).absolute_clone();
    let curve = split.curve();
    let (a, b) = curve.range_tuple();
    let t = (a + b) / 2.0;
    let vertex = truck_topology::Vertex::new(curve.subs(t));
    let (e0, e1) = split.cut_with_parameter(&vertex, t).unwrap();
    let pieces: Wire = vec![e0, e1].into();
    let faces = shell
        .iter()
        .map(|face| {
            let boundary: Wire = face.boundaries()[0]
                .iter()
                .flat_map(|edge| {
                    if edge.id() != split.id() {
                        vec![edge.clone()]
                    } else if edge.orientation() {
                        pieces.iter().cloned().collect()
                    } else {
                        pieces.inverse().iter().cloned().collect()
                    }
                })
                .collect();
            Face::new(vec![boundary], face.oriented_surface())
        })
        .collect();
    let subdivided = Solid::new(vec![faces]);
    assert_solid(&subdivided, 8.0, &[0], TOL);
    let shell = &subdivided.boundaries()[0];
    let r: f64 = 0.2;
    let result = fillet_edges(shell, &edge_ids(shell), r, TOL).unwrap();
    assert_eq!(result.len(), 27);
    assert_tangent(&result);
    let x = 2.0 - 2.0 * r;
    let volume = x * x * x + 6.0 * r * x * x + 3.0 * PI * r * r * x + 4.0 * PI * r.powi(3) / 3.0;
    assert_solid(&Solid::new(vec![result]), volume, &[0], TOL);
}
