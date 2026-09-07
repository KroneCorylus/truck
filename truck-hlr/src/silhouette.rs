use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_modeling::*;

/// A mesh edge, by the position indices of its ends, the smaller first.
type Key = (usize, usize);

/// Near-zero values of `n · d` are assigned this positive sign, so a face perpendicular to
/// the view everywhere, such as a cylinder wall seen along its axis, has no contour.
const SNAP: f64 = 1.0e-9;

/// The silhouette curves of a face: where the normal of `surface` is perpendicular to
/// `direction`, traced over the triangles of `mesh`, the face's tessellation in the parameters
/// of `surface`, so the curves are trimmed to the face. Each crossing of a triangle edge is
/// refined on the exact normal. Rulings of cylinders and cones come back as lines and great
/// circles of spheres as arcs; anything else is a cubic B-spline through the crossings. Curves
/// shorter than `tol` are dropped.
pub(crate) fn silhouettes(
    surface: &Surface,
    mesh: &PolygonMesh,
    direction: Vector3,
    tol: f64,
) -> Vec<Curve> {
    let d = direction.normalize();
    let uv_coords = mesh.uv_coords();
    let mut samples: Vec<Option<(Point2, f64)>> = vec![None; mesh.positions().len()];
    for vertex in mesh.face_iter().flatten() {
        if samples[vertex.pos].is_none() {
            let Some(uv) = vertex.uv.map(|i| Point2::from_vec(uv_coords[i])) else {
                return Vec::new();
            };
            let f = surface.normal(uv.x, uv.y).dot(d);
            let f = if f.abs() < SNAP { SNAP } else { f };
            samples[vertex.pos] = Some((uv, f));
        }
    }
    let sample = |i: usize| samples[i].expect("vertex of a triangle");

    let mut crossings: HashMap<Key, Point2> = HashMap::new();
    let mut segments: Vec<(Key, Key)> = Vec::new();
    for triangle in mesh.faces().triangle_iter() {
        let idx = [triangle[0].pos, triangle[1].pos, triangle[2].pos];
        let mut ends = Vec::with_capacity(2);
        for k in 0..3 {
            let (i, j) = (idx[k], idx[(k + 1) % 3]);
            if (sample(i).1 > 0.0) != (sample(j).1 > 0.0) {
                let key = (i.min(j), i.max(j));
                crossings
                    .entry(key)
                    .or_insert_with(|| crossing(surface, d, sample(key.0), sample(key.1)));
                ends.push(key);
            }
        }
        if let [a, b] = ends[..] {
            segments.push((a, b));
        }
    }

    chains(&segments)
        .into_iter()
        .filter_map(|chain| {
            let mut points: Vec<Point3> = Vec::with_capacity(chain.len());
            for key in chain {
                let uv = crossings[&key];
                let p = surface.subs(uv.x, uv.y);
                if points.last().is_none_or(|q| !q.near(&p)) {
                    points.push(p);
                }
            }
            let length: f64 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
            (length >= tol).then(|| exact_or_fitted(surface, d, points))
        })
        .collect()
}

/// The parameter on the mesh edge from `uv0` to `uv1` where `n · d` changes sign, by bisection
/// on the exact normal; the end nearer zero when the exact values do not change sign.
fn crossing(
    surface: &Surface,
    d: Vector3,
    (uv0, _): (Point2, f64),
    (uv1, _): (Point2, f64),
) -> Point2 {
    let at = |t: f64| uv0 + (uv1 - uv0) * t;
    let g = |t: f64| surface.normal(at(t).x, at(t).y).dot(d);
    let (mut a, mut b) = (0.0, 1.0);
    let (ga, gb) = (g(a), g(b));
    if (ga > 0.0) == (gb > 0.0) {
        return if ga.abs() <= gb.abs() { uv0 } else { uv1 };
    }
    for _ in 0..60 {
        let m = (a + b) / 2.0;
        if (g(m) > 0.0) == (ga > 0.0) {
            a = m;
        } else {
            b = m;
        }
    }
    at((a + b) / 2.0)
}

/// Links segments sharing an end into polylines of keys; a closed loop repeats its first key.
fn chains(segments: &[(Key, Key)]) -> Vec<VecDeque<Key>> {
    let mut adjacency: HashMap<Key, Vec<usize>> = HashMap::new();
    for (i, (a, b)) in segments.iter().enumerate() {
        adjacency.entry(*a).or_default().push(i);
        adjacency.entry(*b).or_default().push(i);
    }
    let mut used = vec![false; segments.len()];
    let next_from = |key: Key, used: &mut [bool]| -> Option<Key> {
        let i = *adjacency[&key].iter().find(|&&i| !used[i])?;
        used[i] = true;
        let (a, b) = segments[i];
        Some(if a == key { b } else { a })
    };
    let mut chains = Vec::new();
    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut chain = VecDeque::from([segments[start].0, segments[start].1]);
        while let Some(key) = next_from(*chain.back().unwrap(), &mut used) {
            chain.push_back(key);
        }
        while let Some(key) = next_from(*chain.front().unwrap(), &mut used) {
            chain.push_front(key);
        }
        chains.push(chain);
    }
    chains
}

fn exact_or_fitted(surface: &Surface, d: Vector3, points: Vec<Point3>) -> Curve {
    match surface.elementary() {
        Some((Elementary::Cylinder { .. } | Elementary::Cone { .. }, _)) => {
            Line(points[0], points[points.len() - 1]).into()
        }
        Some((Elementary::Sphere { center, radius }, _)) => {
            great_circle_arc(center, radius, d, &points)
        }
        _ => interpolate(&points).into(),
    }
}

/// The arc of the great circle of the sphere perpendicular to `d` from the first of `points`
/// through the middle one to the last; the full circle when the first and last coincide.
fn great_circle_arc(center: Point3, radius: f64, d: Vector3, points: &[Point3]) -> Curve {
    let in_plane = |p: Point3| {
        let v = p - center;
        (v - d * v.dot(d)).normalize()
    };
    let x = in_plane(points[0]);
    let mut y = d.cross(x);
    let angle_of = |p: Point3, y: Vector3| {
        let e = in_plane(p);
        let a = e.dot(y).atan2(e.dot(x));
        if a <= 0.0 {
            a + 2.0 * PI
        } else {
            a
        }
    };
    let closed = points[0].near(&points[points.len() - 1]);
    let mut angle = if closed {
        2.0 * PI
    } else {
        angle_of(points[points.len() - 1], y)
    };
    if !closed && angle_of(points[points.len() / 2], y) > angle {
        y = -y;
        angle = 2.0 * PI - angle;
    }
    let transform = Matrix4::from_cols(
        (x * radius).extend(0.0),
        (y * radius).extend(0.0),
        d.extend(0.0),
        center.to_homogeneous(),
    );
    let arc = TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, angle));
    Processor::with_transform(arc, transform).into()
}

/// A cubic B-spline through `points`, parametrised by chord length with averaged knots.
fn interpolate(points: &[Point3]) -> BSplineCurve<Point3> {
    let n = points.len();
    let mut chord = vec![0.0];
    for w in points.windows(2) {
        chord.push(chord[chord.len() - 1] + w[0].distance(w[1]));
    }
    let degree = 3.min(n - 1);
    let mut knots = vec![chord[0]; degree + 1];
    knots.extend((1..n - degree).map(|j| chord[j..j + degree].iter().sum::<f64>() / degree as f64));
    knots.extend(vec![chord[n - 1]; degree + 1]);
    let parameter_points: Vec<_> = chord.into_iter().zip(points.iter().copied()).collect();
    BSplineCurve::interpole(KnotVec::from(knots), parameter_points)
}
