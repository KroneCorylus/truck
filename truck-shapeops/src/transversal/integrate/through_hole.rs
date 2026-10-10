use std::f64::consts::TAU;
use truck_modeling::*;

use super::SubtractionResult;

const ANGULAR_EPS: f64 = 1.0e-12;

struct Cylinder {
    origin: Point3,
    axis: Vector3,
    radius: f64,
    surface: Surface,
}

struct Prism<'a> {
    shell: &'a Shell,
    caps: [usize; 2],
    heights: [f64; 2],
}

fn parallel(a: Vector3, b: Vector3) -> bool { a.cross(b).magnitude2() <= ANGULAR_EPS * ANGULAR_EPS }

fn cylindrical(surface: &Surface) -> Option<(Elementary, bool)> {
    let Surface::Extruded(extrusion) = surface else {
        return None;
    };
    let Curve::Conic(conic) = extrusion.entity_curve() else {
        return None;
    };
    let arc = CircleArc::new(conic)?;
    if !parallel(arc.x.cross(arc.y), extrusion.extruding_vector().normalize()) {
        return None;
    }
    surface.elementary()
}

impl Cylinder {
    fn height(&self, p: Point3) -> f64 { (p - self.origin).dot(self.axis) }
    fn center(&self, h: f64) -> Point3 { self.origin + h * self.axis }

    fn recognize(tool: &Shell) -> Option<(Self, Prism<'_>)> {
        let surface = tool.iter().find_map(|face| {
            let s = face.surface();
            cylindrical(&s).map(|_| s)
        })?;
        let (
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            },
            _,
        ) = surface.elementary()?
        else {
            return None;
        };
        let cylinder = Self {
            origin,
            axis,
            radius,
            surface,
        };
        if !radius.is_finite() || radius <= TOLERANCE || !axis.magnitude2().is_finite() {
            return None;
        }
        let prism = Prism::recognize(tool, &cylinder)?;
        for (i, face) in prism.shell.iter().enumerate() {
            if prism.caps.contains(&i) {
                if face.boundaries().len() != 1 {
                    return None;
                }
                let mut turn = 0.0;
                for edge in face.boundaries()[0].iter() {
                    let Curve::Conic(conic) = edge.oriented_curve() else {
                        return None;
                    };
                    let arc = CircleArc::new(&conic)?;
                    if !parallel(arc.x.cross(arc.y), axis)
                        || !arc.radius.near(&radius)
                        || (arc.center - cylinder.center(cylinder.height(arc.center))).magnitude()
                            > TOLERANCE
                    {
                        return None;
                    }
                    turn += (arc.range.1 - arc.range.0)
                        * arc.x.cross(arc.y).dot(axis).signum()
                        * if conic.orientation() { 1.0 } else { -1.0 };
                }
                let expected = if i == prism.caps[0] { -TAU } else { TAU };
                if (turn - expected).abs() > ANGULAR_EPS * prism.shell.len() as f64 {
                    return None;
                }
            } else {
                let (
                    Elementary::Cylinder {
                        origin,
                        axis: other,
                        radius: r,
                    },
                    outward,
                ) = cylindrical(&face.surface())?
                else {
                    return None;
                };
                if !parallel(axis, other)
                    || !r.near(&radius)
                    || (origin - cylinder.center(cylinder.height(origin))).magnitude() > TOLERANCE
                    || outward != face.orientation()
                {
                    return None;
                }
            }
        }
        Some((cylinder, prism))
    }
}

impl<'a> Prism<'a> {
    fn recognize(shell: &'a Shell, cylinder: &Cylinder) -> Option<Self> {
        let mut caps = Vec::new();
        for (i, face) in shell.iter().enumerate() {
            let surface = face.surface();
            let (support, forward) = match surface {
                Surface::Plane(plane) => (Elementary::Plane(plane), true),
                _ => cylindrical(&surface)?,
            };
            match support {
                Elementary::Plane(plane) => {
                    let normal = plane.normal();
                    if parallel(normal, cylinder.axis) {
                        let outward = normal.dot(cylinder.axis)
                            * if face.orientation() == forward {
                                1.0
                            } else {
                                -1.0
                            };
                        let height = (plane.origin() - cylinder.origin).dot(normal)
                            / normal.dot(cylinder.axis);
                        caps.push((i, height, outward));
                    } else if normal.dot(cylinder.axis).abs() > ANGULAR_EPS {
                        return None;
                    }
                }
                Elementary::Cylinder { axis, .. } if parallel(axis, cylinder.axis) => {}
                _ => return None,
            }
        }
        if caps.len() != 2 {
            return None;
        }
        caps.sort_by(|a, b| a.1.total_cmp(&b.1));
        let [(lo, a, na), (hi, b, nb)] = caps.as_slice() else {
            return None;
        };
        if !a.is_finite() || !b.is_finite() || b - a <= TOLERANCE || *na >= 0.0 || *nb <= 0.0 {
            return None;
        }
        if shell.vertex_iter().any(|v| {
            let point = v.point();
            let h = cylinder.height(point);
            // Bound the positional error admitted by the angular recognition threshold.
            let extent = (point - cylinder.origin).magnitude().max(cylinder.radius);
            !h.is_finite()
                || extent * ANGULAR_EPS > TOLERANCE / 16.0
                || h < a - TOLERANCE
                || h > b + TOLERANCE
        }) {
            return None;
        }
        Some(Self {
            shell,
            caps: [*lo, *hi],
            heights: [*a, *b],
        })
    }
}

struct CircleArc {
    center: Point3,
    x: Vector3,
    y: Vector3,
    radius: f64,
    range: (f64, f64),
}

type Conic = Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>;

impl CircleArc {
    fn new(conic: &Conic) -> Option<Self> {
        let matrix = conic.transform();
        let (x, y) = (matrix[0].truncate(), matrix[1].truncate());
        let radius = x.magnitude();
        let range = conic.range_tuple();
        if !(0..4).all(|i| (0..4).all(|j| matrix[i][j].is_finite()))
            || !radius.is_finite()
            || radius <= TOLERANCE
            || (y.magnitude() / radius - 1.0).abs() > ANGULAR_EPS
            || (x.dot(y) / radius.powi(2)).abs() > ANGULAR_EPS
            || matrix[0].w != 0.0
            || matrix[1].w != 0.0
            || matrix[3].w != 1.0
            || !range.0.is_finite()
            || !range.1.is_finite()
            || range.1 <= range.0
            || range.1 - range.0 > TAU + ANGULAR_EPS
        {
            return None;
        }
        Some(Self {
            center: Point3::from_vec(matrix[3].truncate()),
            x: x / radius,
            y: y / radius,
            radius,
            range,
        })
    }

    fn distance(&self, point: Point3) -> f64 {
        let at = |t: f64| self.center + self.radius * (self.x * t.cos() + self.y * t.sin());
        let delta = point - self.center;
        let angle = delta.dot(self.y).atan2(delta.dot(self.x));
        let angle = angle + TAU * ((self.range.0 - angle) / TAU).ceil();
        let mut distance = point
            .distance(at(self.range.0))
            .min(point.distance(at(self.range.1)));
        if angle <= self.range.1 {
            distance = distance.min(point.distance(at(angle)));
        }
        distance
    }
}

fn disk_inside(face: &Face, center: Point3, axis: Vector3, radius: f64) -> Option<bool> {
    let angle = |p: Point3, q: Point3| {
        let (p, q) = (p - center, q - center);
        axis.dot(p.cross(q)).atan2(p.dot(q))
    };
    let mut winding = 0.0;
    for edge in face.boundaries().iter().flat_map(|wire| wire.iter()) {
        match edge.oriented_curve() {
            Curve::Line(line) => {
                let delta = line.1 - line.0;
                let length2 = delta.magnitude2();
                if !length2.is_finite() || length2 <= TOLERANCE2 {
                    return None;
                }
                let t = ((center - line.0).dot(delta) / length2).clamp(0.0, 1.0);
                if center.distance(line.0 + t * delta) <= radius + TOLERANCE {
                    return None;
                }
                winding += angle(line.0, line.1);
            }
            Curve::Conic(conic) => {
                let arc = CircleArc::new(&conic)?;
                if !parallel(arc.x.cross(arc.y), axis) || arc.distance(center) <= radius + TOLERANCE
                {
                    return None;
                }
                // Clearance is exact. Chords closer than radius/4 to the arc cannot cross
                // the disk center, so their winding equals the curved boundary's winding.
                let step = 2.0 * (1.0 - radius / (4.0 * arc.radius)).clamp(-1.0, 1.0).acos();
                let count = ((arc.range.1 - arc.range.0) / step).ceil().max(1.0);
                if !count.is_finite() || count > 4096.0 {
                    return None;
                }
                let count = count as usize;
                let mut previous = conic.subs(arc.range.0);
                for i in 1..=count {
                    let point = conic
                        .subs(arc.range.0 + (arc.range.1 - arc.range.0) * i as f64 / count as f64);
                    winding += angle(previous, point);
                    previous = point;
                }
            }
            _ => return None,
        }
    }
    let turns = winding / TAU;
    if !turns.is_finite() || (turns - turns.round()).abs() > TOLERANCE {
        return None;
    }
    match turns.round() as i32 {
        0 => Some(false),
        -1 | 1 => Some(true),
        _ => None,
    }
}

fn section(cylinder: &Cylinder, height: f64, segments: usize) -> Option<Face> {
    let center = cylinder.center(height);
    let surface = &cylinder.surface;
    let mut ranges = [surface.try_range_tuple().0?, surface.try_range_tuple().1?];
    ranges[0].1 = ranges[0].0 + TAU;
    let seed = surface.subs(ranges[0].0, ranges[1].0);
    let delta = seed - cylinder.origin;
    let x = (delta - delta.dot(cylinder.axis) * cylinder.axis).normalize();
    let y = cylinder.axis.cross(x);
    let plane = Plane::new(center, center + x, center + y);
    let r = cylinder.radius * 2.0;
    let mut curves = crate::local::intersect_surfaces(
        &plane.into(),
        ((-r, r), (-r, r)),
        surface,
        (ranges[0], ranges[1]),
        TOLERANCE,
    )?;
    if curves.len() != 1 {
        return None;
    }
    let mut curve = curves.pop()?;
    if !matches!(curve, Curve::Conic(_)) {
        return None;
    }
    let (a, b) = curve.range_tuple();
    let t = (a + b) / 2.0;
    if (curve.subs(t) - center)
        .cross(curve.der(t))
        .dot(cylinder.axis)
        < 0.0
    {
        curve.invert();
    }
    let vertices: Vec<_> = (0..segments)
        .map(|i| builder::vertex(curve.subs(a + (b - a) * i as f64 / segments as f64)))
        .collect();
    let wire: Wire = (0..segments)
        .map(|i| {
            let mut part = curve.clone();
            if i > 0 {
                part = part.cut(a + (b - a) * i as f64 / segments as f64);
            }
            if i + 1 < segments {
                let (start, end) = part.range_tuple();
                part.cut(start + (end - start) / (segments - i) as f64);
            }
            Edge::new(&vertices[i], &vertices[(i + 1) % segments], part)
        })
        .collect();
    Face::try_new(vec![wire], plane.into()).ok()
}

pub(super) fn subtract(
    target: &Solid,
    tool: &Solid,
    tol: f64,
) -> Option<SubtractionResult<Curve, Surface>> {
    let [target_shell] = target.boundaries().as_slice() else {
        return None;
    };
    let cutters = tool
        .boundaries()
        .iter()
        .map(Cylinder::recognize)
        .collect::<Option<Vec<_>>>()?;
    let axis = cutters.first()?.0.axis;
    let mut envelopes: Vec<(Point3, f64)> = Vec::with_capacity(cutters.len());
    for (cylinder, cutter) in &cutters {
        if tol >= cylinder.radius || !parallel(axis, cylinder.axis) {
            return None;
        }
        let [a, b] = cutter.heights;
        let center = cylinder.center(a + (b - a) / 2.);
        // Bound each complete cutter's projection, including the admitted axis tilt.
        // Disjoint disks prove that these shells are separate bodies, never cavities.
        let radius = cylinder.radius + (b - a) / 2. * axis.cross(cylinder.axis).magnitude();
        if !radius.is_finite()
            || envelopes.iter().any(|&(other, r)| {
                let distance = (center - other).cross(axis).magnitude();
                !distance.is_finite() || distance <= radius + r + TOLERANCE
            })
        {
            return None;
        }
        envelopes.push((center, radius));
    }
    Solid::try_new(target.boundaries().clone()).ok()?;
    Solid::try_new(tool.boundaries().clone()).ok()?;
    let mut shell = target_shell.clone();
    let mut cap_boundaries: Vec<Option<Vec<Wire>>> = vec![None; shell.len()];
    let mut removed_material = false;
    for (cylinder, cutter) in &cutters {
        let body = Prism::recognize(target_shell, cylinder)?;
        let [lo, hi] = body.heights;
        if cutter.heights[0] >= lo - TOLERANCE || cutter.heights[1] <= hi + TOLERANCE {
            return None;
        }
        let inside0 = disk_inside(
            &body.shell[body.caps[0]],
            cylinder.center(lo),
            cylinder.axis,
            cylinder.radius,
        )?;
        let inside1 = disk_inside(
            &body.shell[body.caps[1]],
            cylinder.center(hi),
            cylinder.axis,
            cylinder.radius,
        )?;
        if inside0 != inside1 {
            return None;
        }
        if !inside0 {
            continue;
        }
        removed_material = true;
        let disk = section(cylinder, lo, cutter.shell.len().checked_sub(2)?.max(2))?;
        let mut bore: Solid = builder::tsweep(&disk, cylinder.axis * (hi - lo));
        bore.not();
        for face in bore.face_iter() {
            if let Surface::Plane(plane) = face.surface() {
                let side = usize::from(cylinder.height(plane.origin()) > (lo + hi) / 2.0);
                let index = body.caps[side];
                let original = &body.shell[index];
                let mut opening = face.boundaries()[0].clone();
                if !original.orientation() {
                    opening.invert();
                }
                cap_boundaries[index]
                    .get_or_insert_with(|| original.absolute_boundaries().clone())
                    .push(opening);
            } else {
                shell.push(face.clone());
            }
        }
    }
    for (index, boundaries) in cap_boundaries.into_iter().enumerate() {
        if let Some(boundaries) = boundaries {
            let original = &target_shell[index];
            let mut cap = Face::try_new(boundaries, original.surface()).ok()?;
            if !original.orientation() {
                cap.invert();
            }
            shell[index] = cap;
        }
    }
    Some(SubtractionResult {
        solid: Solid::try_new(vec![shell]).ok()?,
        removed_material,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cylinder(center: Point3, radius: f64, height: f64) -> Solid {
        let circle = primitive::circle(
            center + radius * Vector3::unit_x(),
            center,
            Vector3::unit_z(),
            2,
        );
        let disk = builder::try_attach_plane(&[circle]).unwrap();
        builder::tsweep(&disk, height * Vector3::unit_z())
    }

    #[test]
    fn section_accepts_reversed_curve_parameters() {
        let curve = Curve::Conic(
            Processor::new(TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., TAU))).inverse(),
        );
        let cylinder = Cylinder {
            origin: Point3::origin(),
            axis: Vector3::unit_z(),
            radius: 1.,
            surface: Surface::Extruded(ExtrudedCurve::by_extrusion(curve, 4. * Vector3::unit_z())),
        };
        let disk = section(&cylinder, 1., 6).unwrap();
        let solid: Solid = builder::tsweep(&disk, Vector3::unit_z());
        assert!(solid.is_geometric_consistent());
    }

    #[test]
    fn batch_rejects_overlapping_touching_nested_and_unsupported_cutters() {
        let body: Solid = primitive::cuboid(BoundingBox::from_iter([
            Point3::origin(),
            Point3::new(12., 8., 2.),
        ]));
        let first = cylinder(Point3::new(3., 3., -1.), 1., 4.);
        let check = |other: Solid| {
            for reverse in [false, true] {
                let mut shells = vec![first.boundaries()[0].clone(), other.boundaries()[0].clone()];
                if reverse {
                    shells.reverse();
                }
                assert!(subtract(&body, &Solid::new(shells), 0.001).is_none());
            }
        };
        for x in [0., 1., 2., 2. + TOLERANCE / 2.] {
            check(builder::translated(&first, x * Vector3::unit_x()));
        }
        let nested = cylinder(Point3::new(3., 3., -1.), 0.5, 4.);
        check(nested.clone());
        let mut cavity = nested;
        cavity.not();
        check(cavity);
        check(cylinder(Point3::new(8., 3., 1.), 1., 2.));
        check(cylinder(Point3::new(8., 3., 0.), 1., 2.));
        let other = cylinder(Point3::new(8., 3., -1.), 1., 4.);
        check(builder::rotated(
            &other,
            Point3::new(8., 3., 1.),
            Vector3::unit_x(),
            Rad(0.01),
        ));
        check(builder::scaled(
            &other,
            Point3::new(8., 3., 0.),
            Vector3::new(1., 1.1, 1.),
        ));
        check(primitive::cuboid(BoundingBox::from_iter([
            Point3::new(7., 2., -1.),
            Point3::new(9., 4., 3.),
        ])));
        let separated = cylinder(Point3::new(5. + 4. * TOLERANCE, 3., -1.), 1., 4.);
        let batch = Solid::new(vec![
            first.boundaries()[0].clone(),
            separated.boundaries()[0].clone(),
        ]);
        let cut = subtract(&body, &batch, 0.001).unwrap();
        assert!(cut.removed_material);
        assert!(cut.solid.is_geometric_consistent());
    }

    #[test]
    fn ambiguous_and_unsupported_cuts_defer_to_general_boolean() {
        let body: Solid = primitive::cuboid(BoundingBox::from_iter([
            Point3::origin(),
            Point3::new(8., 8., 2.),
        ]));
        for (center, height) in [
            (Point3::new(4., 4., 1.), 2.),   // blind
            (Point3::new(4., 4., 0.), 2.),   // coincident caps
            (Point3::new(1., 4., -1.), 4.),  // tangent outer wall
            (Point3::new(0.5, 4., -1.), 4.), // crosses outer wall
        ] {
            assert!(subtract(&body, &cylinder(center, 1., height), 0.001).is_none());
        }
        let tool = cylinder(Point3::new(4., 4., -1.), 1., 4.);
        let first = subtract(&body, &tool, 0.001).unwrap().solid;
        for offset in [1., 2., 2. + TOLERANCE / 2.] {
            assert!(subtract(
                &first,
                &builder::translated(&tool, offset * Vector3::unit_x()),
                0.001
            )
            .is_none());
        }
        let tilted = builder::rotated(&tool, Point3::new(4., 4., 1.), Vector3::unit_x(), Rad(0.01));
        assert!(subtract(&body, &tilted, 0.001).is_none());
        let ellipse = builder::scaled(&tool, Point3::new(4., 4., 0.), Vector3::new(1., 1.1, 1.));
        assert!(subtract(&body, &ellipse, 0.001).is_none());
        let other = builder::translated(&body, Vector3::new(10., 0., 0.));
        let compound = Solid::new(vec![
            body.boundaries()[0].clone(),
            other.boundaries()[0].clone(),
        ]);
        assert!(subtract(&compound, &tool, 0.001).is_none());
        let cavity: Solid = primitive::cuboid(BoundingBox::from_iter([
            Point3::new(3., 3., 0.5),
            Point3::new(5., 5., 1.5),
        ]));
        let hollow = Solid::new(vec![
            body.boundaries()[0].clone(),
            cavity.boundaries()[0].inverse().into(),
        ]);
        assert!(subtract(&hollow, &tool, 0.001).is_none());
        let mut inverted = body.clone();
        inverted.not();
        assert!(subtract(&inverted, &tool, 0.001).is_none());
    }
}
