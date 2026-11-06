//! Candidate construction only. No dispatch or admission until the paired-trace
//! and runtime arithmetic budgets have been implemented.
use std::collections::HashSet;
use std::f64::consts::FRAC_PI_2;
use truck_modeling::*;
#[cfg(test)]
type Evidence = serde_json::Value;
#[cfg(not(test))]
pub(super) struct Evidence;

// Local degree-four Taylor arithmetic. Invalid domains propagate NaN and reject;
// no interval division or square root silently drops a singular branch.
#[derive(Clone, Copy, Debug)]
struct Enclosure {
    lo: f64,
    hi: f64,
}
fn upward(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        x
    } else if x == 0. {
        f64::from_bits(1)
    } else {
        f64::from_bits(if x > 0. {
            x.to_bits() + 1
        } else {
            x.to_bits() - 1
        })
    }
}
fn downward(x: f64) -> f64 { -upward(-x) }
impl Enclosure {
    fn point(x: f64) -> Self { Self { lo: x, hi: x } }
    fn valid(self) -> bool { self.lo.is_finite() && self.hi.is_finite() && self.lo <= self.hi }
    fn norm(self) -> f64 {
        if self.valid() {
            self.lo.abs().max(self.hi.abs())
        } else {
            f64::INFINITY
        }
    }
    fn sqrt(self) -> Self {
        if !self.valid() || self.lo <= 0. {
            return Self::point(f64::NAN);
        }
        Self {
            lo: downward(self.lo.sqrt()),
            hi: upward(self.hi.sqrt()),
        }
    }
    fn error(self, x: f64) -> f64 {
        if !self.valid() || !x.is_finite() {
            f64::INFINITY
        } else {
            upward((self - Self::point(x)).norm())
        }
    }
}
impl std::ops::Add for Enclosure {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self {
            lo: downward(self.lo + b.lo),
            hi: upward(self.hi + b.hi),
        }
    }
}
impl std::ops::Neg for Enclosure {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
}
impl std::ops::Sub for Enclosure {
    type Output = Self;
    fn sub(self, b: Self) -> Self { self + -b }
}
impl std::ops::Mul for Enclosure {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        if !self.valid() || !b.valid() {
            return Self::point(f64::NAN);
        }
        let p = [
            self.lo * b.lo,
            self.lo * b.hi,
            self.hi * b.lo,
            self.hi * b.hi,
        ];
        Self {
            lo: downward(p.into_iter().fold(f64::INFINITY, f64::min)),
            hi: upward(p.into_iter().fold(f64::NEG_INFINITY, f64::max)),
        }
    }
}
impl std::ops::Div for Enclosure {
    type Output = Self;
    fn div(self, b: Self) -> Self {
        if !b.valid() || (b.lo <= 0. && b.hi >= 0.) {
            return Self::point(f64::NAN);
        }
        self * Self {
            lo: downward(1. / b.hi),
            hi: upward(1. / b.lo),
        }
    }
}
#[derive(Clone, Copy)]
struct Jet([Enclosure; 5]);
impl Jet {
    fn constant(x: Enclosure) -> Self {
        let mut a = [Enclosure::point(0.); 5];
        a[0] = x;
        Self(a)
    }
    fn variable(x: Enclosure) -> Self {
        let mut a = Self::constant(x);
        a.0[1] = Enclosure::point(1.);
        a
    }
    fn scale(self, x: Enclosure) -> Self { Self(self.0.map(|v| v * x)) }
    fn sqrt(self) -> Self {
        let mut b = Self::constant(self.0[0].sqrt());
        for k in 1..5 {
            let sum = (1..k).fold(Enclosure::point(0.), |s, j| s + b.0[j] * b.0[k - j]);
            b.0[k] = (self.0[k] - sum) / (Enclosure::point(2.) * b.0[0]);
        }
        b
    }
    fn inverse(self) -> Self {
        let mut b = Self::constant(Enclosure::point(1.) / self.0[0]);
        for k in 1..5 {
            let sum = (1..=k).fold(Enclosure::point(0.), |s, j| s + self.0[j] * b.0[k - j]);
            b.0[k] = -sum / self.0[0];
        }
        b
    }
}
impl std::ops::Add for Jet {
    type Output = Self;
    fn add(self, b: Self) -> Self { Self(std::array::from_fn(|j| self.0[j] + b.0[j])) }
}
impl std::ops::Mul for Jet {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        Self(std::array::from_fn(|k| {
            (0..=k).fold(Enclosure::point(0.), |s, j| s + self.0[j] * b.0[k - j])
        }))
    }
}

impl Input {
    fn scalar_jets(&self, t: Jet) -> [Jet; 4] {
        let e = Enclosure::point;
        let r = e(self.r);
        let m = e(self.m);
        let factor = e(1.) + m * m;
        let q = Jet::constant(e(self.b) - r * factor.sqrt()) + t.scale(-m);
        let h = (q * q + Jet::constant(-r * r)).sqrt();
        let d = ((q * q).scale(factor) + Jet::constant(-r * r)).sqrt();
        [h, h * d.inverse(), (q * d.inverse()).scale(m), t]
    }
    fn homogeneous_box(&self, scalars: [Enclosure; 4], s: f64, column: usize) -> [Enclosure; 4] {
        let e = Enclosure::point;
        let s = e(s);
        let w = [e(1.), e(1.), e(1.) + s * s][column];
        let sine = [e(0.), s, e(2.) * s][column];
        let cosine = [e(1.), e(1.), e(1.) - s * s][column];
        let [h, a, b, t] = scalars;
        let local = [
            h * w + e(self.r) * a * sine,
            e(self.r) * (w - cosine),
            t * w + e(self.r) * b * sine,
        ];
        let mut world = [e(0.); 4];
        for (j, coordinate) in world.iter_mut().enumerate().take(3) {
            *coordinate = e(self.origin[j]) * w;
            for (k, value) in local.iter().enumerate() {
                *coordinate = *coordinate + e(self.axes[k][j]) * *value;
            }
        }
        world[3] = w;
        world
    }
}

#[derive(Debug)]
struct CanalLedger {
    homogeneous_error: [f64; 2],
    intervals: Vec<f64>,
    controls: f64,
    truncation: f64,
    evaluation: f64,
}
impl CanalLedger {
    fn accepts(&self) -> bool {
        [self.controls, self.truncation, self.evaluation]
            .into_iter()
            .all(|v| v.is_finite() && (0. ..1e-10).contains(&v))
    }
}

// Restricted to the knots and controls supplied here: cubic Bezier spans in T,
// a single quadratic span in s, positive weights, no extrapolation. Coefficients
// are the represented Input values; recognition uncertainty is a separate term.
fn certify_canal(input: &Input, ts: &[f64], s: f64, rows: &[Vec<Vector4>]) -> Option<CanalLedger> {
    let e = Enclosure::point;
    if ts.len() < 2
        || ts.len() > 16385
        || rows.len() != 3 * (ts.len() - 1) + 1
        || !s.is_finite()
        || s < 1e-100
        || rows.iter().any(|row| row.len() != 3)
    {
        return None;
    }
    let mut control = 0_f64;
    let mut weight = 0_f64;
    let mut truncation = 0_f64;
    let mut maximum = 0_f64;
    let mut magnitude = 0_f64;
    let mut wmin = f64::INFINITY;
    let mut wmax = 0_f64;
    let frame = input
        .axes
        .iter()
        .map(|a| upward(upward(a.x.abs() + a.y.abs()) + a.z.abs()))
        .fold(0_f64, f64::max);
    for (index, pair) in ts.windows(2).enumerate() {
        let h = e(pair[1]) - e(pair[0]);
        if !h.valid() || h.lo < 1e-100 || h.hi > 1e100 {
            return None;
        }
        let left = input.scalar_jets(Jet::variable(e(pair[0])));
        let right = input.scalar_jets(Jet::variable(e(pair[1])));
        let boxes = [
            left.map(|a| a.0[0]),
            left.map(|a| a.0[0] + h * a.0[1] / e(3.)),
            right.map(|a| a.0[0] - h * a.0[1] / e(3.)),
            right.map(|a| a.0[0]),
        ];
        for (j, box_row) in boxes.into_iter().enumerate() {
            for (col, p) in rows[3 * index + j].iter().copied().enumerate() {
                if [p.x, p.y, p.z, p.w].iter().any(|x| !x.is_finite()) || p.w <= 0. {
                    return None;
                }
                let b = input.homogeneous_box(box_row, s, col);
                let err = (0..3).fold(e(0.), |sum, k| sum + e(b[k].error(p[k]))).hi;
                control = control.max(err);
                weight = weight.max(b[3].error(p.w));
                let norm = (e(p.x.abs()) + e(p.y.abs()) + e(p.z.abs())).hi;
                maximum = maximum.max(norm);
                magnitude = magnitude.max((e(norm) / e(p.w)).hi);
                wmin = wmin.min(p.w);
                wmax = wmax.max(p.w);
            }
        }
        let bounds = input.scalar_jets(Jet::variable(Enclosure {
            lo: pair[0],
            hi: pair[1],
        }));
        let fourth = (e(bounds[0].0[4].norm())
            + e(input.r) * (e(bounds[1].0[4].norm()) + e(bounds[2].0[4].norm())))
            * e(24.);
        let remainder = (e(frame) * fourth * h * h * h * h / e(384.)).hi;
        if !remainder.is_finite() {
            return None;
        }
        truncation = truncation.max(remainder);
    }
    let project = |numerator: f64, denominator: f64| {
        let lower = e(wmin) - e(denominator);
        if lower.lo <= 0. {
            return f64::INFINITY;
        }
        ((e(numerator) + e(magnitude) * e(denominator)) / lower + e(f64::EPSILON) * e(magnitude)).hi
    };
    let u = e(f64::EPSILON / 2.);
    let ledger = CanalLedger {
        homogeneous_error: [control, weight],
        intervals: ts.to_vec(),
        controls: project(control, weight),
        truncation,
        evaluation: project((e(4096.) * u * e(maximum)).hi, (e(4096.) * u * e(wmax)).hi),
    };
    ledger.accepts().then_some(ledger)
}

// |sum B_i dP_i| <= B max|dP_i|. Both rational denominators
// are bounded below, including the weight perturbation of the ideal controls.
fn expanded_projection(
    basis: Enclosure,
    lower: Enclosure,
    maximum: f64,
    error: [f64; 2],
) -> Option<f64> {
    let e = Enclosure::point;
    let dn = basis * e(error[0]);
    let dw = basis * e(error[1]);
    let ideal_lower = lower - dw;
    if !basis.valid()
        || !lower.valid()
        || lower.lo <= 0.
        || !ideal_lower.valid()
        || ideal_lower.lo <= 0.
    {
        return None;
    }
    let result = (dn + e(maximum) * basis / lower * dw) / ideal_lower;
    result.valid().then_some(result.hi)
}

impl CanalLedger {
    fn expanded_fit(
        &self,
        input: &Input,
        surface: &Surface,
        hull: [Enclosure; 2],
    ) -> Option<(f64, f64, f64, f64)> {
        let Surface::NurbsSurface(s) = surface else {
            return None;
        };
        let e = Enclosure::point;
        let ks = s.knot_vecs();
        let degrees = s.degrees();
        // candidate reverses the surface by swapping its two parameter axes.
        let (time, angular) = if degrees == (3, 2) {
            (0, 1)
        } else if degrees == (2, 3) {
            (1, 0)
        } else {
            return None;
        };
        let knots = [&ks.0, &ks.1];
        let degree = [degrees.0, degrees.1];
        let basis = basis_neighborhood(knots[0], degree[0], hull[0])?
            * basis_neighborhood(knots[1], degree[1], hull[1])?;
        let points = s.control_points();
        let wmin = points
            .iter()
            .flatten()
            .map(|p| p.w)
            .fold(f64::INFINITY, f64::min);
        let wmax = points.iter().flatten().map(|p| p.w).fold(0., f64::max);
        let maximum = points
            .iter()
            .flatten()
            .map(|p| box_norm(boxed(p.truncate())).hi)
            .fold(0., f64::max);
        let lower = e(wmin) - e(wmax) * (basis - e(1.));
        let controls = expanded_projection(basis, lower, maximum, self.homogeneous_error)?;
        if self.intervals.len() < 2 || !hull[angular].valid() {
            return None;
        }
        let frame = input
            .axes
            .iter()
            .map(|a| box_norm(boxed(*a)).hi)
            .fold(0., f64::max);
        let mut remainder = 0_f64;
        for (i, span) in self.intervals.windows(2).enumerate() {
            let lo = if i == 0 {
                span[0].min(hull[time].lo)
            } else {
                span[0]
            };
            let hi = if i + 2 == self.intervals.len() {
                span[1].max(hull[time].hi)
            } else {
                span[1]
            };
            let jets = input.scalar_jets(Jet::variable(Enclosure { lo, hi }));
            let fourth = e(frame)
                * (e(jets[0].0[4].norm())
                    + e(input.r) * (e(jets[1].0[4].norm()) + e(jets[2].0[4].norm())));
            let h = e(span[1]) - e(span[0]);
            // Hermite remainder fourth_derivative/24 (t-a)^2(t-b)^2, valid also
            // outside [a,b] when the derivative bound covers its convex hull.
            let left = (e(span[0]) - e(lo)) * (e(span[1]) - e(lo));
            let right = (e(hi) - e(span[0])) * (e(hi) - e(span[1]));
            let product = e((h * h / e(4.)).hi.max(left.norm()).max(right.norm()));
            let bound = fourth * product * product;
            if !bound.valid() {
                return None;
            }
            remainder = remainder.max(bound.hi);
        }
        (controls.is_finite() && remainder.is_finite())
            .then_some((controls, remainder, basis.hi, lower.lo))
    }
}

#[derive(Clone, Copy, Debug)]
struct ConeLedger {
    gram_departure: f64,
    generator_residual: f64,
    displacement: f64,
    #[cfg(test)]
    generator_domain: [f64; 2],
}

// Compare harmonic coefficients of the stored affine revolution with the
// recovered cone. Coefficients are affine in eta, so their endpoint hull covers
// the entire stored line domain [0,1], for every angle. This also charges axis,
// frame, center and generator errors without assuming classifier exactness.
fn certify_cone(
    surface: &Surface,
    origin: Point3,
    axes: [Vector3; 3],
    slope: f64,
    intercept: f64,
) -> Option<ConeLedger> {
    let Surface::RevolutedCurve(rev) = surface else {
        return None;
    };
    let Curve::Line(line) = rev.entity().entity_curve() else {
        return None;
    };
    let e = Enclosure::point;
    let matrix = rev.transform();
    if matrix.x.w != 0. || matrix.y.w != 0. || matrix.z.w != 0. || matrix.w.w != 1. {
        return None;
    }
    let dot = |a: [Enclosure; 3], b: [Enclosure; 3]| (0..3).fold(e(0.), |sum, i| sum + a[i] * b[i]);
    let vector = |v: Vector3| [e(v.x), e(v.y), e(v.z)];
    let transform = |v: [Enclosure; 3]| {
        std::array::from_fn::<_, 3, _>(|j| {
            (0..3).fold(e(0.), |sum, k| sum + e(matrix[k][j]) * v[k])
        })
    };
    let norm = |v: [Enclosure; 3]| v.into_iter().fold(e(0.), |sum, x| sum + e(x.norm()));
    let axis = vector(rev.entity().axis());
    let center = rev.entity().origin();
    let center_world = transform(vector(center.to_vec()));
    let center_world: [Enclosure; 3] = std::array::from_fn(|j| center_world[j] + e(matrix.w[j]));
    let mut gram = e(0.);
    let scale2 = dot(vector(matrix.x.truncate()), vector(matrix.x.truncate()));
    for i in 0..3 {
        for j in 0..3 {
            let gij = dot(vector(matrix[i].truncate()), vector(matrix[j].truncate()));
            gram = gram + e((gij - if i == j { scale2 } else { e(0.) }).norm());
        }
    }
    let dual = dual_frame(axes)?;
    let local = |p: [Enclosure; 3]| {
        std::array::from_fn::<_, 3, _>(|j| {
            let relative = std::array::from_fn(|k| p[k] - e(origin[k]));
            dot(relative, dual[j])
        })
    };
    let endpoints = [line.0, line.1];
    let transformed = endpoints.map(|p| {
        let v = transform(vector(p.to_vec()));
        std::array::from_fn(|j| v[j] + e(matrix.w[j]))
    });
    let p = local(transformed[0]);
    let radius = (p[0] * p[0] + p[1] * p[1]).sqrt();
    if !radius.valid() {
        return None;
    }
    let radial = [p[0] / radius, p[1] / radius];
    let axis_world = transform(axis);
    let direction = dot(axis_world, vector(axes[2]));
    if !direction.valid() || direction.lo <= 0. {
        return None;
    }
    let mut displacement = 0_f64;
    let mut generator_residual = 0_f64;
    for (index, point) in endpoints.into_iter().enumerate() {
        let relative = std::array::from_fn(|j| e(point[j]) - e(center[j]));
        let axial = dot(relative, axis);
        let mean = transform(axis.map(|a| a * axial));
        let cosine = transform(std::array::from_fn(|j| relative[j] - axis[j] * axial));
        let sine = transform([
            axis[1] * relative[2] - axis[2] * relative[1],
            axis[2] * relative[0] - axis[0] * relative[2],
            axis[0] * relative[1] - axis[1] * relative[0],
        ]);
        let q = local(transformed[index]);
        let rho = e(intercept) + e(slope) * q[2];
        if !rho.valid() || rho.lo <= 0. {
            return None;
        }
        generator_residual = generator_residual
            .max(norm([q[0] - rho * radial[0], q[1] - rho * radial[1], e(0.)]).hi);
        let mean_error = norm(std::array::from_fn(|j| {
            center_world[j] + mean[j] - e(origin[j]) - e(axes[2][j]) * q[2]
        }));
        let cosine_error = norm(std::array::from_fn(|j| {
            cosine[j] - rho * (e(axes[0][j]) * radial[0] + e(axes[1][j]) * radial[1])
        }));
        let sine_error = norm(std::array::from_fn(|j| {
            sine[j] - rho * (-e(axes[0][j]) * radial[1] + e(axes[1][j]) * radial[0])
        }));
        displacement = displacement.max((mean_error + cosine_error + sine_error).hi);
    }
    let ledger = ConeLedger {
        gram_departure: gram.hi,
        generator_residual,
        displacement,
        #[cfg(test)]
        generator_domain: [0., 1.],
    };
    [
        ledger.gram_departure,
        ledger.generator_residual,
        ledger.displacement,
    ]
    .iter()
    .all(|x| x.is_finite())
    .then_some(ledger)
    .filter(|l| l.displacement <= 1e-10)
}

#[derive(Clone)]
struct Input {
    origin: Point3,
    axes: [Vector3; 3],
    // D, S, Q, I, O: retain the original, oriented surface objects.
    supports: [Surface; 5],
    #[cfg(test)]
    cone_ledgers: [ConeLedger; 2],
    #[cfg(feature = "i2-reviewed-glibc")]
    source_faces: [FaceID; 5],
    a: f64,
    b: f64,
    n: f64,
    m: f64,
    r: f64,
}

fn finite(p: Point3) -> bool { p.x.is_finite() && p.y.is_finite() && p.z.is_finite() }

impl Input {
    fn point(&self, p: Point3) -> Point3 {
        self.origin + self.axes[0] * p.x + self.axes[1] * p.y + self.axes[2] * p.z
    }
    fn vector(&self, p: Vector3) -> Vector3 {
        self.axes[0] * p.x + self.axes[1] * p.y + self.axes[2] * p.z
    }
    fn height(&self) -> f64 { (self.b - self.a) / (self.m + self.n) }
    fn ring(&self) -> f64 { self.a + self.n * self.r + self.r * (1. + self.n * self.n).sqrt() }

    // This recognizes a candidate family; these checks are NOT an admission certificate.
    fn recognize(shell: &Shell, selected: &HashSet<EdgeID>, r: f64) -> Option<Self> {
        if shell.len() != 5
            || selected.len() != 3
            || !r.is_finite()
            || r <= 0.
            || shell.iter().any(|f| f.boundaries().len() != 1)
        {
            return None;
        }
        let mut seen = HashSet::new();
        let edges: Vec<_> = shell.edge_iter().filter(|e| seen.insert(e.id())).collect();
        if edges.len() != 9
            || shell
                .vertex_iter()
                .map(|v| v.id())
                .collect::<HashSet<_>>()
                .len()
                != 6
            || selected.iter().any(|id| !seen.contains(id))
        {
            return None;
        }
        let planes: Vec<_> = shell
            .iter()
            .filter_map(|f| {
                let (Elementary::Plane(mut p), orientation) = f.oriented_surface().elementary()?
                else {
                    return None;
                };
                if !orientation {
                    p.invert();
                }
                Some((f, p))
            })
            .collect();
        if planes.len() != 3 {
            return None;
        }
        let base = planes.iter().position(|(f, _)| {
            f.edge_iter()
                .any(|e| selected.contains(&e.id()) && !matches!(e.curve(), Curve::Line(_)))
        })?;
        let far = planes
            .iter()
            .position(|(f, _)| !f.edge_iter().any(|e| selected.contains(&e.id())))?;
        let start = (0..3).find(|&i| i != base && i != far)?;
        let axes = [
            -planes[far].1.normal(),
            -planes[start].1.normal(),
            -planes[base].1.normal(),
        ];
        if axes[0].cross(axes[1]).dot(axes[2]) < 1. - 1e-12
            || axes[0].dot(axes[1]).abs() > 1e-12
            || axes[0].dot(axes[2]).abs() > 1e-12
            || axes[1].dot(axes[2]).abs() > 1e-12
        {
            return None;
        }
        let matrix = Matrix3::from_cols(axes[0], axes[1], axes[2]).transpose();
        let rhs = Vector3::new(
            axes[0].dot(planes[far].1.origin().to_vec()),
            axes[1].dot(planes[start].1.origin().to_vec()),
            axes[2].dot(planes[base].1.origin().to_vec()),
        );
        let origin = Point3::from_vec(matrix.invert()? * rhs);
        let local = |p: Point3| {
            let v = p - origin;
            Point3::new(v.dot(axes[0]), v.dot(axes[1]), v.dot(axes[2]))
        };
        let mut cones = Vec::new();
        for face in shell.iter() {
            if planes.iter().any(|(f, _)| f.id() == face.id()) {
                continue;
            }
            let surface = face.oriented_surface();
            let Surface::RevolutedCurve(ref rev) = surface else {
                continue;
            };
            if !matches!(rev.entity().entity_curve(), Curve::Line(_)) {
                return None;
            }
            // Read the stored line directly: Processor inversion swaps surface UV.
            let generator = rev.entity().entity_curve();
            let p = local(rev.transform().transform_point(generator.subs(0.)));
            let q = local(rev.transform().transform_point(generator.subs(1.)));
            if !finite(p) || !finite(q) || q.z == p.z {
                return None;
            }
            let slope = (q.x.hypot(q.y) - p.x.hypot(p.y)) / (q.z - p.z);
            let intercept = p.x.hypot(p.y) - slope * p.z;
            if !slope.is_finite() || !intercept.is_finite() || intercept <= 0. {
                return None;
            }
            let (_, outward) = surface.elementary()?;
            if outward != (slope < 0.) {
                return None;
            }
            let center = local(rev.transform().transform_point(rev.entity().origin()));
            let axis = rev
                .transform()
                .transform_vector(rev.entity().axis())
                .normalize();
            let scale = intercept.max(r);
            if center.x.hypot(center.y) > scale * 1e-12 || axis.cross(axes[2]).magnitude() > 1e-12 {
                return None;
            }
            cones.push((slope, intercept, surface, face.id()));
        }
        if cones.len() != 2 {
            return None;
        }
        cones.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (outer, inner) = (&cones[0], &cones[1]);
        if outer.0 >= 0. || inner.0 <= 0. || outer.1 <= inner.1 {
            return None;
        }
        let expected = [
            (planes[base].0.id(), inner.3),
            (planes[base].0.id(), planes[start].0.id()),
            (planes[start].0.id(), outer.3),
        ];
        for (a, b) in expected {
            let fa = shell.iter().find(|f| f.id() == a)?;
            let fb = shell.iter().find(|f| f.id() == b)?;
            let shared: Vec<_> = fa
                .edge_iter()
                .filter(|e| fb.edge_iter().any(|q| q.id() == e.id()))
                .collect();
            if shared.len() != 1 || !selected.contains(&shared[0].id()) {
                return None;
            }
        }
        let cone_ledgers = [
            certify_cone(&inner.2, origin, axes, inner.0, inner.1)?,
            certify_cone(&outer.2, origin, axes, outer.0, outer.1)?,
        ];
        #[cfg(not(test))]
        let _ = cone_ledgers;
        let input = Self {
            origin,
            axes,
            supports: [
                planes[base].0.oriented_surface(),
                planes[start].0.oriented_surface(),
                planes[far].0.oriented_surface(),
                inner.2.clone(),
                outer.2.clone(),
            ],
            #[cfg(test)]
            cone_ledgers,
            #[cfg(feature = "i2-reviewed-glibc")]
            source_faces: [
                planes[base].0.id(),
                planes[start].0.id(),
                planes[far].0.id(),
                inner.3,
                outer.3,
            ],
            a: inner.1,
            b: outer.1,
            n: inner.0,
            m: -outer.0,
            r,
        };
        if !finite(origin) || !input.height().is_finite() || input.height() <= 2. * r {
            return None;
        }
        source_profile(
            &input,
            shell,
            [
                planes[base].0.id(),
                planes[start].0.id(),
                planes[far].0.id(),
                inner.3,
                outer.3,
            ],
        )?;
        Some(input)
    }

    fn scalars(&self, t: f64) -> Option<([f64; 3], [f64; 3])> {
        let q = self.b - self.m * t - self.r * (1. + self.m * self.m).sqrt();
        if q <= self.r {
            return None;
        }
        let h = (q * q - self.r * self.r).sqrt();
        let d = ((1. + self.m * self.m) * q * q - self.r * self.r).sqrt();
        let hp = -self.m * q / h;
        let dp = -(1. + self.m * self.m) * self.m * q / d;
        let value = [h, h / d, self.m * q / d];
        let derivative = [
            hp,
            hp / d - h * dp / (d * d),
            -self.m * self.m / d - self.m * q * dp / (d * d),
        ];
        if value
            .iter()
            .chain(derivative.iter())
            .any(|v| !v.is_finite())
        {
            return None;
        }
        Some((value, derivative))
    }

    fn canal(&self, t: f64, phi: f64) -> Option<(Point3, Vector3, Vector3)> {
        let ([h, a, b], [ht, at, bt]) = self.scalars(t)?;
        if !phi.is_finite() || !(-4. ..=4.).contains(&phi) {
            return None;
        }
        let (s, c) = phi.sin_cos();
        Some((
            Point3::new(h + self.r * a * s, self.r * (1. - c), t + self.r * b * s),
            Vector3::new(ht + self.r * at * s, 0., 1. + self.r * bt * s),
            Vector3::new(self.r * a * c, self.r * s, self.r * b * c),
        ))
    }
    fn outer_phi(&self, t: f64) -> Option<f64> {
        let q = self.b - self.m * t - self.r * (1. + self.m * self.m).sqrt();
        let c = -self.r / ((1. + self.m * self.m).sqrt() * q);
        if !c.is_finite() || !(-1. ..0.).contains(&c) {
            return None;
        }
        Some(c.acos())
    }
    fn trim(&self, phi: f64, kind: Trim) -> Option<(Point3, Vector3)> {
        let enclosure = self.trim_box(Enclosure::point(phi), kind)?;
        if upward(enclosure.hi - enclosure.lo) > 1e-10 {
            return None;
        }
        let t = enclosure.lo + (enclosure.hi - enclosure.lo) / 2.;
        let (p, pt, pp) = self.canal(t, phi)?;
        let (ft, fp) = match kind {
            Trim::Ci => {
                let rho = p.x.hypot(p.y);
                (
                    self.n * pt.z - (p.x * pt.x + p.y * pt.y) / rho,
                    self.n * pp.z - (p.x * pp.x + p.y * pp.y) / rho,
                )
            }
            Trim::Bc => (pt.z, pp.z + self.r * phi.cos()),
            Trim::Dc => (pt.z, pp.z),
        };
        if !ft.is_finite() || ft <= 0. {
            return None;
        }
        Some((p, pp - pt * (fp / ft)))
    }
    fn ab(&self, phi: f64) -> Option<(Point3, Vector3)> {
        if !phi.is_finite() || !(-4. ..=4.).contains(&phi) {
            return None;
        }
        let (s, c) = phi.sin_cos();
        let y = self.r * (1. - c);
        let rho = self.ring() - self.r * c;
        let x = (rho * rho - y * y).sqrt();
        if !x.is_finite() || x <= 0. {
            return None;
        }
        Some((
            Point3::new(x, y, self.r * (1. - s)),
            Vector3::new(self.r * s * (rho - y) / x, self.r * s, -self.r * c),
        ))
    }
}

fn trig_enclosure(x: Enclosure, cosine: bool) -> Enclosure {
    if !x.valid() || x.lo < 0. || x.hi > upward(std::f64::consts::PI) {
        return Enclosure::point(f64::NAN);
    }
    let point = |t: f64| {
        let e = Enclosure::point;
        let mut term = e(if cosine { 1. } else { t });
        let mut sum = term;
        for j in 1..=30 {
            let k = if cosine { 2 * j } else { 2 * j + 1 };
            term = term * (-e(t) * e(t)) / e((k * (k - 1)) as f64);
            sum = sum + term;
        }
        sum + Enclosure {
            lo: -1e-48,
            hi: 1e-48,
        }
    };
    let a = point(x.lo);
    let b = point(x.hi);
    let mut result = Enclosure {
        lo: a.lo.min(b.lo),
        hi: a.hi.max(b.hi),
    };
    if !cosine && x.lo <= upward(FRAC_PI_2) && x.hi >= downward(FRAC_PI_2) {
        result.hi = 1.;
    }
    if cosine && x.lo <= 3.141592653589794 && x.hi >= 3.141592653589792 {
        result.lo = -1.;
    }
    result
}
// Invert the decreasing cosine on [0, pi], preserving target uncertainty.
fn inverse_cosine(target: Enclosure) -> Option<Enclosure> {
    if !target.valid() || target.lo <= -1. || target.hi >= 1. {
        return None;
    }
    let mut ends = [0.; 2];
    for (side, end) in ends.iter_mut().enumerate() {
        let (mut lo, mut hi) = (0., std::f64::consts::PI);
        for _ in 0..64 {
            let mid = lo + (hi - lo) / 2.;
            if mid == lo || mid == hi {
                break;
            }
            let v = trig_enclosure(Enclosure::point(mid), true);
            if if side == 0 {
                v.lo > target.hi
            } else {
                v.hi >= target.lo
            } {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        *end = if side == 0 { lo } else { hi };
    }
    Some(Enclosure {
        lo: ends[0],
        hi: ends[1],
    })
}

fn endpoint_allowance(
    input: &Input,
    physical: Enclosure,
    stored: f64,
    point: Point3,
    bounds: impl Fn(Enclosure) -> Option<[Jet; 3]>,
) -> Option<(f64, f64)> {
    if !physical.valid() || !stored.is_finite() {
        return None;
    }
    let e = Enclosure::point;
    let neighborhood = Enclosure {
        lo: physical.lo.min(stored),
        hi: physical.hi.max(stored),
    };
    let jets = bounds(neighborhood)?;
    let exact = input.world_box(bounds(physical)?.map(|v| v.0[0]));
    let correction = (0..3)
        .fold(e(0.), |sum, k| sum + e(exact[k].error(point[k])))
        .hi;
    let speed = (0..3).fold(e(0.), |sum, j| {
        sum + e((0..3)
            .fold(e(0.), |v, k| v + e(input.axes[k][j]) * jets[k].0[1])
            .norm())
    });
    let sliver = (speed * e(physical.error(stored))).hi;
    (correction.is_finite() && sliver.is_finite() && correction + sliver <= 1e-10)
        .then_some((correction, sliver))
}

fn trig_jets(phi: Enclosure) -> (Jet, Jet) {
    let e = Enclosure::point;
    let s = trig_enclosure(phi, false);
    let c = trig_enclosure(phi, true);
    (
        Jet([s, c, -s / e(2.), -c / e(6.), s / e(24.)]),
        Jet([c, -s, -c / e(2.), s / e(6.), c / e(24.)]),
    )
}
impl Input {
    fn canal_jets(&self, t: Jet, s: Jet, c: Jet) -> [Jet; 3] {
        let e = Enclosure::point;
        let [h, a, b, _] = self.scalar_jets(t);
        [
            h + (a * s).scale(e(self.r)),
            (Jet::constant(e(1.)) + c.scale(e(-1.))).scale(e(self.r)),
            t + (b * s).scale(e(self.r)),
        ]
    }
    fn equation_jet(&self, t: Jet, s: Jet, c: Jet, kind: Trim) -> Jet {
        let e = Enclosure::point;
        let p = self.canal_jets(t, s, c);
        match kind {
            Trim::Ci => {
                Jet::constant(e(self.a))
                    + p[2].scale(e(self.n))
                    + (p[0] * p[0] + p[1] * p[1]).sqrt().scale(e(-1.))
            }
            Trim::Bc => p[2] + s.scale(e(self.r)) + Jet::constant(e(-self.r)),
            Trim::Dc => p[2],
        }
    }
    fn trim_box(&self, phi: Enclosure, kind: Trim) -> Option<Enclosure> {
        let e = Enclosure::point;
        let s = Jet::constant(trig_enclosure(phi, false));
        let c = Jet::constant(trig_enclosure(phi, true));
        let (lo, hi) = if matches!(kind, Trim::Ci) {
            (self.height() - 2. * self.r, self.height())
        } else {
            (-self.r, self.r)
        };
        let padding = upward(1024. * f64::EPSILON * (lo.abs() + hi.abs() + self.r));
        let lo = downward(lo - padding);
        let hi = upward(hi + padding);
        let value = |t| {
            let r = e(self.r);
            let m = e(self.m);
            let normal2 = e(1.) + m * m;
            let q = e(self.b) - m * e(t) - r * normal2.sqrt();
            let h = (q * q - r * r).sqrt();
            let d = (normal2 * q * q - r * r).sqrt();
            let x = h + r * h * s.0[0] / d;
            let y = r * (e(1.) - c.0[0]);
            let z = e(t) + r * m * q * s.0[0] / d;
            match kind {
                Trim::Ci => e(self.a) + e(self.n) * z - (x * x + y * y).sqrt(),
                Trim::Bc => z + r * s.0[0] - r,
                Trim::Dc => z,
            }
        };
        let a = value(lo);
        let b = value(hi);
        if !a.valid() || !b.valid() || a.hi > 0. || b.lo < 0. {
            return None;
        }
        let derivative = self
            .equation_jet(Jet::variable(Enclosure { lo, hi }), s, c, kind)
            .0[1];
        if !derivative.valid() || derivative.lo <= 0. {
            return None;
        }
        let mut ends = [0.; 2];
        for (side, end) in ends.iter_mut().enumerate() {
            let (mut a, mut b) = (lo, hi);
            for _ in 0..64 {
                let mid = a + (b - a) / 2.;
                if mid == a || mid == b {
                    break;
                }
                let v = value(mid);
                if !v.valid() {
                    return None;
                }
                if if side == 0 { v.hi < 0. } else { v.lo <= 0. } {
                    a = mid;
                } else {
                    b = mid;
                }
            }
            *end = if side == 0 { a } else { b };
        }
        let result = Enclosure {
            lo: ends[0],
            hi: ends[1],
        };
        result.valid().then_some(result)
    }
    fn trim_jets(&self, phi: Enclosure, kind: Trim) -> Option<[Jet; 3]> {
        let (s, c) = trig_jets(phi);
        let root = self.trim_box(phi, kind)?;
        let denominator = self
            .equation_jet(
                Jet::variable(root),
                Jet::constant(s.0[0]),
                Jet::constant(c.0[0]),
                kind,
            )
            .0[1];
        if !denominator.valid() || denominator.lo <= 0. {
            return None;
        }
        let mut t = Jet::constant(root);
        for k in 1..5 {
            t.0[k] = -self.equation_jet(t, s, c, kind).0[k] / denominator;
        }
        let p = self.canal_jets(t, s, c);
        p.iter().all(|v| v.0.iter().all(|v| v.valid())).then_some(p)
    }
}

#[derive(Clone, Copy)]
enum Trim {
    Ci,
    Bc,
    Dc,
}

// Unsquared, sign-bracketed candidate solve. This is not an interval root certificate.
#[cfg(test)]
fn root(f: impl Fn(f64) -> Option<f64>, mut lo: f64, mut hi: f64) -> Option<f64> {
    let a = f(lo)?;
    let b = f(hi)?;
    if !a.is_finite() || !b.is_finite() || a > 0. || b < 0. {
        return None;
    }
    if a == 0. {
        return Some(lo);
    }
    if b == 0. {
        return Some(hi);
    }
    for _ in 0..64 {
        let mid = lo + (hi - lo) / 2.;
        if mid == lo || mid == hi {
            break;
        }
        let v = f(mid)?;
        if !v.is_finite() {
            return None;
        }
        if v < 0. {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(lo + (hi - lo) / 2.)
}

fn knots(values: &[f64]) -> KnotVec {
    let mut result = vec![values[0]];
    for &v in values {
        result.extend([v; 3]);
    }
    result.push(values[values.len() - 1]);
    KnotVec::from(result)
}

impl Input {
    fn ab_jets(&self, phi: Enclosure) -> Option<[Jet; 3]> {
        let e = Enclosure::point;
        let (s, c) = trig_jets(phi);
        let n = e(self.n);
        let r = e(self.r);
        let ring = e(self.a) + n * r + r * (e(1.) + n * n).sqrt();
        let y = (Jet::constant(e(1.)) + c.scale(e(-1.))).scale(r);
        let rho = Jet::constant(ring) + c.scale(-r);
        let p = [
            (rho * rho + (y * y).scale(e(-1.))).sqrt(),
            y,
            (Jet::constant(e(1.)) + s.scale(e(-1.))).scale(r),
        ];
        p.iter().all(|v| v.0.iter().all(|v| v.valid())).then_some(p)
    }
    fn contact_jets(&self, t: Enclosure, outer: bool) -> Option<[Jet; 3]> {
        let e = Enclosure::point;
        let t = Jet::variable(t);
        let [h, _, _, _] = self.scalar_jets(t);
        let p = if outer {
            let m = e(self.m);
            let r = e(self.r);
            let normal = (e(1.) + m * m).sqrt();
            let q = Jet::constant(e(self.b) - r * normal) + t.scale(-m);
            let c = q.inverse().scale(-r / normal);
            let s = (Jet::constant(e(1.)) + (c * c).scale(e(-1.))).sqrt();
            self.canal_jets(t, s, c)
        } else {
            [h, Jet::constant(e(0.)), t]
        };
        p.iter().all(|v| v.0.iter().all(|v| v.valid())).then_some(p)
    }
    fn world_box(&self, p: [Enclosure; 3]) -> [Enclosure; 3] {
        std::array::from_fn(|j| {
            (0..3).fold(Enclosure::point(self.origin[j]), |v, k| {
                v + Enclosure::point(self.axes[k][j]) * p[k]
            })
        })
    }
}
fn reverse_jets(p: [Jet; 3]) -> [Jet; 3] {
    p.map(|mut v| {
        v.0[1] = -v.0[1];
        v.0[3] = -v.0[3];
        v
    })
}

// Both allocations use the same finite subdivision rule. Bounds must include
// every composed spatial/UV coordinate before this controller can admit a pair.
fn fit_count(range: (f64, f64), fourth: f64) -> Option<usize> {
    if !range.0.is_finite()
        || !range.1.is_finite()
        || range.0 >= range.1
        || !fourth.is_finite()
        || fourth < 0.
    {
        return None;
    }
    let mut count = 1_usize;
    while count <= 16384 {
        let h = Enclosure::point(upward((range.1 - range.0) / count as f64));
        let m = Enclosure::point(fourth);
        if h.hi <= 0.01
            && (m * h * h * h * h / Enclosure::point(384.)).hi <= 1e-11
            && (m * h * h * h).hi <= 2e-9
        {
            return Some(count);
        }
        count *= 2;
    }
    None
}

#[derive(Clone, Copy, Debug, Default)]
struct CurveLedger {
    construction: f64,
    truncation: f64,
    evaluation: f64,
    derivative_construction: f64,
    derivative_truncation: f64,
    derivative_evaluation: f64,
}
impl CurveLedger {
    fn accepts(self) -> bool {
        let e = Enclosure::point;
        [
            self.construction,
            self.truncation,
            self.evaluation,
            self.derivative_construction,
            self.derivative_truncation,
            self.derivative_evaluation,
        ]
        .iter()
        .all(|x| x.is_finite() && *x >= 0.)
            && self.construction <= 1e-10
            && self.truncation <= 1e-11
            && self.evaluation <= 1e-10
            && (e(self.derivative_construction)
                + e(self.derivative_truncation)
                + e(self.derivative_evaluation))
            .hi <= 1e-8
    }
    fn include(&mut self, b: Self) {
        self.construction = self.construction.max(b.construction);
        self.truncation = self.truncation.max(b.truncation);
        self.evaluation = self.evaluation.max(b.evaluation);
        self.derivative_construction = self.derivative_construction.max(b.derivative_construction);
        self.derivative_truncation = self.derivative_truncation.max(b.derivative_truncation);
        self.derivative_evaluation = self.derivative_evaluation.max(b.derivative_evaluation);
    }
}

fn certify_span(
    controls: &[Point3],
    ideal: [[Enclosure; 3]; 4],
    h: Enclosure,
    fourth: f64,
) -> Option<CurveLedger> {
    if controls.len() != 4 || !h.valid() || h.lo < 1e-100 || !fourth.is_finite() || fourth < 0. {
        return None;
    }
    let e = Enclosure::point;
    let mut construction = 0_f64;
    let mut derivative_construction = 0_f64;
    let mut maximum = 0_f64;
    for (j, p) in controls.iter().enumerate() {
        construction = construction.max(
            (0..3)
                .fold(e(0.), |sum, k| sum + e(ideal[j][k].error(p[k])))
                .hi,
        );
        maximum = maximum.max((e(p.x.abs()) + e(p.y.abs()) + e(p.z.abs())).hi);
        if j < 3 {
            let error = (0..3).fold(e(0.), |sum, k| {
                let actual = e(3.) * (e(controls[j + 1][k]) - e(p[k])) / h;
                let target = e(3.) * (ideal[j + 1][k] - ideal[j][k]) / h;
                sum + e((actual - target).norm())
            });
            derivative_construction = derivative_construction.max(error.hi);
        }
    }
    // der_n(1) evaluates four signed cubic basis derivatives, then accumulates
    // P_i * basis_i. On a clamped span their absolute sum is <= 6/h.
    // The two quadratic recurrence rounds, derivative round and three coordinate
    // dot products use fewer than 128 operations per path. gamma(128) bounds
    // their absolute roundoff, including cancellation in the signed final sum.
    let unit = e(f64::EPSILON / 2.);
    let gamma = e(128.) * unit / (e(1.) - e(128.) * unit);
    let ledger = CurveLedger {
        construction,
        truncation: (e(fourth) * h * h * h * h / e(384.)).hi,
        evaluation: (e(2048.) * unit * e(maximum)).hi,
        derivative_construction,
        derivative_truncation: (e(fourth) * h * h * h).hi,
        derivative_evaluation: (gamma * e(6.) * e(maximum) / h).hi,
    };
    ledger.accepts().then_some(ledger)
}

// Certified against represented coefficients and stored parameter endpoints.
// Physical endpoint slivers and paired support charts remain separate obligations.
fn fitted(
    input: &Input,
    range: (f64, f64),
    f: impl Fn(f64) -> Option<(Point3, Vector3)>,
    bounds: impl Fn(Enclosure) -> Option<[Jet; 3]>,
    front: &Vertex,
    back: &Vertex,
) -> Option<(Edge, CurveLedger)> {
    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        return None;
    }
    let mut fourth = 0_f64;
    let frame = input
        .axes
        .iter()
        .map(|a| upward(upward(a.x.abs() + a.y.abs()) + a.z.abs()))
        .fold(0_f64, f64::max);
    for i in 0..128 {
        let lo = range.0 + (range.1 - range.0) * i as f64 / 128.;
        let hi = range.0 + (range.1 - range.0) * (i + 1) as f64 / 128.;
        let p = bounds(Enclosure { lo, hi })?;
        let m = p.iter().fold(Enclosure::point(0.), |sum, v| {
            sum + Enclosure::point(v.0[4].norm())
        });
        let bound = m * Enclosure::point(24.) * Enclosure::point(frame);
        if !bound.valid() {
            return None;
        }
        fourth = fourth.max(bound.hi);
    }
    let count = fit_count(range, fourth)?;
    let ts: Vec<_> = (0..=count)
        .map(|i| range.0 + (range.1 - range.0) * i as f64 / count as f64)
        .collect();
    if range.0 >= range.1 {
        return None;
    }
    let mut controls = Vec::with_capacity(3 * count + 1);
    let (mut p, mut d) = f(ts[0])?;
    controls.push(input.point(p));
    for pair in ts.windows(2) {
        let (q, e) = f(pair[1])?;
        let dt = (pair[1] - pair[0]) / 3.;
        controls.extend([
            input.point(p + d * dt),
            input.point(q - e * dt),
            input.point(q),
        ]);
        p = q;
        d = e;
    }
    if controls.iter().any(|p| !finite(*p)) {
        return None;
    }
    let correction =
        upward(controls[0].distance(front.point()) + controls.last()?.distance(back.point()));
    if !correction.is_finite() || correction >= 1e-10 {
        return None;
    }
    controls[0] = front.point();
    *controls.last_mut()? = back.point();
    let e = Enclosure::point;
    let mut ledger = CurveLedger::default();
    for (index, pair) in ts.windows(2).enumerate() {
        let h = e(pair[1]) - e(pair[0]);
        if h.lo < 1e-100 {
            return None;
        }
        let left = bounds(e(pair[0]))?;
        let right = bounds(e(pair[1]))?;
        let boxes = [
            left.map(|v| v.0[0]),
            left.map(|v| v.0[0] + h * v.0[1] / e(3.)),
            right.map(|v| v.0[0] - h * v.0[1] / e(3.)),
            right.map(|v| v.0[0]),
        ];
        let boxes = boxes.map(|b| input.world_box(b));
        // Control error grows as 1/h: failure rejects this finite candidate;
        // blindly subdividing a failed stored-control certificate is unsound.
        ledger.include(certify_span(
            &controls[3 * index..=3 * index + 3],
            boxes,
            h,
            fourth,
        )?);
    }
    if !ledger.accepts() {
        return None;
    }
    Some((
        Edge::new(front, back, BSplineCurve::new(knots(&ts), controls).into()),
        ledger,
    ))
}

fn canal_surface(input: &Input, end_phi: f64) -> Option<(Surface, CanalLedger)> {
    let count = 256;
    let ts: Vec<_> = (0..=count)
        .map(|i| -input.r + (input.height() + input.r) * i as f64 / count as f64)
        .collect();
    let s = (end_phi / 2.).tan();
    if !s.is_finite() || s <= 0. {
        return None;
    }
    let weights = [1., 1., 1. + s * s];
    let sine = [0., s, 2. * s];
    let cosine = [1., 1., 1. - s * s];
    let mut rows = Vec::new();
    let row = |t: f64, v: [f64; 3]| {
        (0..3)
            .map(|j| {
                let w = weights[j];
                let p = input.origin.to_vec() * w
                    + input.vector(Vector3::new(
                        v[0] * w + input.r * v[1] * sine[j],
                        input.r * (w - cosine[j]),
                        t * w + input.r * v[2] * sine[j],
                    ));
                Vector4::new(p.x, p.y, p.z, w)
            })
            .collect::<Vec<_>>()
    };
    rows.push(row(ts[0], input.scalars(ts[0])?.0));
    for pair in ts.windows(2) {
        let (a, da) = input.scalars(pair[0])?;
        let (b, db) = input.scalars(pair[1])?;
        let dt = (pair[1] - pair[0]) / 3.;
        rows.push(row(
            pair[0] + dt,
            std::array::from_fn(|i| a[i] + dt * da[i]),
        ));
        rows.push(row(
            pair[1] - dt,
            std::array::from_fn(|i| b[i] - dt * db[i]),
        ));
        rows.push(row(pair[1], b));
    }
    if rows.iter().flatten().any(|p| {
        !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() || !p.w.is_finite() || p.w <= 0.
    }) {
        return None;
    }
    let ledger = certify_canal(input, &ts, s, &rows)?;
    Some((
        NurbsSurface::new(BSplineSurface::new(
            (knots(&ts), KnotVec::from(vec![0., 0., 0., s, s, s])),
            rows,
        ))
        .into(),
        ledger,
    ))
}

fn circle(input: &Input, center: Point3, cosine: Vector3, sine: Vector3, end: f64) -> Curve {
    let cosine = input.vector(cosine);
    let sine = input.vector(sine);
    let matrix = Matrix4::from_cols(
        cosine.extend(0.),
        sine.extend(0.),
        cosine.cross(sine).normalize().extend(0.),
        input.point(center).to_homogeneous(),
    );
    Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., end)),
        matrix,
    ))
}

const ENDS: [(usize, usize); 18] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 0),
    (1, 8),
    (8, 7),
    (7, 6),
    (6, 0),
    (2, 9),
    (9, 8),
    (3, 11),
    (11, 10),
    (10, 9),
    (4, 5),
    (5, 11),
    (6, 5),
    (7, 10),
];
// D,A,B,C,O,Q,I,S; signed one-based edge indices.
const LOOPS: [&[i32]; 8] = [
    &[-5, -4, -3, -2, -1],
    &[1, 6, 7, 8, 9],
    &[2, 10, 11, -6],
    &[3, 12, 13, 14, -10],
    &[4, 15, 16, -12],
    &[5, -9, 17, -15],
    &[-8, 18, -13, -16, -17],
    &[-7, -11, -14, -18],
];

fn candidate(input: &Input) -> Option<(Shell, Vec<CurveLedger>, CanalLedger)> {
    let r = input.r;
    let height = input.height();
    let ring = input.ring();
    let base_t = -r * input.m / (1. + input.m * input.m).sqrt();
    let ridge_t = height + base_t;
    let phi_base = input.outer_phi(base_t)?;
    let phi_ridge = input.outer_phi(ridge_t)?;
    let ci0 = input.trim(0., Trim::Ci)?.0;
    let ci1 = input.trim(phi_ridge, Trim::Ci)?.0;
    let bc0 = input.trim(0., Trim::Bc)?.0;
    let bc1 = input.trim(FRAC_PI_2, Trim::Bc)?.0;
    let dc1 = input.trim(phi_base, Trim::Dc)?.0;
    let ab0 = input.ab(0.)?.0;
    let ab1 = input.ab(FRAC_PI_2)?.0;
    let z_contact = r + r * input.n / (1. + input.n * input.n).sqrt();
    let rho_contact = input.a + input.n * z_contact;
    if z_contact >= ci0.z || bc1.distance(dc1) == 0. || phi_base >= phi_ridge {
        return None;
    }
    let local = [
        Point3::new(0., ring, 0.),
        ab1,
        bc1,
        dc1,
        Point3::new(0., input.b, 0.),
        Point3::new(0., input.a + input.n * height, height),
        Point3::new(0., rho_contact, z_contact),
        Point3::new(rho_contact, 0., z_contact),
        ab0,
        bc0,
        ci0,
        ci1,
    ];
    if local.iter().any(|p| !finite(*p)) {
        return None;
    }
    let vertices = local.map(|p| Vertex::new(input.point(p)));
    let line = |i: usize| {
        let (a, b) = ENDS[i];
        builder::line(&vertices[a], &vertices[b])
    };
    let arc = |i: usize, mid: Point3| {
        let (a, b) = ENDS[i];
        builder::circle_arc(&vertices[a], &vertices[b], input.point(mid))
    };
    let polar_arc = |i: usize| {
        let (a, b) = ENDS[i];
        let theta = (local[a].y.atan2(local[a].x) + local[b].y.atan2(local[b].x)) / 2.;
        if !theta.is_finite() || !(-4. ..=4.).contains(&theta) {
            return None;
        }
        let rho = local[a].x.hypot(local[a].y);
        Some(arc(
            i,
            Point3::new(rho * theta.cos(), rho * theta.sin(), local[a].z),
        ))
    };
    let mut curve_ledgers = Vec::new();
    let mut fit = |i: usize, range, f: &dyn Fn(f64) -> Option<(Point3, Vector3)>| {
        let (a, b) = ENDS[i];
        let bounds = |x: Enclosure| match i {
            2 => input.trim_jets(x, Trim::Dc),
            5 => input.ab_jets(-x).map(reverse_jets),
            9 => input.trim_jets(-x, Trim::Bc).map(reverse_jets),
            11 => input.contact_jets(x, true),
            12 => input.trim_jets(-x, Trim::Ci).map(reverse_jets),
            13 => input.contact_jets(-x, false).map(reverse_jets),
            _ => None,
        };
        let (edge, ledger) = fitted(input, range, f, bounds, &vertices[a], &vertices[b])?;
        curve_ledgers.push(ledger);
        Some(edge)
    };
    let meridian = input.n.atan();
    let inv_normal = 1. / (1. + input.n * input.n).sqrt();
    let as_curve = circle(
        input,
        Point3::new(ring, 0., r),
        Vector3::new(-r, 0., 0.),
        Vector3::new(0., 0., r),
        meridian,
    );
    let aq_curve = circle(
        input,
        Point3::new(0., ring, r),
        Vector3::new(0., -r * inv_normal, r * input.n * inv_normal),
        Vector3::new(0., -r * input.n * inv_normal, -r * inv_normal),
        meridian + FRAC_PI_2,
    );
    let co = |t: f64| {
        let phi = input.outer_phi(t)?;
        let q = input.b - input.m * t - r * (1. + input.m * input.m).sqrt();
        let c = -r / ((1. + input.m * input.m).sqrt() * q);
        let derivative =
            r * input.m / ((1. + input.m * input.m).sqrt() * q * q * (1. - c * c).sqrt());
        let (p, pt, pp) = input.canal(t, phi)?;
        Some((p, pt + pp * derivative))
    };
    let (mut c, canal_ledger) = canal_surface(input, phi_ridge)?;
    c.invert();
    let cs: Curve = PCurve::new(
        BSplineCurve::new(
            KnotVec::from(vec![-ci0.z, -ci0.z, -r, -r]),
            vec![Point2::new(0., ci0.z), Point2::new(0., r)],
        ),
        Box::new(c.clone()),
    )
    .into();
    if cs.subs(-ci0.z).distance(vertices[10].point()) > 1e-10
        || cs.subs(-r).distance(vertices[9].point()) > 1e-10
    {
        return None;
    }
    let edges = [
        polar_arc(0)?,
        line(1),
        fit(2, (FRAC_PI_2, phi_base), &|p| input.trim(p, Trim::Dc))?,
        polar_arc(3)?,
        line(4),
        fit(5, (-FRAC_PI_2, 0.), &|p| input.ab(-p).map(|(q, d)| (q, -d)))?,
        Edge::new(&vertices[8], &vertices[7], as_curve),
        polar_arc(7)?,
        Edge::new(&vertices[6], &vertices[0], aq_curve),
        fit(9, (-FRAC_PI_2, 0.), &|p| {
            input.trim(-p, Trim::Bc).map(|(q, d)| (q, -d))
        })?,
        line(10),
        fit(11, (base_t, ridge_t), &co)?,
        fit(12, (-phi_ridge, 0.), &|p| {
            input.trim(-p, Trim::Ci).map(|(q, d)| (q, -d))
        })?,
        Edge::new(&vertices[10], &vertices[9], cs),
        line(14),
        polar_arc(15)?,
        line(16),
        line(17),
    ];
    let mut a: Surface = Processor::new(RevolutedCurve::by_revolution(
        edges[8].curve(),
        input.origin,
        input.axes[2],
    ))
    .into();
    let section = circle(
        input,
        Point3::new(0., r, r),
        Vector3::new(0., -r, 0.),
        Vector3::new(0., 0., -r),
        FRAC_PI_2,
    );
    let b: Surface = ExtrudedCurve::by_extrusion(section, input.axes[0] * input.b).into();
    a.invert();
    let surfaces = [
        input.supports[0].clone(),
        a,
        b,
        c,
        input.supports[4].clone(),
        input.supports[2].clone(),
        input.supports[3].clone(),
        input.supports[1].clone(),
    ];
    LOOPS
        .iter()
        .zip(surfaces)
        .map(|(indices, surface)| {
            let wire: Wire = indices
                .iter()
                .map(|&i| {
                    if i > 0 {
                        edges[i as usize - 1].clone()
                    } else {
                        edges[(-i) as usize - 1].inverse()
                    }
                })
                .collect();
            Face::try_new(vec![wire], surface).ok()
        })
        .collect::<Option<Vec<_>>>()
        .map(|faces| (faces.into(), curve_ledgers, canal_ledger))
}

impl Input {
    fn stationary_expression(&self, phi: Enclosure) -> Option<(Enclosure, Enclosure)> {
        let e = Enclosure::point;
        let t = self.trim_box(phi, Trim::Ci)?;
        let r = e(self.r);
        let m = e(self.m);
        let n = e(self.n);
        let q = e(self.b) - m * t - r * (e(1.) + m * m).sqrt();
        let ratio = r / q;
        let k = m / (e(1.) - ratio * ratio).sqrt();
        let normal = (e(1.) + k * k).sqrt();
        let sine = trig_enclosure(phi, false);
        let cosine = trig_enclosure(phi, true);
        let x = (q * q - r * r).sqrt() + r * sine / normal;
        let y = r * (e(1.) - cosine);
        let rho = (x * x + y * y).sqrt();
        let radial_x = (e(1.) - (y / rho) * (y / rho)).sqrt();
        let h = r * ((radial_x - n * k) * cosine / normal + y * sine / rho);
        // At a stationary point T'=0. On cos(phi)>=0, differentiating H
        // with fixed T leaves nonnegative terms plus this strict lower bound.
        let lower = r * sine * (n * k - e(1.)) / normal;
        (h.valid() && lower.valid()).then_some((
            h,
            if cosine.lo >= 0. {
                lower
            } else {
                Enclosure {
                    lo: f64::NEG_INFINITY,
                    hi: lower.hi,
                }
            },
        ))
    }

    fn separation_gate(&self, charge: f64) -> Option<Evidence> {
        if !charge.is_finite() || charge <= 0. {
            return None;
        }
        let e = Enclosure::point;
        let domains = self.physical_domains()?;
        let end = -domains[4].0.lo;
        let mut stationary_lower = f64::INFINITY;
        let mut uncertain = 0;
        // Cover the whole physical CI domain, including every zero-containing
        // derivative box. H_phi>0 at every zero prohibits a second crossing.
        for j in 0..128 {
            let phi = Enclosure {
                lo: end * j as f64 / 128.,
                hi: end * (j + 1) as f64 / 128.,
            };
            let (h, lower) = self.stationary_expression(phi)?;
            if h.lo <= 0. && h.hi >= 0. {
                if lower.lo <= 0. {
                    return None;
                }
                stationary_lower = stationary_lower.min(lower.lo);
                uncertain += 1;
            }
        }
        let (mut lo, mut hi) = (0., end);
        if self.stationary_expression(e(lo))?.0.hi >= 0.
            || self.stationary_expression(e(hi))?.0.lo <= 0.
        {
            return None;
        }
        for _ in 0..30 {
            let mid = lo + (hi - lo) / 2.;
            let h = self.stationary_expression(e(mid))?.0;
            if h.hi < 0. {
                lo = mid;
            } else if h.lo > 0. {
                hi = mid;
            } else {
                break;
            }
        }
        let minimum = self.trim_box(Enclosure { lo, hi }, Trim::Ci)?;
        let start = self.trim_box(e(0.), Trim::Ci)?;
        let seam = self.trim_box(domains[0].0, Trim::Bc)?;
        let gaps = [domains[3].0 - seam, start - minimum, domains[3].1 - start];
        if uncertain == 0
            || !stationary_lower.is_finite()
            || gaps.iter().any(|g| !g.valid() || g.lo <= charge)
        {
            return None;
        }
        Some({
            #[cfg(test)]
            {
                serde_json::json!({"stationary_phi": [lo,hi],
            "zero_containing_boxes": uncertain, "stationary_partial_lower": stationary_lower,
            "minimum_t": [minimum.lo,minimum.hi], "stationary_scope": "analytic_only_uniform_position_transfer",
            "gaps": gaps.map(|g| [g.lo,g.hi]), "charge": charge,
            "order": ["base_minus_seam", "start_minus_minimum", "ridge_minus_start"]})
            }
            #[cfg(not(test))]
            {
                Evidence
            }
        })
    }

    fn physical_domains(&self) -> Option<[(Enclosure, Enclosure); 7]> {
        let e = Enclosure::point;
        let r = e(self.r);
        let m = e(self.m);
        let n = e(self.n);
        let normal = (e(1.) + m * m).sqrt();
        let base = -r * m / normal;
        let ridge = (e(self.b) - e(self.a)) / (m + n) + base;
        let outer = |t| inverse_cosine(-r / (normal * (e(self.b) - m * t - r * normal)));
        let half = inverse_cosine(e(0.))?;
        let start = self.trim_box(e(0.), Trim::Ci)?;
        let meridian = inverse_cosine(e(1.) / (e(1.) + n * n).sqrt())?;
        Some([
            (half, outer(base)?),
            (-half, e(0.)),
            (-half, e(0.)),
            (base, ridge),
            (-outer(ridge)?, e(0.)),
            (-start, -r),
            (e(0.), meridian),
        ])
    }

    fn endpoint_gate(&self, shell: &Shell) -> Option<Vec<EndpointRecord>> {
        let domains = self.physical_domains()?;
        let ids = [2, 5, 9, 11, 12, 13, 6];
        let mut unique = HashSet::new();
        // Face loops encounter edges 4..0 first; retrieve by the fixed signed loops.
        let mut edges = vec![None; 18];
        for (face, indices) in shell.iter().zip(LOOPS) {
            for (edge, &index) in face.boundaries()[0].iter().zip(indices) {
                let k = index.unsigned_abs() as usize - 1;
                if unique.insert(k) {
                    edges[k] = Some(if index > 0 {
                        edge.clone()
                    } else {
                        edge.inverse()
                    });
                }
            }
        }
        let mut rows = Vec::new();
        for (slot, &id) in ids.iter().enumerate() {
            let edge = edges[id].as_ref()?;
            let curve = edge.curve();
            let (lo, hi) = curve.range_tuple();
            let bounds = |x: Enclosure| match id {
                2 => self.trim_jets(x, Trim::Dc),
                5 => self.ab_jets(-x).map(reverse_jets),
                9 => self.trim_jets(-x, Trim::Bc).map(reverse_jets),
                11 => self.contact_jets(x, true),
                12 => self.trim_jets(-x, Trim::Ci).map(reverse_jets),
                13 => self.contact_jets(-x, false).map(reverse_jets),
                6 => {
                    let e = Enclosure::point;
                    let (s, c) = trig_jets(x);
                    let ring = e(self.a)
                        + e(self.n) * e(self.r)
                        + e(self.r) * (e(1.) + e(self.n) * e(self.n)).sqrt();
                    Some([
                        Jet::constant(ring) + c.scale(e(-self.r)),
                        Jet::constant(e(0.)),
                        Jet::constant(e(self.r)) + s.scale(e(self.r)),
                    ])
                }
                _ => None,
            };
            let mut ends = Vec::new();
            for (physical, stored, vertex) in [
                (domains[slot].0, lo, edge.front()),
                (domains[slot].1, hi, edge.back()),
            ] {
                let (correction, sliver) =
                    endpoint_allowance(self, physical, stored, vertex.point(), bounds)?;
                ends.push(EndpointBound {
                    physical,
                    stored,
                    vertex_correction: correction,
                    sliver,
                });
            }
            rows.push(EndpointRecord { edge: id, ends });
        }
        Some(rows)
    }
}

#[derive(Clone, Debug)]
struct EndpointBound {
    physical: Enclosure,
    stored: f64,
    vertex_correction: f64,
    sliver: f64,
}
#[derive(Clone, Debug)]
struct EndpointRecord {
    edge: usize,
    ends: Vec<EndpointBound>,
}
impl EndpointRecord {
    #[cfg(test)]
    fn diagnostic(&self) -> Evidence {
        serde_json::json!({"edge":self.edge,"ends":self.ends.iter().map(|b|
            serde_json::json!({"physical":[b.physical.lo,b.physical.hi],"stored":b.stored,
                "vertex_correction":b.vertex_correction,"sliver":b.sliver})).collect::<Vec<_>>()})
    }
}

// UVs are in the stored candidate charts, on the stored edge parameter.
// In particular reversed AB/BC/CI parameters are not physical positive phi.
#[derive(Clone, Copy, Debug)]
enum PairedChart {
    A,
    B,
    C,
    D,
    S,
    I,
    O,
}

fn angle_jet(x: Jet, y: Jet) -> Option<Jet> {
    let e = Enclosure::point;
    let ratio = y.0[0] / x.0[0];
    if x.0[0].lo <= 0. || !ratio.valid() || ratio.norm() >= 0.2 {
        return None;
    }
    // Alternating atan series; the absolute remainder also covers negative y.
    let mut term = ratio;
    let mut angle = ratio;
    for k in 1..20 {
        term = term * (-ratio * ratio);
        angle = angle + term / e((2 * k + 1) as f64);
    }
    let mut remainder = e(1.);
    for _ in 0..41 {
        remainder = remainder * e(ratio.norm());
    }
    remainder = remainder / e(41.);
    let differentiate = |v: Jet| {
        Jet(std::array::from_fn(|k| {
            if k == 4 {
                e(0.)
            } else {
                v.0[k + 1] * e((k + 1) as f64)
            }
        }))
    };
    let derivative =
        (x * differentiate(y) + (y * differentiate(x)).scale(e(-1.))) * (x * x + y * y).inverse();
    let result = Jet(std::array::from_fn(|k| {
        if k == 0 {
            angle
                + Enclosure {
                    lo: -remainder.hi,
                    hi: remainder.hi,
                }
        } else {
            derivative.0[k - 1] / e(k as f64)
        }
    }));
    result.0.iter().all(|v| v.valid()).then_some(result)
}

impl Input {
    fn trim_parameter_jet(&self, phi: Enclosure, kind: Trim) -> Option<Jet> {
        let (s, c) = trig_jets(phi);
        let root = self.trim_box(phi, kind)?;
        let denominator = self
            .equation_jet(
                Jet::variable(root),
                Jet::constant(s.0[0]),
                Jet::constant(c.0[0]),
                kind,
            )
            .0[1];
        if !denominator.valid() || denominator.lo <= 0. {
            return None;
        }
        let mut t = Jet::constant(root);
        for k in 1..5 {
            t.0[k] = -self.equation_jet(t, s, c, kind).0[k] / denominator;
        }
        t.0.iter().all(|v| v.valid()).then_some(t)
    }

    fn paired_uv_jets(
        &self,
        edge: usize,
        chart: PairedChart,
        parameter: Enclosure,
    ) -> Option<[Jet; 2]> {
        let e = Enclosure::point;
        let constant = |x| Jet::constant(e(x));
        let reverse = matches!(edge, 5 | 9 | 12 | 13);
        let physical = if reverse { -parameter } else { parameter };
        let variable = Jet::variable(physical);
        let (s, c) = trig_jets(physical);
        let spatial = || -> Option<[Jet; 3]> {
            match edge {
                2 => self.trim_jets(physical, Trim::Dc),
                5 => self.ab_jets(physical),
                9 => self.trim_jets(physical, Trim::Bc),
                11 => self.contact_jets(physical, true),
                12 => self.trim_jets(physical, Trim::Ci),
                13 => self.contact_jets(physical, false),
                6 => Some([
                    constant(self.ring()) + c.scale(e(-self.r)),
                    constant(0.),
                    constant(self.r) + s.scale(e(self.r)),
                ]),
                _ => None,
            }
        };
        let uv = match chart {
            PairedChart::D | PairedChart::S
                if matches!(
                    &self.supports[if matches!(chart, PairedChart::D) {
                        0
                    } else {
                        1
                    }],
                    Surface::Plane(_)
                ) =>
            {
                let index = if matches!(chart, PairedChart::D) {
                    0
                } else {
                    1
                };
                let Surface::Plane(plane) = &self.supports[index] else {
                    return None;
                };
                let p = spatial()?;
                let world: [Jet; 3] = std::array::from_fn(|j| {
                    (0..3).fold(constant(self.origin[j]), |sum, k| {
                        sum + p[k].scale(e(self.axes[k][j]))
                    }) + constant(-plane.origin()[j])
                });
                let axes = [plane.u_axis(), plane.v_axis()];
                let dot =
                    |a: Vector3, b: Vector3| (0..3).fold(e(0.), |sum, k| sum + e(a[k]) * e(b[k]));
                let aa = dot(axes[0], axes[0]);
                let ab = dot(axes[0], axes[1]);
                let bb = dot(axes[1], axes[1]);
                let det = aa * bb - ab * ab;
                if !det.valid() || det.lo <= 0. {
                    return None;
                }
                let rhs =
                    axes.map(|a| (0..3).fold(constant(0.), |sum, k| sum + world[k].scale(e(a[k]))));
                [
                    (rhs[0].scale(bb) + rhs[1].scale(-ab)).scale(e(1.) / det),
                    (rhs[1].scale(aa) + rhs[0].scale(-ab)).scale(e(1.) / det),
                ]
            }
            PairedChart::I | PairedChart::O | PairedChart::D => {
                let index = match chart {
                    PairedChart::I => 3,
                    PairedChart::O => 4,
                    _ => 0,
                };
                let Surface::RevolutedCurve(rev) = &self.supports[index] else {
                    return None;
                };
                let Curve::Line(line) = rev.entity().entity_curve() else {
                    return None;
                };
                let matrix = rev.transform();
                let dual = dual_frame(self.axes)?;
                let local = |point: Point3| -> [Enclosure; 3] {
                    let world: [Enclosure; 3] = std::array::from_fn(|j| {
                        (0..3).fold(e(matrix.w[j]) - e(self.origin[j]), |sum, k| {
                            sum + e(matrix[k][j]) * e(point[k])
                        })
                    });
                    dual.map(|axis| box_dot(world, axis))
                };
                let start = local(line.0);
                let end = local(line.1);
                let dz = end[2] - start[2];
                let p = spatial()?;
                let eta = if dz.valid() && (dz.lo > 0. || dz.hi < 0.) {
                    (p[2] + Jet::constant(-start[2])).scale(e(1.) / dz)
                } else {
                    let r0 = (start[0] * start[0] + start[1] * start[1]).sqrt();
                    let r1 = (end[0] * end[0] + end[1] * end[1]).sqrt();
                    ((p[0] * p[0] + p[1] * p[1]).sqrt() + Jet::constant(-r0))
                        .scale(e(1.) / (r1 - r0))
                };
                let x = p[0].scale(start[0]) + p[1].scale(start[1]);
                let y = p[1].scale(start[0]) + p[0].scale(-start[1]);
                let theta = angle_jet(x, y)?;
                // certify_cone already requires the stored axis to point along +Z.
                if rev.orientation() {
                    [eta, theta]
                } else {
                    [theta, eta]
                }
            }
            PairedChart::S => return None,
            PairedChart::A => {
                let phase = inverse_cosine(e(1.) / (e(1.) + e(self.n) * e(self.n)).sqrt())?;
                let phase = Jet::constant(phase);
                let half_pi = Jet::constant(Enclosure {
                    lo: downward(FRAC_PI_2),
                    hi: upward(FRAC_PI_2),
                });
                match edge {
                    5 => {
                        let p = self.ab_jets(physical)?;
                        [
                            angle_jet(p[0], p[1])? + half_pi.scale(e(-1.)),
                            phase + variable,
                        ]
                    }
                    6 => [half_pi.scale(e(-1.)), phase + variable.scale(e(-1.))],
                    _ => return None,
                }
            }
            PairedChart::B => {
                let p = match edge {
                    5 => self.ab_jets(physical)?,
                    9 => self.trim_jets(physical, Trim::Bc)?,
                    _ => return None,
                };
                if self.b <= 0. {
                    return None;
                }
                [variable, p[0].scale(e(1.) / e(self.b))]
            }
            PairedChart::C => match edge {
                2 | 9 | 12 => {
                    let kind = match edge {
                        2 => Trim::Dc,
                        9 => Trim::Bc,
                        _ => Trim::Ci,
                    };
                    [
                        s * (constant(1.) + c).inverse(),
                        self.trim_parameter_jet(physical, kind)?,
                    ]
                }
                11 => {
                    let q = constant(self.b)
                        + variable.scale(e(-self.m))
                        + Jet::constant(-e(self.r) * (e(1.) + e(self.m) * e(self.m)).sqrt());
                    let cosine = q
                        .inverse()
                        .scale(-e(self.r) / (e(1.) + e(self.m) * e(self.m)).sqrt());
                    let sine = (constant(1.) + (cosine * cosine).scale(e(-1.))).sqrt();
                    [sine * (constant(1.) + cosine).inverse(), variable]
                }
                13 => [constant(0.), variable],
                _ => return None,
            },
        };
        let uv = if reverse {
            uv.map(|mut v| {
                v.0[1] = -v.0[1];
                v.0[3] = -v.0[3];
                v
            })
        } else {
            uv
        };
        uv.iter()
            .all(|v| v.0.iter().all(|x| x.valid()))
            .then_some(uv)
    }
}

// This ledger certifies UV interpolation and evaluation only. Its constructor
// cannot mint a support or a paired certificate: those require additional bounds.
struct StoredUv {
    curve: BSplineCurve<Point2>,
    construction: [f64; 2],
    truncation: [f64; 2],
    evaluation: [f64; 2],
}
impl StoredUv {
    fn composed(&self, lipschitz: [f64; 2]) -> Option<f64> {
        let e = Enclosure::point;
        let mut total = e(0.);
        for (j, lip) in lipschitz.into_iter().enumerate() {
            if !lip.is_finite() || lip < 0. {
                return None;
            }
            for term in [self.construction[j], self.truncation[j], self.evaluation[j]] {
                if !term.is_finite() || term < 0. {
                    return None;
                }
                total = total + e(lip) * e(term);
            }
        }
        total.valid().then_some(total.hi)
    }
}

fn stored_uv(
    input: &Input,
    support: &Surface,
    edge: usize,
    chart: PairedChart,
    ts: &[f64],
) -> Option<StoredUv> {
    if ts.len() < 2 || ts.len() > 16385 {
        return None;
    }
    let e = Enclosure::point;
    let mut points = Vec::with_capacity(3 * (ts.len() - 1) + 1);
    let mut construction = [0_f64; 2];
    let mut truncation = [0_f64; 2];
    let mut maximum = [0_f64; 2];
    for (i, pair) in ts.windows(2).enumerate() {
        let h = e(pair[1]) - e(pair[0]);
        if !h.valid() || h.lo < 1e-100 {
            return None;
        }
        let left = chart_jets(
            support,
            chart,
            input.paired_uv_jets(edge, chart, e(pair[0]))?,
        )?;
        let right = chart_jets(
            support,
            chart,
            input.paired_uv_jets(edge, chart, e(pair[1]))?,
        )?;
        let whole = chart_jets(
            support,
            chart,
            input.paired_uv_jets(
                edge,
                chart,
                Enclosure {
                    lo: pair[0],
                    hi: pair[1],
                },
            )?,
        )?;
        let boxes = [
            left.map(|v| v.0[0]),
            left.map(|v| v.0[0] + h * v.0[1] / e(3.)),
            right.map(|v| v.0[0] - h * v.0[1] / e(3.)),
            right.map(|v| v.0[0]),
        ];
        for (k, b) in boxes.into_iter().enumerate() {
            let p = if i > 0 && k == 0 {
                *points.last()?
            } else {
                Point2::new(
                    b[0].lo + (b[0].hi - b[0].lo) / 2.,
                    b[1].lo + (b[1].hi - b[1].lo) / 2.,
                )
            };
            for j in 0..2 {
                construction[j] = construction[j].max(b[j].error(p[j]));
                maximum[j] = maximum[j].max(p[j].abs());
            }
            if i == 0 || k > 0 {
                points.push(p);
            }
        }
        for j in 0..2 {
            truncation[j] =
                truncation[j].max((e(24.) * e(whole[j].0[4].norm()) * h * h * h * h / e(384.)).hi);
        }
    }
    let ledger = StoredUv {
        curve: BSplineCurve::new(knots(ts), points),
        construction,
        truncation,
        evaluation: maximum.map(|m| (e(2048.) * e(f64::EPSILON / 2.) * e(m)).hi),
    };
    ledger.composed([1., 1.])?;
    Some(ledger)
}

fn paired_uv_rows(
    input: &Input,
    shell: &Shell,
    curves: &[CurveLedger],
    canal: &CanalLedger,
    endpoints: &[EndpointRecord],
) -> Option<(Vec<PairBudget>, Evidence)> {
    let mut edges = vec![None; 18];
    for (face, indices) in shell.iter().zip(LOOPS) {
        for (edge, index) in face.boundaries()[0].iter().zip(indices) {
            let k = index.unsigned_abs() as usize - 1;
            if edges[k].is_none() {
                edges[k] = Some(if *index > 0 {
                    edge.clone()
                } else {
                    edge.inverse()
                });
            }
        }
    }
    let pairs = [
        (2, [(0, PairedChart::D), (3, PairedChart::C)]),
        (5, [(1, PairedChart::A), (2, PairedChart::B)]),
        (9, [(2, PairedChart::B), (3, PairedChart::C)]),
        (11, [(3, PairedChart::C), (4, PairedChart::O)]),
        (12, [(3, PairedChart::C), (6, PairedChart::I)]),
        (13, [(3, PairedChart::C), (7, PairedChart::S)]),
        (6, [(1, PairedChart::A), (7, PairedChart::S)]),
    ];
    let mut rows = Vec::new();
    let mut budgets = Vec::new();
    for (id, supports) in pairs {
        let curve = edges[id].as_ref()?.curve();
        let (lo, hi) = curve.range_tuple();
        let ts = if let Curve::BSplineCurve(c) = &curve {
            let (knots, _) = c.clone().destruct();
            let mut values = knots.to_vec();
            values.dedup();
            values
        } else {
            let count = 1024;
            (0..=count)
                .map(|k| lo + (hi - lo) * k as f64 / count as f64)
                .collect()
        };
        for (face, chart) in supports {
            let support = shell.get(face)?.surface();
            let uv = if id == 13 && matches!(chart, PairedChart::C) {
                let Curve::PCurve(p) = &curve else {
                    return None;
                };
                let c = p.curve();
                if c.degree() != 1
                    || c.control_points().len() != 2
                    || c.control_points()[0] != Point2::new(0., -lo)
                    || c.control_points()[1] != Point2::new(0., -hi)
                {
                    return None;
                }
                StoredUv {
                    curve: c.clone(),
                    construction: [0.; 2],
                    truncation: [0.; 2],
                    evaluation: [
                        0.,
                        upward(2048. * (f64::EPSILON / 2.) * lo.abs().max(hi.abs())),
                    ],
                }
            } else {
                stored_uv(input, &support, id, chart, &ts)?
            };
            let e = Enclosure::point;
            let mut hull = std::array::from_fn(|j| {
                let lo = uv
                    .curve
                    .control_points()
                    .iter()
                    .map(|p| p[j])
                    .fold(f64::INFINITY, f64::min);
                let hi = uv
                    .curve
                    .control_points()
                    .iter()
                    .map(|p| p[j])
                    .fold(f64::NEG_INFINITY, f64::max);
                let error = e(uv.construction[j]) + e(uv.truncation[j]) + e(uv.evaluation[j]);
                Enclosure {
                    lo: (e(lo) - error).lo,
                    hi: (e(hi) + error).hi,
                }
            });
            let slot = [2, 5, 9, 11, 12, 13, 6].iter().position(|k| *k == id)?;
            let physical = input.physical_domains()?[slot];
            for (box_, stored) in [(physical.0, lo), (physical.1, hi)] {
                let neighborhood = Enclosure {
                    lo: box_.lo.min(stored),
                    hi: box_.hi.max(stored),
                };
                let jets = chart_jets(
                    &support,
                    chart,
                    input.paired_uv_jets(id, chart, neighborhood)?,
                )?;
                for j in 0..2 {
                    hull[j].lo = hull[j].lo.min(jets[j].0[0].lo);
                    hull[j].hi = hull[j].hi.max(jets[j].0[0].hi);
                }
            }
            let (lip, mut factor, magnitude, coefficient) =
                support_composition(input, &support, chart, hull)?;
            let mut arguments = support_arguments(&support, hull)?;
            if id == 6 {
                arguments.extend(curve_arguments(
                    &curve,
                    Enclosure {
                        lo: lo.min(physical.0.lo),
                        hi: hi.max(physical.1.hi),
                    },
                )?);
            }
            let uv_composition = uv.composed(lip)?;
            let maximum: [f64; 2] = std::array::from_fn(|j| {
                uv.curve
                    .control_points()
                    .iter()
                    .map(|p| p[j].abs())
                    .fold(0_f64, f64::max)
            });
            // 512 exceeds the <=129 operation evaluator path plus orientation
            // argument rounding and its propagation through bounded derivatives.
            let gamma = basic_gamma(512);
            let basic = if matches!(chart, PairedChart::C) {
                (e(2048.) * e(f64::EPSILON / 2.) * e(magnitude)).hi
            } else {
                (gamma * e(magnitude) * (e(1.) + e(maximum[0]) + e(maximum[1]))).hi
            };
            let projection = if let Surface::Plane(plane) = &support {
                let mut error = 0_f64;
                for span in ts.windows(2) {
                    error = error.max(plane_projection_error(
                        input,
                        plane,
                        ideal_trace(
                            input,
                            id,
                            Enclosure {
                                lo: span[0],
                                hi: span[1],
                            },
                        )?,
                    )?);
                }
                for (box_, stored) in [(physical.0, lo), (physical.1, hi)] {
                    error = error.max(plane_projection_error(
                        input,
                        plane,
                        ideal_trace(
                            input,
                            id,
                            Enclosure {
                                lo: box_.lo.min(stored),
                                hi: box_.hi.max(stored),
                            },
                        )?,
                    )?);
                }
                error
            } else {
                coefficient
            };
            let expanded_canal = if matches!(chart, PairedChart::C) {
                Some(canal.expanded_fit(input, &support, hull)?)
            } else {
                None
            };
            let support_fit = if let Some((control, truncation, _, _)) = expanded_canal {
                (e(control) + e(truncation)).hi
            } else {
                projection
            };
            let recognition = 0.; // Included once in the typed coefficient comparison.
            let mut spatial = if let Some(i) = [2, 5, 9, 11, 12].iter().position(|k| *k == id) {
                let c = &curves[i];
                (e(c.construction) + e(c.truncation) + e(c.evaluation)).hi
            } else if id == 13 {
                (e(canal.controls) + e(canal.truncation) + e(canal.evaluation)).hi
            } else {
                0.
            };
            if id == 6 {
                let [w, a, b] = conic_coefficients(&curve)?;
                let ring = e(input.a)
                    + e(input.n) * e(input.r)
                    + e(input.r) * (e(1.) + e(input.n) * e(input.n)).sqrt();
                let expected = [
                    [ring, e(0.), e(input.r)],
                    [e(-input.r), e(0.), e(0.)],
                    [e(0.), e(0.), e(input.r)],
                ];
                let expected =
                    std::array::from_fn(|j| world_coefficient(input, expected[j], j == 0));
                spatial = (coefficient_error([w, a, b], expected)
                    + gamma
                        * box_norm(curve_absolute(
                            &curve,
                            Enclosure {
                                lo: lo.min(physical.0.lo),
                                hi: hi.max(physical.1.hi),
                            },
                        )?))
                .hi;
                let raw = raw_conic_coefficients(&curve)?;
                factor = (e(factor) + box_norm(raw[1]) + box_norm(raw[2])).hi;
            }
            let row = endpoints.iter().find(|row| row.edge == id)?;
            let mut endpoint = 0_f64;
            for end in &row.ends {
                let physical = end.physical;
                let stored = end.stored;
                let neighborhood = Enclosure {
                    lo: physical.lo.min(stored),
                    hi: physical.hi.max(stored),
                };
                let jets = chart_jets(
                    &support,
                    chart,
                    input.paired_uv_jets(id, chart, neighborhood)?,
                )?;
                let speed = e(lip[0]) * e(jets[0].0[1].norm()) + e(lip[1]) * e(jets[1].0[1].norm());
                endpoint = endpoint.max(
                    (e(end.vertex_correction) + e(end.sliver) + speed * e(physical.error(stored)))
                        .hi,
                );
            }
            let structural = id == 13 && matches!(chart, PairedChart::C);
            let terms = if structural {
                let Curve::PCurve(p) = &curve else {
                    return None;
                };
                let (Surface::NurbsSurface(a), Surface::NurbsSurface(b)) =
                    (p.surface().as_ref(), &support)
                else {
                    return None;
                };
                if a.knot_vecs() != b.knot_vecs() || a.control_points() != b.control_points() {
                    return None;
                }
                [0.; 6]
            } else {
                [
                    spatial,
                    uv_composition,
                    endpoint,
                    recognition,
                    support_fit,
                    basic,
                ]
            };
            let budget = PairBudget::new(&terms, factor)?;
            budgets.push(budget);
            // These are regression witnesses, not a bound on the support evaluator.
            #[cfg(test)]
            let samples: Vec<_> = (0..=8)
                .map(|k| {
                    let t = lo + (hi - lo) * k as f64 / 8.;
                    let q = uv.curve.subs(t);
                    curve.subs(t).distance(support.subs(q.x, q.y))
                })
                .collect();
            rows.push({
#[cfg(test)] { serde_json::json!({"edge": id, "face": face,
                "uv_curve": uv.curve, "construction": uv.construction,
                "expanded_canal_controls_truncation_B_Wlower":expanded_canal,
                "trig_argument_hulls":arguments.iter().map(|a|[a.lo,a.hi]).collect::<Vec<_>>(),
                "truncation": uv.truncation, "evaluation": uv.evaluation,
                "sample_residuals_not_certificate": samples,
                "paired_certificate": {"K":budget.known,"F":budget.factor,"epsilon_max":budget.epsilon_max(),"terms":terms,"lipschitz":lip,"structural":structural,"runtime_contract_supplied":false,"conditional_2_pow_minus_40":budget.charge(Some(2_f64.powi(-40))),"accepted_without_trig":budget.charge(None).is_some()}}) }
#[cfg(not(test))] { let _ = (expanded_canal, arguments, lip, structural); Evidence }
});
        }
    }
    Some((budgets, {
        #[cfg(test)]
        {
            serde_json::json!(rows)
        }
        #[cfg(not(test))]
        {
            let _ = rows;
            Evidence
        }
    }))
}

// Private conditional gate. Production callers must establish runtime eligibility.
pub(super) fn build_candidate(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    radius: f64,
) -> Option<Shell> {
    build_candidate_with_ledgers(shell, selected, radius).map(|(shell, _)| shell)
}

pub(super) fn build_candidate_with_ledgers(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    radius: f64,
) -> Option<(Shell, Evidence)> {
    let input = Input::recognize(shell, selected, radius)?;
    let (shell, curves, canal) = candidate(&input)?;
    let endpoints = input.endpoint_gate(&shell)?;
    let (mut budgets, paired_uv) = paired_uv_rows(&input, &shell, &curves, &canal, &endpoints)?;
    let (analytic_budgets, analytic_edges) = analytic_inventory(&input, &shell)?;
    budgets.extend(analytic_budgets);
    // Charge the retained stored derivative errors as well as position and
    // endpoint displacements; do not substitute an oracle's controller ledger.
    let e = Enclosure::point;
    let mut charge = e(0.);
    for c in &curves {
        charge = charge
            + e(c.construction)
            + e(c.truncation)
            + e(c.evaluation)
            + e(c.derivative_construction)
            + e(c.derivative_truncation)
            + e(c.derivative_evaluation);
    }
    for row in &endpoints {
        for end in &row.ends {
            charge = charge + e(end.vertex_correction) + e(end.sliver);
        }
    }
    // Runtime eligibility remains a public-dispatch prerequisite. Even this
    // private conditional candidate must enforce the reviewed error allowance.
    charge = charge
        + e(paired_position_charge(
            &budgets,
            ScalarRuntimeContract::ReviewedGlibc,
        )?);
    // Every T is a dual-frame height of an actual spatial point. Uniform
    // position error bounds the minimum value without stored stationary uniqueness.
    // The stored curves approximate the original analytic trace at the same
    // parameter. No perturbed implicit intersection is solved here, so adding
    // delta_F/inf|F_T| would charge a different construction. trim_box and jets
    // already enclose the actual analytic root used to build the controls.
    let dual = dual_frame(input.axes)?;
    let sensitivity = dual[2].iter().map(|x| x.norm()).fold(0_f64, f64::max);
    let separation = input.separation_gate((e(2.) * e(sensitivity) * charge).hi)?;
    Some((shell, {
        #[cfg(test)]
        {
            serde_json::json!({"analytic_edges": analytic_edges, "paired_uv": paired_uv, "physical_endpoints": endpoints.iter().map(EndpointRecord::diagnostic).collect::<Vec<_>>(), "separation": separation, "curves": curves.iter().map(|c| serde_json::json!({
            "construction": c.construction, "truncation": c.truncation, "evaluation": c.evaluation,
            "derivative_construction": c.derivative_construction,
            "derivative_truncation": c.derivative_truncation,
            "derivative_evaluation": c.derivative_evaluation})).collect::<Vec<_>>(),
        "canal": {"controls": canal.controls, "truncation": canal.truncation, "evaluation": canal.evaluation},
        "cones": input.cone_ledgers.map(|c| serde_json::json!({"gram_departure": c.gram_departure,
            "generator_residual": c.generator_residual, "displacement": c.displacement,
            "generator_domain": c.generator_domain})), "full_admission": false,
        "missing": ["independent completed-gate review before public admission"],
        "conditional_runtime": {"epsilon":2_f64.powi(-40),"domain":[-4.,4.],"assumptions":"matched scalar glibc contract; RN; gradual underflow; no interposition or fast math; final executable mapping required"}})
        }
        #[cfg(not(test))]
        {
            let _ = (analytic_edges, paired_uv, separation);
            Evidence
        }
    }))
}

// Bounded scratch composition model: binary64 round-to-nearest basic operations,
// gradual underflow, correctly rounded sqrt; sin/cos/sin_cos require a separate
// uniform absolute-error contract. No runtime contract is inferred from samples.
#[derive(Clone, Copy, Debug)]
struct PairBudget {
    known: f64,
    factor: f64,
}
impl PairBudget {
    fn new(terms: &[f64], factor: f64) -> Option<Self> {
        if terms
            .iter()
            .chain(std::iter::once(&factor))
            .any(|x| !x.is_finite() || *x < 0.)
        {
            return None;
        }
        let known = terms
            .iter()
            .fold(Enclosure::point(0.), |s, x| s + Enclosure::point(*x))
            .hi;
        Some(Self { known, factor })
    }
    #[cfg(test)]
    fn epsilon_max(self) -> Option<f64> {
        if self.known >= 1e-9 || self.factor <= 0. {
            return None;
        }
        let e = Enclosure::point;
        let x = ((e(1e-9) - e(self.known)) / e(self.factor)).lo;
        (x.is_finite() && x > 0.).then_some(x.min(1e-6))
    }
    fn charge(self, epsilon: Option<f64>) -> Option<f64> {
        let epsilon = if self.factor == 0. { 0. } else { epsilon? };
        if !epsilon.is_finite() || !(0. ..=1e-6).contains(&epsilon) {
            return None;
        }
        let e = Enclosure::point;
        let charge = (e(self.known) + e(self.factor) * e(epsilon)).hi;
        (charge <= 1e-9).then_some(charge)
    }
}

#[derive(Clone, Copy)]
enum ScalarRuntimeContract {
    // Matched scalar glibc on [-4, 4], RN and gradual underflow; deployment
    // eligibility is separate from satisfying this numerical allowance.
    ReviewedGlibc,
}
impl ScalarRuntimeContract {
    fn epsilon(self) -> f64 {
        match self {
            Self::ReviewedGlibc => 2_f64.powi(-40),
        }
    }
}

fn paired_position_charge(budgets: &[PairBudget], runtime: ScalarRuntimeContract) -> Option<f64> {
    let e = Enclosure::point;
    let mut total = e(0.);
    for budget in budgets {
        budget.charge(Some(runtime.epsilon()))?;
        total = total
            + e(if budget.factor > 0. {
                1e-9
            } else {
                budget.known
            });
    }
    total.valid().then_some(total.hi)
}

type Box3 = [Enclosure; 3];
fn boxed(v: Vector3) -> Box3 {
    [
        Enclosure::point(v.x),
        Enclosure::point(v.y),
        Enclosure::point(v.z),
    ]
}
fn box_norm(v: Box3) -> Enclosure {
    v.into_iter()
        .fold(Enclosure::point(0.), |s, x| s + Enclosure::point(x.norm()))
}
fn box_dot(a: Box3, b: Box3) -> Enclosure {
    (0..3).fold(Enclosure::point(0.), |s, j| s + a[j] * b[j])
}
fn box_cross(a: Box3, b: Box3) -> Box3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn affine_matrix(m: Matrix4) -> Option<()> {
    (m.x.w == 0.
        && m.y.w == 0.
        && m.z.w == 0.
        && m.w.w == 1.
        && (0..4).all(|i| (0..4).all(|j| m[i][j].is_finite())))
    .then_some(())
}
fn mapped(m: Matrix4, v: Box3) -> Box3 {
    std::array::from_fn(|j| {
        (0..3).fold(Enclosure::point(0.), |s, k| {
            s + Enclosure::point(m[k][j]) * v[k]
        })
    })
}
fn world_coefficient(input: &Input, v: Box3, point: bool) -> Box3 {
    std::array::from_fn(|j| {
        (0..3).fold(
            Enclosure::point(if point { input.origin[j] } else { 0. }),
            |s, k| s + Enclosure::point(input.axes[k][j]) * v[k],
        )
    })
}
fn dual_frame(axes: [Vector3; 3]) -> Option<[Box3; 3]> {
    let a = axes.map(boxed);
    let cof = [
        box_cross(a[1], a[2]),
        box_cross(a[2], a[0]),
        box_cross(a[0], a[1]),
    ];
    let det = box_dot(a[0], cof[0]);
    if !det.valid() || det.lo <= 0. && det.hi >= 0. {
        return None;
    }
    let dual = cof.map(|v| v.map(|x| x / det));
    dual.iter().flatten().all(|x| x.valid()).then_some(dual)
}

// Exact stored-coefficient Rodrigues algebra: P=aa^T, C=I-P, S=[a]cross.
// No unit-axis substitution is made, including for imported processors.
fn revolution_coefficients(
    m: Matrix4,
    origin: Point3,
    axis: Vector3,
    generator: [Box3; 3],
) -> Option<[[Box3; 3]; 3]> {
    affine_matrix(m)?;
    let e = Enclosure::point;
    let a = boxed(axis);
    let o = boxed(origin.to_vec());
    let mut h = [[[e(0.); 3]; 3]; 3];
    for j in 0..3 {
        let v = std::array::from_fn(|k| generator[j][k] - if j == 0 { o[k] } else { e(0.) });
        let axial = box_dot(a, v);
        h[0][j] = mapped(m, a.map(|x| x * axial));
        h[1][j] = mapped(m, std::array::from_fn(|k| v[k] - a[k] * axial));
        h[2][j] = mapped(m, box_cross(a, v));
    }
    let center = mapped(m, o);
    for k in 0..3 {
        h[0][0][k] = h[0][0][k] + center[k] + e(m.w[k]);
    }
    h.iter().flatten().flatten().all(|x| x.valid()).then_some(h)
}
fn raw_conic_coefficients(curve: &Curve) -> Option<[Box3; 3]> {
    let Curve::Conic(c) = curve else {
        return None;
    };
    let m = *c.transform();
    affine_matrix(m)?;
    Some([
        boxed(m.w.truncate()),
        boxed(m.x.truncate()),
        boxed(m.y.truncate()),
    ])
}
fn conic_coefficients(curve: &Curve) -> Option<[Box3; 3]> {
    let Curve::Conic(c) = curve else {
        return None;
    };
    let m = *c.transform();
    affine_matrix(m)?;
    let mut result = [
        boxed(m.w.truncate()),
        boxed(m.x.truncate()),
        boxed(m.y.truncate()),
    ];
    if !c.orientation() {
        let (lo, hi) = c.range_tuple();
        let phase = Enclosure::point(lo) + Enclosure::point(hi);
        if !phase.valid() || phase.lo < -4. || phase.hi > 4. {
            return None;
        }
        let cs = trig_enclosure(phase, true);
        let sn = trig_enclosure(phase, false);
        result[1] =
            std::array::from_fn(|k| Enclosure::point(m.x[k]) * cs + Enclosure::point(m.y[k]) * sn);
        result[2] =
            std::array::from_fn(|k| Enclosure::point(m.x[k]) * sn - Enclosure::point(m.y[k]) * cs);
    }
    Some(result)
}
fn rephase_coefficients(curve: &Curve, mut h: [Box3; 3]) -> Option<[Box3; 3]> {
    let Curve::Conic(c) = curve else {
        return None;
    };
    if !c.orientation() {
        let (lo, hi) = c.range_tuple();
        let phase = Enclosure::point(lo) + Enclosure::point(hi);
        if !phase.valid() || phase.lo < -4. || phase.hi > 4. {
            return None;
        }
        let cs = trig_enclosure(phase, true);
        let sn = trig_enclosure(phase, false);
        let [_, a, b] = h;
        h[1] = std::array::from_fn(|j| a[j] * cs + b[j] * sn);
        h[2] = std::array::from_fn(|j| a[j] * sn - b[j] * cs);
    }
    Some(h)
}
fn chart_jets(support: &Surface, chart: PairedChart, mut uv: [Jet; 2]) -> Option<[Jet; 2]> {
    let e = Enclosure::point;
    let (curve, inner, swap) = match (support, chart) {
        (Surface::RevolutedCurve(r), PairedChart::A) => {
            (r.entity().entity_curve(), 1, r.orientation())
        }
        (Surface::Extruded(r), PairedChart::B) => (r.entity_curve(), 0, false),
        _ => return Some(uv),
    };
    let Curve::Conic(c) = curve else {
        return None;
    };
    if !c.orientation() {
        let (lo, hi) = c.range_tuple();
        uv[inner] = Jet::constant(e(lo) + e(hi)) + uv[inner].scale(e(-1.));
    }
    if swap {
        uv.swap(0, 1);
    }
    Some(uv)
}
fn coefficient_error(a: [Box3; 3], b: [Box3; 3]) -> Enclosure {
    (0..3).fold(Enclosure::point(0.), |s, i| {
        s + box_norm(std::array::from_fn(|j| a[i][j] - b[i][j]))
    })
}
// On each endpoint polynomial piece, the absolute Bernstein basis sum is
// (1+2 delta/h)^degree. Interior Bezier extraction is convex. Multiplying
// the two axis bounds covers the tensor basis and also derivative bases.
fn basis_neighborhood(knots: &KnotVec, degree: usize, hull: Enclosure) -> Option<Enclosure> {
    let e = Enclosure::point;
    if degree == 0
        || knots.len() < 2 * (degree + 1)
        || !hull.valid()
        || knots.iter().any(|x| !x.is_finite())
    {
        return None;
    }
    if knots.windows(2).any(|x| x[1] < x[0]) {
        return None;
    }
    let lo = knots[degree];
    let hi = knots[knots.len() - degree - 1];
    if lo >= hi
        || knots[..=degree].iter().any(|x| *x != lo)
        || knots[knots.len() - degree - 1..].iter().any(|x| *x != hi)
    {
        return None;
    }
    let h = knots
        .windows(2)
        .filter(|x| x[1] > x[0])
        .map(|x| (e(x[1]) - e(x[0])).lo)
        .fold(f64::INFINITY, f64::min);
    if !h.is_finite() || h <= 0. || !(e(1.) / e(h)).valid() {
        return None;
    }
    let delta = (e(lo) - e(hull.lo)).hi.max((e(hull.hi) - e(hi)).hi).max(0.);
    // A whole escaped knot span is a domain escape, not endpoint roundoff.
    if delta >= h {
        return None;
    }
    let growth = e(1.) + e(2.) * e(delta) / e(h);
    let bound = (0..degree).fold(e(1.), |a, _| a * growth);
    bound.valid().then_some(bound)
}

fn absolute_map(m: Matrix4, v: Box3) -> Box3 {
    std::array::from_fn(|j| {
        (0..3).fold(Enclosure::point(m.w[j].abs()), |s, k| {
            s + Enclosure::point(m[k][j].abs()) * v[k]
        })
    })
}
fn curve_arguments(curve: &Curve, domain: Enclosure) -> Option<Vec<Enclosure>> {
    let e = Enclosure::point;
    let mut arguments = Vec::new();
    if let Curve::Conic(c) = curve {
        if c.orientation() {
            arguments.push(domain);
        } else {
            let (lo, hi) = c.range_tuple();
            let phase = e(lo) + e(hi);
            let rounding = basic_gamma(3) * (e(lo.abs()) + e(hi.abs()) + e(domain.norm()));
            let angle = phase - domain;
            arguments.push(phase);
            arguments.push(Enclosure {
                lo: (e(angle.lo) - rounding).lo,
                hi: (e(angle.hi) + rounding).hi,
            });
        }
    }
    arguments
        .iter()
        .all(|x| x.valid() && x.lo >= -4. && x.hi <= 4.)
        .then_some(arguments)
}
fn support_arguments(support: &Surface, hull: [Enclosure; 2]) -> Option<Vec<Enclosure>> {
    let arguments = match support {
        Surface::RevolutedCurve(r) => {
            let inner = if r.orientation() { 0 } else { 1 };
            let mut args = curve_arguments(r.entity().entity_curve(), hull[inner])?;
            args.push(hull[1 - inner]);
            args
        }
        Surface::Extruded(s) => curve_arguments(s.entity_curve(), hull[0])?,
        _ => Vec::new(),
    };
    if arguments
        .iter()
        .any(|x| !x.valid() || x.lo < -4. || x.hi > 4.)
    {
        return None;
    }
    Some(arguments)
}

fn curve_absolute(curve: &Curve, domain: Enclosure) -> Option<Box3> {
    let e = Enclosure::point;
    curve_arguments(curve, domain)?;
    match curve {
        Curve::Line(line) => Some(std::array::from_fn(|j| {
            e(line.0[j].abs()) + (e(line.1[j].abs()) + e(line.0[j].abs())) * e(domain.norm())
        })),
        Curve::Conic(c) => {
            let m = *c.transform();
            affine_matrix(m)?;
            let cap = e(1.) + e(1e-6);
            // Processor evaluates lo+hi-t before the circle. Its argument
            // error is at most gamma(3)(|lo|+|hi|+|t|). Carry that formation
            // magnitude through the same absolute transforms as circle error.
            let formation = if c.orientation() {
                e(0.)
            } else {
                let (lo, hi) = c.range_tuple();
                e(lo.abs()) + e(hi.abs()) + e(domain.norm())
            };
            Some(absolute_map(m, [cap + formation, cap + formation, e(0.)]))
        }
        _ => None,
    }
}
fn basic_gamma(operations: usize) -> Enclosure {
    let e = Enclosure::point;
    let nu = e(operations as f64) * e(f64::EPSILON / 2.);
    nu / (e(1.) - nu)
}
fn revolution_absolute(
    m: Matrix4,
    origin: Point3,
    axis: Vector3,
    generator: &Curve,
    domain: Enclosure,
) -> Option<f64> {
    affine_matrix(m)?;
    let e = Enclosure::point;
    let g = curve_absolute(generator, domain)?;
    let a = boxed(axis).map(|x| e(x.norm()));
    let o = boxed(origin.to_vec()).map(|x| e(x.norm()));
    let cap = e(1.) + e(1e-6);
    // cgmath forms 1-c, nine matrix entries, then R*(g-o)+o and M*p+t.
    // Absolute values precede every subtraction, so affine cancellation cannot
    // erase the work magnitude. A path uses <=132 basic operations: generator
    // <=40, Rodrigues <=37, vector product/add <=21, affine homogeneous <=31,
    // and orientation <=3.
    let rotated = std::array::from_fn(|j| {
        (0..3).fold(o[j], |s, k| {
            let r = (e(1.) + cap) * a[j] * a[k] + if j == k { cap } else { cap * a[3 - j - k] };
            s + r * (g[k] + o[k])
        })
    });
    let bound = box_norm(absolute_map(m, rotated));
    bound.valid().then_some(bound.hi)
}

fn ideal_trace(input: &Input, edge: usize, t: Enclosure) -> Option<Box3> {
    let e = Enclosure::point;
    let p = match edge {
        2 => {
            let mut p = input.trim_jets(t, Trim::Dc)?;
            // trim_box brackets the unsquared z=0 equation and proves a positive
            // t derivative over its domain. Every accepted angle (including
            // endpoint neighborhoods) therefore has one root with exactly z=0.
            // Tighten only this ideal trace; stored spatial errors stay charged.
            p[2] = Jet::constant(e(0.));
            p
        }
        5 => input.ab_jets(-t)?,
        9 => input.trim_jets(-t, Trim::Bc)?,
        11 => input.contact_jets(t, true)?,
        12 => input.trim_jets(-t, Trim::Ci)?,
        13 => input.contact_jets(-t, false)?,
        6 => {
            let (sn, cs) = trig_jets(t);
            [
                Jet::constant(e(input.ring())) + cs.scale(e(-input.r)),
                Jet::constant(e(0.)),
                Jet::constant(e(input.r)) + sn.scale(e(input.r)),
            ]
        }
        _ => return None,
    };
    Some(p.map(|j| j.0[0]))
}
fn plane_projection_error(input: &Input, plane: &Plane, local: Box3) -> Option<f64> {
    let e = Enclosure::point;
    let a = boxed(plane.u_axis());
    let b = boxed(plane.v_axis());
    let aa = box_dot(a, a);
    let ab = box_dot(a, b);
    let bb = box_dot(b, b);
    let determinant = aa * bb - ab * ab;
    if !determinant.valid() || determinant.lo <= 0. {
        return None;
    }
    let inverse = [
        [bb / determinant, -ab / determinant],
        [-ab / determinant, aa / determinant],
    ];
    let project = |v: Box3| {
        let rhs = [box_dot(a, v), box_dot(b, v)];
        let uv = inverse.map(|row| row[0] * rhs[0] + row[1] * rhs[1]);
        std::array::from_fn::<_, 3, _>(|j| a[j] * uv[0] + b[j] * uv[1] - v[j])
    };
    // Expand (A (A^T A)^-1 A^T-I)(origin-plane.origin+E*x).
    // Keeping the constant and E columns separate preserves exact plane
    // cancellation; an interval box for world X before projection would not.
    let constant = project(std::array::from_fn(|j| {
        e(input.origin[j]) - e(plane.origin()[j])
    }));
    let columns = input.axes.map(|v| project(boxed(v)));
    let residual =
        std::array::from_fn(|j| (0..3).fold(constant[j], |s, k| s + columns[k][j] * local[k]));
    let bound = box_norm(residual);
    bound.valid().then_some(bound.hi)
}

fn support_composition(
    input: &Input,
    support: &Surface,
    chart: PairedChart,
    hull: [Enclosure; 2],
) -> Option<([f64; 2], f64, f64, f64)> {
    let e = Enclosure::point;
    if hull.iter().any(|x| !x.valid()) {
        return None;
    }
    let dual = dual_frame(input.axes)?;
    match support {
        Surface::RevolutedCurve(rev) => {
            let m = *rev.transform();
            affine_matrix(m)?;
            let origin = rev.entity().origin();
            let axis = rev.entity().axis();
            let eta = if rev.orientation() { 0 } else { 1 };
            if hull[1 - eta].lo < -4. || hull[1 - eta].hi > 4. {
                return None;
            }
            let absolute =
                revolution_absolute(m, origin, axis, rev.entity().entity_curve(), hull[eta])?;
            if let Curve::Line(line) = rev.entity().entity_curve() {
                let (lo, hi) = line.range_tuple();
                let escape = (e(lo) - e(hull[eta].lo))
                    .hi
                    .max((e(hull[eta].hi) - e(hi)).hi)
                    .max(0.);
                let span = e(hi) - e(lo);
                if !span.valid() || span.lo <= 0. || escape >= span.lo {
                    return None;
                }
                let extension = e(1.) + e(2.) * e(escape) / span;
                let g = [boxed(line.0.to_vec()), boxed(line.1 - line.0), [e(0.); 3]];
                let h = revolution_coefficients(m, origin, axis, g)?;
                let angular =
                    (box_norm(std::array::from_fn(|k| h[1][0][k] + h[1][1][k] * hull[eta]))
                        + box_norm(std::array::from_fn(|k| h[2][0][k] + h[2][1][k] * hull[eta])))
                    .hi;
                let linear = (box_norm(h[0][1]) + box_norm(h[1][1]) + box_norm(h[2][1])).hi;
                let discrepancy = match chart {
                    PairedChart::I => {
                        (e(
                            certify_cone(support, input.origin, input.axes, input.n, input.a)?
                                .displacement,
                        ) * extension)
                            .hi
                    }
                    PairedChart::O => {
                        (e(
                            certify_cone(support, input.origin, input.axes, -input.m, input.b)?
                                .displacement,
                        ) * extension)
                            .hi
                    }
                    // A disk has constant local height; compare each actual harmonic coefficient
                    // to its reconstruction in the recovered radial frame, over the whole hull.
                    PairedChart::D => {
                        let mut error = e(0.);
                        for u in [hull[eta].lo, hull[eta].hi] {
                            let actual = std::array::from_fn(|i| {
                                std::array::from_fn(|k| h[i][0][k] + h[i][1][k] * e(u))
                            });
                            let p = mapped(m, std::array::from_fn(|k| g[0][k] + g[1][k] * e(u)));
                            let relative =
                                std::array::from_fn(|k| p[k] + e(m.w[k]) - e(input.origin[k]));
                            let local = dual.map(|v| box_dot(v, relative));
                            let expected = [
                                boxed(input.origin.to_vec()),
                                world_coefficient(input, [local[0], local[1], e(0.)], false),
                                world_coefficient(input, [-local[1], local[0], e(0.)], false),
                            ];
                            error = e(error.hi.max(coefficient_error(actual, expected).hi));
                        }
                        error.hi
                    }
                    _ => return None,
                };
                let magnitude = absolute;
                Some((
                    if eta == 0 {
                        [linear, angular]
                    } else {
                        [angular, linear]
                    },
                    angular,
                    magnitude,
                    discrepancy,
                ))
            } else {
                if !matches!(chart, PairedChart::A) {
                    return None;
                }
                let g = conic_coefficients(rev.entity().entity_curve())?;
                let h = revolution_coefficients(m, origin, axis, g)?;
                // Runtime errors occur in the original sin/cos channels, before
                // exact phase expansion. Rephased coefficients are comparison
                // coefficients, not a replacement for those independent errors.
                let runtime_h = revolution_coefficients(
                    m,
                    origin,
                    axis,
                    raw_conic_coefficients(rev.entity().entity_curve())?,
                )?;
                let inv = e(1.) / (e(1.) + e(input.n) * e(input.n)).sqrt();
                let ring = e(input.a)
                    + e(input.n) * e(input.r)
                    + e(input.r) * (e(1.) + e(input.n) * e(input.n)).sqrt();
                let desired = [
                    [e(0.), ring, e(input.r)],
                    [e(0.), -e(input.r) * inv, e(input.r) * e(input.n) * inv],
                    [e(0.), -e(input.r) * e(input.n) * inv, -e(input.r) * inv],
                ];
                let desired = rephase_coefficients(rev.entity().entity_curve(), desired)?;
                // Expected rotation is in the Input frame; never project using E^T.
                let mut expected = [[[e(0.); 3]; 3]; 3];
                for j in 0..3 {
                    let local = desired[j];
                    expected[0][j] = world_coefficient(input, [e(0.), e(0.), local[2]], j == 0);
                    expected[1][j] = world_coefficient(input, [local[0], local[1], e(0.)], false);
                    expected[2][j] = world_coefficient(input, [-local[1], local[0], e(0.)], false);
                }
                let mut outer = e(0.);
                let mut inner = e(0.);
                let mut q = e(0.);
                let mut error = e(0.);
                for i in 0..3 {
                    error = error + coefficient_error(h[i], expected[i]);
                    for (j, v) in runtime_h[i].iter().enumerate() {
                        let n = box_norm(*v);
                        if i > 0 {
                            outer = outer + n;
                        }
                        if j > 0 {
                            inner = inner + n;
                        }
                        if i > 0 && j > 0 {
                            q = q + n;
                        }
                    }
                }
                Some((
                    if eta == 0 {
                        [inner.hi, outer.hi]
                    } else {
                        [outer.hi, inner.hi]
                    },
                    (outer + inner + q * e(1e-6)).hi,
                    absolute,
                    error.hi,
                ))
            }
        }
        Surface::Extruded(s) => {
            let g = conic_coefficients(s.entity_curve())?;
            let d = boxed(s.extruding_vector());
            let expected = [
                [e(0.), e(input.r), e(input.r)],
                [e(0.), e(-input.r), e(0.)],
                [e(0.), e(0.), e(-input.r)],
            ];
            let expected = std::array::from_fn(|j| world_coefficient(input, expected[j], j == 0));
            let expected = rephase_coefficients(s.entity_curve(), expected)?;
            let ed = world_coefficient(input, [e(input.b), e(0.), e(0.)], false);
            let discrepancy = coefficient_error(g, expected)
                + box_norm(std::array::from_fn(|j| d[j] - ed[j])) * e(hull[1].norm());
            let raw = raw_conic_coefficients(s.entity_curve())?;
            let factor = box_norm(raw[1]) + box_norm(raw[2]);
            Some((
                [factor.hi, box_norm(d).hi],
                factor.hi,
                (box_norm(curve_absolute(s.entity_curve(), hull[0])?)
                    + box_norm(d) * e(hull[1].norm()))
                .hi,
                discrepancy.hi,
            ))
        }
        Surface::Plane(p) => {
            let a = boxed(p.u_axis());
            let b = boxed(p.v_axis());
            let normal = box_cross(a, b);
            let nn = box_dot(normal, normal);
            if !nn.valid() || nn.lo <= 0. {
                return None;
            }
            Some((
                [box_norm(a).hi, box_norm(b).hi],
                0.,
                (box_norm(boxed(p.origin().to_vec()))
                    + box_norm(a) * e(hull[0].norm())
                    + box_norm(b) * e(hull[1].norm()))
                .hi,
                0.,
            ))
        }
        Surface::NurbsSurface(s) => {
            let points = s.control_points();
            let kv = s.knot_vecs();
            let degrees = s.degrees();
            let ks = [&kv.0, &kv.1];
            let degree = [degrees.0, degrees.1];
            let mut basis = e(1.);
            for j in 0..2 {
                basis = basis * basis_neighborhood(ks[j], degree[j], hull[j])?;
            }
            let wmin = points
                .iter()
                .flatten()
                .map(|p| p.w)
                .fold(f64::INFINITY, f64::min);
            if !wmin.is_finite()
                || wmin <= 0.
                || points
                    .iter()
                    .flatten()
                    .any(|p| (0..4).any(|j| !p[j].is_finite()))
            {
                return None;
            }
            let wmax = points.iter().flatten().map(|p| p.w).fold(0_f64, f64::max);
            let lower = e(wmin) - e(wmax) * (basis - e(1.));
            if !lower.valid() || lower.lo <= 0. {
                return None;
            }
            let maximum = points
                .iter()
                .flatten()
                .map(|p| box_norm(boxed(p.truncate())).hi)
                .fold(0_f64, f64::max);
            let mut lip = [0.; 2];
            for axis in 0..2 {
                let count = if axis == 0 {
                    points.len()
                } else {
                    points[0].len()
                };
                let mut dn = 0_f64;
                let mut dw = 0_f64;
                for i in 0..count - 1 {
                    let den = e(ks[axis][i + degree[axis] + 1]) - e(ks[axis][i + 1]);
                    if !den.valid() || den.lo <= 0. {
                        return None;
                    }
                    let mut include = |a: Vector4, b: Vector4| {
                        let diff = std::array::from_fn(|k| e(b[k]) - e(a[k]));
                        dn = dn.max((e(degree[axis] as f64) * box_norm(diff) / den).hi);
                        dw =
                            dw.max((e(degree[axis] as f64) * e((e(b.w) - e(a.w)).norm()) / den).hi);
                    };
                    if axis == 0 {
                        for (&a, &b) in points[i].iter().zip(&points[i + 1]) {
                            include(a, b);
                        }
                    } else {
                        for row in points {
                            include(row[i], row[i + 1]);
                        }
                    }
                }
                lip[axis] = (e(dn) * basis / lower
                    + e(maximum) * basis * e(dw) * basis / (lower * lower))
                    .hi;
            }
            // Absolute numerator and denominator evaluation errors both enter N/W.
            // The inherited spline contract bounds each homogeneous sum; the
            // denominator perturbation must remain smaller than its lower bound.
            // Each axis basis recurrence is triangular (<=32(p+1)^2
            // elementary operations); the active tensor sum costs <=16(p+1)(q+1).
            // Keep the inherited 2048-operation floor, but scale for actual degrees.
            let p = e((degree[0] + 1) as f64);
            let q = e((degree[1] + 1) as f64);
            let operations = e(32.) * (p * p + q * q) + e(16.) * p * q + e(4.);
            let count = e(operations.hi.max(2048.));
            let nu = count * e(f64::EPSILON / 2.);
            if !nu.valid() || nu.hi >= 1. {
                return None;
            }
            let gamma = nu / (e(1.) - nu);
            let denominator_error = gamma * e(wmax) * basis;
            let safe = lower - denominator_error;
            if !safe.valid() || safe.lo <= 0. {
                return None;
            }
            let magnitude = e(maximum) * basis / safe * (e(1.) + e(wmax) * basis / lower) * gamma
                / (e(2048.) * e(f64::EPSILON / 2.));
            Some((lip, 0., magnitude.hi, 0.))
        }
        _ => None,
    }
}

#[cfg(test)]
mod typed_composition_tests {
    use super::*;
    #[test]
    fn typed_rodrigues_keeps_stored_axis_and_affine_transform() {
        let e = Enclosure::point;
        let m = Matrix4::from_cols(
            Vector4::new(0., 2., 0., 0.),
            Vector4::new(-2., 0., 0., 0.),
            Vector4::new(0., 0., 2., 0.),
            Vector4::new(13., -7., 4., 1.),
        );
        let origin = Point3::new(1., 2., 3.);
        let axis = Vector3::new(0., 0., 1. + 1e-13);
        let g = [
            boxed(Vector3::new(3., 2., 4.)),
            boxed(Vector3::new(1., 0., 0.)),
            [e(0.); 3],
        ];
        let h = revolution_coefficients(m, origin, axis, g).unwrap();
        assert!(h.iter().flatten().flatten().all(|x| x.valid()));
        let identity = revolution_coefficients(Matrix4::identity(), origin, axis, g).unwrap();
        assert_ne!(h[0][0][0].lo, identity[0][0][0].lo);
        let bad = Matrix4::from_nonuniform_scale(f64::NAN, 1., 1.);
        assert!(revolution_coefficients(bad, origin, axis, g).is_none());
    }
    #[test]
    fn reversed_conic_chart_expands_phase() {
        let mut c: Curve = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., FRAC_PI_2)),
            Matrix4::from_scale(2.),
        )
        .into();
        c.invert();
        let h = conic_coefficients(&c).unwrap();
        // cos(pi/2-u)=sin(u), sin(pi/2-u)=cos(u).
        assert!(h[1][1].lo <= 2. && h[1][1].hi >= 2.);
        assert!(h[2][0].lo <= 2. && h[2][0].hi >= 2.);
    }
    #[test]
    fn dual_frame_rejects_singular_and_retains_skew() {
        assert!(dual_frame([Vector3::unit_x(); 3]).is_none());
        let axes = [
            Vector3::unit_x(),
            Vector3::new(1e-13, 1., 0.),
            Vector3::unit_z(),
        ];
        let dual = dual_frame(axes).unwrap();
        assert!(dual[0][1].hi < 0.);
        for (i, d) in dual.iter().enumerate() {
            for (j, axis) in axes.iter().enumerate() {
                let dot = box_dot(*d, boxed(*axis));
                let target = if i == j { 1. } else { 0. };
                assert!(dot.lo <= target && dot.hi >= target);
            }
        }
    }
    #[test]
    fn imported_coefficient_perturbation_obeys_budget() {
        let e = Enclosure::point;
        let zero = [[e(0.); 3]; 3];
        let mut imported = zero;
        imported[1][0] = e(1e-12);
        let small = coefficient_error(imported, zero).hi;
        assert!(PairBudget::new(&[small], 0.)
            .unwrap()
            .charge(None)
            .is_some());
        imported[1][0] = e(1e-8);
        let large = coefficient_error(imported, zero).hi;
        assert!(PairBudget::new(&[large], 0.)
            .unwrap()
            .charge(None)
            .is_none());
        assert!(PairBudget::new(&[f64::NAN], 0.).is_none());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn composition_contract_is_required() {
        let p = PairBudget::new(&[1e-11, 2e-11], 50.).unwrap();
        assert!(p.charge(None).is_none());
        assert!(p.charge(Some(p.epsilon_max().unwrap() / 2.)).is_some());
        assert!(p.charge(Some(1e-6)).is_none());
        assert!(PairBudget::new(&[f64::NAN], 0.).is_none());
        assert!(PairBudget::new(&[-1.], 0.).is_none());
        assert!(PairBudget::new(&[2e-9], 0.).unwrap().charge(None).is_none());
        assert!(PairBudget::new(&[1e-11], 0.)
            .unwrap()
            .charge(None)
            .is_some());
    }

    use super::*;

    #[test]
    fn uv_angle_series_rejects_unknown_and_encloses_signs() {
        let e = Enclosure::point;
        for y in [-0.1_f64, 0., 0.1] {
            let angle = angle_jet(Jet::constant(e(1.)), Jet::variable(e(y))).unwrap();
            assert!(angle.0[0].lo <= y.atan() && y.atan() <= angle.0[0].hi);
            let derivative = 1. / (1. + y * y);
            assert!(angle.0[1].lo <= derivative && derivative <= angle.0[1].hi);
        }
        assert!(angle_jet(Jet::constant(e(0.)), Jet::constant(e(0.))).is_none());
        assert!(angle_jet(Jet::constant(e(1.)), Jet::constant(e(0.2))).is_none());
        assert!(angle_jet(Jet::constant(e(1.)), Jet::constant(e(f64::NAN))).is_none());
    }

    #[test]
    fn uv_composition_does_not_drop_unknown_allowances() {
        let mut uv = StoredUv {
            curve: BSplineCurve::new(
                KnotVec::from(vec![0., 0., 1., 1.]),
                vec![Point2::new(0., 0.), Point2::new(1., 1.)],
            ),
            construction: [1e-12; 2],
            truncation: [2e-12; 2],
            evaluation: [3e-12; 2],
        };
        let value = uv.composed([2., 3.]).unwrap();
        assert!((3e-11..3.01e-11).contains(&value));
        assert!(uv.composed([1., f64::NAN]).is_none());
        uv.evaluation[1] = f64::NAN;
        assert!(uv.composed([1., 0.]).is_none());
    }

    #[test]
    fn stored_cone_rejects_anisotropy_and_skew_generator() {
        let support = |skew: f64, transform: Matrix4| -> Surface {
            let line: Curve = Line(Point3::new(20., 0., 0.), Point3::new(25., skew, 5.)).into();
            let mut rev = Processor::new(RevolutedCurve::by_revolution(
                line,
                Point3::origin(),
                Vector3::unit_z(),
            ));
            rev.transform_by(transform);
            rev.into()
        };
        let axes = [Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z()];
        let exact = support(0., Matrix4::identity());
        assert!(certify_cone(&exact, Point3::origin(), axes, 1., 20.).is_some());
        let anisotropic = support(0., Matrix4::from_nonuniform_scale(1., 1. + 1e-8, 1.));
        assert!(certify_cone(&anisotropic, Point3::origin(), axes, 1., 20.).is_none());
        let skew = support(1e-7, Matrix4::identity());
        assert!(certify_cone(&skew, Point3::origin(), axes, 1., 20.).is_none());
        let rotation = Matrix4::from_angle_z(Rad(0.3));
        let translation = Vector3::new(100., -50., 20.);
        let transformed = support(
            0.,
            Matrix4::from_translation(translation) * rotation * Matrix4::from_scale(2.),
        );
        let axes = [
            rotation.x.truncate(),
            rotation.y.truncate(),
            rotation.z.truncate(),
        ];
        assert!(certify_cone(&transformed, Point3::from_vec(translation), axes, 1., 40.).is_some());
    }
    #[test]
    fn supported_runtime_must_fit_every_pair_budget() {
        let runtime = ScalarRuntimeContract::ReviewedGlibc;
        let inadequate = PairBudget::new(&[1e-11], 2000.).unwrap();
        assert!(inadequate.epsilon_max().unwrap() > 0.);
        assert!(inadequate.epsilon_max().unwrap() < runtime.epsilon());
        assert!(paired_position_charge(&[inadequate], runtime).is_none());
        let supported = PairBudget::new(&[1e-11], 50.).unwrap();
        assert!(paired_position_charge(&[supported], runtime).is_some());
        assert!(paired_position_charge(&[supported, inadequate], runtime).is_none());
        let structural = PairBudget::new(&[1e-11], 0.).unwrap();
        assert!(paired_position_charge(&[structural], runtime).is_some());
    }

    #[test]
    fn stored_endpoint_derivative_is_not_truncation() {
        let e = Enclosure::point;
        let h = 1e-4;
        let points: [Point3; 4] = std::array::from_fn(|j| Point3::new(h * j as f64 / 3., 0., 0.));
        let ideal = points.map(|p| [e(p.x), e(p.y), e(p.z)]);
        assert!(certify_span(&points, ideal, e(h), 0.).unwrap().accepts());
        let mut perturbed = points;
        perturbed[0].x += 1e-11;
        assert!(ideal[0][0].error(perturbed[0].x) < 1e-10);
        assert!(certify_span(&perturbed, ideal, e(h), 0.).is_none());
        assert!((3. * (perturbed[1].x - perturbed[0].x) / h - 1.).abs() > 2.99e-7);
    }
    #[test]
    fn inverse_cosine_keeps_target_uncertainty() {
        let e = Enclosure::point;
        let half = inverse_cosine(e(0.)).unwrap();
        assert!(half.lo < FRAC_PI_2 && half.hi > FRAC_PI_2);
        let wide = inverse_cosine(Enclosure {
            lo: -1e-6,
            hi: 1e-6,
        })
        .unwrap();
        assert!(wide.lo < half.lo && wide.hi > half.hi);
        assert!(wide.hi - wide.lo > 1.99e-6);
        assert!(inverse_cosine(e(f64::NAN)).is_none());
        assert!(inverse_cosine(e(1.01)).is_none());
    }
    #[test]
    fn finite_controller_and_unknown_arithmetic_reject() {
        let e = Enclosure::point;
        assert!(!(e(1.) / Enclosure { lo: -1., hi: 1. }).valid());
        assert!(!e(-1.).sqrt().valid());
        assert!(upward(0.) > 0. && downward(0.) < 0.);
        assert!(fit_count((0., 1.), 1.).is_some());
        assert!(fit_count((0., 1.), f64::NAN).is_none());
        assert!(fit_count((0., 1.), 1e100).is_none());
        assert!(fit_count((1., 1.), 1.).is_none());
        let x = Jet::variable(Enclosure { lo: 1., hi: 2. });
        let fourth = (x * x * x * x).0[4];
        assert!(fourth.lo <= 1. && fourth.hi >= 1.);
        let sine = trig_enclosure(Enclosure { lo: 1.5, hi: 1.6 }, false);
        assert!(sine.hi >= 1. && sine.lo < 1.);
        assert!(!trig_enclosure(e(4.), false).valid());
    }
    #[test]
    fn opposite_uses_and_closed_loops() {
        let mut uses = [0; 18];
        let mut counts = [0; 18];
        for boundary in LOOPS {
            let oriented: Vec<_> = boundary
                .iter()
                .map(|&i| {
                    let k = i.unsigned_abs() as usize - 1;
                    uses[k] += i.signum();
                    counts[k] += 1;
                    let (a, b) = ENDS[k];
                    if i > 0 {
                        (a, b)
                    } else {
                        (b, a)
                    }
                })
                .collect();
            for i in 0..oriented.len() {
                assert_eq!(oriented[i].1, oriented[(i + 1) % oriented.len()].0);
            }
        }
        assert_eq!(uses, [0; 18]);
        assert_eq!(counts, [2; 18]);
    }
    #[test]
    fn procedural_candidate_retains_shared_edges() {
        let vs = [
            Point3::new(0., 0., 0.),
            Point3::new(10., 0., 0.),
            Point3::new(5., 0., 5.),
        ]
        .map(Vertex::new);
        let profile = builder::try_attach_plane(&[wire![
            builder::line(&vs[0], &vs[1]),
            builder::line(&vs[1], &vs[2]),
            builder::line(&vs[2], &vs[0])
        ]])
        .unwrap();
        let source: Solid = builder::rsweep(
            &profile.inverse(),
            Point3::new(-20., 0., 0.),
            Vector3::unit_z(),
            Rad(FRAC_PI_2),
            1,
        );
        let shell = &source.boundaries()[0];
        let mut selected = HashSet::new();
        for e in shell.edge_iter() {
            let p = e.front().point();
            let q = e.back().point();
            if (p.z.abs() < 1e-10
                && q.z.abs() < 1e-10
                && (p - Point3::new(-20., 0., 0.)).magnitude() < 20.0000001
                && (q - Point3::new(-20., 0., 0.)).magnitude() < 20.0000001)
                || (p.y.abs() < 1e-10
                    && q.y.abs() < 1e-10
                    && ((p.z.abs() < 1e-10 && q.z.abs() < 1e-10) || (p.x > 0. && q.x > 0.)))
            {
                selected.insert(e.id());
            }
        }
        for r in [0.5, 1., 1.5] {
            let result = build_candidate(shell, &selected, r).expect("candidate topology");
            assert_eq!(result.len(), 8);
            assert_eq!(
                result
                    .edge_iter()
                    .map(|e| e.id())
                    .collect::<HashSet<_>>()
                    .len(),
                18
            );
            assert_eq!(
                result
                    .vertex_iter()
                    .map(|v| v.id())
                    .collect::<HashSet<_>>()
                    .len(),
                12
            );
            assert!(result.is_geometric_consistent());
            let input = Input::recognize(shell, &selected, r).unwrap();
            let (_, curves, canal) = candidate(&input).unwrap();
            closure_tests::actual_charts(&input, &result);
            if r == 1. {
                closure_tests::transformed_inputs(&source, &selected);
            }
            let endpoints = input.endpoint_gate(&result).unwrap();
            let (_, paired) = paired_uv_rows(&input, &result, &curves, &canal, &endpoints).unwrap();
            assert_eq!(paired.as_array().unwrap().len(), 14);
            for row in paired.as_array().unwrap() {
                for residual in row["sample_residuals_not_certificate"].as_array().unwrap() {
                    assert!(residual.as_f64().unwrap() <= 1e-9, "{row}");
                }
                assert!(row["paired_certificate"]["K"].as_f64().unwrap() < 1e-9);
                assert!(
                    row["paired_certificate"]["conditional_2_pow_minus_40"]
                        .as_f64()
                        .unwrap()
                        < 1e-9
                );
            }
            assert!(stored_uv(
                &input,
                &result.get(1).unwrap().surface(),
                5,
                PairedChart::A,
                &[0., -1.]
            )
            .is_none());
            assert!(stored_uv(
                &input,
                &result.get(1).unwrap().surface(),
                5,
                PairedChart::A,
                &[0., f64::NAN]
            )
            .is_none());
            assert!(input.separation_gate(1e-7).is_some());
            assert!(input.separation_gate(1.).is_none());
            assert!(input.separation_gate(f64::NAN).is_none());
            let e = Enclosure::point;
            let physical = input.physical_domains().unwrap()[1].0;
            let stored = -FRAC_PI_2;
            let point = input.point(input.ab(-stored).unwrap().0);
            let bounds = |x: Enclosure| input.ab_jets(-x).map(reverse_jets);
            assert!(endpoint_allowance(&input, physical, stored, point, bounds).is_some());
            assert!(endpoint_allowance(&input, physical, stored + 1e-7, point, bounds).is_none());
            assert!(endpoint_allowance(
                &input,
                physical,
                stored,
                point + Vector3::new(1e-7, 0., 0.),
                bounds
            )
            .is_none());
            assert!(endpoint_allowance(&input, e(f64::NAN), stored, point, bounds).is_none());

            let phi_end = input
                .outer_phi(input.height() - r * input.m / (1. + input.m * input.m).sqrt())
                .unwrap();
            let (Surface::NurbsSurface(net), _) = canal_surface(&input, phi_end).unwrap() else {
                panic!("canal representation")
            };
            let ts: Vec<_> = (0..=256)
                .map(|i| -r + (input.height() + r) * i as f64 / 256.)
                .collect();
            let s = (phi_end / 2.).tan();
            assert!(certify_canal(&input, &ts, s, net.control_points())
                .unwrap()
                .accepts());
            let mut wrong = net.control_points().clone();
            wrong[5][1].x += 1e-5;
            assert!(certify_canal(&input, &ts, s, &wrong).is_none());
            let mut wrong = net.control_points().clone();
            wrong[5][1].w = 0.;
            assert!(certify_canal(&input, &ts, s, &wrong).is_none());
            let mut wrong_knots = ts.clone();
            wrong_knots[10] = wrong_knots[9];
            assert!(certify_canal(&input, &wrong_knots, s, net.control_points()).is_none());
            let start = input.trim(0., Trim::Ci).unwrap().0.z;
            let mut minimum = start;
            // CI's stored parameter is -phi; retain its interior remote lobe.
            let ci = result[3].boundaries()[0][2].curve();
            for j in 0..=128 {
                let phi = phi_end * j as f64 / 128.;
                let (p, _) = input.trim(phi, Trim::Ci).unwrap();
                assert!(ci.subs(-phi).distance(input.point(p)) < 1e-8);
                let t = root(
                    |t| {
                        let (q, _, _) = input.canal(t, phi)?;
                        Some(input.a + input.n * q.z - q.x.hypot(q.y))
                    },
                    input.height() - 2. * r,
                    input.height(),
                )
                .unwrap();
                minimum = minimum.min(t);
            }
            assert!(minimum < start - 1e-8, "CI must preserve the remote lobe");
        }
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;
    #[test]
    fn reversed_phase_formation_is_charged_before_cancellation() {
        let mut curve: Curve = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (-1e6, 1e6 + 1.)),
            Matrix4::identity(),
        )
        .into();
        curve.invert();
        let domain = Enclosure { lo: 0., hi: 1. };
        assert!(curve_arguments(&curve, domain).is_some());
        let magnitude = box_norm(curve_absolute(&curve, domain).unwrap());
        assert!(magnitude.lo > 1e6);
        assert!(PairBudget::new(&[(basic_gamma(512) * magnitude).hi], 0.)
            .unwrap()
            .charge(None)
            .is_none());
    }
    #[test]
    fn runtime_domain_includes_reversal_phase() {
        let mut curve: Curve = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (2., 3.)),
            Matrix4::identity(),
        )
        .into();
        assert!(curve_arguments(&curve, Enclosure { lo: 2., hi: 3. }).is_some());
        curve.invert();
        assert!(curve_arguments(&curve, Enclosure { lo: 2., hi: 3. }).is_none());
        assert!(curve_arguments(
            &curve,
            Enclosure {
                lo: f64::NAN,
                hi: 3.
            }
        )
        .is_none());
    }
    #[test]
    fn endpoint_basis_and_denominator_controls() {
        let e = Enclosure::point;
        let k = KnotVec::bezier_knot(2);
        let b = basis_neighborhood(
            &k,
            2,
            Enclosure {
                lo: -1e-13,
                hi: 1. + 1e-13,
            },
        )
        .unwrap();
        assert!(b.hi > 1. && b.hi < 1.00000000001);
        assert!(basis_neighborhood(&k, 2, Enclosure { lo: -2., hi: 1. }).is_none());
        let collapsed = KnotVec::from(vec![0.; 6]);
        assert!(basis_neighborhood(&collapsed, 2, e(0.)).is_none());
        assert!(basis_neighborhood(&k, 2, e(f64::INFINITY)).is_none());
    }
    #[test]
    fn affine_cancellation_does_not_cancel_work() {
        let e = Enclosure::point;
        let c: Curve = Line(Point3::new(1e12, 0., 0.), Point3::new(1e12, 0., 1.)).into();
        let m = Matrix4::from_translation(Vector3::new(-1e12, 0., 0.));
        let bound = revolution_absolute(m, Point3::origin(), Vector3::unit_z(), &c, e(0.)).unwrap();
        assert!(bound > 1e12);
        assert!(PairBudget::new(&[(basic_gamma(512) * e(bound)).hi], 0.)
            .unwrap()
            .charge(None)
            .is_none());
    }
    pub(super) fn transformed_inputs(source: &Solid, selected: &HashSet<EdgeID>) {
        for (index, scale) in [0.25, 1., 3.].into_iter().enumerate() {
            let matrix = Matrix4::from_translation(Vector3::new(2., -3., 1.))
                * Matrix4::from_angle_x(Rad(0.2))
                * Matrix4::from_angle_z(Rad(0.1))
                * Matrix4::from_scale(scale);
            let transformed = builder::transformed(source, matrix);
            let endpoints: Vec<_> = source
                .edge_iter()
                .filter(|e| selected.contains(&e.id()))
                .map(|e| {
                    (
                        matrix.transform_point(e.front().point()),
                        matrix.transform_point(e.back().point()),
                    )
                })
                .collect();
            let selected: HashSet<_> = transformed
                .edge_iter()
                .filter(|e| {
                    endpoints.iter().any(|(a, b)| {
                        (e.front().point().distance(*a) < 1e-9
                            && e.back().point().distance(*b) < 1e-9)
                            || (e.front().point().distance(*b) < 1e-9
                                && e.back().point().distance(*a) < 1e-9)
                    })
                })
                .map(|e| e.id())
                .collect();
            if scale == 3. {
                // The unchanged derivative certificate rejects DC: its measured
                // construction+truncation+evaluation enclosure exceeds 1e-8.
                assert!(build_candidate_with_ledgers(
                    &transformed.boundaries()[0],
                    &selected,
                    scale
                )
                .is_none());
                continue;
            }
            let (result, ledger) =
                build_candidate_with_ledgers(&transformed.boundaries()[0], &selected, scale)
                    .unwrap_or_else(|| panic!("transformed scale {scale}"));
            assert!(result.is_geometric_consistent());
            for row in ledger["paired_uv"].as_array().unwrap() {
                assert!(
                    row["paired_certificate"]["conditional_2_pow_minus_40"]
                        .as_f64()
                        .unwrap()
                        < 1e-9
                );
            }
            if let Some(dir) = std::env::var_os("I2_CLOSURE_EVIDENCE") {
                let path = std::path::PathBuf::from(dir).join(format!("transformed-{index}.json"));
                let evidence = serde_json::json!({
                    "builder_source": include_str!("i2.rs"),
                    "input": transformed.compress(), "candidate": Solid::new(vec![result.clone()]).compress(),
                    "scale": scale, "transform": matrix, "ledgers": ledger,
                    "full_admission": false
                });
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .unwrap();
                file.write_all(&serde_json::to_vec_pretty(&evidence).unwrap())
                    .unwrap();
            }
        }
    }
    pub(super) fn actual_charts(input: &Input, shell: &Shell) {
        let e = Enclosure::point;
        if let Surface::Plane(plane) = &input.supports[1] {
            let trace = [e(2.), e(0.), e(3.)];
            assert!(plane_projection_error(input, plane, trace).unwrap() < 1e-10);
            let shifted = plane.transformed(Matrix4::from_translation(plane.normal() * 1e-12));
            let error = plane_projection_error(input, &shifted, trace).unwrap();
            assert!(error > 1e-13 && error < 1e-10);
            let singular = Plane::new(
                plane.origin(),
                plane.origin() + plane.u_axis(),
                plane.origin() + plane.u_axis(),
            );
            assert!(plane_projection_error(input, &singular, trace).is_none());
        }
        let canal = shell.get(3).unwrap().surface();
        let Surface::NurbsSurface(n) = &canal else {
            panic!("canal")
        };
        let kv = n.knot_vecs();
        let deg = n.degrees();
        let hull = [
            Enclosure {
                lo: kv.0[deg.0] - 1e-13,
                hi: kv.0[kv.0.len() - deg.0 - 1] + 1e-13,
            },
            Enclosure {
                lo: kv.1[deg.1] - 1e-13,
                hi: kv.1[kv.1.len() - deg.1 - 1] + 1e-13,
            },
        ];
        assert!(support_composition(input, &canal, PairedChart::C, hull).is_some());
        let mut bad = n.clone();
        bad.control_points_mut().next().unwrap().w = 0.;
        assert!(
            support_composition(input, &Surface::NurbsSurface(bad), PairedChart::C, hull).is_none()
        );
        let mut escaped = hull;
        escaped[0].lo -= 100.;
        assert!(support_composition(input, &canal, PairedChart::C, escaped).is_none());
        for (face, edge, chart) in [
            (1, 5, PairedChart::A),
            (2, 5, PairedChart::B),
            (6, 12, PairedChart::I),
        ] {
            let original = shell.get(face).unwrap().surface();
            let mut small = original.clone();
            small.transform_by(Matrix4::from_translation(Vector3::new(1e-12, -1e-12, 0.)));
            let mut variants = vec![original.clone(), small];
            match &original {
                Surface::RevolutedCurve(r) => {
                    let mut swapped = original.clone();
                    swapped.invert();
                    variants.push(swapped);
                    if matches!(chart, PairedChart::I) {
                        let o = r.entity().origin().to_vec();
                        let rotation = Matrix4::from_translation(o)
                            * Matrix4::from_axis_angle(r.entity().axis(), Rad(0.1))
                            * Matrix4::from_translation(-o);
                        let mut generator = r.entity().entity_curve().clone();
                        generator.transform_by(rotation);
                        let mut p = Processor::with_transform(
                            RevolutedCurve::by_revolution(
                                generator,
                                r.entity().origin(),
                                r.entity().axis(),
                            ),
                            *r.transform(),
                        );
                        if !r.orientation() {
                            p.invert();
                        }
                        variants.push(p.into());
                    }
                    if matches!(chart, PairedChart::A) {
                        let mut generator = r.entity().entity_curve().clone();
                        generator.invert();
                        let mut p = Processor::with_transform(
                            RevolutedCurve::by_revolution(
                                generator,
                                r.entity().origin(),
                                r.entity().axis(),
                            ),
                            *r.transform(),
                        );
                        if !r.orientation() {
                            p.invert();
                        }
                        variants.push(p.into());
                    }
                }
                Surface::Extruded(r) => {
                    let mut generator = r.entity_curve().clone();
                    generator.invert();
                    variants
                        .push(ExtrudedCurve::by_extrusion(generator, r.extruding_vector()).into());
                }
                _ => panic!("unexpected support"),
            }
            let slot = [2, 5, 9, 11, 12, 13, 6]
                .iter()
                .position(|i| *i == edge)
                .unwrap();
            let (lo, hi) = input.physical_domains().unwrap()[slot];
            let ts: Vec<_> = (0..=1024)
                .map(|j| lo.hi + (hi.lo - lo.hi) * j as f64 / 1024.)
                .collect();
            for support in variants {
                // Original cone charts come from Input supports; swapping its
                // surface exchanges UV coordinates without changing geometry.
                let mut chart_input = input.clone();
                if matches!(chart, PairedChart::I) {
                    chart_input.supports[3] = support.clone();
                }
                let uv = stored_uv(&chart_input, &support, edge, chart, &ts).unwrap();
                let hull = std::array::from_fn(|j| Enclosure {
                    lo: uv
                        .curve
                        .control_points()
                        .iter()
                        .map(|p| p[j])
                        .fold(f64::INFINITY, f64::min),
                    hi: uv
                        .curve
                        .control_points()
                        .iter()
                        .map(|p| p[j])
                        .fold(f64::NEG_INFINITY, f64::max),
                });
                let (lip, f, m, c) = support_composition(input, &support, chart, hull).unwrap();
                let budget = PairBudget::new(
                    &[c, uv.composed(lip).unwrap(), (basic_gamma(512) * e(m)).hi],
                    f,
                )
                .unwrap();
                assert!(
                    budget.charge(Some(2_f64.powi(-40))).is_some(),
                    "{:?}",
                    budget
                );
                let mut bad = support.clone();
                bad.transform_by(Matrix4::from_translation(Vector3::new(1e-7, 0., 0.)));
                if let Some((_, _, _, error)) = support_composition(input, &bad, chart, hull) {
                    assert!(PairBudget::new(&[error], 0.)
                        .unwrap()
                        .charge(None)
                        .is_none());
                }
                for t in [ts[0], ts[512], ts[1024]] {
                    let q = uv.curve.subs(t);
                    let ideal =
                        world_coefficient(input, ideal_trace(input, edge, e(t)).unwrap(), true);
                    let p = support.subs(q.x, q.y);
                    assert!(
                        ideal
                            .iter()
                            .enumerate()
                            .map(|(j, x)| x.error(p[j]))
                            .sum::<f64>()
                            < 1e-9
                    );
                }
            }
        }
    }
}

// Semantic labels are recovered from the input supports, never fixture coordinates.
fn source_profile(input: &Input, shell: &Shell, faces: [FaceID; 5]) -> Option<()> {
    let e = Enclosure::point;
    let h = (e(input.b) - e(input.a)) / (e(input.m) + e(input.n));
    let ridge = e(input.a) + e(input.n) * h;
    let local = [
        [e(input.a), e(0.), e(0.)],
        [e(input.b), e(0.), e(0.)],
        [ridge, e(0.), h],
        [e(0.), e(input.a), e(0.)],
        [e(0.), e(input.b), e(0.)],
        [e(0.), ridge, h],
    ];
    let world = local.map(|p| world_coefficient(input, p, true));
    let mut vertices = Vec::new();
    let mut ids = HashSet::new();
    for v in shell.vertex_iter().filter(|v| ids.insert(v.id())) {
        let matches: Vec<_> = world
            .iter()
            .enumerate()
            .filter(|(_, p)| box_norm(std::array::from_fn(|j| p[j] - e(v.point()[j]))).hi <= 1e-10)
            .map(|(i, _)| i)
            .collect();
        if matches.len() != 1 || vertices.iter().any(|(_, i)| *i == matches[0]) {
            return None;
        }
        vertices.push((v.id(), matches[0]));
    }
    if vertices.len() != 6 {
        return None;
    }
    let label = |v: &Vertex| {
        vertices
            .iter()
            .find(|(id, _)| *id == v.id())
            .map(|(_, i)| *i)
    };
    let loops: [&[usize]; 5] = [
        &[0, 3, 4, 1],
        &[0, 1, 2],
        &[3, 5, 4],
        &[0, 2, 5, 3],
        &[1, 4, 5, 2],
    ];
    let mut incidences = Vec::new();
    for (slot, (face_id, expected)) in faces.into_iter().zip(loops).enumerate() {
        let face = shell.iter().find(|f| f.id() == face_id)?;
        let edges: Vec<_> = face.edge_iter().collect();
        if edges.len() != expected.len() {
            return None;
        }
        let start = expected
            .iter()
            .position(|i| Some(*i) == label(edges[0].front()))?;
        for (k, edge) in edges.iter().enumerate() {
            let a = label(edge.front())?;
            let b = label(edge.back())?;
            if a != expected[(start + k) % expected.len()]
                || b != expected[(start + k + 1) % expected.len()]
            {
                return None;
            }
            incidences.push((edge.id(), a, b));
            let curve = edge.oriented_curve();
            let (lo, hi) = curve.range_tuple();
            if !lo.is_finite() || !hi.is_finite() || hi <= lo {
                return None;
            }
            let domain = Enclosure { lo, hi };
            let gamma = basic_gamma(512);
            let evaluation = gamma * box_norm(curve_absolute(&curve, domain)?);
            let (coefficient, factor) = if a % 3 == b % 3 {
                let actual = conic_coefficients(&curve)?;
                let center = world_coefficient(input, [e(0.), e(0.), local[a][2]], true);
                let radial = world_coefficient(input, [local[a][0], local[a][1], e(0.)], false);
                let sign = if a < 3 { e(1.) } else { e(-1.) };
                let tangent = world_coefficient(
                    input,
                    [-sign * local[a][1], sign * local[a][0], e(0.)],
                    false,
                );
                let cs = trig_enclosure(e(lo), true);
                let sn = trig_enclosure(e(lo), false);
                let expected = [
                    center,
                    std::array::from_fn(|j| radial[j] * cs - tangent[j] * sn),
                    std::array::from_fn(|j| radial[j] * sn + tangent[j] * cs),
                ];
                // A quarter span, with range error charged in spatial units.
                let phase = (e(hi)
                    - e(lo)
                    - Enclosure {
                        lo: downward(FRAC_PI_2),
                        hi: upward(FRAC_PI_2),
                    })
                .norm();
                let raw = raw_conic_coefficients(&curve)?;
                (
                    coefficient_error(actual, expected) + e(phase) * box_norm(radial),
                    box_norm(raw[1]) + box_norm(raw[2]),
                )
            } else {
                let Curve::Line(line) = &curve else {
                    return None;
                };
                if lo != 0. || hi != 1. {
                    return None;
                }
                (
                    box_norm(std::array::from_fn(|j| world[a][j] - e(line.0[j])))
                        + box_norm(std::array::from_fn(|j| world[b][j] - e(line.1[j]))),
                    e(0.),
                )
            };
            let mut span: Box3 = std::array::from_fn(|j| Enclosure {
                lo: local[a][j].lo.min(local[b][j].lo),
                hi: local[a][j].hi.max(local[b][j].hi),
            });
            let radius = if a % 3 == b % 3 {
                let rho = if a < 3 { local[a][0] } else { local[a][1] };
                span[0] = Enclosure { lo: 0., hi: rho.hi };
                span[1] = span[0];
                rho
            } else {
                e(0.)
            };
            let (support_error, support_factor, support_basic, _) = analytic_support(
                input,
                &face.oriented_surface(),
                [0, 7, 5, 6, 4][slot],
                span,
                radius,
                if a % 3 == b % 3 {
                    angular_extension(input, coefficient.hi, radius)?
                } else {
                    0.
                },
            )?;
            PairBudget::new(
                &[coefficient.hi, evaluation.hi, support_error, support_basic],
                (factor + e(support_factor)).hi,
            )?
            .charge(Some(ScalarRuntimeContract::ReviewedGlibc.epsilon()))?;
        }
    }
    for (id, a, b) in &incidences {
        if incidences
            .iter()
            .filter(|(other, _, _)| other == id)
            .count()
            != 2
            || !incidences
                .iter()
                .any(|(other, c, d)| other == id && a == d && b == c)
        {
            return None;
        }
    }
    Some(())
}

fn analytic_vertices(input: &Input) -> Option<[Box3; 12]> {
    let e = Enclosure::point;
    let mut vertices = [[e(f64::NAN); 3]; 12];
    let domains = input.physical_domains()?;
    for (slot, id) in [2, 5, 9, 11, 12, 13].into_iter().enumerate() {
        let (a, b) = ENDS[id];
        vertices[a] = ideal_trace(input, id, domains[slot].0)?;
        vertices[b] = ideal_trace(input, id, domains[slot].1)?;
    }
    let h = (e(input.b) - e(input.a)) / (e(input.m) + e(input.n));
    let ring = e(input.a)
        + e(input.n) * e(input.r)
        + e(input.r) * (e(1.) + e(input.n) * e(input.n)).sqrt();
    let contact = e(input.r) + e(input.r) * e(input.n) / (e(1.) + e(input.n) * e(input.n)).sqrt();
    let rho = e(input.a) + e(input.n) * contact;
    vertices[0] = [e(0.), ring, e(0.)];
    vertices[4] = [e(0.), e(input.b), e(0.)];
    vertices[5] = [e(0.), e(input.a) + e(input.n) * h, h];
    vertices[6] = [e(0.), rho, contact];
    vertices[7] = [rho, e(0.), contact];
    vertices
        .iter()
        .flatten()
        .all(|x| x.valid())
        .then_some(vertices)
}

fn analytic_inventory(input: &Input, shell: &Shell) -> Option<(Vec<PairBudget>, Evidence)> {
    let e = Enclosure::point;
    let vertices = analytic_vertices(input)?;
    let world = vertices.map(|p| world_coefficient(input, p, true));
    let mut edges = vec![None; 18];
    for (face, indices) in shell.iter().zip(LOOPS) {
        for (edge, &index) in face.boundaries()[0].iter().zip(indices) {
            let id = index.unsigned_abs() as usize - 1;
            edges[id] = Some(if index > 0 {
                edge.clone()
            } else {
                edge.inverse()
            });
        }
    }
    let mut rows = Vec::new();
    let mut budgets = Vec::new();
    for id in [0, 1, 3, 4, 7, 8, 10, 14, 15, 16, 17] {
        let edge = edges[id].as_ref()?;
        let curve = edge.curve();
        let (a, b) = ENDS[id];
        let (lo, hi) = curve.range_tuple();
        let domain = Enclosure { lo, hi };
        if !domain.valid() || hi <= lo {
            return None;
        }
        let work = box_norm(curve_absolute(&curve, domain)?);
        let evaluation = basic_gamma(512) * work;
        let endpoint = box_norm(std::array::from_fn(|j| {
            world[a][j] - e(edge.front().point()[j])
        })) + box_norm(std::array::from_fn(|j| {
            world[b][j] - e(edge.back().point()[j])
        }));
        let mut factor = e(0.);
        let coefficient = if matches!(id, 1 | 4 | 10 | 14 | 16 | 17) {
            let Curve::Line(line) = &curve else {
                return None;
            };
            if lo != 0. || hi != 1. {
                return None;
            }
            box_norm(std::array::from_fn(|j| world[a][j] - e(line.0[j])))
                + box_norm(std::array::from_fn(|j| world[b][j] - e(line.1[j])))
        } else {
            let actual = conic_coefficients(&curve)?;
            let raw = raw_conic_coefficients(&curve)?;
            factor = box_norm(raw[1]) + box_norm(raw[2]);
            let expected = if id == 8 {
                let inv = e(1.) / (e(1.) + e(input.n) * e(input.n)).sqrt();
                let ring = vertices[0][1];
                let local = [
                    [e(0.), ring, e(input.r)],
                    [e(0.), -e(input.r) * inv, e(input.r) * e(input.n) * inv],
                    [e(0.), -e(input.r) * e(input.n) * inv, -e(input.r) * inv],
                ];
                std::array::from_fn(|j| world_coefficient(input, local[j], j == 0))
            } else {
                if (e(hi) - e(lo)).hi >= downward(std::f64::consts::PI) {
                    return None;
                }
                let center = world_coefficient(input, [e(0.), e(0.), vertices[a][2]], true);
                let radial =
                    world_coefficient(input, [vertices[a][0], vertices[a][1], e(0.)], false);
                let sign = e(if matches!(id, 0 | 15) { -1. } else { 1. });
                let tangent = world_coefficient(
                    input,
                    [-sign * vertices[a][1], sign * vertices[a][0], e(0.)],
                    false,
                );
                let cs = trig_enclosure(e(lo), true);
                let sn = trig_enclosure(e(lo), false);
                [
                    center,
                    std::array::from_fn(|j| radial[j] * cs - tangent[j] * sn),
                    std::array::from_fn(|j| radial[j] * sn + tangent[j] * cs),
                ]
            };
            // Both range endpoints are checked with directed trigonometric enclosures.
            // Coefficients bound every interior parameter, including folded constants.
            let mut endpoint_curve = e(0.);
            for (t, vertex) in [(lo, a), (hi, b)] {
                let cs = trig_enclosure(e(t), true);
                let sn = trig_enclosure(e(t), false);
                endpoint_curve = endpoint_curve
                    + box_norm(std::array::from_fn(|j| {
                        actual[0][j] + actual[1][j] * cs + actual[2][j] * sn - world[vertex][j]
                    }));
            }
            coefficient_error(actual, expected) + endpoint_curve
        };
        let local_hull: Box3 = std::array::from_fn(|j| Enclosure {
            lo: vertices[a][j].lo.min(vertices[b][j].lo),
            hi: vertices[a][j].hi.max(vertices[b][j].hi),
        });
        let radius = if matches!(id, 0 | 3 | 7 | 15) {
            (vertices[a][0] * vertices[a][0] + vertices[a][1] * vertices[a][1]).sqrt()
        } else {
            e(0.)
        };
        let mut span = local_hull;
        if matches!(id, 0 | 3 | 7 | 15) {
            span[0] = Enclosure {
                lo: 0.,
                hi: radius.hi,
            };
            span[1] = span[0];
        } else if id == 8 {
            span = [
                e(0.),
                Enclosure {
                    lo: vertices[6][1].lo.min(vertices[0][1].lo),
                    hi: vertices[0][1].hi + input.r,
                },
                Enclosure {
                    lo: 0.,
                    hi: vertices[6][2].hi,
                },
            ];
        }
        for (face, indices) in LOOPS.into_iter().enumerate().filter(|(_, indices)| {
            indices
                .iter()
                .any(|index| index.unsigned_abs() as usize == id + 1)
        }) {
            let _ = indices;
            let support = shell[face].surface();
            let (support_error, support_factor, support_basic, hull) = analytic_support(
                input,
                &support,
                face,
                span,
                radius,
                if id == 8 {
                    angular_extension(input, coefficient.hi, e(input.r))?
                } else if radius.lo > 0. {
                    angular_extension(input, coefficient.hi, radius)?
                } else {
                    0.
                },
            )?;
            let budget = PairBudget::new(
                &[
                    coefficient.hi,
                    evaluation.hi,
                    endpoint.hi,
                    support_error,
                    support_basic,
                ],
                (factor + e(support_factor)).hi,
            )?;
            budget.charge(Some(ScalarRuntimeContract::ReviewedGlibc.epsilon()))?;
            let mut arguments = curve_arguments(&curve, domain)?;
            arguments.extend(support_arguments(&support, hull)?);
            rows.push({
                #[cfg(test)]
                {
                    serde_json::json!({"edge":id,"face":face,"range":[lo,hi],
                "trig_argument_hulls":arguments.iter().map(|x|[x.lo,x.hi]).collect::<Vec<_>>(),
                "physical_endpoints":[world[a].map(|x|[x.lo,x.hi]),world[b].map(|x|[x.lo,x.hi])],
                "support_hull":hull.map(|x|[x.lo,x.hi]),"coefficient":coefficient.hi,
                "endpoint":endpoint.hi,"evaluation":evaluation.hi,"support":support_error,
                "support_evaluation":support_basic,"K":budget.known,"F":budget.factor})
                }
                #[cfg(not(test))]
                {
                    let _ = (arguments, world);
                    Evidence
                }
            });
            budgets.push(budget);
        }
    }
    (rows.len() == 22).then_some((budgets, {
        #[cfg(test)]
        {
            serde_json::json!(rows)
        }
        #[cfg(not(test))]
        {
            let _ = rows;
            Evidence
        }
    }))
}
fn analytic_support(
    input: &Input,
    support: &Surface,
    face: usize,
    span: Box3,
    radius: Enclosure,
    angular_padding: f64,
) -> Option<(f64, f64, f64, [Enclosure; 2])> {
    let e = Enclosure::point;
    let half = Enclosure {
        lo: downward(FRAC_PI_2),
        hi: upward(FRAC_PI_2),
    };
    let square = |x: Enclosure| Enclosure {
        lo: if x.lo <= 0. && x.hi >= 0. {
            0.
        } else {
            downward(x.lo.abs().min(x.hi.abs()).powi(2))
        },
        hi: upward(x.norm().powi(2)),
    };
    if let Surface::Plane(plane) = support {
        let world = world_coefficient(input, span, true);
        let relative = std::array::from_fn(|j| world[j] - e(plane.origin()[j]));
        let a = boxed(plane.u_axis());
        let b = boxed(plane.v_axis());
        let aa = box_dot(a, a);
        let ab = box_dot(a, b);
        let bb = box_dot(b, b);
        let det = aa * bb - ab * ab;
        if !det.valid() || det.lo <= 0. {
            return None;
        }
        let ra = box_dot(a, relative);
        let rb = box_dot(b, relative);
        let hull = [(ra * bb - rb * ab) / det, (rb * aa - ra * ab) / det];
        let (_, _, work, _) = support_composition(input, support, PairedChart::S, hull)?;
        let basic = basic_gamma(512) * e(work) * (e(1.) + e(hull[0].norm()) + e(hull[1].norm()));
        return Some((
            plane_projection_error(input, plane, span)?,
            0.,
            basic.hi,
            hull,
        ));
    }
    let (chart, hull) = match face {
        1 => {
            let phase = inverse_cosine(e(1.) / (e(1.) + e(input.n) * e(input.n)).sqrt())?;
            let nominal = [
                Enclosure {
                    lo: (-half - e(angular_padding)).lo,
                    hi: angular_padding,
                },
                Enclosure {
                    lo: 0.,
                    hi: (phase + half + e(angular_padding)).hi,
                },
            ];
            let jets = chart_jets(support, PairedChart::A, nominal.map(Jet::constant))?;
            (PairedChart::A, jets.map(|j| j.0[0]))
        }
        2 => {
            let nominal = [
                Enclosure {
                    lo: 0.,
                    hi: half.hi,
                },
                span[0] / e(input.b),
            ];
            let jets = chart_jets(support, PairedChart::B, nominal.map(Jet::constant))?;
            (PairedChart::B, jets.map(|j| j.0[0]))
        }
        0 | 4 | 6 => {
            let Surface::RevolutedCurve(rev) = support else {
                return None;
            };
            let Curve::Line(line) = rev.entity().entity_curve() else {
                return None;
            };
            let dual = dual_frame(input.axes)?;
            let local = |p: Point3| {
                let w = mapped(*rev.transform(), boxed(p.to_vec()));
                let relative =
                    std::array::from_fn(|j| w[j] + e(rev.transform().w[j]) - e(input.origin[j]));
                dual.map(|axis| box_dot(axis, relative))
            };
            let start = local(line.0);
            let end = local(line.1);
            // The original quarter-profile chart starts on the positive radial axis.
            // A represented phase departure is bounded, not rounded away.
            if start[0].lo <= 0. {
                return None;
            }
            let phase = (e(start[1].norm()) / start[0]).hi;
            let theta = Enclosure {
                lo: (-e(phase) - e(angular_padding)).lo,
                hi: (half + e(phase) + e(angular_padding)).hi,
            };
            let eta = if face == 0 {
                let r0 = (square(start[0]) + square(start[1])).sqrt();
                let r1 = (square(end[0]) + square(end[1])).sqrt();
                let rho = if radius.lo > 0. {
                    radius
                } else {
                    (square(span[0]) + square(span[1])).sqrt()
                };
                (rho - r0) / (r1 - r0)
            } else {
                (span[2] - start[2]) / (end[2] - start[2])
            };
            let hull = if rev.orientation() {
                [eta, theta]
            } else {
                [theta, eta]
            };
            (
                match face {
                    0 => PairedChart::D,
                    4 => PairedChart::O,
                    _ => PairedChart::I,
                },
                hull,
            )
        }
        _ => return None,
    };
    support_arguments(support, hull)?;
    let (_, factor, work, coefficient) = support_composition(input, support, chart, hull)?;
    let basic = basic_gamma(512) * e(work) * (e(1.) + e(hull[0].norm()) + e(hull[1].norm()));
    Some((coefficient, factor, basic.hi, hull))
}

#[cfg(test)]
mod whole_boundary_tests {
    use super::*;
    fn fixture() -> (Shell, HashSet<EdgeID>) {
        let vertices = [
            Point3::new(0., 0., 0.),
            Point3::new(10., 0., 0.),
            Point3::new(5., 0., 5.),
        ]
        .map(Vertex::new);
        let profile = builder::try_attach_plane(&[wire![
            builder::line(&vertices[0], &vertices[1]),
            builder::line(&vertices[1], &vertices[2]),
            builder::line(&vertices[2], &vertices[0])
        ]])
        .unwrap();
        let solid: Solid = builder::rsweep(
            &profile.inverse(),
            Point3::new(-20., 0., 0.),
            Vector3::unit_z(),
            Rad(FRAC_PI_2),
            1,
        );
        let shell = solid.boundaries()[0].clone();
        let mut selected = HashSet::new();
        for edge in shell.edge_iter() {
            let p = edge.front().point();
            let q = edge.back().point();
            if (p.z.abs() < 1e-10
                && q.z.abs() < 1e-10
                && (p - Point3::new(-20., 0., 0.)).magnitude() < 20.0000001
                && (q - Point3::new(-20., 0., 0.)).magnitude() < 20.0000001)
                || (p.y.abs() < 1e-10
                    && q.y.abs() < 1e-10
                    && ((p.z.abs() < 1e-10 && q.z.abs() < 1e-10) || (p.x > 0. && q.x > 0.)))
            {
                selected.insert(edge.id());
            }
        }
        (shell, selected)
    }
    #[test]
    fn dc_plane_all_ledger_rows() {
        let (shell, selected) = fixture();
        let face = shell
            .iter()
            .find(|f| f.vertex_iter().all(|v| v.point().z.abs() < 1e-10))
            .unwrap();
        let sign = face.surface().normal(0., 0.).z.signum();
        let o = Point3::new(10., 30., 0.);
        face.set_surface(
            Plane::new(
                o,
                o + Vector3::unit_x() * 30.,
                o + Vector3::unit_y() * (30. * sign),
            )
            .into(),
        );
        let (shell, selected) = if let Some(path) = std::env::var_os("I2_ACTUAL_SOURCE") {
            let solid = Solid::extract(
                serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap(),
            )
            .unwrap();
            let mut ids = HashSet::new();
            for edge in solid.edge_iter() {
                let p = edge.front().point();
                let q = edge.back().point();
                if (p.z.abs() < 1e-10
                    && q.z.abs() < 1e-10
                    && p.to_vec().magnitude() < 20.0000001
                    && q.to_vec().magnitude() < 20.0000001)
                    || (p.y.abs() < 1e-10
                        && q.y.abs() < 1e-10
                        && ((p.z.abs() < 1e-10 && q.z.abs() < 1e-10)
                            || (p.x > 20.0000001 && q.x > 20.0000001)))
                {
                    ids.insert(edge.id());
                }
            }
            assert_eq!(ids.len(), 3);
            (solid.boundaries()[0].clone(), ids)
        } else {
            (shell, selected)
        };
        let input = Input::recognize(&shell, &selected, 1.).unwrap();
        let (candidate, curves, canal) = candidate(&input).unwrap();
        let endpoints = input.endpoint_gate(&candidate).unwrap();
        let (mut budgets, paired) =
            paired_uv_rows(&input, &candidate, &curves, &canal, &endpoints).unwrap();
        let (analytic_budgets, analytic) = analytic_inventory(&input, &candidate).unwrap();
        budgets.extend(analytic_budgets);
        assert_eq!(budgets.len(), 36);
        if let Some(path) = std::env::var_os("I2_PLANE_LEDGER") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap();
            serde_json::to_writer_pretty(
                file,
                &serde_json::json!({"paired": paired, "analytic": analytic,
                "budgets": budgets.iter().map(|b| [b.known, b.factor]).collect::<Vec<_>>() }),
            )
            .unwrap();
        }
        for (i, budget) in budgets.iter().enumerate() {
            assert!(
                budget.charge(Some(2_f64.powi(-40))).is_some(),
                "row {i}: {}",
                budget.known
            );
        }
    }

    #[test]
    fn dc_ideal_plane_keeps_displacement_charge() {
        let (shell, selected) = fixture();
        let input = Input::recognize(&shell, &selected, 1.).unwrap();
        let domain = Enclosure {
            lo: 1.7,
            hi: 1.7001,
        };
        let raw = input.trim_jets(domain, Trim::Dc).unwrap()[2].0[0];
        assert!(raw.norm() > 1e-7);
        let trace = ideal_trace(&input, 2, domain).unwrap();
        assert_eq!((trace[2].lo, trace[2].hi), (0., 0.));
        let o = input.origin;
        for (u, v) in [(30., 30.), (-60., 15.)] {
            let plane = Plane::new(o, o + input.axes[0] * u, o + input.axes[1] * v);
            assert!(plane_projection_error(&input, &plane, trace).unwrap() < 1e-9);
            let shifted = plane.transformed(Matrix4::from_translation(input.axes[2] * 1e-5));
            assert!(plane_projection_error(&input, &shifted, trace).unwrap() >= 1e-5);
        }
        assert!(ideal_trace(&input, 2, Enclosure { lo: 4., hi: 4.1 }).is_none());
    }

    #[test]
    fn source_vertex_only_change_is_not_a_profile() {
        let (shell, selected) = fixture();
        assert!(Input::recognize(&shell, &selected, 1.).is_some());
        let vertex = shell.vertex_iter().next().unwrap();
        vertex.set_point(vertex.point() + Vector3::new(1e-5, 0., 0.));
        assert!(Input::recognize(&shell, &selected, 1.).is_none());
    }
    #[test]
    fn source_interior_trim_is_not_certified_by_endpoints() {
        let (shell, selected) = fixture();
        let edge = shell
            .edge_iter()
            .find(|edge| !selected.contains(&edge.id()) && matches!(edge.curve(), Curve::Line(_)))
            .unwrap();
        let Curve::Line(line) = edge.curve() else {
            unreachable!()
        };
        edge.set_curve(
            BSplineCurve::new(
                KnotVec::from(vec![0., 0., 0., 1., 1., 1.]),
                vec![
                    line.0,
                    Point3::from_vec((line.0.to_vec() + line.1.to_vec()) / 2.)
                        + Vector3::new(0., 0., 1e-4),
                    line.1,
                ],
            )
            .into(),
        );
        assert!(Input::recognize(&shell, &selected, 1.).is_none());
    }
    #[test]
    fn source_arc_range_must_cover_the_quarter() {
        let (shell, selected) = fixture();
        let edge = shell
            .edge_iter()
            .find(|edge| matches!(edge.curve(), Curve::Conic(_)))
            .unwrap();
        let Curve::Conic(c) = edge.curve() else {
            unreachable!()
        };
        let (lo, hi) = c.range_tuple();
        let mut wrong = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::<Point3>::new(), (lo, hi - 1e-4)),
            *c.transform(),
        );
        if !c.orientation() {
            wrong.invert();
        }
        edge.set_curve(wrong.into());
        assert!(Input::recognize(&shell, &selected, 1.).is_none());
    }
    #[test]
    fn source_directed_loop_must_match_material_side() {
        let (mut shell, selected) = fixture();
        let face = &shell[0];
        let boundary = face.boundaries()[0].inverse();
        shell[0] = Face::new_unchecked(vec![boundary], face.oriented_surface());
        assert!(Input::recognize(&shell, &selected, 1.).is_none());
    }

    #[test]
    fn all_analytic_edges_and_q_are_consumed() {
        let (shell, selected) = fixture();
        let input = Input::recognize(&shell, &selected, 1.).unwrap();
        let (output, _, _) = candidate(&input).unwrap();
        let (rows, _) = analytic_inventory(&input, &output).unwrap();
        assert_eq!(rows.len(), 22);
        let mut q = output[5].surface();
        q.transform_by(Matrix4::from_translation(Vector3::new(1e-6, 0., 0.)));
        output[5].set_surface(q);
        assert!(analytic_inventory(&input, &output).is_none());
    }
    #[test]
    fn each_constructed_analytic_coefficient_is_consumed() {
        for target in [0, 1, 3, 4, 7, 8, 10, 14, 15, 16, 17] {
            let (shell, selected) = fixture();
            let input = Input::recognize(&shell, &selected, 1.).unwrap();
            let (output, _, _) = candidate(&input).unwrap();
            let mut found = None;
            for (face, indices) in output.iter().zip(LOOPS) {
                for (edge, index) in face.boundaries()[0].iter().zip(indices) {
                    if index.unsigned_abs() as usize == target + 1 {
                        found = Some(edge.clone());
                    }
                }
            }
            let edge = found.unwrap();
            let mut curve = edge.curve();
            curve.transform_by(Matrix4::from_translation(Vector3::new(1e-6, 2e-6, 3e-6)));
            edge.set_curve(curve);
            assert!(
                analytic_inventory(&input, &output).is_none(),
                "edge {target}"
            );
        }
    }

    #[test]
    fn original_far_support_perturbation_is_checked() {
        for shift in [1e-12, 1e-6] {
            let (shell, selected) = fixture();
            let face = shell
                .iter()
                .find(|face| !face.edge_iter().any(|edge| selected.contains(&edge.id())))
                .unwrap();
            let mut support = face.surface();
            support.transform_by(Matrix4::from_translation(Vector3::new(shift, 0., 0.)));
            face.set_surface(support);
            assert_eq!(
                Input::recognize(&shell, &selected, 1.).is_some(),
                shift < 1e-10
            );
        }
    }

    #[test]
    fn positive_weights_do_not_imply_factor_two_projection() {
        let e = Enclosure::point;
        let amplification = expanded_projection(e(3.), e(0.5), 1., [1e-12, 0.]).unwrap();
        assert!(amplification >= 6e-12);
        assert!(amplification > 2e-12);
        assert!(expanded_projection(e(3.), e(0.5), 1., [1e-12, 0.2]).is_none());
    }
}

// For a circular arc of less than pi, chord = 2 rho sin(delta/2),
// sin(x)>=2x/pi gives delta<=2 chord/rho. The dual-frame norm
// converts the world L1 enclosure to a local chord enclosure.
fn angular_extension(input: &Input, error: f64, radius: Enclosure) -> Option<f64> {
    let e = Enclosure::point;
    let dual = dual_frame(input.axes)?;
    let norm = dual.iter().map(|row| box_norm(*row).hi).fold(0., f64::max);
    if radius.lo <= 0. {
        return None;
    }
    let extension = e(2.) * e(norm) * e(error) / radius;
    (extension.valid() && extension.hi < FRAC_PI_2).then_some(extension.hi)
}

#[cfg(feature = "i2-reviewed-glibc")]
pub(crate) fn build_for_solid(
    shell: &Shell,
    selected: &HashSet<EdgeID>,
    radius: f64,
) -> Option<Shell> {
    let input = Input::recognize(shell, selected, radius)?;
    let candidate = build_candidate(shell, selected, radius)?;
    // Candidate semantic order is D,A,B,C,O,Q,I,S; finish expects source order first.
    let slots = [0, 7, 5, 6, 4];
    let mut faces = Vec::with_capacity(8);
    for source in shell.iter() {
        let semantic = input
            .source_faces
            .iter()
            .position(|id| *id == source.id())?;
        faces.push(candidate[slots[semantic]].clone());
    }
    faces.extend([1, 2, 3].map(|i| candidate[i].clone()));
    Some(faces.into())
}
