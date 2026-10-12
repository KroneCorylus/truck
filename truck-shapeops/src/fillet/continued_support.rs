use super::along_wire::EdgeRadius;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_geometry::prelude::ContactCircle;
use truck_modeling::*;

struct Patch {
    sides: [usize; 2],
    surface: ApproxFilletSurface<Surface, Surface>,
}

struct Crossing {
    parameter: f64,
    neighbor: usize,
    uv: Point2,
}

fn crossings(
    shell: &Shell,
    face: usize,
    contact: &Curve,
    exact: impl Fn(f64) -> Option<Point3>,
    selected: &HashSet<EdgeID>,
    normal: Vector3,
    tol: f64,
) -> Option<Vec<Crossing>> {
    let (parameters, points) = contact.parameter_division(contact.range_tuple(), tol / 4.);
    let mut result = Vec::new();
    for edge in shell[face]
        .edge_iter()
        .filter(|e| !selected.contains(&e.id()))
    {
        let curve = edge.curve();
        let (bounds, vertices) = curve.parameter_division(curve.range_tuple(), tol / 4.);
        for (p, t) in points.windows(2).zip(parameters.windows(2)) {
            for (q, s) in vertices.windows(2).zip(bounds.windows(2)) {
                let (a, b, d) = (p[1] - p[0], q[1] - q[0], q[0] - p[0]);
                let determinant = a.cross(b).dot(normal);
                if determinant.abs() <= f64::EPSILON * a.magnitude() * b.magnitude() {
                    continue;
                }
                let u = d.cross(b).dot(normal) / determinant;
                let v = d.cross(a).dot(normal) / determinant;
                if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                    continue;
                }
                // Chords bracket the crossing; the rolling contacts locate its actual station.
                let range = contact.range_tuple();
                let (parameter, _) = super::along_wire::intersect_samples(
                    range,
                    curve.range_tuple(),
                    (t[0] + u * (t[1] - t[0]), s[0] + v * (s[1] - s[0])),
                    &exact,
                    |t| Some(curve.subs(t)),
                    |t| {
                        let h = (range.1 - range.0) * 1e-5;
                        let a = (t - h).max(range.0);
                        let b = (t + h).min(range.1);
                        Some((exact(b)? - exact(a)?) / (b - a))
                    },
                    |t| Some(curve.der(t)),
                )?;
                if result
                    .iter()
                    .any(|c: &Crossing| (c.parameter - parameter).abs() < 1e-9)
                {
                    continue;
                }
                let neighbor = shell.iter().enumerate().find_map(|(i, f)| {
                    (i != face && f.edge_iter().any(|e| e.id() == edge.id())).then_some(i)
                })?;
                let surface = shell[neighbor].oriented_surface();
                let point = exact(parameter)?;
                let uv = surface.search_parameter(point, None, 100)?;
                if surface.normal(uv.0, uv.1).dot(normal) < 1. - 1e-7 {
                    return None;
                }
                result.push(Crossing {
                    parameter,
                    neighbor,
                    uv: uv.into(),
                });
            }
        }
    }
    Some(result)
}

pub(super) fn fillet<R: ScalarFunctionD1>(
    shell: &Shell,
    wire: &Wire,
    radius: R,
    tol: f64,
) -> Option<Shell> {
    if !wire.is_cyclic() {
        return None;
    }
    let selected: HashSet<_> = wire.iter().map(Edge::id).collect();
    let mut patches = Vec::new();
    let mut changed = false;
    for (k, edge) in wire.iter().enumerate() {
        let mut sides = [None; 2];
        for (i, face) in shell.iter().enumerate() {
            for candidate in face.edge_iter().filter(|e| e.id() == edge.id()) {
                let side = usize::from(candidate.front() != edge.front());
                if sides[side].replace(i).is_some() {
                    return None;
                }
            }
        }
        let sides = [sides[0]?, sides[1]?];
        let curve = edge.oriented_curve();
        let (a, b) = curve.range_tuple();
        let law = EdgeRadius {
            radius: radius.clone(),
            k: k as f64,
            t0: a,
            scale: 1. / (b - a),
        };
        let rbf = RbfSurface::new(
            curve.clone(),
            shell[sides[0]].oriented_surface(),
            shell[sides[1]].oriented_surface(),
            law.clone(),
        );
        let baseline = ApproxFilletSurface::approx_rolling_ball_fillet(&rbf, (a, b), tol)?;
        let contacts: [Curve; 2] = [
            baseline.side_pcurve0().to_same_geometry(),
            baseline.side_pcurve1().to_same_geometry(),
        ];
        let surfaces = [rbf.surface0(), rbf.surface1()];
        let mut events: [Vec<Crossing>; 2] = [Vec::new(), Vec::new()];
        let mut outer = [Vec::new(), Vec::new()];
        for side in 0..2 {
            if let Surface::Plane(plane) = surfaces[side] {
                outer[side] =
                    super::planar::sample_wire(&shell[sides[side]].boundaries()[0], tol / 4.);
                events[side] = crossings(
                    shell,
                    sides[side],
                    &contacts[side],
                    |t| {
                        let cc = rbf.contact_circle(t)?;
                        Some(if side == 0 {
                            cc.contact_point0().point
                        } else {
                            cc.contact_point1().point
                        })
                    },
                    &selected,
                    plane.normal(),
                    tol,
                )?;
            }
        }
        let mut stations = vec![a, b];
        stations.extend(events.iter().flatten().map(|e| e.parameter.clamp(a, b)));
        stations.sort_by(f64::total_cmp);
        stations.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        for range in stations.windows(2) {
            let mid = (range[0] + range[1]) / 2.;
            let mut next_sides = sides;
            let mut seeds = [None, None];
            for side in 0..2 {
                if let Surface::Plane(plane) = surfaces[side] {
                    if !super::planar::inside(
                        contacts[side].subs(mid),
                        &outer[side],
                        plane.normal(),
                        tol / 2.,
                    ) {
                        let first = events[side].first()?;
                        if events[side].iter().any(|c| c.neighbor != first.neighbor) {
                            return None;
                        }
                        next_sides[side] = first.neighbor;
                        seeds[side] = Some(first.uv);
                        changed = true;
                    }
                }
            }
            let next = next_sides.map(|i| shell[i].oriented_surface());
            let solve = |t| {
                let previous = rbf.contact_circle(t)?;
                let old = [previous.contact_point0(), previous.contact_point1()];
                let sign = (previous.center() - old[0].point)
                    .dot(surfaces[0].normal(old[0].uv.x, old[0].uv.y))
                    .signum();
                ContactCircle::try_from_contacts(
                    (curve.subs(t), curve.der(t)),
                    t,
                    &next[0],
                    &next[1],
                    sign * law.subs(t),
                    std::array::from_fn(|i| seeds[i].unwrap_or(old[i].uv)),
                )
            };
            let surface = ApproxFilletSurface::approx_contact_circles(
                &next[0],
                &next[1],
                (range[0], range[1]),
                tol / 4.,
                solve,
            )?;
            patches.push(Patch {
                sides: next_sides,
                surface,
            });
        }
    }
    if !changed {
        return None;
    }
    assemble(shell, &selected, patches, tol)
}

fn assemble(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    patches: Vec<Patch>,
    tol: f64,
) -> Option<Shell> {
    let n = patches.len();
    let curves: Vec<[Curve; 2]> = patches
        .iter()
        .map(|p| {
            [
                p.surface.side_pcurve0().to_same_geometry(),
                p.surface.side_pcurve1().to_same_geometry(),
            ]
        })
        .collect();
    let mut regions = HashMap::default();
    for (patch, contacts) in patches.iter().zip(&curves) {
        for (side, contact) in contacts.iter().enumerate() {
            let index = patch.sides[side];
            if let std::collections::hash_map::Entry::Vacant(entry) = regions.entry(index) {
                entry.insert(ContactRegion::new(&shell[index], tol / 4.)?);
            }
            let points = contact
                .parameter_division(contact.range_tuple(), tol / 4.)
                .1;
            if !points
                .into_iter()
                .all(|point| regions[&index].contains(point))
            {
                return None;
            }
        }
    }
    let old_vertices: Vec<_> = shell.vertex_iter().collect();
    let mut vertices = Vec::new();
    for i in 0..n {
        let mut pair = Vec::new();
        for (first, last) in curves[i].iter().zip(&curves[(i + n - 1) % n]) {
            let a = first.subs(first.range_tuple().0);
            let b = last.subs(last.range_tuple().1);
            if !a.near(&b) {
                return None;
            }
            let point = a.midpoint(b);
            pair.push(
                old_vertices
                    .iter()
                    .find(|v| v.point().near(&point))
                    .cloned()
                    .unwrap_or_else(|| Vertex::new(point)),
            );
        }
        vertices.push([pair[0].clone(), pair[1].clone()]);
    }
    let mut added = vec![Vec::new(); shell.len()];
    let mut contacts = Vec::new();
    let mut cross = Vec::new();
    for i in 0..n {
        let pair: [Edge; 2] = std::array::from_fn(|s| {
            Edge::new(
                &vertices[i][s],
                &vertices[(i + 1) % n][s],
                curves[i][s].clone(),
            )
        });
        added[patches[i].sides[0]].push(pair[0].clone());
        added[patches[i].sides[1]].push(pair[1].inverse());
        contacts.push(pair);
        let previous = &patches[(i + n - 1) % n].surface;
        cross.push(Edge::new(
            &vertices[i][0],
            &vertices[i][1],
            previous
                .fillet_bezier(previous.range_tuple().1 .1)
                .to_same_geometry(),
        ));
    }
    let mut pieces = HashMap::default();
    for edge in shell.edge_iter() {
        if pieces.contains_key(&edge.id()) {
            continue;
        }
        if selected.contains(&edge.id()) {
            pieces.insert(edge.id(), Vec::new());
            continue;
        }
        let edge = edge.absolute_clone();
        let curve = edge.curve();
        let (a, b) = curve.range_tuple();
        let mut cuts: Vec<_> = vertices
            .iter()
            .flatten()
            .filter_map(|v| {
                if v == edge.front() || v == edge.back() {
                    return None;
                }
                let t = curve.search_parameter(v.point(), None, 100)?;
                (t > a && t < b && curve.subs(t).near(&v.point())).then_some((t, v))
            })
            .collect();
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        cuts.dedup_by(|a, b| a.1 == b.1);
        let id = edge.id();
        let mut tail = edge;
        let mut split = Vec::new();
        for (_, vertex) in cuts {
            let (front, back) = tail.cut(vertex)?;
            split.push(front);
            tail = back;
        }
        split.push(tail);
        pieces.insert(id, split);
    }
    let mut faces = Vec::new();
    for (i, face) in shell.iter().enumerate() {
        let old: Vec<_> = face
            .edge_iter()
            .flat_map(|edge| {
                let split = &pieces[&edge.id()];
                if edge.orientation() {
                    split.clone()
                } else {
                    split.iter().rev().map(Edge::inverse).collect()
                }
            })
            .collect();
        if added[i].is_empty() && face.edge_iter().all(|e| pieces[&e.id()].len() == 1) {
            faces.push(face.clone());
            continue;
        }
        let loops = reconnect(&old, &added[i])?;
        if loops.is_empty() {
            return None;
        }
        let surface = face.oriented_surface();
        let loops = order_loops(loops, &surface, tol)?;
        faces.push(Face::try_new(loops, surface).ok()?);
    }
    for (i, patch) in patches.into_iter().enumerate() {
        let boundary = wire![
            contacts[i][0].inverse(),
            cross[i].clone(),
            contacts[i][1].clone(),
            cross[(i + 1) % n].inverse()
        ];
        faces.push(Face::try_new(vec![boundary], patch.surface.to_same_geometry()).ok()?);
    }
    Some(faces.into())
}

fn reconnect(old: &[Edge], added: &[Edge]) -> Option<Vec<Wire>> {
    let mut next = HashMap::default();
    for edge in old {
        if next.insert(edge.front().id(), edge.clone()).is_some() {
            return None;
        }
    }
    // A contact replaces the original outgoing path through the removed region.
    let mut contact_starts = HashSet::default();
    for edge in added {
        if !contact_starts.insert(edge.front().id()) {
            return None;
        }
        next.insert(edge.front().id(), edge.clone());
    }
    let mut used = HashSet::default();
    let mut loops = Vec::new();
    for start in added.iter().chain(old) {
        if used.contains(&start.id()) {
            continue;
        }
        let mut edge = start.clone();
        let mut wire = Wire::new();
        let mut seen = HashSet::default();
        let closed = loop {
            if used.contains(&edge.id()) || !seen.insert(edge.id()) {
                break false;
            }
            wire.push_back(edge.clone());
            if edge.back() == start.front() {
                break true;
            }
            match next.get(&edge.back().id()) {
                Some(e) => edge = e.clone(),
                None => break false,
            }
        };
        if closed {
            used.extend(seen);
            loops.push(wire);
        } else if added.iter().any(|e| e.id() == start.id()) {
            return None;
        }
    }
    Some(loops)
}

fn order_loops(mut loops: Vec<Wire>, surface: &Surface, tol: f64) -> Option<Vec<Wire>> {
    if let Surface::Plane(plane) = surface {
        let mut outer = None;
        for (i, wire) in loops.iter().enumerate() {
            let points = super::planar::sample_wire(wire, tol);
            let area = super::planar::polygon_area(&points).dot(plane.normal());
            if !super::planar::valid_polygon(&points, plane.normal() * area.signum(), TOLERANCE) {
                return None;
            }
            if area > 0. && outer.replace(i).is_some() {
                return None;
            }
        }
        loops.swap(0, outer?);
    } else if loops.len() != 1 {
        return None;
    }
    Some(loops)
}

struct ContactRegion {
    surface: Surface,
    loops: Vec<Vec<Point3>>,
    reference: Point2,
}

fn unwrap_parameters(surface: &Surface, mut uv: Point2, reference: Point2) -> Point2 {
    for (i, period) in [surface.u_period(), surface.v_period()].iter().enumerate() {
        if let Some(period) = period {
            uv[i] += period * ((reference[i] - uv[i]) / period).round();
        }
    }
    uv
}

impl ContactRegion {
    fn new(face: &Face, tol: f64) -> Option<Self> {
        let surface = face.oriented_surface();
        let mut loops = Vec::new();
        let mut reference = None;
        for boundary in face.boundaries() {
            let mut points = Vec::new();
            let mut hint = reference;
            for point in super::planar::sample_wire(&boundary, tol) {
                let uv = Point2::from(surface.search_parameter(point, hint, 100)?);
                let uv = hint.map_or(uv, |hint| {
                    unwrap_parameters(&surface, uv, Point2::from(hint))
                });
                hint = Some(uv.into());
                points.push(Point3::new(uv.x, uv.y, 0.));
            }
            if points.len() < 3 {
                return None;
            }
            if reference.is_none() {
                let bounds = BoundingBox::from_iter(points.iter().copied());
                let center = bounds.center();
                reference = Some((center.x, center.y));
            }
            loops.push(points);
        }
        Some(Self {
            surface,
            loops,
            reference: reference?.into(),
        })
    }

    fn contains(&self, point: Point3) -> bool {
        let Some(uv) = self
            .surface
            .search_parameter(point, Some(self.reference.into()), 100)
        else {
            return false;
        };
        let uv = unwrap_parameters(&self.surface, uv.into(), self.reference);
        let query = Point3::new(uv.x, uv.y, 0.);
        super::planar::inside(query, &self.loops[0], Vector3::unit_z(), TOLERANCE)
            && self.loops[1..]
                .iter()
                .all(|hole| !super::planar::inside(query, hole, Vector3::unit_z(), TOLERANCE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continued_contacts_respect_a_trimmed_cylinder_and_its_hole_across_the_seam() {
        let surface: Surface = ExtrudedCurve::by_extrusion(
            Curve::Conic(Processor::new(TrimmedCurve::new(
                UnitCircle::new(),
                (0., std::f64::consts::TAU),
            ))),
            Vector3::unit_z(),
        )
        .into();
        let boundary = |points: [Point2; 4]| {
            let vertices = points.map(|p| Vertex::new(surface.subs(p.x, p.y)));
            (0..4)
                .map(|i| {
                    let j = (i + 1) % 4;
                    let curve = PCurve::new(
                        BSplineCurve::new(KnotVec::bezier_knot(1), vec![points[i], points[j]]),
                        surface.clone(),
                    );
                    Edge::new(&vertices[i], &vertices[j], curve.to_same_geometry())
                })
                .collect::<Wire>()
        };
        let outer = boundary([
            Point2::new(6., 0.),
            Point2::new(6.5, 0.),
            Point2::new(6.5, 1.),
            Point2::new(6., 1.),
        ]);
        let hole = boundary([
            Point2::new(6.18, 0.4),
            Point2::new(6.18, 0.6),
            Point2::new(6.32, 0.6),
            Point2::new(6.32, 0.4),
        ]);
        let face = Face::new(vec![outer, hole], surface.clone());
        let region = ContactRegion::new(&face, 0.0001).unwrap();
        for (u, v, expected) in [
            (6.1, 0.5, true),
            (6.4, 0.5, true),
            (6.25, 0.7, true),
            (6.25, 0.5, false),
            (6.6, 0.5, false),
            (6.25, 1.1, false),
        ] {
            assert_eq!(region.contains(surface.subs(u, v)), expected, "({u}, {v})");
        }
    }
}
