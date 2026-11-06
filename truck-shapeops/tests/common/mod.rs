//! Shared checks and primitives for geometric tests.
#![allow(dead_code)]

pub mod blend;

use rustc_hash::FxHashSet as HashSet;
use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_topology::{Shell, Solid, shell::ShellCondition};

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
    for (i, shell) in solid.boundaries().iter().enumerate() {
        let mut mesh = shell.triangulation(tol).to_polygon();
        mesh.put_together_same_attrs(TOLERANCE)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(
            mesh.shell_condition(),
            ShellCondition::Closed,
            "tessellated mesh of shell {i} is not closed"
        );
    }
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

/// Asserts the number of distinct vertices, edges and faces over all boundary shells of `solid`.
/// A spurious split of a boundary shows up as extra vertices and edges with the same faces.
pub fn assert_counts<P, C, S>(solid: &Solid<P, C, S>, vertices: usize, edges: usize, faces: usize) {
    let vs: HashSet<_> = solid.vertex_iter().map(|v| v.id()).collect();
    let es: HashSet<_> = solid.edge_iter().map(|e| e.id()).collect();
    let fs: usize = solid.boundaries().iter().map(|shell| shell.len()).sum();
    assert_eq!(
        (vs.len(), es.len(), fs),
        (vertices, edges, faces),
        "(vertices, edges, faces)"
    );
}

/// Convex polytopes `{x : n·x <= d}` with unit outward normals, as independent oracles.
pub mod polytope {
    use truck_meshalgo::prelude::*;

    /// Vertices where three planes meet inside every half-space.
    pub fn vertices(planes: &[(Vector3, f64)]) -> Vec<Point3> {
        let mut points: Vec<Point3> = Vec::new();
        for a in 0..planes.len() {
            for b in a + 1..planes.len() {
                for c in b + 1..planes.len() {
                    let [(n, d), (m, e), (l, f)] = [planes[a], planes[b], planes[c]];
                    let matrix = Matrix3::from_cols(n, m, l).transpose();
                    if matrix.determinant().abs() < 1e-12 {
                        continue;
                    }
                    let p = Point3::from_vec(matrix.invert().unwrap() * Vector3::new(d, e, f));
                    if planes.iter().all(|(n, d)| n.dot(p.to_vec()) <= d + 1e-9)
                        && !points.iter().any(|q| q.distance(p) < 1e-9)
                    {
                        points.push(p);
                    }
                }
            }
        }
        points
    }

    /// Volume from `V = Σ d·area / 3` over the faces.
    pub fn volume(planes: &[(Vector3, f64)]) -> f64 {
        let points = vertices(planes);
        planes
            .iter()
            .map(|&(n, d)| {
                let mut face: Vec<_> = points
                    .iter()
                    .map(|p| p.to_vec())
                    .filter(|p| (n.dot(*p) - d).abs() <= 1e-9)
                    .collect();
                if face.len() < 3 {
                    return 0.0;
                }
                let center = face.iter().sum::<Vector3>() / face.len() as f64;
                let x = (face[0] - center).normalize();
                let y = n.cross(x);
                face.sort_by(|a, b| {
                    let angle = |p: &Vector3| (p - center).dot(y).atan2((p - center).dot(x));
                    angle(a).total_cmp(&angle(b))
                });
                let area = (0..face.len())
                    .map(|i| (face[i] - center).cross(face[(i + 1) % face.len()] - center))
                    .sum::<Vector3>()
                    .dot(n)
                    / 2.0;
                d * area / 3.0
            })
            .sum()
    }

    /// The plane through `points`, oriented away from `inside`.
    pub fn plane(points: &[Point3], inside: Point3) -> (Vector3, f64) {
        let n = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize();
        let n = if n.dot(points[0] - inside) < 0.0 {
            -n
        } else {
            n
        };
        (n, n.dot(points[0].to_vec()))
    }
}
