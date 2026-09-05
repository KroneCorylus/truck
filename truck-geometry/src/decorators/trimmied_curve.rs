use super::*;

impl<C> TrimmedCurve<C> {
    /// constructor
    #[inline(always)]
    pub const fn new(curve: C, range: (f64, f64)) -> Self { Self { curve, range } }
    /// Returns the reference of non-trimmed curve
    #[inline(always)]
    pub const fn curve(&self) -> &C { &self.curve }
    /// Returns the mutable reference of non-trimmed curve
    #[inline(always)]
    pub fn curve_mut(&mut self) -> &mut C { &mut self.curve }
}

impl<C: ParametricCurve> ParametricCurve for TrimmedCurve<C> {
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.curve.der_n(n, t) }
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.curve.subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.curve.der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { self.curve.der2(t) }
    #[inline(always)]
    fn period(&self) -> Option<f64> { self.curve.period() }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(self.range.0), Bound::Included(self.range.1))
    }
}

impl<C: ParametricCurve> BoundedCurve for TrimmedCurve<C> {}

impl<C: ParametricCurve> Cut for TrimmedCurve<C> {
    fn cut(&mut self, t: f64) -> Self {
        let (t0, t1) = self.range;
        self.range = (t0, t);
        Self::new(self.curve.clone(), (t, t1))
    }
}

impl<C> TrimmedCurve<C> {
    /// Replaces a missing hint by the trimmed range, so a periodic curve answers inside it.
    fn hint_or_range<H: Into<SPHint1D>>(&self, hint: H) -> SPHint1D {
        match hint.into() {
            SPHint1D::None => SPHint1D::Range(self.range.0, self.range.1),
            hint => hint,
        }
    }
}

impl<C: SearchNearestParameter<D1>> SearchNearestParameter<D1> for TrimmedCurve<C> {
    type Point = C::Point;
    /// Nearest parameter within the trimmed range.
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: C::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = self.hint_or_range(hint);
        let t = self.curve.search_nearest_parameter(pt, hint, trials)?;
        Some(t.clamp(self.range.0, self.range.1))
    }
}

impl<C: SearchParameter<D1>> SearchParameter<D1> for TrimmedCurve<C> {
    type Point = C::Point;
    /// Parameter of `pt` within the trimmed range, `None` if `pt` lies outside it.
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        pt: C::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = self.hint_or_range(hint);
        let t = self.curve.search_parameter(pt, hint, trials)?;
        let (t0, t1) = self.range;
        (t0 - TOLERANCE..=t1 + TOLERANCE).contains(&t).then_some(t)
    }
}

impl<C: ParameterDivision1D> ParameterDivision1D for TrimmedCurve<C> {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        self.curve.parameter_division(range, tol)
    }
}
