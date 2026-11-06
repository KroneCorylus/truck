use truck_geometry::prelude::*;

#[derive(Clone, Copy)]
struct Interval {
    lo: f64,
    hi: f64,
}
impl Interval {
    fn point(x: f64) -> Self { Self { lo: x, hi: x } }
    fn valid(self) -> bool { self.lo.is_finite() && self.hi.is_finite() && self.lo <= self.hi }
    fn norm(self) -> f64 {
        if self.valid() {
            self.lo.abs().max(self.hi.abs())
        } else {
            f64::INFINITY
        }
    }
    fn distance(self, x: f64) -> f64 { (self - Self::point(x)).norm() }
}
impl std::ops::Add for Interval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            lo: (self.lo + rhs.lo).next_down(),
            hi: (self.hi + rhs.hi).next_up(),
        }
    }
}
impl std::ops::Sub for Interval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            lo: (self.lo - rhs.hi).next_down(),
            hi: (self.hi - rhs.lo).next_up(),
        }
    }
}
impl std::ops::Mul for Interval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let products = [
            self.lo * rhs.lo,
            self.lo * rhs.hi,
            self.hi * rhs.lo,
            self.hi * rhs.hi,
        ];
        if !self.valid() || !rhs.valid() || products.iter().any(|x| !x.is_finite()) {
            return Self::point(f64::NAN);
        }
        Self {
            lo: products
                .into_iter()
                .fold(f64::INFINITY, f64::min)
                .next_down(),
            hi: products
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max)
                .next_up(),
        }
    }
}
impl std::ops::Div for Interval {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if !rhs.valid() || (rhs.lo <= 0. && rhs.hi >= 0.) {
            return Self::point(f64::NAN);
        }
        self * Self {
            lo: rhs.hi.recip().next_down(),
            hi: rhs.lo.recip().next_up(),
        }
    }
}

// Encloses real stored coefficients, not floating evaluation of the source PCurve.
// The map term uses paired knot-cell transport; cuts compare actual output poles
// with interval Boehm insertion, including normalization and subtractive rounding.
pub(super) struct BoundaryCertificate {
    knots: Vec<f64>,
    points: Vec<[Interval; 3]>,
    degree: usize,
    map_error: f64,
}
impl BoundaryCertificate {
    pub(super) fn new(
        source_knots: &KnotVec,
        poles: &[Vector4],
        mapped: &BSplineCurve<Point3>,
        domain: (f64, f64),
        values: (f64, f64),
    ) -> Option<Self> {
        let i = Interval::point;
        let (a, b) = domain;
        let (p, q) = values;
        let slope = (i(q) - i(p)) / (i(b) - i(a));
        if !slope.valid() || (slope.lo <= 0. && slope.hi >= 0.) {
            return None;
        }
        let speed = if slope.lo > 0. {
            slope
        } else {
            Interval {
                lo: -slope.hi,
                hi: -slope.lo,
            }
        };
        let mut spacing = f64::INFINITY;
        for pair in source_knots.windows(2).filter(|p| p[0] < p[1]) {
            let width = (i(pair[1]) - i(pair[0])) / speed;
            if !width.valid() || width.lo <= 0. {
                return None;
            }
            spacing = spacing.min(width.lo);
        }
        let mut delta = 0_f64;
        for (index, &knot) in source_knots.iter().enumerate() {
            let exact = i(a) + (i(knot) - i(p)) / slope;
            let index = if p < q {
                index
            } else {
                source_knots.len() - 1 - index
            };
            delta = delta.max(exact.distance(mapped.knot_vec()[index]));
        }
        let mut points: Vec<_> = poles
            .iter()
            .map(|p| [p.x, p.y, p.z].map(|x| if p.w == 1. { i(x) } else { i(x) / i(p.w) }))
            .collect();
        let coordinate_norm = points
            .iter()
            .map(|p| p.iter().fold(i(0.), |sum, x| sum + i(x.norm())).hi)
            .fold(0_f64, f64::max);
        let degree = mapped.degree();
        let map_error = (i(10. * degree as f64) * i(coordinate_norm) * i(delta) / i(spacing)).hi;
        if !map_error.is_finite() || !spacing.is_finite() {
            return None;
        }
        if p > q {
            points.reverse();
        }
        let mut certificate = Self {
            knots: mapped.knot_vec().to_vec(),
            points,
            degree,
            map_error,
        };
        if a > certificate.knots[0] {
            certificate.cut(a, true)?;
        }
        if b < *certificate.knots.last()? {
            certificate.cut(b, false)?;
        }
        Some(certificate)
    }

    fn cut(&mut self, t: f64, keep_right: bool) -> Option<()> {
        let multiplicity = self.knots.iter().filter(|&&k| k == t).count();
        for _ in multiplicity..=self.degree {
            let index = self.knots.partition_point(|&k| k <= t);
            let mut points = Vec::with_capacity(self.points.len() + 1);
            for j in 0..=self.points.len() {
                points.push(if j < index - self.degree {
                    self.points[j]
                } else if j >= index {
                    self.points[j - 1]
                } else {
                    let i = Interval::point;
                    let alpha = (i(t) - i(self.knots[j]))
                        / (i(self.knots[j + self.degree]) - i(self.knots[j]));
                    if !alpha.valid() {
                        return None;
                    }
                    std::array::from_fn(|axis| {
                        alpha * self.points[j][axis] + (i(1.) - alpha) * self.points[j - 1][axis]
                    })
                });
            }
            self.knots.insert(index, t);
            self.points = points;
        }
        let last = self.knots.partition_point(|&k| k <= t) - 1;
        let split = last - self.degree;
        if keep_right {
            self.knots.drain(..split);
            self.points.drain(..split);
        } else {
            self.knots.truncate(last + 1);
            self.points.truncate(split);
        }
        Some(())
    }

    pub(super) fn error(&self, result: &BSplineCurve<Point3>) -> Option<f64> {
        if self.knots != result.knot_vec().as_slice()
            || self.points.len() != result.control_points().len()
        {
            return None;
        }
        let mut controls = 0_f64;
        for (enclosure, actual) in self.points.iter().zip(result.control_points()) {
            let error = (0..3)
                .fold(Interval::point(0.), |sum, axis| {
                    sum + Interval::point(enclosure[axis].distance(actual[axis]))
                })
                .hi;
            if !error.is_finite() {
                return None;
            }
            controls = controls.max(error);
        }
        let total = (Interval::point(self.map_error) + Interval::point(controls)).hi;
        total.is_finite().then_some(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_canal_uses_the_actual_cut_controls() {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/i2-stored-canal.json"))
                .unwrap();
        let truck_modeling::Curve::PCurve(raw) =
            serde_json::from_value(value["curve"].clone()).unwrap()
        else {
            panic!("stored canal trace")
        };
        let truck_modeling::Surface::NurbsSurface(surface) = raw.surface().as_ref() else {
            panic!("stored canal")
        };
        let surface = surface.non_rationalized();
        let poles = &surface.control_points()[0];
        let knots = surface.vknot_vec();
        let mapped = BSplineCurve::new(
            knots.iter().rev().map(|k| -*k).collect::<Vec<_>>().into(),
            poles.iter().rev().map(|p| p.to_point()).collect(),
        );
        let domain = raw.range_tuple();
        let values = (
            raw.curve().control_point(0).y,
            raw.curve().control_point(1).y,
        );
        let proof = BoundaryCertificate::new(knots, poles, &mapped, domain, values).unwrap();
        let output = super::super::affine_boundary(&raw, 0.00005).unwrap();
        let error = proof.error(&output).unwrap();
        eprintln!(
            "stored boundary: map={} total={} controls={}",
            proof.map_error,
            error,
            output.control_points().len()
        );
        assert!(error < 5e-8);
        assert_eq!(output.control_points().len(), 424);
        if std::env::var_os("TRUCK_BOUNDARY_AUDIT").is_some() {
            eprintln!(
                "BOUNDARY_AUDIT {}",
                serde_json::json!({
                    "map_error": proof.map_error,
                    "total": error,
                    "knots": proof.knots,
                    "enclosures": proof.points.iter().map(|p| p.map(|x| [x.lo, x.hi])).collect::<Vec<_>>(),
                    "actual": output.control_points().iter().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>(),
                })
            );
        }
        let mut corrupted = output.clone();
        corrupted.control_point_mut(0).z += 1e-4;
        assert!(proof.error(&corrupted).unwrap() > 5e-8);
    }

    #[test]
    fn exact_quadratic_restriction_and_corrupted_control() {
        let knots = KnotVec::bezier_knot(2);
        let poles = [
            Vector4::new(0., 0., 0., 3.),
            Vector4::new(3., 6., 0., 3.),
            Vector4::new(6., 0., 0., 3.),
        ];
        let mut curve =
            BSplineCurve::new(knots.clone(), poles.iter().map(|p| p.to_point()).collect());
        let proof =
            BoundaryCertificate::new(&knots, &poles, &curve, (0.25, 0.75), (0.25, 0.75)).unwrap();
        curve = curve.cut(0.25);
        curve.cut(0.75);
        let expected = [
            Point3::new(0.5, 0.75, 0.),
            Point3::new(1., 1.25, 0.),
            Point3::new(1.5, 0.75, 0.),
        ];
        assert_eq!(curve.control_points(), &expected);
        for (control, exact) in proof.points.iter().zip(expected) {
            for axis in 0..3 {
                assert!(control[axis].lo <= exact[axis] && exact[axis] <= control[axis].hi);
            }
        }
        assert!(proof.error(&curve).unwrap() < 1e-10);
        curve.control_point_mut(1).y += 1e-4;
        assert!(proof.error(&curve).unwrap() > 5e-8);
    }

    #[test]
    fn knot_transport_covers_repeated_knots_and_both_directions() {
        for degree in 1..=3 {
            for multiplicity in 1..=degree {
                for reverse in [false, true] {
                    let mut knots = vec![0.; degree + 1];
                    knots.extend(vec![0.5; multiplicity]);
                    knots.extend(vec![1.; degree + 1]);
                    let knots = KnotVec::from(knots);
                    let poles: Vec<_> = (0..knots.len() - degree - 1)
                        .map(|j| Vector4::new(j as f64, (j % 2) as f64, -(j as f64), 1.))
                        .collect();
                    let source = BSplineCurve::new(
                        knots.clone(),
                        poles.iter().map(|p| p.to_point()).collect(),
                    );
                    let (a, b, p, q) = if reverse {
                        (2., 5., 1., 0.)
                    } else {
                        (2., 5., 0., 1.)
                    };
                    let mut mapped: Vec<_> = knots
                        .iter()
                        .map(|k| a + (k - p) * (b - a) / (q - p))
                        .collect();
                    let mut points = source.control_points().clone();
                    if reverse {
                        mapped.reverse();
                        points.reverse();
                    }
                    for k in &mut mapped {
                        if *k > a && *k < b {
                            *k += 1e-5;
                        }
                    }
                    let candidate = BSplineCurve::new(mapped.into(), points);
                    let proof =
                        BoundaryCertificate::new(&knots, &poles, &candidate, (a, b), (p, q))
                            .unwrap();
                    let bound = proof.error(&candidate).unwrap();
                    assert!(bound > 0. && bound < 0.01);
                    for j in 0..=2048 {
                        let t = a + (b - a) * j as f64 / 2048.;
                        let value = p + (q - p) * (t - a) / (b - a);
                        assert!(candidate.subs(t).distance(source.subs(value)) <= bound);
                    }
                }
            }
        }
    }

    #[test]
    fn interval_invalid_arithmetic_cannot_authorize_a_bound() {
        let i = Interval::point;
        for value in [i(1.) / i(0.), i(f64::MAX) * i(2.), i(f64::NAN) * i(0.)] {
            assert!(!value.valid());
            assert!(value.norm().is_infinite());
        }
        let exact = i(f64::from_bits(1)) / i(2.);
        assert!(exact.lo <= 0. && exact.hi >= f64::from_bits(1));
    }
}
