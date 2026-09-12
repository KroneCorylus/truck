use super::*;
use array_macro::array;
use std::ops::ControlFlow;

/// Find collisions between two polygon meshes and extract interference lines.
///
/// # Details
///
/// The algorithm for mesh extraction is as follows.
/// All the triangles are projected toward the axis to form intervals.
/// By looking at the overlap of these intervals, we can narrow down the interfering triangles.
/// The overlap of the intervals is obtained by sorting the endpoints.
///
/// # Remarks
///
/// We see that the surfaces that are in contact are not interfering with each other.
/// Therefore, polygon meshes included in one plane are not interfering.
pub trait Collision {
    /// If `self` and `other` collide, then returns only one interference line.
    /// Otherwise, returns `None`.
    fn collide_with(&self, other: &PolygonMesh) -> Option<(Point3, Point3)>;
    /// Extract all interference lines between `self` and `other`.
    /// # Remarks
    /// The results is not arranged so that included lines make continuous maximal polyline curve.
    fn extract_interference(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)>;
}

impl Collision for PolygonMesh {
    #[inline(always)]
    fn collide_with(&self, other: &PolygonMesh) -> Option<(Point3, Point3)> {
        are_colliding(self, other)
    }
    #[inline(always)]
    fn extract_interference(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)> {
        collision(self, other)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EndPointType {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct EndPoint {
    entity: f64,
    r#type: EndPointType,
    segnum: usize,
    index: usize,
}

impl EndPoint {
    #[inline(always)]
    fn new(entity: f64, r#type: EndPointType, segnum: usize, index: usize) -> EndPoint {
        EndPoint {
            entity,
            r#type,
            segnum,
            index,
        }
    }
    #[inline(always)]
    fn from_seg(seg: (f64, f64), segnum: usize, index: usize) -> [EndPoint; 2] {
        [
            EndPoint::new(seg.0, EndPointType::Front, segnum, index),
            EndPoint::new(seg.1, EndPointType::Back, segnum, index),
        ]
    }
}

fn tri_to_seg(tri: [Point3; 3], unit: Vector3) -> (f64, f64) {
    let a = tri[0].to_vec().dot(unit);
    let b = tri[1].to_vec().dot(unit);
    let c = tri[2].to_vec().dot(unit);
    (f64::min(f64::min(a, b), c), f64::max(f64::max(a, b), c))
}

fn endpoints(
    tris: &[[Point3; 3]],
    segnum: usize,
    unit: Vector3,
) -> impl Iterator<Item = EndPoint> + '_ {
    tris.iter()
        .enumerate()
        .filter(|(_, tri)| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small())
        .flat_map(move |(i, tri)| EndPoint::from_seg(tri_to_seg(*tri, unit), segnum, i))
}

fn sorted_endpoints(tris0: &[[Point3; 3]], tris1: &[[Point3; 3]], unit: Vector3) -> Vec<EndPoint> {
    let mut res: Vec<EndPoint> = endpoints(tris0, 0, unit)
        .chain(endpoints(tris1, 1, unit))
        .collect();
    res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater));
    res
}

fn disjoint_bdbs(bdb0: &BoundingBox<Point3>, bdb1: &BoundingBox<Point3>) -> bool {
    bdb0.max()[0] < bdb1.min()[0]
        || bdb1.max()[0] < bdb0.min()[0]
        || bdb0.max()[1] < bdb1.min()[1]
        || bdb1.max()[1] < bdb0.min()[1]
        || bdb0.max()[2] < bdb1.min()[2]
        || bdb1.max()[2] < bdb0.min()[2]
}

fn collide_seg_triangle(seg: [Point3; 2], tri: [Point3; 3]) -> Option<Point3> {
    let ab = tri[1] - tri[0];
    let bc = tri[2] - tri[1];
    let ca = tri[0] - tri[2];
    let nor = ab.cross(ca);
    if nor.so_small() {
        return None;
    }
    let ap = seg[0] - tri[0];
    let aq = seg[1] - tri[0];
    let dotapnor = ap.dot(nor);
    let dotaqnor = aq.dot(nor);
    if dotapnor * dotaqnor > 0.0 {
        return None;
    }
    let h = seg[0] + dotapnor / (dotapnor - dotaqnor) * (seg[1] - seg[0]);
    if f64::signum(ab.cross(nor).dot(h - tri[0]) + TOLERANCE2)
        + f64::signum(bc.cross(nor).dot(h - tri[1]) + TOLERANCE2)
        + f64::signum(ca.cross(nor).dot(h - tri[2]) + TOLERANCE2)
        >= 2.0
    {
        Some(h)
    } else {
        None
    }
}

fn collide_triangles(tri0: [Point3; 3], tri1: [Point3; 3]) -> Option<(Point3, Point3)> {
    let mut tuple = (None, None);
    [
        collide_seg_triangle([tri0[0], tri0[1]], tri1),
        collide_seg_triangle([tri0[1], tri0[2]], tri1),
        collide_seg_triangle([tri0[2], tri0[0]], tri1),
        collide_seg_triangle([tri1[0], tri1[1]], tri0),
        collide_seg_triangle([tri1[1], tri1[2]], tri0),
        collide_seg_triangle([tri1[2], tri1[0]], tri0),
    ]
    .into_iter()
    .for_each(|pt| match tuple {
        (None, _) => tuple.0 = pt,
        (Some(_), None) => tuple.1 = pt,
        (Some(ref mut p), Some(ref mut q)) => {
            if let Some(pt) = pt {
                let dist0 = pt.distance2(*p);
                let dist1 = pt.distance2(*q);
                let dist2 = p.distance2(*q);
                if dist2 < dist0 {
                    *q = pt;
                } else if dist2 < dist1 {
                    *p = pt;
                }
            }
        }
    });
    match tuple {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

fn make_pos_tri(poly: &PolygonMesh, face: [StandardVertex; 3]) -> [Point3; 3] {
    array![i => poly.positions()[face[i].pos]; 3]
}

/// The triangles of `poly`, degenerate ones included, so that indices follow the faces.
fn triangles(poly: &PolygonMesh) -> Vec<[Point3; 3]> {
    poly.faces()
        .triangle_iter()
        .map(|face| make_pos_tri(poly, face))
        .collect()
}

/// Calls `f` on the pairs of triangles, one of each mesh, whose bounding boxes overlap, in the
/// order of a sweep along one axis, until `f` breaks.
fn for_each_close_pair(
    poly0: &PolygonMesh,
    poly1: &PolygonMesh,
    mut f: impl FnMut([Point3; 3], [Point3; 3]) -> ControlFlow<()>,
) {
    let unit = match poly0.positions().first() {
        Some(point) => hash::take_one_unit(*point),
        None => return,
    };
    let tris = [triangles(poly0), triangles(poly1)];
    let boxes = tris.each_ref().map(|tris| {
        tris.iter()
            .map(|tri| tri.iter().collect::<BoundingBox<Point3>>())
            .collect::<Vec<_>>()
    });
    // A triangle outside the box of the other mesh meets none of its triangles. It still takes
    // part in the sweep, since the order of the pairs decides how the segments are joined.
    let hulls: [BoundingBox<Point3>; 2] =
        [poly0, poly1].map(|poly| poly.positions().iter().collect());
    let near: [Vec<bool>; 2] = [0, 1].map(|s| {
        boxes[s]
            .iter()
            .map(|bdb| !disjoint_bdbs(bdb, &hulls[1 - s]))
            .collect()
    });
    let mut current = [Vec::<usize>::new(), Vec::<usize>::new()];
    for EndPoint {
        r#type,
        segnum,
        index,
        ..
    } in sorted_endpoints(&tris[0], &tris[1], unit)
    {
        match r#type {
            EndPointType::Front => {
                current[segnum].push(index);
                if !near[segnum][index] {
                    continue;
                }
                let other = 1 - segnum;
                for &i in &current[other] {
                    let [i0, i1] = if segnum == 0 { [index, i] } else { [i, index] };
                    if near[other][i]
                        && !disjoint_bdbs(&boxes[0][i0], &boxes[1][i1])
                        && f(tris[0][i0], tris[1][i1]).is_break()
                    {
                        return;
                    }
                }
            }
            EndPointType::Back => {
                let i = current[segnum]
                    .iter()
                    .position(|idx| *idx == index)
                    .unwrap();
                current[segnum].swap_remove(i);
            }
        }
    }
}

fn collision(poly0: &PolygonMesh, poly1: &PolygonMesh) -> Vec<(Point3, Point3)> {
    let mut lines = Vec::new();
    for_each_close_pair(poly0, poly1, |tri0, tri1| {
        lines.extend(collide_triangles(tri0, tri1));
        ControlFlow::Continue(())
    });
    lines
}

fn are_colliding(poly0: &PolygonMesh, poly1: &PolygonMesh) -> Option<(Point3, Point3)> {
    let mut line = None;
    for_each_close_pair(poly0, poly1, |tri0, tri1| {
        line = collide_triangles(tri0, tri1);
        match line {
            Some(_) => ControlFlow::Break(()),
            None => ControlFlow::Continue(()),
        }
    });
    line
}

#[test]
fn collide_triangles_test() {
    let tri0 = [
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let tri1 = [
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(-1.0, -1.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    assert!(collide_triangles(tri0, tri1).is_some());
    let tri0 = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let tri1 = [
        Point3::new(0.0, 0.0, 0.5),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    assert!(collide_triangles(tri0, tri1).is_none());
}

#[test]
fn sweep_finds_every_crossing_pair() {
    let n = 20;
    let positions = (0..=n)
        .flat_map(|i| {
            (0..=n).map(move |j| {
                let (x, y) = (i as f64 / n as f64, j as f64 / n as f64);
                Point3::new(10.0 * x - 5.0, 10.0 * y - 5.0, 0.0)
            })
        })
        .collect();
    let idx = |i: usize, j: usize| i * (n + 1) + j;
    let faces: Faces = (0..n)
        .flat_map(|i| {
            (0..n).flat_map(move |j| {
                [
                    [idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)],
                    [idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)],
                ]
            })
        })
        .collect();
    let grid = PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    );
    let c = Point3::new(0.3, 0.1, 0.05);
    let octahedron = PolygonMesh::new(
        StandardAttributes {
            positions: [
                Vector3::unit_x(),
                -Vector3::unit_x(),
                Vector3::unit_y(),
                -Vector3::unit_y(),
                Vector3::unit_z(),
                -Vector3::unit_z(),
            ]
            .map(|v| c + 0.7 * v)
            .to_vec(),
            ..Default::default()
        },
        [
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ]
        .into_iter()
        .collect(),
    );
    let all_pairs = |poly0: &PolygonMesh, poly1: &PolygonMesh| {
        let proper = |tri: &&[Point3; 3]| !(tri[1] - tri[0]).cross(tri[2] - tri[0]).so_small();
        let tris1 = triangles(poly1);
        triangles(poly0)
            .iter()
            .filter(proper)
            .flat_map(|tri0| {
                tris1.iter().filter(proper).filter_map(|tri1| {
                    let bdb0 = tri0.iter().collect();
                    let bdb1 = tri1.iter().collect();
                    match disjoint_bdbs(&bdb0, &bdb1) {
                        true => None,
                        false => collide_triangles(*tri0, *tri1),
                    }
                })
            })
            .collect::<Vec<_>>()
    };
    let sorted = |lines: Vec<(Point3, Point3)>| {
        let mut keys: Vec<[f64; 6]> = lines
            .iter()
            .map(|(a, b)| [a.x, a.y, a.z, b.x, b.y, b.z])
            .collect();
        keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        keys
    };
    for (poly0, poly1) in [(&grid, &octahedron), (&octahedron, &grid)] {
        let expected = all_pairs(poly0, poly1);
        assert!(!expected.is_empty());
        assert_eq!(sorted(poly0.extract_interference(poly1)), sorted(expected));
        assert!(poly0.collide_with(poly1).is_some());
    }
}
