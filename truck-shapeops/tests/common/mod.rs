//! Shared checks and primitives for geometric tests.
#![allow(dead_code)]

pub mod blend;

use rustc_hash::FxHashSet as HashSet;
use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_topology::{shell::ShellCondition, Shell, Solid};

/// Asserts that `solid` is a well-formed closed solid with the expected volume.
///
/// Runs [`assert_topology`], [`assert_mesh_closed`] and [`assert_volume`] in that order.
pub fn assert_solid<C, S>(
    solid: &Solid<Point3, C, S>,
    expected_volume: f64,
    expected_genera: &[usize],
    tol: f64,
) where
    C: PolylineableCurve,
    S: MeshableSurface,
{
    assert_topology(solid, expected_genera);
    assert_mesh_closed(solid, tol);
    assert_volume(solid, expected_volume, tol);
}

/// Asserts, per boundary shell, that the shell is `Closed`, every face boundary is a closed wire,
/// and the Euler characteristic matches a closed surface of the expected genus. The genera are
/// compared as a sorted multiset since shell order is not meaningful.
pub fn assert_topology<P, C, S>(solid: &Solid<P, C, S>, expected_genera: &[usize]) {
    let mut genera: Vec<usize> = solid.boundaries().iter().map(shell_genus).collect();
    genera.sort_unstable();
    let mut expected_genera = expected_genera.to_vec();
    expected_genera.sort_unstable();
    assert_eq!(genera, expected_genera, "genus of boundary shells");
}

/// Asserts that the tessellation of `solid` at `tol` closes up once coincident vertices are merged
/// and degenerate triangles, such as those at the poles of a sphere, are dropped.
/// This catches faces whose boundary polylines overlap or leave gaps.
pub fn assert_mesh_closed<C, S>(solid: &Solid<Point3, C, S>, tol: f64)
where
    C: PolylineableCurve,
    S: MeshableSurface, {
    let mut mesh = solid.triangulation(tol).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    assert_eq!(
        mesh.shell_condition(),
        ShellCondition::Closed,
        "tessellated mesh is not closed"
    );
}

/// Asserts that the signed volume of the tessellation of `solid` at `tol` matches `expected`.
/// The allowed error is `tol` times the area of the curved faces, which bounds the chord error,
/// with a floor of `TOLERANCE` for rounding. Planar faces tessellate exactly and do not contribute.
pub fn assert_volume<C, S>(solid: &Solid<Point3, C, S>, expected: f64, tol: f64)
where
    C: PolylineableCurve,
    S: MeshableSurface, {
    let meshed = solid.triangulation(tol);
    let volume = meshed.to_polygon().volume();
    let curved_area: f64 = meshed
        .face_iter()
        .filter_map(|face| face.surface())
        .filter(|mesh| !is_planar(mesh))
        .map(|mesh| surface_area(&mesh))
        .sum();
    let allowed = (curved_area * tol).max(TOLERANCE);
    assert!(
        (volume - expected).abs() <= allowed,
        "volume {volume} differs from expected {expected} by more than {allowed}"
    );
}

fn triangles(mesh: &PolygonMesh) -> impl Iterator<Item = [Point3; 3]> + '_ {
    let positions = mesh.positions();
    mesh.faces().triangle_iter().map(move |tri| {
        [
            positions[tri[0].pos],
            positions[tri[1].pos],
            positions[tri[2].pos],
        ]
    })
}

fn surface_area(mesh: &PolygonMesh) -> f64 {
    triangles(mesh)
        .map(|[p, q, r]| (q - p).cross(r - p).magnitude() / 2.0)
        .sum()
}

fn is_planar(mesh: &PolygonMesh) -> bool {
    let mut normals = triangles(mesh)
        .map(|[p, q, r]| (q - p).cross(r - p))
        .filter(|n| !n.so_small())
        .map(|n| n.normalize());
    let Some(first) = normals.next() else {
        return true;
    };
    normals.all(|n| n.cross(first).so_small())
}

/// Checks a shell is closed with consistent loops and returns its genus.
fn shell_genus<P, C, S>(shell: &Shell<P, C, S>) -> usize {
    assert_eq!(
        shell.shell_condition(),
        ShellCondition::Closed,
        "shell is not closed"
    );

    let mut inner_loops = 0;
    for face in shell.face_iter() {
        let boundaries = face.boundaries();
        assert!(!boundaries.is_empty(), "face without boundary");
        for wire in &boundaries {
            assert!(wire.is_closed(), "face boundary is not a closed wire");
        }
        inner_loops += boundaries.len() - 1;
    }

    let vertices: HashSet<_> = shell.vertex_iter().map(|v| v.id()).collect();
    let edges: HashSet<_> = shell.edge_iter().map(|e| e.id()).collect();
    let euler =
        vertices.len() as i64 - edges.len() as i64 + shell.len() as i64 - inner_loops as i64;
    assert!(
        euler <= 2 && euler % 2 == 0,
        "impossible Euler characteristic {euler}"
    );
    ((2 - euler) / 2) as usize
}

/// Volume of a cylinder.
pub fn cylinder_volume(radius: f64, height: f64) -> f64 { PI * radius * radius * height }

/// Volume of a sphere.
pub fn sphere_volume(radius: f64) -> f64 { 4.0 / 3.0 * PI * radius * radius * radius }

/// Volume removed by a constant-radius fillet of length `length` on a right-angle edge.
pub fn fillet_removed_volume(radius: f64, length: f64) -> f64 {
    radius * radius * (1.0 - PI / 4.0) * length
}

/// Primitive solids built with `truck-modeling`.
pub mod modeling {
    use super::PI;
    use truck_modeling::*;

    /// Axis-aligned box.
    pub fn cuboid(min: Point3, max: Point3) -> Solid {
        primitive::cuboid(BoundingBox::from_iter([min, max]))
    }

    /// Cylinder with the given base center, axis direction, radius and height.
    pub fn cylinder(base: Point3, axis: Vector3, radius: f64, height: f64) -> Solid {
        let axis = axis.normalize();
        let radial = if axis.x.abs() < 0.9 {
            axis.cross(Vector3::unit_x()).normalize()
        } else {
            axis.cross(Vector3::unit_y()).normalize()
        };
        let vertex = builder::vertex(base + radial * radius);
        let circle = builder::rsweep(&vertex, base, axis, Rad(7.0), 2);
        let disk = builder::try_attach_plane(&[circle]).unwrap();
        builder::tsweep(&disk, axis * height)
    }

    /// Sphere with the given center and radius.
    pub fn sphere(center: Point3, radius: f64) -> Solid {
        let v = builder::vertex(center + Vector3::unit_y() * radius);
        let wire: Wire = builder::rsweep(&v, center, Vector3::unit_x(), Rad(PI), 3);
        let shell = builder::cone(&wire, Vector3::unit_y(), Rad(7.0), 4);
        Solid::new(vec![shell])
    }
}
