use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_modeling::*;

struct Profile {
    origin: Point3,
    x: Vector3,
    y: Vector3,
    z: Vector3,
    length: f64,
    radius: f64,
    inner: f64,
    center: Vector2,
    limits: [f64; 5],
    distances: [[f64; 7]; 2],
    sides: [usize; 7],
    caps: [usize; 2],
    budget: f64,
}

impl Profile {
    fn point(&self, p: Vector2, z: f64) -> Point3 {
        self.origin + self.x * p.x + self.y * p.y + self.z * z
    }

    fn section(&self, z: f64) -> ([Vector2; 7], f64, f64) {
        let h: [f64; 7] = std::array::from_fn(|j| {
            (self.distances[0][j] - z).max(0.) + (self.distances[1][j] - (self.length - z)).max(0.)
        });
        let [left, high, low, right, flat] = self.limits;
        let (a, b, l, high, flat) = (
            left + h[0],
            right - h[5],
            low + h[6],
            high - h[1],
            flat - h[4],
        );
        let (outer, inner) = (self.radius - h[2], self.inner + h[3]);
        let p = self.center.normalize() * outer;
        (
            [
                Vector2::new(a, l),
                Vector2::new(a, high),
                Vector2::new((outer * outer - high * high).sqrt(), high),
                p,
                Vector2::new(self.center.x, flat),
                Vector2::new(b, flat),
                Vector2::new(b, l),
            ],
            outer,
            inner,
        )
    }

    fn stations(&self) -> Vec<f64> {
        let mut stations = vec![0., self.length];
        stations.extend(self.distances[0]);
        stations.extend(self.distances[1].map(|d| self.length - d));
        stations.sort_by(f64::total_cmp);
        stations.dedup();
        stations
    }

    fn certify(&self, stations: &[f64]) -> Option<()> {
        let max = |d: &[f64; 7]| d.iter().copied().fold(0., f64::max);
        if max(&self.distances[0]) + max(&self.distances[1]) + self.budget >= self.length {
            return None;
        }
        // Every offset is affine between breakpoints. These conservative interval
        // enclosures bound the square root, hence all roof branch ordering gaps.
        let mut intervals: Vec<_> = stations.windows(2).map(|w| (w[0], w[1], 0)).collect();
        while let Some((a, b, depth)) = intervals.pop() {
            let w = [a, b];
            let (p, ro, ri) = self.section(w[0]);
            let (q, so, si) = self.section(w[1]);
            let min = |j: usize, k: usize| p[j][k].min(q[j][k]);
            let max = |j: usize, k: usize| p[j][k].max(q[j][k]);
            let qlo = (ro.min(so).powi(2) - max(1, 1).powi(2)).sqrt();
            let qhi = (ro.max(so).powi(2) - min(1, 1).powi(2)).sqrt();
            let gaps = [
                ro.min(so),
                ri.min(si),
                min(1, 1),
                qlo,
                qlo - max(0, 0),
                min(3, 0) - qhi,
                self.center.x - max(3, 0),
                min(5, 0) - self.center.x,
                min(4, 1) - max(0, 1),
                min(1, 1) - max(4, 1),
            ];
            if gaps.iter().any(|v| !v.is_finite() || *v <= self.budget) {
                if depth >= 20 {
                    return None;
                }
                let mid = (a + b) / 2.;
                intervals.extend([(a, mid, depth + 1), (mid, b, depth + 1)]);
            }
        }
        Some(())
    }
}

fn recognize(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Option<Profile> {
    if shell.len() != 9 || selected.len() != 14 || shell.iter().any(|f| f.boundaries().len() != 1) {
        return None;
    }
    let data =
        super::planar::collect_neighborhood(shell, selected, "chamfer_solid_edges", false).ok()?;
    if data.original_edges.len() != 21
        || data.vertices.len() != 14
        || data.incident.iter().any(|v| v.len() != 3)
    {
        return None;
    }
    let cylinders: Vec<_> = shell
        .iter()
        .enumerate()
        .filter_map(|(f, face)| match face.oriented_surface().elementary() {
            Some((
                Elementary::Cylinder {
                    origin,
                    axis,
                    radius,
                },
                positive,
            )) => Some((f, origin, axis.normalize(), radius, positive)),
            _ => None,
        })
        .collect();
    if cylinders.len() != 2 {
        return None;
    }
    let &(outer, origin, axis, radius, _) = cylinders.iter().find(|c| c.4)?;
    let &(inner_face, center, inner_axis, inner, _) = cylinders.iter().find(|c| !c.4)?;
    let scale = data
        .vertices
        .iter()
        .map(|v| v.point().distance(origin))
        .fold(radius, f64::max);
    let magnitude = data
        .vertices
        .iter()
        .map(|v| v.point().to_vec().magnitude())
        .fold(0., f64::max);
    let budget = 512. * f64::EPSILON * (magnitude + scale);
    if !budget.is_finite()
        || budget > tol.min(TOLERANCE) / 16.
        || axis.cross(inner_axis).magnitude() * scale > budget
    {
        return None;
    }
    let caps: Vec<_> = data
        .planes
        .iter()
        .enumerate()
        .filter_map(|(f, p)| {
            p.filter(|(n, _)| n.cross(axis).magnitude() * scale <= budget)
                .map(|_| f)
        })
        .collect();
    if caps.len() != 2 || data.planes.iter().flatten().count() != 7 {
        return None;
    }
    let low = data.sides.iter().find_map(|s| {
        if !s.contains(&inner_face) {
            return None;
        }
        s.iter()
            .copied()
            .find(|&f| f != inner_face && !caps.contains(&f) && data.planes[f].is_some())
    })?;
    let y = data.planes[low]?.0;
    let mut z = axis;
    let mut x = y.cross(z).normalize();
    if (center - origin).dot(x) < 0. {
        z = -z;
        x = -x;
    }
    if y.dot(z).abs() * scale > budget {
        return None;
    }
    let mut caps = [caps[0], caps[1]];
    let cap_point = |f: usize| shell[f].boundaries()[0][0].front().point();
    if (cap_point(caps[1]) - cap_point(caps[0])).dot(z) < 0. {
        caps.swap(0, 1);
    }
    let origin = origin + z * (cap_point(caps[0]) - origin).dot(z);
    let length = (cap_point(caps[1]) - origin).dot(z);
    let c = Vector2::new((center - origin).dot(x), (center - origin).dot(y));
    // Along P(t), the inner-cylinder radial residual is bounded by
    // |D-R-r| for the entire affine generator; no center is snapped.
    // Axis/normal errors above contribute at most budget over the full span.
    if (c.magnitude() - radius - inner).abs() > budget {
        return None;
    }
    let mut sides = [usize::MAX; 7];
    sides[2] = outer;
    sides[3] = inner_face;
    sides[4] = low;
    let mut limits = [0.; 5];
    for (f, p) in data.planes.iter().enumerate() {
        if caps.contains(&f) {
            continue;
        }
        let Some((n, _)) = p else { continue };
        let (j, k, dir) = if f == low {
            (4, 4, y)
        } else if n.dot(x) < -0.9 {
            (0, 0, -x)
        } else if n.dot(x) > 0.9 {
            (5, 3, x)
        } else if n.dot(y) > 0.9 {
            (1, 1, y)
        } else {
            (6, 2, -y)
        };
        if (*n - dir).magnitude() * scale > budget || (sides[j] != usize::MAX && sides[j] != f) {
            return None;
        }
        sides[j] = f;
        limits[k] = (cap_point(f) - origin).dot(if j == 0 || j == 5 { x } else { y });
    }
    if sides.contains(&usize::MAX) || (limits[4] - (c.y - inner)).abs() > budget {
        return None;
    }
    let mut ds = [[0.; 7]; 2];
    for k in 0..2 {
        if shell[caps[k]].boundaries()[0].len() != 7 {
            return None;
        }
        for j in 0..7 {
            let e = data
                .sides
                .iter()
                .position(|s| s.contains(&caps[k]) && s.contains(&sides[j]))?;
            let edge = &data.original_edges[e];
            if !selected.contains(&edge.id()) {
                return None;
            }
            let d = distances.get(&edge.id())?;
            if d.iter().any(|d| !d.is_finite() || *d <= budget) || d[0] != d[1] {
                return None;
            }
            ds[k][j] = d[0];
        }
        if ds[k][2] != ds[k][3] || ds[k][3] != ds[k][4] {
            return None;
        }
    }
    let p = Profile {
        origin,
        x,
        y,
        z,
        length,
        radius,
        inner,
        center: c,
        limits,
        distances: ds,
        sides,
        caps,
        budget: budget.max(1e-10 * scale),
    };
    // Check the complete source graph against the derived profile, including the
    // longitudinal edges and both cap contours, without changing source geometry.
    if p.budget > tol.min(TOLERANCE) / 16. {
        return None;
    }
    let original = Profile {
        distances: [[0.; 7]; 2],
        ..p
    };
    for k in 0..2 {
        let z = k as f64 * length;
        let (points, _, _) = original.section(z);
        for j in 0..7 {
            let e = data
                .sides
                .iter()
                .position(|s| s.contains(&p.caps[k]) && s.contains(&p.sides[j]))?;
            let edge = &data.original_edges[e];
            let a = p.point(points[j], z);
            let b = p.point(points[(j + 1) % 7], z);
            if !((edge.front().point().distance(a) <= p.budget
                && edge.back().point().distance(b) <= p.budget)
                || (edge.front().point().distance(b) <= p.budget
                    && edge.back().point().distance(a) <= p.budget))
            {
                return None;
            }
            match edge.curve() {
                Curve::Line(_) if j != 2 && j != 3 => (),
                Curve::Conic(conic) if j == 2 || j == 3 => {
                    let transform = conic.transform();
                    let c = if j == 2 { Vector2::zero() } else { p.center };
                    let r = if j == 2 { p.radius } else { p.inner };
                    let u = transform.x.truncate();
                    let v = transform.y.truncate();
                    let (lo, hi) = conic.range_tuple();
                    if (transform.w.truncate() - p.point(c, z).to_vec()).magnitude() > p.budget
                        || (u.magnitude() - r).abs() > p.budget
                        || (v.magnitude() - r).abs() > p.budget
                        || u.dot(v).abs() / r > p.budget
                        || u.dot(p.z).abs() > p.budget
                        || v.dot(p.z).abs() > p.budget
                        || !lo.is_finite()
                        || !hi.is_finite()
                        || hi - lo <= 0.
                        || hi - lo >= std::f64::consts::PI
                        || transform.x.w != 0.
                        || transform.y.w != 0.
                        || transform.w.w != 1.
                    {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    }
    for (k, edge) in data.original_edges.iter().enumerate() {
        if selected.contains(&edge.id()) {
            continue;
        }
        if !matches!(edge.curve(), Curve::Line(_)) {
            return None;
        }
        let j = (0..7).find(|&j| {
            data.sides[k].contains(&p.sides[j]) && data.sides[k].contains(&p.sides[(j + 6) % 7])
        })?;
        let point = original.section(0.).0[j];
        let a = p.point(point, 0.);
        let b = p.point(point, p.length);
        if !((edge.front().point().distance(a) <= p.budget
            && edge.back().point().distance(b) <= p.budget)
            || (edge.front().point().distance(b) <= p.budget
                && edge.back().point().distance(a) <= p.budget))
        {
            return None;
        }
    }
    if p.sides.iter().any(|&f| shell[f].boundaries()[0].len() != 4) {
        return None;
    }
    Some(p)
}

fn section_curve(p: &Profile, a: &Vertex, b: &Vertex, j: usize, z: f64, radius: f64) -> Curve {
    if j != 2 && j != 3 {
        return Curve::Line(Line(a.point(), b.point()));
    }
    let c = if j == 2 { Vector2::zero() } else { p.center };
    let center = p.point(c, z);
    let angle = |v: &Vertex| {
        let d = v.point() - center;
        d.dot(p.y).atan2(d.dot(p.x))
    };
    let start = angle(a);
    let mut end = angle(b);
    if j == 2 && end > start {
        end -= std::f64::consts::TAU;
    }
    if j == 3 && end < start {
        end += std::f64::consts::TAU;
    }
    let mut curve = Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(
            UnitCircle::<Point3>::new(),
            (start.min(end), start.max(end)),
        ),
        Matrix4::from_cols(
            (p.x * radius).extend(0.),
            (p.y * radius).extend(0.),
            p.z.extend(0.),
            center.to_homogeneous(),
        ),
    ));
    if start > end {
        curve.invert();
    }
    curve
}

fn intersection(
    plane: &Surface,
    curved: &Surface,
    a: Point3,
    b: Point3,
    reach: f64,
    tol: f64,
) -> Option<Curve> {
    let curves = crate::local::try_intersect_surfaces(
        plane,
        ((-reach, reach), (-reach, reach)),
        curved,
        ((0., 1.), (0., std::f64::consts::TAU)),
        tol,
    )
    .ok()?;
    for mut curve in curves {
        if matches!(curve, Curve::Line(_)) {
            if curve.search_parameter(a, None, 100).is_some()
                && curve.search_parameter(b, None, 100).is_some()
            {
                return Some(Curve::Line(Line(a, b)));
            }
            continue;
        }
        let Some(u) = curve.search_parameter(a, None, 100) else {
            continue;
        };
        let Some(v) = curve.search_parameter(b, None, 100) else {
            continue;
        };
        let (lo, hi) = curve.range_tuple();
        if u.min(v) < lo - TOLERANCE || u.max(v) > hi + TOLERANCE {
            continue;
        }
        if u.max(v) < hi {
            curve.cut(u.max(v));
        }
        if u.min(v) > lo {
            curve = curve.cut(u.min(v));
        }
        if u > v {
            curve.invert();
        }
        return Some(curve);
    }
    None
}

pub(super) fn construct(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    distances: &HashMap<EdgeID, [f64; 2]>,
    tol: f64,
) -> Option<Shell> {
    let p = recognize(shell, selected, distances, tol)?;
    let stations = p.stations();
    p.certify(&stations)?;
    let vertices: Vec<[Vertex; 7]> = stations
        .iter()
        .map(|&z| p.section(z).0.map(|v| Vertex::new(p.point(v, z))))
        .collect();
    let contours: Vec<Vec<Edge>> = stations
        .iter()
        .enumerate()
        .map(|(k, &z)| {
            let (_, ro, ri) = p.section(z);
            (0..7)
                .map(|j| {
                    Edge::new(
                        &vertices[k][j],
                        &vertices[k][(j + 1) % 7],
                        section_curve(
                            &p,
                            &vertices[k][j],
                            &vertices[k][(j + 1) % 7],
                            j,
                            z,
                            if j == 2 { ro } else { ri },
                        ),
                    )
                })
                .collect()
        })
        .collect();
    let mut surfaces: Vec<Option<Surface>> =
        shell.iter().map(|f| Some(f.oriented_surface())).collect();
    surfaces.resize(23, None);
    let mut boundaries: Vec<Vec<Edge>> = vec![vec![]; 23];
    boundaries[p.caps[0]] = contours[0].clone();
    boundaries[p.caps[1]] = contours.last()?.iter().rev().map(Edge::inverse).collect();
    for k in 0..stations.len() - 1 {
        let mid = (stations[k] + stations[k + 1]) / 2.;
        let ids: [usize; 7] = std::array::from_fn(|j| {
            if mid < p.distances[0][j] {
                9 + j
            } else if mid > p.length - p.distances[1][j] {
                16 + j
            } else {
                p.sides[j]
            }
        });
        for j in 0..7 {
            let id = ids[j];
            if surfaces[id].is_some() {
                continue;
            }
            let a = vertices[k][j].point();
            let b = vertices[k][(j + 1) % 7].point();
            let c = vertices[k + 1][j].point();
            let mut surface = if j == 2 || j == 3 {
                let center = if j == 2 { Vector2::zero() } else { p.center };
                let cap = usize::from(id >= 16);
                let d = p.distances[cap][j];
                let (za, zb) = if cap == 0 {
                    (0., d)
                } else {
                    (p.length - d, p.length)
                };
                let radius = |z| {
                    let (_, ro, ri) = p.section(z);
                    if j == 2 {
                        ro
                    } else {
                        ri
                    }
                };
                Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
                    Curve::Line(Line(
                        p.point(center + Vector2::new(radius(za), 0.), za),
                        p.point(center + Vector2::new(radius(zb), 0.), zb),
                    )),
                    p.point(center, 0.),
                    p.z,
                )))
            } else {
                Surface::Plane(Plane::new(a, c, b))
            };
            let uv = surface.search_nearest_parameter(a, None, 100)?;
            let outward = (c - a).cross(b - a);
            if surface.normal(uv.0, uv.1).dot(outward) < 0. {
                surface.invert();
            }
            surfaces[id] = Some(surface);
        }
        let seams: Vec<Edge> = (0..7)
            .map(|j| {
                let a = &vertices[k][j];
                let b = &vertices[k + 1][j];
                let curve = if j == 2 {
                    // Use a fresh analytic revolution with a unit axial domain even
                    // when the retained imported cylinder uses a different chart.
                    let (_, ro, _) = p.section(stations[k]);
                    let (_, so, _) = p.section(stations[k + 1]);
                    let curved =
                        Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
                            Curve::Line(Line(
                                p.point(Vector2::new(ro, 0.), stations[k]),
                                p.point(Vector2::new(so, 0.), stations[k + 1]),
                            )),
                            p.origin,
                            p.z,
                        )));
                    intersection(
                        surfaces[ids[1]].as_ref()?,
                        &curved,
                        a.point(),
                        b.point(),
                        4. * (p.length + p.radius),
                        tol,
                    )?
                } else {
                    Curve::Line(Line(a.point(), b.point()))
                };
                Some(Edge::new(a, b, curve))
            })
            .collect::<Option<_>>()?;
        for j in 0..7 {
            boundaries[ids[j]].extend([
                contours[k][j].inverse(),
                seams[j].clone(),
                contours[k + 1][j].clone(),
                seams[(j + 1) % 7].inverse(),
            ]);
        }
    }
    let mut result = Shell::new();
    for (edges, surface) in boundaries.into_iter().zip(surfaces) {
        let mut remaining: HashMap<EdgeID, Edge> = HashMap::default();
        for edge in edges {
            if remaining.remove(&edge.id()).is_none() {
                remaining.insert(edge.id(), edge);
            }
        }
        let first = remaining.values().next()?.clone();
        remaining.remove(&first.id());
        let mut wire = Wire::from(vec![first]);
        while !remaining.is_empty() {
            let id = wire.back()?.back().id();
            let next = remaining.values().find(|e| e.front().id() == id)?.clone();
            remaining.remove(&next.id());
            wire.push_back(next);
        }
        result.push(Face::try_new(vec![wire], surface?).ok()?);
    }
    (result.shell_condition() == ShellCondition::Closed).then_some(result)
}
