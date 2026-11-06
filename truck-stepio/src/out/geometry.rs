use super::{Result, *};
use truck_geometry::prelude::*;
use truck_modeling::{Curve as ModelingCurve, Elementary, Surface as ModelingSurface};
use truck_polymesh::PolylineCurve;

impl DisplayByStep for Point2 {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            coordinates = SliceDisplay(AsRef::<[f64; 2]>::as_ref(self)),
        ))
    }
}
impl_const_step_length!(Point2, 1);

impl DisplayByStep for Point3 {
    fn step_point_coordinates(&self) -> Option<[f64; 3]> {
        Some([
            step_number(self.x)?,
            step_number(self.y)?,
            step_number(self.z)?,
        ])
    }
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = CARTESIAN_POINT('', {coordinates});\n",
            coordinates = SliceDisplay(AsRef::<[f64; 3]>::as_ref(self)),
        ))
    }
}
impl_const_step_length!(Point3, 1);

/// class for display `DIRECTION`.
#[derive(Clone, Debug, Copy)]
pub struct VectorAsDirection<V>(pub V);

impl DisplayByStep for VectorAsDirection<Vector2> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            direction_ratios = SliceDisplay(AsRef::<[f64; 2]>::as_ref(&self.0)),
        ))
    }
}

impl DisplayByStep for VectorAsDirection<Vector3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = DIRECTION('', {direction_ratios});\n",
            direction_ratios = SliceDisplay(AsRef::<[f64; 3]>::as_ref(&self.0)),
        ))
    }
}
impl_const_step_length!(VectorAsDirection<V>, 1, <V>);

impl<V> DisplayByStep for V
where
    V: InnerSpace<Scalar = f64>,
    VectorAsDirection<V>: DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let magnitude = self.magnitude();
        let direction_idx = idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = VECTOR('', #{direction_idx}, {magnitude});\n{direction}",
            direction = StepDataDisplay::new(VectorAsDirection(*self / magnitude), direction_idx),
            magnitude = FloatDisplay(magnitude),
        ))
    }
}
impl_const_step_length!(Vector2, 2);
impl_const_step_length!(Vector3, 2);

/// class for display `Matrix`.
#[derive(Clone, Debug, Copy)]
pub struct MatrixAsAxis<M>(pub M);

impl DisplayByStep for MatrixAsAxis<Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let location_idx = idx + 1;
        let ref_direction_idx = idx + 2;
        let location = self.0[2].to_point();
        let ref_direction = VectorAsDirection(self.0[0].truncate());
        f.write_fmt(format_args!(
            "#{idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(MatrixAsAxis<Matrix3>, 3);

impl DisplayByStep for MatrixAsAxis<Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let location_idx = idx + 1;
        let axis_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let location = self.0[3].to_point();
        let axis = VectorAsDirection(self.0[2].truncate());
        let ref_direction = VectorAsDirection(self.0[0].truncate());
        f.write_fmt(format_args!(
            "#{idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(MatrixAsAxis<Matrix4>, 4);

impl<P> DisplayByStep for Line<P>
where
    P: EuclideanSpace + ConstStepLength + DisplayByStep,
    P::Diff: DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let pnt_idx = idx + 1;
        let dir_idx = idx + 1 + P::LENGTH;
        f.write_fmt(format_args!(
            "#{idx} = LINE('', #{pnt_idx}, #{dir_idx});\n{pnt}{dir}",
            pnt = StepDataDisplay::new(self.0, pnt_idx),
            dir = StepDataDisplay::new(self.1 - self.0, dir_idx),
        ))
    }
}

impl<P> StepLength for Line<P>
where
    P: EuclideanSpace + ConstStepLength,
    P::Diff: ConstStepLength,
{
    #[inline(always)]
    fn step_length(&self) -> usize { <Self as ConstStepLength>::LENGTH }
}

impl<P> ConstStepLength for Line<P>
where
    P: EuclideanSpace + ConstStepLength,
    P::Diff: ConstStepLength,
{
    const LENGTH: usize = 1 + P::LENGTH + P::Diff::LENGTH;
}

impl<P> StepCurve for Line<P> {}

impl<P> DisplayByStep for PolylineCurve<P>
where P: Copy + ConstStepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!(
            "#{idx} = POLYLINE('', {range});\n",
            range = IndexSliceDisplay(idx + 1..=idx + self.0.len())
        ))?;
        let closure = |(i, p): (usize, &P)| p.fmt(idx + 1 + i * P::LENGTH, f);
        self.0.iter().enumerate().try_for_each(closure)
    }
}

impl<P: ConstStepLength> StepLength for PolylineCurve<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.0.len() * P::LENGTH }
}

impl<P> StepCurve for PolylineCurve<P> {}

impl<P> DisplayByStep for BSplineCurve<P>
where P: Copy + ConstStepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        fmt_bspline_curve(self, idx, f, false)
    }
}

fn fmt_bspline_curve<P>(
    curve: &BSplineCurve<P>,
    idx: usize,
    f: &mut Formatter<'_>,
    open: bool,
) -> Result
where
    P: Copy + ConstStepLength + DisplayByStep,
{
    let (knots, multi) = curve.knot_vec().to_single_multi();
    let control_points_instances = curve
        .control_points()
        .iter()
        .enumerate()
        .map(|(i, p)| StepDataDisplay::new(*p, idx + 1 + i * P::LENGTH))
        .collect::<Vec<_>>();
    f.write_fmt(format_args!(
            "#{idx} = B_SPLINE_CURVE_WITH_KNOTS('', {degree}, {control_points_list}, .UNSPECIFIED., {closed}, .U., {knot_multiplicities}, {knots}, .UNSPECIFIED.);\n{control_points_instances}",
            closed = if open { ".F." } else { ".U." },
            degree = curve.degree(),
            control_points_list = IndexSliceDisplay((idx + 1..=idx + curve.control_points().len() * P::LENGTH).step_by(P::LENGTH)),
			knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            control_points_instances = SliceDisplay(&control_points_instances),
		))
}

impl<P> StepLength for BSplineCurve<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<P> StepCurve for BSplineCurve<P> {}

impl<V> DisplayByStep for NurbsCurve<V>
where
    V: Homogeneous<Scalar = f64>,
    V::Point: ConstStepLength + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let (knots, multi) = self.knot_vec().to_single_multi();
        let control_points_instances = self
            .control_points()
            .iter()
            .enumerate()
            .map(|(i, v)| StepDataDisplay::new(v.to_point(), idx + 1 + i * V::Point::LENGTH))
            .collect::<Vec<_>>();
        let weights = self
            .control_points()
            .iter()
            .map(|v| v.weight())
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "#{idx} = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE({degree}, {control_points_list}, .UNSPECIFIED., .U., .U.)
    B_SPLINE_CURVE_WITH_KNOTS({knot_multiplicities}, {knots}, .UNSPECIFIED.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE({weights})
    REPRESENTATION_ITEM('')
);\n{control_points_instances}",
            degree = self.degree(),
            control_points_list = IndexSliceDisplay(
                (idx + 1..=idx + self.control_points().len() * V::Point::LENGTH)
                    .step_by(V::Point::LENGTH)
            ),
            knot_multiplicities = SliceDisplay(&multi),
            knots = SliceDisplay(&knots),
            weights = SliceDisplay(&weights),
            control_points_instances = SliceDisplay(&control_points_instances),
        ))
    }
}

impl<V> StepLength for NurbsCurve<V> {
    #[inline(always)]
    fn step_length(&self) -> usize { self.control_points().len() + 1 }
}

impl<V> StepCurve for NurbsCurve<V> {}

impl DisplayByStep for Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let location = transform[2].to_point();
        if r0.near(&r1) {
            let r = FloatDisplay(r0);
            f.write_fmt(format_args!("#{idx} = CIRCLE('', #{position_idx}, {r});\n"))?;
        } else {
            let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
            f.write_fmt(format_args!(
                "#{idx} = ELLIPSE('', #{position_idx}, {r0}, {r1});\n"
            ))?;
        }
        f.write_fmt(format_args!(
            "#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point();
        let (a, b, axis, _) = ellipse_axes(transform);
        let r0 = a.magnitude();
        let r1 = b.magnitude();
        let ref_direction = VectorAsDirection(a / r0);
        if r0.near(&r1) {
            let r = FloatDisplay(r0);
            f.write_fmt(format_args!("#{idx} = CIRCLE('', #{position_idx}, {r});\n"))?;
        } else {
            let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
            f.write_fmt(format_args!(
                "#{idx} = ELLIPSE('', #{position_idx}, {r0}, {r1});\n"
            ))?;
        }
        f.write_fmt(format_args!(
            "#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>, 5);

impl DisplayByStep for Processor<TrimmedCurve<UnitHyperbola<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDataDisplay::new(ref_direction_raw, ref_direction_idx);
        let location = StepDataDisplay::new(transform[2].to_point(), location_idx);
        let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
        f.write_fmt(format_args!(
            "#{idx} = HYPERBOLA('', #{position_idx}, {r0}, {r1});
#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});
{location}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitHyperbola<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitHyperbola<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = StepDataDisplay::new(transform[3].to_point(), location_idx);
        let axis_raw = VectorAsDirection(transform[2].truncate().normalize());
        let axis = StepDataDisplay::new(axis_raw, axis_idx);
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDataDisplay::new(ref_direction_raw, ref_direction_idx);
        let (r0, r1) = (FloatDisplay(r0), FloatDisplay(r1));
        f.write_fmt(format_args!(
            "#{idx} = HYPERBOLA('', #{position_idx}, {r0}, {r1});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitHyperbola<Point3>>, Matrix4>, 5);

impl DisplayByStep for Processor<TrimmedCurve<UnitParabola<Point2>>, Matrix3> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = *self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let ref_direction_idx = idx + 3;
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let focal_dist = FloatDisplay(r1 * r1 / r0);
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDataDisplay::new(ref_direction_raw, ref_direction_idx);
        let location = StepDataDisplay::new(transform[2].to_point(), location_idx);
        f.write_fmt(format_args!(
            "#{idx} = PARABOLA('', #{position_idx}, {focal_dist});
#{position_idx} = AXIS2_PLACEMENT_2D('', #{location_idx}, #{ref_direction_idx});
{location}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitParabola<Point2>>, Matrix3>, 4);

impl DisplayByStep for Processor<TrimmedCurve<UnitParabola<Point3>>, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = StepDataDisplay::new(transform[3].to_point(), location_idx);
        let axis_raw = VectorAsDirection(transform[2].truncate().normalize());
        let axis = StepDataDisplay::new(axis_raw, axis_idx);
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        let focal_dist = FloatDisplay(r1 * r1 / r0);
        let ref_direction_raw = VectorAsDirection(transform[0].truncate() / r0);
        let ref_direction = StepDataDisplay::new(ref_direction_raw, ref_direction_idx);
        f.write_fmt(format_args!(
            "#{idx} = PARABOLA('', #{position_idx}, {focal_dist});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});
{location}{axis}{ref_direction}"
        ))
    }
}
impl_const_step_length!(Processor<TrimmedCurve<UnitParabola<Point3>>, Matrix4>, 5);

impl<C, M: One> StepCurve for Processor<C, M> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

impl<C, S0, S1> DisplayByStep for IntersectionCurve<C, S0, S1>
where
    C: StepLength + DisplayByStep,
    S0: StepLength + DisplayByStep,
    S1: StepLength + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve_idx = idx + 1;
        let surface0_idx = curve_idx + self.leader().step_length();
        let surface1_idx = surface0_idx + self.surface0().step_length();
        f.write_fmt(format_args!(
            "#{idx} = INTERSECTION_CURVE('', #{curve_idx}, (#{surface0_idx}, #{surface1_idx}), .CURVE_3D.);\n"
        ))?;
        self.leader().fmt(curve_idx, f)?;
        self.surface0().fmt(surface0_idx, f)?;
        self.surface1().fmt(surface0_idx, f)
    }
}

impl<C: StepLength, S0: StepLength, S1: StepLength> StepLength for IntersectionCurve<C, S0, S1> {
    #[inline(always)]
    fn step_length(&self) -> usize {
        1 + self.leader().step_length()
            + self.surface0().step_length()
            + self.surface1().step_length()
    }
}

impl<C, S0, S1> ConstStepLength for IntersectionCurve<C, S0, S1>
where
    C: ConstStepLength,
    S0: ConstStepLength,
    S1: ConstStepLength,
{
    const LENGTH: usize = 1 + C::LENGTH + S0::LENGTH + S1::LENGTH;
}

impl<C: StepCurve, S0, S1> StepCurve for IntersectionCurve<C, S0, S1> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.leader().same_sense() }
}

impl<C, S> DisplayByStep for PCurve<C, S>
where
    C: DisplayByStep,
    S: DisplayByStep + StepLength,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let surface_idx = idx + 1;
        let repr_idx = surface_idx + self.surface().step_length();
        let context_idx = repr_idx + 1;
        let curve_idx = repr_idx + 2;
        let curve = StepDataDisplay::new(self.curve(), curve_idx);
        let surface = StepDataDisplay::new(self.surface(), surface_idx);
        f.write_fmt(format_args!(
            "#{idx} = PCURVE('', #{surface_idx}, #{repr_idx});
{surface}#{repr_idx} = DEFINITIONAL_REPRESENTATION('', (#{curve_idx}), #{context_idx});
#{context_idx} = (
    GEOMETRIC_REPRESENTATION_CONTEXT(2)
    PARAMETRIC_REPRESENTATION_CONTEXT()
    REPRESENTATION_CONTEXT('2D SPACE', '')
);
{curve}"
        ))
    }
}

impl<C: StepLength, S: StepLength> StepLength for PCurve<C, S> {
    fn step_length(&self) -> usize { 3 + self.curve().step_length() + self.surface().step_length() }
}

impl<C, S> ConstStepLength for PCurve<C, S>
where
    C: ConstStepLength,
    S: ConstStepLength,
{
    const LENGTH: usize = 3 + C::LENGTH + S::LENGTH;
}

impl<C: StepCurve, S> StepCurve for PCurve<C, S> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.curve().same_sense() }
}

impl DisplayByStep for ModelingCurve {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            ModelingCurve::PCurve(_) => ERR,
            ModelingCurve::Line(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::BSplineCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::NurbsCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::Conic(x) => DisplayByStep::fmt(x, idx, f),
            ModelingCurve::IntersectionCurve(x) => DisplayByStep::fmt(x, idx, f),
        }
    }
}

impl StepLength for ModelingCurve {
    fn step_length(&self) -> usize {
        match self {
            ModelingCurve::PCurve(_) => 0,
            ModelingCurve::Line(_) => Line::<Point3>::LENGTH,
            ModelingCurve::BSplineCurve(x) => x.step_length(),
            ModelingCurve::NurbsCurve(x) => x.step_length(),
            ModelingCurve::Conic(x) => x.step_length(),
            ModelingCurve::IntersectionCurve(x) => x.step_length(),
        }
    }
}

impl StepCurve for ModelingCurve {
    fn step_ellipse(&self) -> Option<StepEllipse> {
        let ModelingCurve::Conic(conic) = self else {
            return None;
        };
        StepEllipse::extract(conic)
    }
    fn same_sense(&self) -> bool {
        match self {
            ModelingCurve::Conic(x) => x.same_sense(),
            _ => true,
        }
    }
}

impl DisplayByStep for Plane {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let axis2_placement_idx = idx + 1;
        let location_idx = idx + 2;
        let z_axis_idx = idx + 3;
        let x_axis_idx = idx + 4;
        f.write_fmt(format_args!(
            "#{idx} = PLANE('', #{axis2_placement_idx});
#{axis2_placement_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{z_axis_idx}, #{x_axis_idx});
{location}{z_axis}{x_axis}",
            location = StepDataDisplay::new(self.origin(), location_idx),
            z_axis = StepDataDisplay::new(VectorAsDirection(self.normal()), z_axis_idx),
            x_axis = StepDataDisplay::new(VectorAsDirection(self.u_axis().normalize()), x_axis_idx)
        ))
    }
}
impl_const_step_length!(Plane, 5);

impl StepSurface for Plane {}

impl DisplayByStep for Processor<Sphere, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let sphere = *self.entity();
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point() + sphere.center().to_vec();
        let axis = VectorAsDirection(transform[2].truncate().normalize());
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        if !r0.near(&r1) {
            f.write_str("The transform of sphere includes non-uniform scale.")?;
            return ERR;
        }
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let r = FloatDisplay(r0 * sphere.radius());
        f.write_fmt(format_args!(
            "#{idx} = SPHERICAL_SURFACE('', #{position_idx}, {r});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n"
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<Sphere, Matrix4>, 5);

impl DisplayByStep for Sphere {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        DisplayByStep::fmt(&Processor::new(*self), idx, f)
    }
}
impl_const_step_length!(Sphere, 5);
impl StepSurface for Sphere {}

impl DisplayByStep for Processor<Torus, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let torus = *self.entity();
        let transform = self.transform();
        let position_idx = idx + 1;
        let location_idx = idx + 2;
        let axis_idx = idx + 3;
        let ref_direction_idx = idx + 4;
        let location = transform[3].to_point() + torus.center().to_vec();
        let axis = VectorAsDirection(transform[2].truncate().normalize());
        let r0 = transform[0].magnitude();
        let r1 = transform[1].magnitude();
        if !r0.near(&r1) {
            f.write_str("The transform of sphere includes non-uniform scale.")?;
            return ERR;
        }
        let ref_direction = VectorAsDirection(transform[0].truncate() / r0);
        let greater = FloatDisplay(r0 * torus.large_radius());
        let lesser = FloatDisplay(r0 * torus.small_radius());
        f.write_fmt(format_args!(
            "#{idx} = TOROIDAL_SURFACE('', #{position_idx}, {greater}, {lesser});
#{position_idx} = AXIS2_PLACEMENT_3D('', #{location_idx}, #{axis_idx}, #{ref_direction_idx});\n",
        ))?;
        DisplayByStep::fmt(&location, location_idx, f)?;
        DisplayByStep::fmt(&axis, axis_idx, f)?;
        DisplayByStep::fmt(&ref_direction, ref_direction_idx, f)
    }
}
impl_const_step_length!(Processor<Torus, Matrix4>, 5);

impl StepSurface for Processor<Torus, Matrix4> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

impl DisplayByStep for Torus {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        DisplayByStep::fmt(&Processor::new(*self), idx, f)
    }
}
impl_const_step_length!(Torus, 5);
impl StepSurface for Torus {}

impl<P> DisplayByStep for BSplineSurface<P>
where P: Copy + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let control_points = self.control_points();
        let control_points_instances = self
            .control_points()
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, p)| StepDataDisplay::new(*p, idx + i + 1))
            .collect::<Vec<_>>();
        let mut counter = 0;
        let control_points_list = control_points
            .iter()
            .map(|slice| {
                counter += slice.len();
                IndexSliceDisplay(idx + counter - slice.len() + 1..=idx + counter)
            })
            .collect::<Vec<_>>();
        let (uknots, umulti) = self.uknot_vec().to_single_multi();
        let (vknots, vmulti) = self.vknot_vec().to_single_multi();
        f.write_fmt(format_args!(
            "#{idx} = B_SPLINE_SURFACE_WITH_KNOTS('', {u_degree}, {v_degree}, {control_points_list}, .UNSPECIFIED., .U., .U., .U., \
{u_multiplicities}, {v_multiplicities}, {u_knots}, {v_knots}, .UNSPECIFIED.);\n{control_points_instances}",
            u_degree = self.udegree(),
            v_degree = self.vdegree(),
            control_points_list = SliceDisplay(&control_points_list),
            u_multiplicities = SliceDisplay(&umulti),
            v_multiplicities = SliceDisplay(&vmulti),
            u_knots = SliceDisplay(&uknots),
            v_knots = SliceDisplay(&vknots),
            control_points_instances = SliceDisplay(&control_points_instances),
        ))
    }
}

impl<P> StepLength for BSplineSurface<P> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}
impl<P> StepSurface for BSplineSurface<P> {}

impl<V> DisplayByStep for NurbsSurface<V>
where
    V: Homogeneous<Scalar = f64>,
    V::Point: Copy + DisplayByStep,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let control_points_instances = self
            .control_points()
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, v)| StepDataDisplay::new(v.to_point(), idx + i + 1))
            .collect::<Vec<_>>();
        let mut counter = 0;
        let control_points_list = self
            .control_points()
            .iter()
            .map(|slice| {
                counter += slice.len();
                IndexSliceDisplay(idx + counter - slice.len() + 1..=idx + counter)
            })
            .collect::<Vec<_>>();
        let weights = self
            .control_points()
            .iter()
            .map(|slice| slice.iter().map(|v| v.weight()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let weights = weights
            .iter()
            .map(|slice| SliceDisplay(slice))
            .collect::<Vec<_>>();
        let (uknots, umulti) = self.uknot_vec().to_single_multi();
        let (vknots, vmulti) = self.vknot_vec().to_single_multi();
        f.write_fmt(format_args!(
            "#{idx} = (
    BOUNDED_SURFACE()
    B_SPLINE_SURFACE({u_degree}, {v_degree}, {control_points_list}, .UNSPECIFIED., .U., .U., .U.)
    B_SPLINE_SURFACE_WITH_KNOTS({u_multiplicities}, {v_multiplicities}, {u_knots}, {v_knots}, .UNSPECIFIED.)
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_SURFACE({weights})
    REPRESENTATION_ITEM('')
    SURFACE()
);\n{control_points_instances}",
            u_degree = self.udegree(),
            v_degree = self.vdegree(),
            control_points_list = SliceDisplay(&control_points_list),
            u_multiplicities = SliceDisplay(&umulti),
            v_multiplicities = SliceDisplay(&vmulti),
            u_knots = SliceDisplay(&uknots),
            v_knots = SliceDisplay(&vknots),
            control_points_instances = SliceDisplay(&control_points_instances),
            weights = SliceDisplay(&weights),
        ))
    }
}

impl<V> StepLength for NurbsSurface<V> {
    #[inline(always)]
    fn step_length(&self) -> usize { 1 + self.control_points().iter().map(Vec::len).sum::<usize>() }
}
impl<V> StepSurface for NurbsSurface<V> {}

impl<C> DisplayByStep for ExtrudedCurve<C, Vector3>
where C: StepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve = self.entity_curve();
        let curve_idx = idx + 1;
        let vector_idx = idx + 1 + curve.step_length();
        let vector = self.extruding_vector();
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_LINEAR_EXTRUSION('', #{curve_idx}, #{vector_idx});\n{}{}",
            StepDataDisplay::new(curve, curve_idx),
            StepDataDisplay::new(vector, vector_idx),
        ))
    }
}
impl<C: StepLength> StepLength for ExtrudedCurve<C, Vector3> {
    fn step_length(&self) -> usize { 1 + self.entity_curve().step_length() + Vector3::LENGTH }
}
impl<C: ConstStepLength> ConstStepLength for ExtrudedCurve<C, Vector3> {
    const LENGTH: usize = 1 + C::LENGTH + Vector3::LENGTH;
}
impl<C: StepCurve> StepSurface for ExtrudedCurve<C, Vector3> {
    fn same_sense(&self) -> bool { self.entity_curve().same_sense() }
}

impl<C: StepCurve, T: One> StepSurface for Processor<ExtrudedCurve<C, Vector3>, T> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() == self.entity().same_sense() }
}

impl<C> DisplayByStep for RevolutedCurve<C>
where C: StepLength + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let curve = self.entity_curve();
        let curve_idx = idx + 1;
        let axis_idx = curve_idx + curve.step_length();
        let location_idx = axis_idx + 1;
        let dir_idx = location_idx + 1;
        f.write_fmt(format_args!(
            "#{idx} = SURFACE_OF_REVOLUTION('', #{curve_idx}, #{axis_idx});
{curve}#{axis_idx} = AXIS1_PLACEMENT('', #{location_idx}, #{dir_idx});\n{location}{dir}",
            curve = StepDataDisplay::new(curve, curve_idx),
            location = StepDataDisplay::new(self.origin(), location_idx),
            dir = StepDataDisplay::new(VectorAsDirection(self.axis()), dir_idx),
        ))
    }
}

impl<C: StepLength> StepLength for RevolutedCurve<C> {
    #[inline(always)]
    fn step_length(&self) -> usize { 4 + self.entity_curve().step_length() }
}

impl<C: ConstStepLength> ConstStepLength for RevolutedCurve<C> {
    const LENGTH: usize = 4 + C::LENGTH;
}

impl<C> StepSurface for RevolutedCurve<C> {
    #[inline(always)]
    fn same_sense(&self) -> bool { false }
}

impl<C> DisplayByStep for Processor<RevolutedCurve<C>, Matrix4>
where C: StepLength + Transformed<Matrix4> + DisplayByStep
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let surface = self.entity();
        let transform = self.transform();
        let (k, a, _) = match transform.iwasawa_decomposition() {
            Some(x) => x,
            None => {
                f.write_str("Transform is not regular")?;
                return ERR;
            }
        };
        if !a[0][0].near(&a[1][1]) || !a[1][1].near(&a[2][2]) {
            f.write_str("Transform contains non-uniform scale.")?;
            return ERR;
        }
        let curve = surface.entity_curve().transformed(*transform);
        let axis = k.transform_vector(surface.axis());
        let origin = transform.transform_point(surface.origin());
        let surface = RevolutedCurve::by_revolution(curve, origin, axis);
        DisplayByStep::fmt(&surface, idx, f)
    }
}
impl<C: StepLength> StepLength for Processor<RevolutedCurve<C>, Matrix4> {
    fn step_length(&self) -> usize { self.entity().step_length() }
}

impl<C, T: One> StepSurface for Processor<RevolutedCurve<C>, T> {
    #[inline(always)]
    fn same_sense(&self) -> bool { !self.orientation() }
}

/// Axis placement with `axis` as the z direction and any perpendicular unit vector as the
/// reference direction.
fn placement(origin: Point3, axis: Vector3) -> MatrixAsAxis<Matrix4> {
    let seed = match axis.x.abs() < 0.9 {
        true => Vector3::unit_x(),
        false => Vector3::unit_y(),
    };
    let x = (seed - axis * seed.dot(axis)).normalize();
    MatrixAsAxis(Matrix4::from_cols(
        x.extend(0.0),
        axis.cross(x).extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    ))
}

/// The elementary surfaces of `truck-modeling` as the STEP entities of the same name. The
/// normal of each entity is the canonical normal of [`Elementary`].
impl DisplayByStep for Elementary {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let position_idx = idx + 1;
        match *self {
            Elementary::Plane(plane) => DisplayByStep::fmt(&plane, idx, f),
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            } => f.write_fmt(format_args!(
                "#{idx} = CYLINDRICAL_SURFACE('', #{position_idx}, {radius});\n{position}",
                radius = FloatDisplay(radius),
                position = StepDataDisplay::new(placement(origin, axis), position_idx),
            )),
            Elementary::Cone {
                apex,
                axis,
                half_angle,
            } => f.write_fmt(format_args!(
                "#{idx} = CONICAL_SURFACE('', #{position_idx}, 0.0, {semi_angle});\n{position}",
                semi_angle = FloatDisplay(half_angle.0),
                position = StepDataDisplay::new(placement(apex, axis), position_idx),
            )),
            Elementary::Sphere { center, radius } => {
                DisplayByStep::fmt(&Sphere::new(center, radius), idx, f)
            }
            Elementary::Torus {
                center,
                axis,
                major,
                minor,
            } => {
                let torus = Torus::new(Point3::origin(), major, minor);
                let placed = Processor::with_transform(torus, placement(center, axis).0);
                DisplayByStep::fmt(&placed, idx, f)
            }
        }
    }
}
impl_const_step_length!(Elementary, 1 + MatrixAsAxis::<Matrix4>::LENGTH);

impl DisplayByStep for ModelingSurface {
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        // Preserve the patch placement so its poles and seam stay away from the trimming loops.
        if let ModelingSurface::Sphere(x) = self {
            return DisplayByStep::fmt(x, idx, f);
        }
        if let Some((elementary, _)) = self.elementary() {
            return DisplayByStep::fmt(&elementary, idx, f);
        }
        match self {
            ModelingSurface::Fillet(_) => ERR,
            ModelingSurface::Sphere(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::Plane(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::BSplineSurface(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::NurbsSurface(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::RevolutedCurve(x) => DisplayByStep::fmt(x, idx, f),
            ModelingSurface::Extruded(x) => DisplayByStep::fmt(x, idx, f),
        }
    }
}

impl StepLength for ModelingSurface {
    fn step_length(&self) -> usize {
        if self.elementary().is_some() {
            return Elementary::LENGTH;
        }
        match self {
            ModelingSurface::Fillet(_) => 0,
            ModelingSurface::Sphere(x) => x.step_length(),
            ModelingSurface::Plane(_) => Plane::LENGTH,
            ModelingSurface::BSplineSurface(x) => x.step_length(),
            ModelingSurface::NurbsSurface(x) => x.step_length(),
            ModelingSurface::RevolutedCurve(x) => x.step_length(),
            ModelingSurface::Extruded(x) => x.step_length(),
        }
    }
}

impl StepSurface for ModelingSurface {
    fn step_cylinder(&self) -> Option<StepCylinder> {
        let (
            Elementary::Cylinder {
                origin,
                axis,
                radius,
            },
            _,
        ) = self.elementary()?
        else {
            return None;
        };
        StepCylinder::extract(origin, axis, radius)
    }
    /// Whether the face normal is the normal of the written entity: the canonical normal for
    /// an elementary surface, otherwise the convention of the generic writer, which for a
    /// revolved surface is the one the reader and OpenCASCADE use.
    fn same_sense(&self) -> bool {
        match self.elementary() {
            Some((_, outward)) => outward,
            None => match self {
                ModelingSurface::RevolutedCurve(x) => StepSurface::same_sense(x),
                ModelingSurface::Extruded(x) => StepSurface::same_sense(x),
                _ => true,
            },
        }
    }
}

fn ellipse_axes(transform: &Matrix4) -> (Vector3, Vector3, VectorAsDirection<Vector3>, f64) {
    let (mut a, mut b) = (transform[0].truncate(), transform[1].truncate());
    let mut phase = 0.0;
    let axis =
        VectorAsDirection(
            a.cross(b)
                .normalize()
                .map(|value| if value == 0.0 { 0.0 } else { value }),
        );
    // A shear preserves a conic but not its orthogonal axes. Diagonalizing their
    // Gram matrix recovers the exact ellipse without changing its handedness.
    if a.dot(b).abs() > 1e-12 * a.magnitude() * b.magnitude() {
        let angle = 0.5 * (2.0 * a.dot(b)).atan2(a.magnitude2() - b.magnitude2());
        phase = angle;
        let (sin, cos) = angle.sin_cos();
        (a, b) = (a * cos + b * sin, b * cos - a * sin);
    }

    (a, b, axis, phase)
}

// These descriptors model decimal coefficients read back from the existing formatter.
// Normalization is the reader's AXIS2_PLACEMENT_3D convention.
fn step_number(x: f64) -> Option<f64> {
    let x = FloatDisplay(x).to_string().parse::<f64>().ok()?;
    x.is_finite().then_some(x)
}
fn step_vector(v: Vector3) -> Option<Vector3> {
    Some(Vector3::new(
        step_number(v.x)?,
        step_number(v.y)?,
        step_number(v.z)?,
    ))
}
fn step_frame(z: Vector3, x: Vector3) -> Option<(Vector3, Vector3, Vector3)> {
    let frame = ellipse_reader_frame(z, x, false)?;
    Some((
        frame[0].map(|v| v.lo + (v.hi - v.lo) * 0.5).into(),
        frame[1].map(|v| v.lo + (v.hi - v.lo) * 0.5).into(),
        frame[2].map(|v| v.lo + (v.hi - v.lo) * 0.5).into(),
    ))
}

/// Internal, formatter-derived ellipse descriptor; not a runtime curve representation.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct StepEllipse {
    center: Point3,
    a: Vector3,
    b: Vector3,
    range: (f64, f64),
    reader_coefficients: [[EllipseInterval; 3]; 2],
    source_mapping_error: f64,
    trim_parameters: (f64, f64),
    trim_shift: f64,
    source_parameters: (f64, f64),
    source_shift: f64,
    reader_frame: [[EllipseInterval; 3]; 3],
    reader_radii: [f64; 2],
}
impl StepEllipse {
    fn extract(conic: &Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>) -> Option<Self> {
        let transform = conic.transform();
        let (a, b, z, rotation) = ellipse_axes(transform);
        let ra = a.magnitude();
        let rb = b.magnitude();
        if !ra.is_finite() || !rb.is_finite() || ra <= 0.0 || rb <= 0.0 || ra.near(&rb) {
            return None;
        }
        let swapped = step_number(rb)? > step_number(ra)?;
        let frame = ellipse_reader_frame(z.0, a / ra, swapped)?;
        let midpoint =
            |v: [EllipseInterval; 3]| Vector3::from(v.map(|q| q.lo + (q.hi - q.lo) * 0.5));
        let (x, y) = (midpoint(frame[0]), midpoint(frame[1]));
        let center = Point3::from_vec(step_vector(transform[3].to_point().to_vec())?);
        let radii = if swapped {
            [step_number(rb)?, step_number(ra)?]
        } else {
            [step_number(ra)?, step_number(rb)?]
        };
        let (a, b) = (x * radii[0], y * radii[1]);
        let (lo, hi) = conic.entity().range_tuple();
        let mut shift = -rotation;
        // StepToGeom canonicalizes minor-first ellipses with X'=-Y, Y'=X.
        if swapped {
            shift += std::f64::consts::FRAC_PI_2;
        }
        let range = (lo + shift, hi + shift);
        if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
            return None;
        }
        let reader_coefficients = [
            frame[0].map(|v| v * EllipseInterval::point(radii[0])),
            frame[1].map(|v| v * EllipseInterval::point(radii[1])),
        ];
        let e = EllipseInterval::point;
        let c = EllipseInterval::trig(shift, true)?;
        let sn = EllipseInterval::trig(shift, false)?;
        let mut defect = e(0.0);
        for j in 0..3 {
            let ac = reader_coefficients[0][j];
            let bc = reader_coefficients[1][j];
            defect = defect
                + e((ac * c + bc * sn - e(transform[0][j])).norm())
                + e((-ac * sn + bc * c - e(transform[1][j])).norm())
                + e((e(center[j]) - e(transform[3][j])).norm());
        }
        let source_mapping_error = defect.hi;
        source_mapping_error.is_finite().then_some(Self {
            center,
            a,
            b,
            range,
            reader_coefficients,
            source_mapping_error,
            trim_parameters: (lo - rotation, hi - rotation),
            trim_shift: if swapped {
                std::f64::consts::FRAC_PI_2
            } else {
                0.0
            },
            source_parameters: (lo, hi),
            source_shift: shift,
            reader_frame: frame,
            reader_radii: radii,
        })
    }
}

/// Conditional on the reviewed scalar policy and the reader operation mapping.
#[derive(Clone, Debug)]
pub(super) struct EllipseTrim {
    pub(super) parameters: (f64, f64),
    pub(super) domain: (f64, f64),
    pub(super) source_trim_error: f64,
}
impl StepEllipse {
    fn reader_endpoint(&self, t: f64) -> Option<[EllipseInterval; 3]> {
        let e = EllipseInterval::point;
        let scalar = EllipseInterval {
            lo: -2.0_f64.powi(-48),
            hi: 2.0_f64.powi(-48),
        };
        let ac = e(self.reader_radii[0]) * (EllipseInterval::trig(t, true)? + scalar);
        let bs = e(self.reader_radii[1]) * (EllipseInterval::trig(t, false)? + scalar);
        let p = std::array::from_fn(|j| {
            ac * self.reader_frame[0][j] + bs * self.reader_frame[1][j] + e(self.center[j])
        });
        p.iter().all(|v| v.valid()).then_some(p)
    }

    pub(super) fn bounded_trim(&self, vertices: [Point3; 2]) -> Option<EllipseTrim> {
        [0.0, std::f64::consts::TAU, -std::f64::consts::TAU]
            .into_iter()
            .find_map(|turn| self.bounded_trim_in_chart(vertices, turn))
    }

    fn bounded_trim_in_chart(&self, vertices: [Point3; 2], turn: f64) -> Option<EllipseTrim> {
        let parameters = (
            step_number(self.trim_parameters.0 + turn)?,
            step_number(self.trim_parameters.1 + turn)?,
        );
        // ExtractParameter uses Shift + Factor * parameter; writer units have Factor=1.
        let domain = (
            self.trim_shift + parameters.0,
            self.trim_shift + parameters.1,
        );
        let (lo, hi) = domain;
        if self.range.0 >= self.range.1 {
            return None;
        }
        let span = hi - lo;
        let period = std::f64::consts::TAU;
        // Both SetTrim and BRepLib AdjustPeriodic are identities under these tests.
        // Check the actual rounded divisions used by Floor, not a real-only span.
        if !lo.is_finite()
            || !hi.is_finite()
            || lo < 0.0
            || hi > 8.0
            || (lo / period).floor() != 0.0
            || period - lo < 1e-9
            || (span / period).floor() != 0.0
            || span <= 2e-9
            || self.source_mapping_error >= 5e-8
            || !self.source_mapping_error.is_finite()
        {
            return None;
        }
        let points = [self.reader_endpoint(lo)?, self.reader_endpoint(hi)?];
        let distance = |a: [EllipseInterval; 3], b: [EllipseInterval; 3]| {
            let mut sum = EllipseInterval::point(0.0);
            for j in 0..3 {
                let q = a[j] - b[j];
                let near = if q.lo <= 0.0 && q.hi >= 0.0 {
                    0.0
                } else {
                    q.lo.abs().min(q.hi.abs())
                };
                let far = q.norm();
                sum = sum
                    + EllipseInterval {
                        lo: (near * near).next_down().max(0.0),
                        hi: (far * far).next_up(),
                    };
            }
            EllipseInterval {
                lo: sum.lo.max(0.0).sqrt().next_down().max(0.0),
                hi: sum.hi.sqrt().next_up(),
            }
        };
        let vertices = vertices.map(|p| [p.x, p.y, p.z].map(EllipseInterval::point));
        if vertices.iter().flatten().any(|v| !v.valid()) {
            return None;
        }
        // LowBound is tested first even for the high vertex. Also exclude the
        // closed-edge topology-repair branch in BRepLib_MakeEdge.
        if distance(points[0], vertices[0]).hi >= 1e-7
            || distance(points[1], vertices[1]).hi >= 1e-7
            || distance(points[0], vertices[1]).lo <= 1e-7
            || distance(points[0], points[1]).lo <= 1e-7
        {
            return None;
        }
        let e = EllipseInterval::point;
        let slip = [
            (self.source_parameters.0, lo),
            (self.source_parameters.1, hi),
        ]
        .into_iter()
        .map(|(t, u)| (e(t) + e(self.source_shift) + e(turn) - e(u)).norm())
        .fold(0.0_f64, f64::max);
        let speed = self
            .reader_coefficients
            .iter()
            .flatten()
            .fold(e(0.0), |sum, q| sum + e(q.norm()));
        // A stored TAU is a chart proposal, not an assumed exact period.
        let period_defect = e((EllipseInterval::trig(turn, true)? - e(1.0)).norm())
            + e(EllipseInterval::trig(turn, false)?.norm());
        let source_trim_error =
            (e(self.source_mapping_error) + speed * (e(slip) + period_defect)).hi;
        if !source_trim_error.is_finite() || source_trim_error >= 5e-8 {
            return None;
        }
        Some(EllipseTrim {
            parameters,
            domain,
            source_trim_error,
        })
    }
}

/// Internal chart of the existing emitted cylindrical surface.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct StepCylinder {
    origin: Point3,
    x: Vector3,
    y: Vector3,
    z: Vector3,
    radius: f64,
    reader_frame: [[EllipseInterval; 3]; 3],
}
impl StepCylinder {
    fn extract(origin: Point3, axis: Vector3, radius: f64) -> Option<Self> {
        let matrix = placement(origin, axis).0;
        let (x, y, z) = step_frame(matrix[2].truncate(), matrix[0].truncate())?;
        let reader_frame = ellipse_reader_frame(matrix[2].truncate(), matrix[0].truncate(), false)?;
        let origin = Point3::from_vec(step_vector(matrix[3].to_point().to_vec())?);
        let radius = step_number(radius)?;
        (radius > 0.0).then_some(Self {
            origin,
            x,
            y,
            z,
            radius,
            reader_frame,
        })
    }
}

// Outward binary64 intervals, local to this export certificate. No library trig
// accuracy is assumed for construction: Taylor enclosures check stored controls.
#[derive(Clone, Copy, Debug)]
struct EllipseInterval {
    lo: f64,
    hi: f64,
}
impl EllipseInterval {
    fn point(x: f64) -> Self { Self { lo: x, hi: x } }
    fn valid(self) -> bool { self.lo.is_finite() && self.hi.is_finite() && self.lo <= self.hi }
    fn norm(self) -> f64 {
        if self.valid() {
            self.lo.abs().max(self.hi.abs())
        } else {
            f64::INFINITY
        }
    }
    fn error(self, x: f64) -> f64 { (self - Self::point(x)).norm() }
    fn trig(x: f64, cosine: bool) -> Option<Self> {
        if !x.is_finite() || x.abs() > 8.0 {
            return None;
        }
        let x = Self::point(x);
        let xx = x * x;
        let mut term = if cosine { Self::point(1.0) } else { x };
        let mut sum = term;
        for k in 1..32 {
            let n = 2 * k + usize::from(!cosine);
            term = -term * xx / Self::point((n * (n - 1)) as f64);
            sum = sum + term;
        }
        // Lagrange remainder at degree 62/63; all real derivatives <= 1.
        let degree = if cosine { 62 } else { 63 };
        let mut remainder = Self::point(1.0);
        for n in 1..=degree + 1 {
            remainder = remainder * Self::point(x.norm()) / Self::point(n as f64);
        }
        let result = sum
            + Self {
                lo: -remainder.hi,
                hi: remainder.hi,
            };
        result.valid().then_some(result)
    }
}
impl std::ops::Add for EllipseInterval {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self {
            lo: (self.lo + b.lo).next_down(),
            hi: (self.hi + b.hi).next_up(),
        }
    }
}
impl std::ops::Neg for EllipseInterval {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
}
impl std::ops::Sub for EllipseInterval {
    type Output = Self;
    fn sub(self, b: Self) -> Self { self + -b }
}
impl std::ops::Mul for EllipseInterval {
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
            lo: p.into_iter().fold(f64::INFINITY, f64::min).next_down(),
            hi: p.into_iter().fold(f64::NEG_INFINITY, f64::max).next_up(),
        }
    }
}
impl std::ops::Div for EllipseInterval {
    type Output = Self;
    fn div(self, b: Self) -> Self {
        if !b.valid() || (b.lo <= 0.0 && b.hi >= 0.0) {
            return Self::point(f64::NAN);
        }
        self * Self {
            lo: (1.0 / b.hi).next_down(),
            hi: (1.0 / b.lo).next_up(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct EllipseStoredBound {
    control_u: f64,
    control_v: f64,
    interpolation: f64,
    reader_u: (f64, f64),
}
impl EllipseStoredBound {
    fn exact_evaluation(self, radius: f64) -> f64 {
        (EllipseInterval::point(self.interpolation)
            + EllipseInterval::point(self.control_v)
            + EllipseInterval::point(radius) * EllipseInterval::point(self.control_u))
        .hi
    }
    fn compute(curve: &BSplineCurve<Point2>, phase: f64, sign: f64, v: [f64; 3]) -> Option<Self> {
        let (knots, multiplicities) = curve.knot_vec().to_single_multi();
        let knots: Vec<_> = knots.into_iter().map(step_number).collect::<Option<_>>()?;
        let points: Vec<_> = curve
            .control_points()
            .iter()
            .map(|p| Some([step_number(p.x)?, step_number(p.y)?]))
            .collect::<Option<_>>()?;
        if knots.len() < 2
            || points.len() != 3 * (knots.len() - 1) + 1
            || multiplicities
                .iter()
                .enumerate()
                .any(|(i, &m)| m != if i == 0 || i + 1 == knots.len() { 4 } else { 3 })
        {
            return None;
        }
        let e = EllipseInterval::point;
        let value = |t| -> Option<_> {
            let s = EllipseInterval::trig(t, false)?;
            let c = EllipseInterval::trig(t, true)?;
            Some((
                e(v[0]) + e(v[1]) * c + e(v[2]) * s,
                -e(v[1]) * s + e(v[2]) * c,
            ))
        };
        let mut bound = Self {
            control_u: 0.0,
            control_v: 0.0,
            interpolation: 0.0,
            reader_u: (f64::INFINITY, f64::NEG_INFINITY),
        };
        for (i, span) in knots.windows(2).enumerate() {
            if span[0] >= span[1] {
                return None;
            }
            let h = e(span[1]) - e(span[0]);
            if h.lo <= 0.0 {
                return None;
            }
            let third = h / e(3.0);
            let (a, da) = value(span[0])?;
            let (b, db) = value(span[1])?;
            let ideals_v = [a, a + da * third, b - db * third, b];
            let ideals_t = [
                e(span[0]),
                e(span[0]) + third,
                e(span[1]) - third,
                e(span[1]),
            ];
            for j in 0..4 {
                let [u, w] = points[3 * i + j];
                bound.control_u = bound
                    .control_u
                    .max((e(phase) + e(sign) * ideals_t[j]).error(u));
                bound.control_v = bound.control_v.max(ideals_v[j].error(w));
                bound.reader_u.0 = bound.reader_u.0.min(u);
                bound.reader_u.1 = bound.reader_u.1.max(u);
            }
            bound.interpolation = bound
                .interpolation
                .max(((e(v[1].abs()) + e(v[2].abs())) * h * h * h * h / e(384.0)).hi);
        }
        (bound.exact_evaluation(1.0).is_finite()).then_some(bound)
    }
}

// Conditional OCCT nonrational cubic evaluator contract. These bounds do not
// establish reader mutation exclusion or the deployed floating-point policy.
#[derive(Clone, Copy, Debug)]
struct EllipseEvaluation {
    error: [f64; 2],
    hull: [EllipseInterval; 2],
}

fn ellipse_up_add(a: f64, b: f64) -> f64 { (a + b).next_up() }
fn ellipse_up_mul(a: f64, b: f64) -> f64 { (a * b).next_up() }
fn ellipse_round(m: f64) -> f64 {
    if !m.is_finite() || !(0.0..f64::MAX / 2.0).contains(&m) {
        return f64::INFINITY;
    }
    // A full minimum subnormal safely majorizes the half-subnormal allowance.
    ellipse_up_add(ellipse_up_mul(2.0_f64.powi(-53), m), f64::from_bits(1))
}

#[derive(Clone, Copy)]
struct EllipseCoefficient {
    center: EllipseInterval,
    error: f64,
}
impl EllipseCoefficient {
    fn exact(x: f64) -> Self {
        Self {
            center: EllipseInterval::point(x),
            error: 0.0,
        }
    }
    fn add(self, b: Self) -> Self {
        let center = self.center + b.center;
        let error = ellipse_up_add(self.error, b.error);
        Self {
            center,
            error: ellipse_up_add(error, ellipse_round(ellipse_up_add(center.norm(), error))),
        }
    }
    fn mul(self, b: Self) -> Self {
        let center = self.center * b.center;
        let error = ellipse_up_add(
            ellipse_up_add(
                ellipse_up_mul(self.center.norm(), b.error),
                ellipse_up_mul(b.center.norm(), self.error),
            ),
            ellipse_up_mul(self.error, b.error),
        );
        Self {
            center,
            error: ellipse_up_add(error, ellipse_round(ellipse_up_add(center.norm(), error))),
        }
    }
    fn div(self, b: Self) -> Self {
        let center = self.center / b.center;
        let minimum = if b.center.lo > 0.0 {
            b.center.lo
        } else {
            -b.center.hi
        };
        let denominator = (minimum - b.error).next_down();
        let error = if denominator > 0.0 {
            (ellipse_up_add(self.error, ellipse_up_mul(center.norm(), b.error)) / denominator)
                .next_up()
        } else {
            f64::INFINITY
        };
        Self {
            center,
            error: ellipse_up_add(error, ellipse_round(ellipse_up_add(center.norm(), error))),
        }
    }
}

impl EllipseEvaluation {
    fn normalization(a: f64, b: f64, hull: EllipseInterval) -> f64 {
        let e = EllipseInterval::point;
        let h = e(b) - e(a);
        let dh = ellipse_round(h.norm());
        let d = (h.lo - dh).next_down();
        if d <= 0.0 {
            return f64::INFINITY;
        }
        let m = (hull - e(a)).norm();
        let q = (e(m) / h).hi;
        let error = (ellipse_up_add(ellipse_round(m), ellipse_up_mul(q, dh)) / d).next_up();
        ellipse_up_add(error, ellipse_round(ellipse_up_add(q, error)))
    }

    fn span(p: [f64; 4], a: f64, b: f64, hull: EllipseInterval) -> (f64, f64) {
        let e = EllipseInterval::point;
        let h = e(b) - e(a);
        let big_h = EllipseCoefficient {
            center: h,
            error: ellipse_round(h.norm()),
        };
        let inv = EllipseCoefficient::exact(1.0).div(big_h);
        let mut c = p.map(EllipseCoefficient::exact);
        for level in 0..3 {
            for j in (level + 1..4).rev() {
                let negative = EllipseCoefficient {
                    center: -c[j - 1].center,
                    error: c[j - 1].error,
                };
                c[j] = c[j].add(negative).mul(inv);
            }
        }
        // Repeated local knots make Bohm's second-phase coefficients exactly zero.
        for (j, factor) in [1.0, 3.0, 6.0, 6.0].into_iter().enumerate().skip(1) {
            c[j] = c[j].mul(EllipseCoefficient::exact(factor));
        }
        let mut scale = EllipseCoefficient::exact(1.0);
        for (j, coefficient) in c.iter_mut().enumerate() {
            *coefficient = coefficient.mul(scale);
            scale = scale.mul(big_h.div(EllipseCoefficient::exact((j + 1) as f64)));
        }
        if !scale.error.is_finite() || !scale.center.valid() {
            return (f64::INFINITY, f64::INFINITY);
        }
        let q = ((hull - e(a)) / h).norm();
        let ez = Self::normalization(a, b, hull);
        let z = ellipse_up_add(q, ez);
        let mut m = c[3].center.norm();
        let mut error = c[3].error;
        for j in (0..3).rev() {
            let product = ellipse_up_mul(z, ellipse_up_add(m, error));
            let rounded = ellipse_round(product);
            let ep = ellipse_up_add(
                ellipse_up_add(ellipse_up_mul(z, error), ellipse_up_mul(m, ez)),
                rounded,
            );
            // Explicitly include product rounding in the subsequent add magnitude.
            error = ellipse_up_add(
                ellipse_up_add(ep, c[j].error),
                ellipse_round(ellipse_up_add(
                    ellipse_up_add(product, rounded),
                    ellipse_up_add(c[j].center.norm(), c[j].error),
                )),
            );
            m = ellipse_up_add(ellipse_up_mul(m, q), c[j].center.norm());
        }
        let ex = Self::normalization(-b, -a, -hull);
        let xm = ((e(b) - hull) / h).norm();
        let ey = ellipse_up_add(ex, ellipse_round(ellipse_up_add(q, ex)));
        let mut magnitude = p.into_iter().map(f64::abs).fold(0.0, f64::max);
        let mut local = 0.0;
        for _ in 0..3 {
            let product = |weight, weight_error| {
                let w = ellipse_up_add(weight, weight_error);
                ellipse_up_add(
                    ellipse_up_add(
                        ellipse_up_mul(w, local),
                        ellipse_up_mul(magnitude, weight_error),
                    ),
                    ellipse_round(ellipse_up_mul(w, ellipse_up_add(magnitude, local))),
                )
            };
            let ep = ellipse_up_add(product(xm, ex), product(q, ey));
            local = ellipse_up_add(
                ep,
                ellipse_round(ellipse_up_add(
                    ellipse_up_mul(ellipse_up_add(xm, q), magnitude),
                    ep,
                )),
            );
            magnitude = ellipse_up_mul(magnitude, ellipse_up_add(xm, q));
        }
        let mut derivative = e(0.0);
        let mut power = e(1.0);
        for (j, coefficient) in c.iter().enumerate().skip(1) {
            derivative = derivative + e(j as f64) * e(coefficient.center.norm()) * power;
            power = power * e(q.max(1.0));
        }
        (error.max(local), (derivative / h).hi)
    }

    fn compute(curve: &BSplineCurve<Point2>, domain: (f64, f64)) -> Option<Self> {
        // Point2, no weights and this multiplicity pattern fix the polynomial type.
        if curve.degree() != 3 || curve.control_points().len() > 12289 {
            return None;
        }
        let (knots, mults) = curve.knot_vec().to_single_multi();
        if knots.len() < 2
            || curve.control_points().len() != 3 * (knots.len() - 1) + 1
            || mults
                .iter()
                .enumerate()
                .any(|(i, &m)| m != if i == 0 || i + 1 == knots.len() { 4 } else { 3 })
        {
            return None;
        }
        let knots = knots
            .into_iter()
            .map(step_number)
            .collect::<Option<Vec<_>>>()?;
        let poles = curve
            .control_points()
            .iter()
            .map(|p| Some([step_number(p.x)?, step_number(p.y)?]))
            .collect::<Option<Vec<_>>>()?;
        let hull = EllipseInterval {
            lo: domain.0,
            hi: domain.1,
        };
        if !hull.valid() {
            return None;
        }
        let scale = knots
            .iter()
            .copied()
            .map(f64::abs)
            .fold(hull.norm(), f64::max);
        if !scale.is_finite() {
            return None;
        }
        let delta = ellipse_up_add(
            1e-7 * 0.01 / 2.0,
            ellipse_up_mul(8.0, scale.next_up() - scale),
        );
        let e = EllipseInterval::point;
        for pair in knots.windows(2) {
            let h = e(pair[1]) - e(pair[0]);
            if !h.valid() || h.lo <= ellipse_up_mul(4.0, delta) {
                return None;
            }
        }
        let last = knots.len() - 1;
        if hull.lo < (e(knots[0]) - (e(knots[1]) - e(knots[0])) / e(4.0)).hi
            || hull.hi > (e(knots[last]) + (e(knots[last]) - e(knots[last - 1])) / e(4.0)).lo
        {
            return None;
        }
        let mut result = Self {
            error: [0.0; 2],
            hull: [EllipseInterval {
                lo: f64::INFINITY,
                hi: f64::NEG_INFINITY,
            }; 2],
        };
        let mut derivatives = [0.0_f64; 2];
        for (i, pair) in knots.windows(2).enumerate() {
            let (a, b) = (pair[0], pair[1]);
            let selected = EllipseInterval {
                lo: if i == 0 {
                    hull.lo
                } else {
                    hull.lo.max((a - delta).next_down())
                },
                hi: if i + 1 == last {
                    hull.hi
                } else {
                    hull.hi.min((b + delta).next_up())
                },
            };
            if selected.lo > selected.hi {
                continue;
            }
            for (j, derivative) in derivatives.iter_mut().enumerate() {
                let ps = std::array::from_fn(|k| poles[3 * i + k][j]);
                let (error, d) = Self::span(ps, a, b, selected);
                if !error.is_finite() || !d.is_finite() || error < 0.0 || d < 0.0 {
                    return None;
                }
                result.error[j] = result.error[j].max(error);
                *derivative = derivative.max(d);
                let extension = (e(a) - e(selected.lo))
                    .hi
                    .max((e(selected.hi) - e(b)).hi)
                    .max(0.0);
                let allowance = ellipse_up_add(ellipse_up_mul(extension, d), error);
                result.hull[j].lo = result.hull[j]
                    .lo
                    .min((ps.into_iter().fold(f64::INFINITY, f64::min) - allowance).next_down());
                result.hull[j].hi = result.hull[j]
                    .hi
                    .max((ps.into_iter().fold(f64::NEG_INFINITY, f64::max) + allowance).next_up());
            }
        }
        for (j, d) in derivatives.into_iter().enumerate() {
            result.error[j] = ellipse_up_add(
                result.error[j],
                ellipse_up_mul(ellipse_up_mul(2.0, delta), d),
            );
        }
        (result.hull.iter().all(|h| h.valid())
            && result.error.iter().all(|e| e.is_finite())
            && result.hull[0].lo >= -8.0
            && result.hull[0].hi <= 8.0)
            .then_some(result)
    }
}

#[derive(Clone, Copy, Debug)]
struct EllipsePositionLedger {
    evaluation: EllipseEvaluation,
    stored: f64,
    cylinder: f64,
    ellipse: f64,
    total: f64,
}
impl EllipsePositionLedger {
    fn support(
        origin: [f64; 3],
        frame: [[f64; 3]; 3],
        radii: [f64; 2],
        vmax: f64,
        errors: [f64; 2],
    ) -> f64 {
        let mut total = 0.0;
        for (j, o) in origin.into_iter().enumerate() {
            let mut terms = [0.0; 3];
            let mut error = 0.0;
            for k in 0..2 {
                let amp = radii[k].abs();
                let d = frame[k][j];
                let er = ellipse_up_add(
                    ellipse_up_mul(amp, ellipse_up_add(2.0_f64.powi(-48), errors[0])),
                    ellipse_round(ellipse_up_mul(amp, ellipse_up_add(1.0, 2.0_f64.powi(-48)))),
                );
                terms[k] = ellipse_up_mul(amp, d);
                error = ellipse_up_add(
                    error,
                    ellipse_up_add(
                        ellipse_up_mul(d, er),
                        ellipse_round(ellipse_up_mul(d, ellipse_up_add(amp, er))),
                    ),
                );
            }
            terms[2] = ellipse_up_mul(frame[2][j], vmax);
            error = ellipse_up_add(
                error,
                ellipse_up_add(
                    ellipse_up_mul(frame[2][j], errors[1]),
                    ellipse_round(ellipse_up_mul(frame[2][j], ellipse_up_add(vmax, errors[1]))),
                ),
            );
            let mut magnitude = terms[0];
            for t in [terms[1], terms[2], o.abs()] {
                magnitude = ellipse_up_add(magnitude, t);
                error = ellipse_up_add(error, ellipse_round(ellipse_up_add(magnitude, error)));
            }
            total = ellipse_up_add(total, error);
        }
        total
    }
    fn compute(
        candidate: &EllipsePcurve,
        ellipse: &StepEllipse,
        cylinder: &StepCylinder,
        domain: (f64, f64),
        source_trim_error: f64,
    ) -> Option<Self> {
        if !source_trim_error.is_finite()
            || source_trim_error < 0.0
            || domain.0 < -8.0
            || domain.1 > 8.0
        {
            return None;
        }
        let evaluation = EllipseEvaluation::compute(&candidate.curve, domain)?;
        let frame = cylinder.reader_frame.map(|a| a.map(|x| x.norm()));
        let ef = ellipse.reader_frame.map(|a| a.map(|x| x.norm()));
        let e = EllipseInterval::point;
        let sum = |a: [f64; 3]| a.into_iter().fold(e(0.0), |s, x| s + e(x));
        let stored = (e(candidate.residual)
            + e(source_trim_error)
            + sum(frame[2]) * (e(candidate.stored.interpolation) + e(candidate.stored.control_v))
            + e(cylinder.radius) * (sum(frame[0]) + sum(frame[1])) * e(candidate.stored.control_u))
        .hi;
        let cylinder_error = Self::support(
            [cylinder.origin.x, cylinder.origin.y, cylinder.origin.z],
            frame,
            [cylinder.radius; 2],
            evaluation.hull[1].norm(),
            evaluation.error,
        );
        // Ellipse D0 has no axial coordinate or argument map: domain is imported U.
        let ellipse_error = Self::support(
            [ellipse.center.x, ellipse.center.y, ellipse.center.z],
            [ef[0], ef[1], [0.0; 3]],
            ellipse.reader_radii,
            0.0,
            [0.0; 2],
        );
        let total = ellipse_up_add(ellipse_up_add(stored, cylinder_error), ellipse_error);
        let ledger = Self {
            evaluation,
            stored,
            cylinder: cylinder_error,
            ellipse: ellipse_error,
            total,
        };
        (total.is_finite() && total <= 5e-8).then_some(ledger)
    }
}

#[derive(Clone, Debug)]
pub(super) struct EllipsePcurve {
    curve: BSplineCurve<Point2>,
    residual: f64,
    interpolation: f64,
    stored: EllipseStoredBound,
}
impl EllipsePcurve {
    fn descriptor_residual(
        ellipse: &StepEllipse,
        cylinder: &StepCylinder,
        phase: f64,
        sign: f64,
        v: [f64; 3],
    ) -> Option<f64> {
        let e = EllipseInterval::point;
        let sin = EllipseInterval::trig(phase, false)?;
        let cos = EllipseInterval::trig(phase, true)?;
        let mut residual = e(0.0);
        for j in 0..3 {
            let x = e(cylinder.x[j]);
            let y = e(cylinder.y[j]);
            let z = e(cylinder.z[j]);
            let d = e(ellipse.center[j]) - e(cylinder.origin[j]);
            let a = e(ellipse.a[j]) - z * e(v[1]) - e(cylinder.radius) * (x * cos + y * sin);
            let b =
                e(ellipse.b[j]) - z * e(v[2]) - e(sign) * e(cylinder.radius) * (-x * sin + y * cos);
            residual = residual + e((d - z * e(v[0])).norm()) + e(a.norm()) + e(b.norm());
        }
        residual.valid().then_some(residual.hi)
    }

    #[cfg(test)]
    pub(super) fn candidate(ellipse: &StepEllipse, cylinder: &StepCylinder) -> Option<Self> {
        Self::candidate_on_domain(ellipse, cylinder, ellipse.range)
    }

    // reader_domain must enclose imported U1/U2, not merely the source trim.
    fn candidate_on_domain(
        ellipse: &StepEllipse,
        cylinder: &StepCylinder,
        reader_domain: (f64, f64),
    ) -> Option<Self> {
        let StepEllipse { center, a, b, .. } = *ellipse;
        let (lo, hi) = ellipse_cover_knots(reader_domain)?;
        let StepCylinder {
            origin,
            x,
            y,
            z,
            radius,
            ..
        } = *cylinder;
        let d = center - origin;
        let proposed_phase = a.dot(y).atan2(a.dot(x));
        let sign = if a.dot(x) * b.dot(y) - a.dot(y) * b.dot(x) > 0.0 {
            1.0
        } else {
            -1.0
        };
        let (v0, vc, vs) = (d.dot(z), a.dot(z), b.dot(z));
        // Verify the represented phase and selected sign directly. No atan2
        // accuracy assumption or ill-conditioned angular division enters this bound.
        // Each candidate is checked by coefficients; rounded TAU is only a proposal.
        let (phase, residual) = [-1.0, 0.0, 1.0]
            .into_iter()
            .filter_map(|turn| {
                let phase = proposed_phase + turn * std::f64::consts::TAU;
                let r = Self::descriptor_residual(ellipse, cylinder, phase, sign, [v0, vc, vs])?;
                let u = EllipseInterval::point(phase)
                    + EllipseInterval::point(sign) * EllipseInterval { lo, hi };
                (phase.abs() <= 8.0 && u.valid() && u.lo >= -8.0 && u.hi <= 8.0)
                    .then_some((phase, r))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))?;
        let e = EllipseInterval::point;
        let v_bound = e(v0.abs()) + e(vc.abs()) + e(vs.abs());
        let mut reader_error = e(0.0);
        for j in 0..3 {
            reader_error = reader_error
                + e(ellipse.reader_coefficients[0][j].error(a[j]))
                + e(ellipse.reader_coefficients[1][j].error(b[j]))
                + e(radius)
                    * (e(cylinder.reader_frame[0][j].error(x[j]))
                        + e(cylinder.reader_frame[1][j].error(y[j])))
                + v_bound * e(cylinder.reader_frame[2][j].error(z[j]));
        }
        let residual = (e(residual) + reader_error).hi;
        let amplitude = vc.abs() + vs.abs();
        // Half of the target reader's 1e-7 precision, not an ISO tolerance.
        let target = 5e-8;
        if !residual.is_finite() || residual >= target / 4.0 || !amplitude.is_finite() {
            return None;
        }
        // Allocate the interpolation budget in spatial coordinates.
        let axial_norm = cylinder.reader_frame[2]
            .iter()
            .fold(EllipseInterval::point(0.0), |s, q| {
                s + EllipseInterval::point(q.norm())
            })
            .hi;
        let step = if amplitude == 0.0 {
            hi - lo
        } else {
            (384.0 * target / (4.0 * amplitude * axial_norm))
                .sqrt()
                .sqrt()
        };
        let count = ((hi - lo) / step).ceil().max(1.0);
        if !count.is_finite() || count > 4096.0 || hi - lo > std::f64::consts::TAU {
            return None;
        }
        let count = count as usize;
        let mut knots = Vec::with_capacity(3 * count + 5);
        let mut points = Vec::with_capacity(3 * count + 1);
        let uv = |t: f64| {
            let (s, c) = t.sin_cos();
            (
                Point2::new(phase + sign * t, v0 + vc * c + vs * s),
                Vector2::new(sign, -vc * s + vs * c),
            )
        };
        let mut previous = lo;
        let mut max_h: f64 = 0.0;
        for k in 0..count {
            let next = if k + 1 == count {
                hi
            } else {
                lo + (hi - lo) * ((k + 1) as f64 / count as f64)
            };
            let h = next - previous;
            if !h.is_finite() || h <= 0.0 {
                return None;
            }
            max_h = max_h.max(h);
            let (p, dp) = uv(previous);
            let (q, dq) = uv(next);
            if k == 0 {
                points.push(p);
                knots.extend([previous; 4]);
            }
            points.extend([p + dp * (h / 3.0), q - dq * (h / 3.0), q]);
            knots.extend(std::iter::repeat_n(
                next,
                if k + 1 == count { 4 } else { 3 },
            ));
            previous = next;
        }
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return None;
        }
        let curve = BSplineCurve::new(KnotVec::from(knots), points);
        let stored = EllipseStoredBound::compute(&curve, phase, sign, [v0, vc, vs])?;
        // This hull covers exact spline u only. Reader rounding must be added
        // before invoking a reviewed scalar evaluator contract on [-8,8].
        if stored.reader_u.0 < -8.0 || stored.reader_u.1 > 8.0 {
            return None;
        }
        if stored.exact_evaluation(radius) >= target {
            return None;
        }
        EllipseEvaluation::compute(&curve, reader_domain)?;
        Some(Self {
            curve,
            stored,
            residual,
            interpolation: amplitude * max_h.powi(4) / 384.0,
        })
    }

    #[cfg(test)]
    pub(super) fn staged(
        ellipse: &StepEllipse,
        cylinder: &StepCylinder,
        trim: &EllipseTrim,
    ) -> Option<Self> {
        let candidate = Self::candidate_on_domain(ellipse, cylinder, trim.domain)?;
        let ledger = EllipsePositionLedger::compute(
            &candidate,
            ellipse,
            cylinder,
            trim.domain,
            trim.source_trim_error,
        )?;
        println!("STAGED_LEDGER {:?}", ledger);
        println!(
            "FRAME_INTERVALS {}",
            serde_json::json!({
                "ellipse": ellipse.reader_frame.map(|a| a.map(|q| [q.lo,q.hi])),
                "cylinder": cylinder.reader_frame.map(|a| a.map(|q| [q.lo,q.hi]))
            })
        );
        Some(candidate)
    }

    #[cfg(test)]
    pub(super) fn certified(ellipse: &StepEllipse, cylinder: &StepCylinder) -> Option<Self> {
        Self::certified_on_domain(
            ellipse,
            cylinder,
            ellipse.range,
            ellipse.source_mapping_error,
        )
    }

    pub(super) fn certified_on_domain(
        ellipse: &StepEllipse,
        cylinder: &StepCylinder,
        domain: (f64, f64),
        source_trim_error: f64,
    ) -> Option<Self> {
        if !ellipse_runtime::eligible() {
            return None;
        }
        let candidate = Self::candidate_on_domain(ellipse, cylinder, domain)?;
        if candidate.stored.reader_u.0 < -8.0 || candidate.stored.reader_u.1 > 8.0 {
            return None;
        }
        if !ellipse.source_mapping_error.is_finite() || ellipse.source_mapping_error >= 5e-8 {
            return None;
        }
        if !candidate.residual.is_finite()
            || !candidate.interpolation.is_finite()
            || !candidate
                .stored
                .exact_evaluation(cylinder.radius)
                .is_finite()
        {
            return None;
        }
        let ledger = EllipsePositionLedger::compute(
            &candidate,
            ellipse,
            cylinder,
            domain,
            source_trim_error,
        )?;
        if !ledger.evaluation.error.iter().all(|x| x.is_finite())
            || ![ledger.stored, ledger.cylinder, ledger.ellipse, ledger.total]
                .into_iter()
                .all(f64::is_finite)
        {
            return None;
        }
        ellipse_runtime::eligible().then_some(candidate)
    }

    pub(super) fn reference_length(&self) -> usize { 3 + self.curve.step_length() }

    pub(super) fn fmt_reference(
        &self,
        idx: usize,
        surface: usize,
        f: &mut Formatter<'_>,
    ) -> Result {
        let repr = idx + 1;
        let context = idx + 2;
        let uv = idx + 3;
        f.write_fmt(format_args!(
            "#{idx} = PCURVE('', #{surface}, #{repr});\n\
#{repr} = DEFINITIONAL_REPRESENTATION('', (#{uv}), #{context});\n\
#{context} = (GEOMETRIC_REPRESENTATION_CONTEXT(2) PARAMETRIC_REPRESENTATION_CONTEXT() REPRESENTATION_CONTEXT('2D SPACE', ''));\n"
        ))?;
        fmt_bspline_curve(&self.curve, uv, f, true)
    }
}

#[cfg(test)]
mod ellipse_pcurve_tests {
    use super::*;

    fn ellipse(range: (f64, f64), reverse: bool) -> StepEllipse {
        let transform = Matrix4::from_cols(
            Vector4::new(2.0, 0.0, 2.0, 0.0),
            Vector4::new(0.0, if reverse { -2.0 } else { 2.0 }, 0.0, 0.0),
            Vector4::unit_z(),
            Vector4::unit_w(),
        );
        StepEllipse::extract(&Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), range),
            transform,
        ))
        .unwrap()
    }

    #[test]
    fn certified_pcurve_emits_explicit_open_logical() {
        struct Reference<'a>(&'a EllipsePcurve);
        impl Display for Reference<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result { self.0.fmt_reference(100, 10, f) }
        }
        let ellipse = ellipse((0.2, 1.0), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        let candidate =
            EllipsePcurve::candidate_on_domain(&ellipse, &cylinder, ellipse.range).unwrap();
        let text = format!("{}", Reference(&candidate));
        assert!(text.contains(".UNSPECIFIED., .F., .U.,"));
        let generic = format!("{}", StepDataDisplay::new(&candidate.curve, 1));
        assert!(generic.contains(".UNSPECIFIED., .U., .U.,"));
    }

    #[test]
    fn whole_span_hermite_candidate_and_reversed_axes() {
        for reverse in [false, true] {
            for axis in [Vector3::unit_x(), Vector3::unit_z(), -Vector3::unit_z()] {
                let ellipse = ellipse((5.9, 6.5), reverse);
                let cylinder = StepCylinder::extract(Point3::origin(), axis, 2.0).unwrap();
                let candidate = EllipsePcurve::candidate(&ellipse, &cylinder).unwrap();
                assert!(candidate.interpolation > 0.0 && candidate.interpolation <= 1.25e-8);
                println!(
                    "reverse={reverse} axis={axis:?} descriptor_residual={:.17e} stored={:?}",
                    candidate.residual, candidate.stored
                );
                for i in 0..1001 {
                    let t = 5.9 + 0.6 * i as f64 / 1000.0;
                    let uv = candidate.curve.subs(t);
                    let (s, c) = t.sin_cos();
                    let expected = ellipse.center + ellipse.a * c + ellipse.b * s;
                    let actual = cylinder.origin
                        + cylinder.radius * (cylinder.x * uv.x.cos() + cylinder.y * uv.x.sin())
                        + cylinder.z * uv.y;
                    assert!(expected.distance(actual) <= candidate.interpolation + 1e-12);
                }
                assert_eq!(
                    EllipsePcurve::certified(&ellipse, &cylinder).is_some(),
                    ellipse_runtime::eligible()
                        && EllipsePositionLedger::compute(
                            &candidate,
                            &ellipse,
                            &cylinder,
                            ellipse.range,
                            ellipse.source_mapping_error
                        )
                        .is_some()
                );
            }
        }
    }

    #[test]
    fn stored_certificate_detects_serialized_control_perturbation() {
        let ellipse = ellipse((0.001234567890123, 1.1), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        let candidate = EllipsePcurve::candidate(&ellipse, &cylinder).unwrap();
        assert!(candidate.stored.exact_evaluation(2.0) < 5e-8);
        let knots = candidate.curve.knot_vec().clone();
        let mut points = candidate.curve.control_points().to_vec();
        points[1].y += 0.001;
        let bad = BSplineCurve::new(knots, points);
        let bound = EllipseStoredBound::compute(&bad, 0.0, 1.0, [0.0, 2.0, 0.0]).unwrap();
        assert!(bound.exact_evaluation(2.0) > 0.0009);
        assert!(EllipseInterval::trig(8.0, false).is_some());
        assert!(EllipseInterval::trig(8.0_f64.next_up(), false).is_none());
        assert!(!(EllipseInterval::point(1.0) / EllipseInterval { lo: -1.0, hi: 1.0 }).valid());
    }

    #[test]
    fn evaluator_rejects_unresolved_reader_span() {
        let ellipse = ellipse((0.1, 0.1000000001), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        assert!(EllipsePcurve::candidate(&ellipse, &cylinder).is_none());
    }

    #[test]
    fn serialized_endpoint_knots_cover_whole_descriptor_domain() {
        let ellipse = ellipse((0.001, 0.002234567890126), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        let candidate = EllipsePcurve::candidate(&ellipse, &cylinder).unwrap();
        let end = candidate.curve.range_tuple().1;
        let serialized_end = step_number(end).unwrap();
        assert!(serialized_end >= ellipse.range.1);
        println!(
            "endpoint_sliver original={end:.17e} serialized={serialized_end:.17e} missing={:.17e}",
            end - serialized_end
        );
        // Admission still requires a certified reader-projected domain.
        assert_eq!(
            EllipsePcurve::certified(&ellipse, &cylinder).is_some(),
            ellipse_runtime::eligible()
                && EllipsePositionLedger::compute(
                    &candidate,
                    &ellipse,
                    &cylinder,
                    ellipse.range,
                    ellipse.source_mapping_error
                )
                .is_some()
        );
    }

    #[test]
    fn phase_residual_rejects_wrong_phase_and_orientation() {
        let ellipse = ellipse((0.1, 1.1), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        let phase = ellipse.a.dot(cylinder.y).atan2(ellipse.a.dot(cylinder.x));
        let d = ellipse.center - cylinder.origin;
        let v = [
            d.dot(cylinder.z),
            ellipse.a.dot(cylinder.z),
            ellipse.b.dot(cylinder.z),
        ];
        assert!(
            EllipsePcurve::descriptor_residual(&ellipse, &cylinder, phase, 1.0, v).unwrap() < 5e-8
        );
        assert!(
            EllipsePcurve::descriptor_residual(&ellipse, &cylinder, phase + 0.01, 1.0, v).unwrap()
                > 0.01
        );
        assert!(
            EllipsePcurve::descriptor_residual(&ellipse, &cylinder, phase, -1.0, v).unwrap() > 1.0
        );
    }

    #[test]
    fn perturbation_and_unbounded_allocation_decline() {
        let mut ellipse = ellipse((0.1, 2.0), false);
        let cylinder = StepCylinder::extract(Point3::origin(), Vector3::unit_z(), 2.0).unwrap();
        ellipse.center.x += 1e-5;
        assert!(EllipsePcurve::candidate(&ellipse, &cylinder).is_none());
        ellipse.center.x = 0.0;
        ellipse.range.1 = 1e200;
        assert!(EllipsePcurve::candidate(&ellipse, &cylinder).is_none());
        ellipse.range = (0.0, 1.0);
        ellipse.a.z = 1e100;
        assert!(EllipsePcurve::candidate(&ellipse, &cylinder).is_none());
    }

    #[test]
    fn rotated_coefficients_and_reversal_keep_spatial_parameter_map() {
        let (sin, cos) = 0.4_f64.sin_cos();
        let a = Vector3::new(3.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 2.0, 0.0);
        let matrix = Matrix4::from_cols(
            (a * cos + b * sin).extend(0.0),
            (b * cos - a * sin).extend(0.0),
            Vector4::unit_z(),
            Vector4::unit_w(),
        );
        let mut conic =
            Processor::with_transform(TrimmedCurve::new(UnitCircle::new(), (0.2, 1.7)), matrix);
        let ellipse = StepEllipse::extract(&conic).unwrap();
        for i in 0..21 {
            let t = 0.2 + 1.5 * i as f64 / 20.0;
            let mapped = t + ellipse.range.0 - 0.2;
            let actual = ellipse.center + ellipse.a * mapped.cos() + ellipse.b * mapped.sin();
            assert!(conic.subs(t).distance(actual) < 5e-15);
        }
        conic.invert();
        let reversed = StepEllipse::extract(&conic).unwrap();
        assert_eq!(ellipse.range, reversed.range);
        assert_eq!(ellipse.a, reversed.a);
        assert_eq!(ellipse.b, reversed.b);
    }

    #[test]
    fn minor_first_parameter_mapping_preserves_points() {
        let transform = Matrix4::from_nonuniform_scale(2.0, 3.0, 1.0);
        let conic =
            Processor::with_transform(TrimmedCurve::new(UnitCircle::new(), (0.2, 1.7)), transform);
        let ellipse = StepEllipse::extract(&conic).unwrap();
        assert_eq!(ellipse.range.0, 0.2 + std::f64::consts::FRAC_PI_2);
        for i in 0..21 {
            let t = 0.2 + 1.5 * i as f64 / 20.0;
            let mapped = t + std::f64::consts::FRAC_PI_2;
            let expected = conic.subs(t);
            let actual = ellipse.center + ellipse.a * mapped.cos() + ellipse.b * mapped.sin();
            assert!(expected.distance(actual) < 2e-15);
        }
    }
}

// The inputs are the actual decimal direction ratios. Interval arithmetic follows
// gp_Dir and gp_XYZ operation trees, including normalization after canonical swap.
fn ellipse_reader_frame(z: Vector3, x: Vector3, swap: bool) -> Option<[[EllipseInterval; 3]; 3]> {
    let e = EllipseInterval::point;
    let input = |v: Vector3| -> Option<[EllipseInterval; 3]> {
        Some([
            e(step_number(v.x)?),
            e(step_number(v.y)?),
            e(step_number(v.z)?),
        ])
    };
    let normalize = |v: [EllipseInterval; 3]| -> Option<[EllipseInterval; 3]> {
        let n2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        // Conservative well-conditioned subset; avoids reader default-axis branches.
        if !n2.valid() || n2.lo < 0.25 || n2.hi > 4.0 {
            return None;
        }
        let n = EllipseInterval {
            lo: n2.lo.sqrt().next_down(),
            hi: n2.hi.sqrt().next_up(),
        };
        let result = v.map(|q| q / n);
        result.iter().all(|q| q.valid()).then_some(result)
    };
    let cross = |a: [EllipseInterval; 3], b: [EllipseInterval; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    // gp_XYZ::CrossCross evaluates a x (b x c), without normalizing the inner cross.
    let z = normalize(normalize(input(z)?)?)?;
    let raw_x = normalize(normalize(input(x)?)?)?;
    let mut x = normalize(cross(z, cross(raw_x, z)))?;
    let _placement_y = normalize(cross(z, x))?;
    // Geom_Axis2Placement::Ax2 reconstructs gp_Ax2 a second time.
    x = normalize(cross(z, cross(x, z)))?;
    let mut y = normalize(cross(z, x))?;
    if swap {
        let vx = normalize(cross(x, z))?;
        x = normalize(cross(z, cross(vx, z)))?;
        y = normalize(cross(z, x))?;
    }
    Some([x, y, z])
}

// Extend the representation, never the edge trim. Certify the read-back endpoints
// rather than assuming one binary64 ulp survives decimal formatting.
fn ellipse_cover_knots(hull: (f64, f64)) -> Option<(f64, f64)> {
    if !hull.0.is_finite() || !hull.1.is_finite() || hull.0 >= hull.1 {
        return None;
    }
    let cover = |value: f64, upper: bool| -> Option<f64> {
        let mut delta = (value.abs().max(1.0) * f64::EPSILON).next_up();
        for _ in 0..64 {
            let proposed = if upper {
                (value + delta).next_up()
            } else {
                (value - delta).next_down()
            };
            let stored = step_number(proposed)?;
            if (upper && stored >= value) || (!upper && stored <= value) {
                return (stored.abs() <= 8.0).then_some(stored);
            }
            delta *= 2.0;
        }
        None
    };
    Some((cover(hull.0, false)?, cover(hull.1, true)?))
}

#[cfg(test)]
mod ellipse_reader_domain_tests {
    use super::*;
    #[test]
    fn decimal_endpoint_cover_and_domain_rejection() {
        let h = (0.001, 0.002234567890126);
        let (a, b) = ellipse_cover_knots(h).unwrap();
        assert!(step_number(a).unwrap() <= h.0);
        assert!(step_number(b).unwrap() >= h.1);
        assert!(ellipse_cover_knots((7.9, 8.0000000001)).is_none());
    }
    #[test]
    fn reader_frame_rejects_degenerate_axes() {
        assert!(ellipse_reader_frame(Vector3::unit_z(), Vector3::unit_z(), false).is_none());
        for swap in [false, true] {
            let f = ellipse_reader_frame(Vector3::unit_z(), Vector3::unit_x(), swap).unwrap();
            let expected = if swap {
                -Vector3::unit_y()
            } else {
                Vector3::unit_x()
            };
            for j in 0..3 {
                assert!(f[0][j].lo <= expected[j] && f[0][j].hi >= expected[j]);
            }
        }
    }
}

#[cfg(test)]
mod bounded_trim_tests {
    use super::*;
    #[test]
    fn bounded_endpoint_order_and_negative_controls() {
        let conic = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0.0, 1.7)),
            Matrix4::from_nonuniform_scale(3.0, 2.0, 1.0),
        );
        let ellipse = StepEllipse::extract(&conic).unwrap();
        let vertices = [conic.subs(0.0), conic.subs(1.7)];
        let trim = ellipse.bounded_trim(vertices).unwrap();
        assert_eq!(trim.domain.0, 0.0);
        assert!(ellipse.bounded_trim([vertices[1], vertices[0]]).is_none());
        let mut wrong = vertices;
        wrong[1].x += 1e-4;
        assert!(ellipse.bounded_trim(wrong).is_none());
        let mut wrong = ellipse.clone();
        wrong.trim_parameters.1 += 0.1;
        assert!(wrong.bounded_trim(vertices).is_none());
        wrong = ellipse.clone();
        wrong.trim_parameters.1 = 1e-12;
        assert!(wrong.bounded_trim(vertices).is_none());
        wrong = ellipse.clone();
        wrong.trim_shift += std::f64::consts::FRAC_PI_2;
        assert!(wrong.bounded_trim(vertices).is_none());
        wrong = ellipse.clone();
        wrong.trim_parameters.0 = f64::NAN;
        assert!(wrong.bounded_trim(vertices).is_none());
    }
}

#[cfg(test)]
mod retained_endpoint_ledgers {
    use super::*;
    #[test]
    fn actual_serialized_frames_endpoints_and_candidate_ledgers() {
        let fixtures = [
            (
                "G4",
                [16.390818326419698, 10.33169233219575, -2.2204460493e-16],
                [0.0, 0.0, 0.9999999999999999],
                [2.1773264873e-15, 1.0, 0.0],
                [10.198039027185569, 9.999999999999998],
                [1.09925857222641, 3.157997706991],
                [7.48211026267222, 14.964220525344441, 0.0],
                [16.554861502159554, 0.13502555182322062, -2.2204460493e-16],
                [16.390818326419705, 9.93431955018822, -1.986863910037644],
                [-1.7763568394e-17, 0.19611613513818402, 0.9805806756909201],
                [1.0, 3.4837223797e-18, 1.7418611898e-17],
                10.000000000000005,
            ),
            (
                "G4",
                [16.3908183264197, 30.33169233219575, 100.0],
                [0.0, 0.0, 1.0],
                [-1.1757563031e-15, -1.0, 0.0],
                [10.198039027185573, 10.000000000000002],
                [5.90085289615864, 6.2995903605808],
                [12.659963464915183, 20.869982609186717, 99.99999999999999],
                [16.554861502159554, 20.13502555182322, 100.0],
                [16.390818326419705, 9.93431955018822, -1.986863910037644],
                [-1.7763568394e-17, 0.19611613513818402, 0.9805806756909201],
                [1.0, 3.4837223797e-18, 1.7418611898e-17],
                10.000000000000005,
            ),
            (
                "G4",
                [16.390818326419705, 28.33169233219575, 89.99999999999999],
                [0.29167889974076266, 0.711267458521669, 0.6395482952007174],
                [
                    -0.8908708063747481,
                    0.44543540318737374,
                    -0.08908708063747454,
                ],
                [10.000000000000004, 13.044276948453827],
                [3.552713678800501e-15, 1.6600016703424334],
                [7.48211026267222, 32.78604636406949, 89.10912919362524],
                [12.659963464915183, 20.869982609186717, 99.99999999999999],
                [16.390818326419705, 9.93431955018822, -1.986863910037644],
                [-1.7763568394e-17, 0.19611613513818402, 0.9805806756909201],
                [1.0, 3.4837223797e-18, 1.7418611898e-17],
                10.000000000000005,
            ),
            (
                "G8",
                [6.160353002686564, 5.099019513592786, -4.4408920985e-16],
                [-8.5880387187e-49, -5.4433162183e-17, -1.0],
                [-9.390830899e-17, -1.0, 5.4433162183e-17],
                [5.099019513592787, 5.000000000000001],
                [0.0, 1.7844069032771],
                [6.160353002686563, 0.0, 0.0],
                [1.273993597583394, 6.179959583497775, -4.4408920985e-16],
                [6.160353002686561, 4.9029033784546, -0.9805806756909203],
                [-1.5322889601e-17, -0.1961161351381841, -0.9805806756909202],
                [1.0, -3.0050658877e-18, -1.5025329438e-17],
                5.0,
            ),
            (
                "G8",
                [6.160353002686556, 25.099019513592786, 100.0],
                [0.0, -5.4433162183e-17, -1.0],
                [-1.9595938386e-16, -1.0, 5.4433162183e-17],
                [5.099019513592779, 4.999999999999993],
                [1.37340076694502, 1.7844069032771],
                [1.2574496242319624, 24.099019513592786, 100.0],
                [1.273993597583394, 26.179959583497777, 99.99999999999999],
                [6.160353002686561, 4.9029033784546, -0.9805806756909203],
                [-1.5322889601e-17, -0.1961161351381841, -0.9805806756909202],
                [1.0, -3.0050658877e-18, -1.5025329438e-17],
                5.0,
            ),
            (
                "G8",
                [6.160353002686561, 24.099019513592786, 95.0],
                [
                    -0.7071067811865475,
                    -0.13867504905630731,
                    -0.6933752452815365,
                ],
                [2.8972705903e-16, -0.9805806756909202, 0.19611613513818382],
                [4.999999999999999, 7.071067811865474],
                [3.552713678800501e-15, 1.3734007669450135],
                [6.160353002686563, 19.196116135138187, 95.98058067569092],
                [1.2574496242319624, 24.099019513592786, 100.0],
                [6.160353002686561, 4.9029033784546, -0.9805806756909203],
                [-1.5322889601e-17, -0.1961161351381841, -0.9805806756909202],
                [1.0, -3.0050658877e-18, -1.5025329438e-17],
                5.0,
            ),
        ];
        for (index, (case, o, z, x, radii, parameters, first, last, co, cz, cx, radius)) in
            fixtures.into_iter().enumerate()
        {
            let swapped = radii[1] > radii[0];
            let frame = ellipse_reader_frame(Vector3::from(z), Vector3::from(x), swapped).unwrap();
            let shift = if swapped {
                std::f64::consts::FRAC_PI_2
            } else {
                0.0
            };
            let radii = if swapped { [radii[1], radii[0]] } else { radii };
            let midpoint =
                |v: [EllipseInterval; 3]| Vector3::from(v.map(|q| q.lo + (q.hi - q.lo) * 0.5));
            let range = (parameters[0] + shift, parameters[1] + shift);
            let ellipse = StepEllipse {
                center: Point3::from(o),
                a: midpoint(frame[0]) * radii[0],
                b: midpoint(frame[1]) * radii[1],
                range,
                reader_coefficients: [
                    frame[0].map(|q| q * EllipseInterval::point(radii[0])),
                    frame[1].map(|q| q * EllipseInterval::point(radii[1])),
                ],
                source_mapping_error: 0.0,
                trim_parameters: (parameters[0], parameters[1]),
                trim_shift: shift,
                source_parameters: (parameters[0], parameters[1]),
                source_shift: shift,
                reader_frame: frame,
                reader_radii: radii,
            };
            // This fixture binds emitted analytic coefficients, not the original pre-Gram transform.
            let trim = ellipse
                .bounded_trim([Point3::from(first), Point3::from(last)])
                .unwrap();
            let (xx, yy, zz) = step_frame(Vector3::from(cz), Vector3::from(cx)).unwrap();
            let cylinder = StepCylinder {
                origin: Point3::from(co),
                x: xx,
                y: yy,
                z: zz,
                radius,
                reader_frame: ellipse_reader_frame(Vector3::from(cz), Vector3::from(cx), false)
                    .unwrap(),
            };
            let candidate =
                EllipsePcurve::candidate_on_domain(&ellipse, &cylinder, trim.domain).unwrap();
            assert_eq!(
                EllipsePcurve::certified_on_domain(
                    &ellipse,
                    &cylinder,
                    trim.domain,
                    trim.source_trim_error
                )
                .is_some(),
                ellipse_runtime::eligible()
            );
            let norms = cylinder.reader_frame.map(|axis| axis.map(|q| q.norm()));
            let en = ellipse.reader_frame.map(|axis| axis.map(|q| q.norm()));
            println!("LEDGER {{\"case\":\"{}\",\"index\":{},\"domain\":{:?},\"residual\":{:?},\"stored\":{:?},\"origin\":{:?},\"frame\":{:?},\"radius\":{:?},\"ellipse_origin\":{:?},\"ellipse_frame\":{:?},\"radii\":{:?}}}",
                case, index, [trim.domain.0,trim.domain.1], candidate.residual,
                [candidate.stored.interpolation,candidate.stored.control_u,candidate.stored.control_v], co,norms,radius,o,en,radii);
            println!(
                "BEGIN_CURVE {}\n{}END_CURVE",
                index,
                StepDataDisplay::new(&candidate.curve, 1)
            );
        }
    }
}

#[cfg(test)]
mod original_endpoint_fixtures {
    use super::*;
    #[test]
    fn original_gram_frames_and_topological_vertices() {
        let fixtures = [
            (
                "G4",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764597]},"transform":{"x":{"x":-8.908708063747477,"y":4.632528193148691,"z":0.0,"w":0.0},"y":{"x":-4.542567625794981,"y":-9.085135251589957,"z":0.0,"w":0.0},"z":{"x":0.0,"y":0.0,"z":1.0,"w":0.0},"w":{"x":16.390818326419698,"y":10.33169233219575,"z":-2.220446049250313e-16,"w":1.0}},"orientation":true}},[{"x":7.48211026267222,"y":14.964220525344441,"z":0.0},{"x":16.554861502159554,"y":0.13502555182322062,"z":-2.220446049250313e-16}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                "G4",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,0.3987374644221573]},"transform":{"x":{"x":-3.7308548615045143,"y":-9.461709723009035,"z":0.0,"w":0.0},"y":{"x":9.277969713379548,"y":-3.8047403482388047,"z":0.0,"w":0.0},"z":{"x":0.0,"y":0.0,"z":1.0,"w":0.0},"w":{"x":16.3908183264197,"y":30.33169233219575,"z":100.0,"w":1.0}},"orientation":true}},[{"x":12.659963464915183,"y":20.869982609186717,"z":99.99999999999999},{"x":16.554861502159554,"y":20.13502555182322,"z":100.0}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                "G4",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.6600016703424378]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873739,"z":-0.8908708063747458,"w":0.0},"y":{"x":-4.5425676257949785,"y":-7.093087569365973,"z":9.960238411119947,"w":0.0},"z":{"x":0.29167889974076266,"y":0.711267458521669,"z":0.6395482952007175,"w":0.0},"w":{"x":16.390818326419705,"y":28.33169233219575,"z":89.99999999999999,"w":1.0}},"orientation":true}},[{"x":7.48211026267222,"y":32.78604636406949,"z":89.10912919362524},{"x":12.659963464915183,"y":20.869982609186717,"z":99.99999999999999}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                "G8",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.784406903277099]},"transform":{"x":{"x":-4.788403000283334e-16,"y":-5.099019513592787,"z":2.775557561562892e-16,"w":0.0},"y":{"x":-5.000000000000001,"y":4.883232067478193e-16,"z":-2.6580976310493685e-32,"w":0.0},"z":{"x":0.0,"y":-5.443316218272764e-17,"z":-1.0,"w":0.0},"w":{"x":6.160353002686564,"y":5.099019513592786,"z":-4.440892098500627e-16,"w":1.0}},"orientation":true}},[{"x":6.160353002686563,"y":0.0,"z":0.0},{"x":1.273993597583394,"y":6.179959583497775,"z":-4.440892098500626e-16}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
            (
                "G8",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,0.4110061363320847]},"transform":{"x":{"x":-4.902903378454593,"y":-1.000000000000001,"z":5.4433162182727695e-17,"w":0.0},"y":{"x":-0.9805806756909209,"y":4.999999999999994,"z":-2.7216581091363785e-16,"w":0.0},"z":{"x":0.0,"y":-5.443316218272764e-17,"z":-1.0,"w":0.0},"w":{"x":6.160353002686556,"y":25.099019513592786,"z":100.0,"w":1.0}},"orientation":true}},[{"x":1.2574496242319624,"y":24.099019513592786,"z":100.0},{"x":1.273993597583394,"y":26.179959583497777,"z":99.99999999999999}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
            (
                "G8",
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.3734007669450161]},"transform":{"x":{"x":1.4486352951323498e-15,"y":-4.9029033784546,"z":0.9805806756909189,"w":0.0},"y":{"x":-4.999999999999999,"y":0.9805806756909186,"z":4.9029033784545994,"w":0.0},"z":{"x":-0.7071067811865476,"y":-0.13867504905630734,"z":-0.6933752452815365,"w":0.0},"w":{"x":6.160353002686561,"y":24.099019513592786,"z":95.0,"w":1.0}},"orientation":true}},[{"x":6.160353002686563,"y":19.196116135138187,"z":95.98058067569092},{"x":1.2574496242319624,"y":24.099019513592786,"z":100.0}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
        ];
        for (index, (case, data)) in fixtures.into_iter().enumerate() {
            let (curve, vertices, surface): (
                truck_modeling::Curve,
                [Point3; 2],
                truck_modeling::Surface,
            ) = serde_json::from_str(data).unwrap();
            let ellipse = curve.step_ellipse().unwrap();
            let vertices = vertices.map(|p| Point3::from(p.step_point_coordinates().unwrap()));
            let vertices = if curve.same_sense() {
                vertices
            } else {
                [vertices[1], vertices[0]]
            };
            println!(
                "ORIGINAL {case} range={:?} precanonical={:?} shift={:?}",
                ellipse.range, ellipse.trim_parameters, ellipse.trim_shift
            );
            let trim = ellipse.bounded_trim(vertices).unwrap();
            let cylinder = surface.step_cylinder().unwrap();
            let candidate =
                EllipsePcurve::candidate_on_domain(&ellipse, &cylinder, trim.domain).unwrap();
            println!(
                "ORIGINAL bound={:?} mapping={:?} trim={:?}",
                candidate.stored, ellipse.source_mapping_error, trim
            );
            let ledger = EllipsePositionLedger::compute(
                &candidate,
                &ellipse,
                &cylinder,
                trim.domain,
                trim.source_trim_error,
            )
            .unwrap();
            for invalid_charge in [-1.0, f64::NAN, f64::INFINITY, 5e-8] {
                assert!(EllipsePositionLedger::compute(
                    &candidate,
                    &ellipse,
                    &cylinder,
                    trim.domain,
                    invalid_charge
                )
                .is_none());
            }
            assert_eq!(
                EllipsePcurve::certified_on_domain(
                    &ellipse,
                    &cylinder,
                    trim.domain,
                    trim.source_trim_error
                )
                .is_some(),
                ellipse_runtime::eligible()
            );
            println!(
                "TYPED {} {:?} {:?} {:?} {:?} {:?} {:?}",
                index,
                ledger.evaluation.error,
                ledger.evaluation.hull.map(|h| [h.lo, h.hi]),
                ledger.stored,
                ledger.cylinder,
                ledger.ellipse,
                ledger.total
            );
            let co = [cylinder.origin.x, cylinder.origin.y, cylinder.origin.z];
            let o = [ellipse.center.x, ellipse.center.y, ellipse.center.z];
            let radius = cylinder.radius;
            let radii = ellipse.reader_radii;
            let norms = cylinder.reader_frame.map(|axis| axis.map(|q| q.norm()));
            let en = ellipse.reader_frame.map(|axis| axis.map(|q| q.norm()));
            println!("LEDGER {{\"case\":\"{}\",\"index\":{},\"domain\":{:?},\"residual\":{:?},\"stored\":{:?},\"origin\":{:?},\"frame\":{:?},\"radius\":{:?},\"ellipse_origin\":{:?},\"ellipse_frame\":{:?},\"radii\":{:?}}}",
                case, index, [trim.domain.0,trim.domain.1], (EllipseInterval::point(candidate.residual) + EllipseInterval::point(trim.source_trim_error)).hi,
                [candidate.stored.interpolation,candidate.stored.control_u,candidate.stored.control_v], co,norms,radius,o,en,radii);
            println!(
                "BEGIN_CURVE {}\n{}END_CURVE",
                index,
                StepDataDisplay::new(&candidate.curve, 1)
            );
        }
    }
}

#[cfg(test)]
mod ellipse_evaluator_tests {
    use super::*;
    #[test]
    fn negative_controls_and_continuation() {
        for (name, knots, poles, domain, accepted) in [
            (
                "negative",
                vec![-2.0, -1.0, 0.0],
                vec![
                    [-2.0, -4.0],
                    [-1.7, 2.0],
                    [-1.3, -3.0],
                    [-1.0, 1.0],
                    [-0.7, -2.0],
                    [-0.3, 3.0],
                    [0.0, -1.0],
                ],
                (-2.01, 0.01),
                true,
            ),
            (
                "narrow",
                vec![0.0, 1e-10],
                vec![[0.0, 0.0]; 4],
                (0.0, 1e-10),
                false,
            ),
            (
                "extension",
                vec![0.0, 1.0],
                vec![[0.0, 0.0]; 4],
                (-0.3, 1.0),
                false,
            ),
            (
                "argument",
                vec![0.0, 1.0],
                vec![[8.0, 0.0]; 4],
                (0.0, 1.0),
                false,
            ),
            (
                "overflow",
                vec![0.0, 1.0],
                vec![[0.0, 1e308]; 4],
                (0.0, 1.0),
                false,
            ),
            (
                "nonfinite",
                vec![0.0, 1.0],
                vec![[0.0, f64::NAN]; 4],
                (0.0, 1.0),
                false,
            ),
        ] {
            let mut flat = Vec::new();
            let mut mults = Vec::new();
            for (i, k) in knots.iter().enumerate() {
                let n = if i == 0 || i + 1 == knots.len() { 4 } else { 3 };
                mults.push(n);
                flat.extend(std::iter::repeat_n(*k, n));
            }
            let curve = BSplineCurve::new(
                KnotVec::from(flat),
                poles.iter().map(|p| Point2::new(p[0], p[1])).collect(),
            );
            let result = EllipseEvaluation::compute(&curve, domain);
            assert_eq!(result.is_some(), accepted, "{name}");
            if name != "nonfinite" {
                println!(
                    "CONTROL {}",
                    serde_json::json!({"name":name,"knots":knots,"poles":poles,"mults":mults,"domain":[domain.0,domain.1],"accepted":accepted,"error":result.map(|r|r.error),"hull":result.map(|r|r.hull.map(|h|[h.lo,h.hi]))})
                );
            }
        }
        assert!(
            !EllipseEvaluation::normalization(1.0, 1.0, EllipseInterval::point(1.0)).is_finite()
        );
    }
}

#[cfg(test)]
mod all_association_fixtures {
    use super::*;
    #[test]
    fn every_original_cylinder_side_has_current_ledger() {
        let fixtures = [
            (
                r#"{"case":"G4","edge":8,"face":4,"wire":0,"occurrence":1,"edge_orientation":true,"face_orientation":true,"vertices":[5,4],"input_sha256":"d590efb671deb93ffc187001a068045d7301ff7ce93a791984dbbcc4754bf34c"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764597]},"transform":{"x":{"x":-8.908708063747477,"y":4.632528193148691,"z":0.0,"w":0.0},"y":{"x":-4.542567625794981,"y":-9.085135251589957,"z":0.0,"w":0.0},"z":{"x":0.0,"y":0.0,"z":1.0,"w":0.0},"w":{"x":16.390818326419698,"y":10.33169233219575,"z":-2.220446049250313e-16,"w":1.0}},"orientation":true}},[{"x":7.48211026267222,"y":14.964220525344441,"z":0.0},{"x":16.554861502159554,"y":0.13502555182322062,"z":-2.220446049250313e-16}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                r#"{"case":"G4","edge":9,"face":4,"wire":0,"occurrence":3,"edge_orientation":false,"face_orientation":true,"vertices":[7,3],"input_sha256":"d590efb671deb93ffc187001a068045d7301ff7ce93a791984dbbcc4754bf34c"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,0.3987374644221573]},"transform":{"x":{"x":-3.7308548615045143,"y":-9.461709723009035,"z":0.0,"w":0.0},"y":{"x":9.277969713379548,"y":-3.8047403482388047,"z":0.0,"w":0.0},"z":{"x":0.0,"y":0.0,"z":1.0,"w":0.0},"w":{"x":16.3908183264197,"y":30.33169233219575,"z":100.0,"w":1.0}},"orientation":true}},[{"x":12.659963464915183,"y":20.869982609186717,"z":99.99999999999999},{"x":16.554861502159554,"y":20.13502555182322,"z":100.0}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                r#"{"case":"G4","edge":11,"face":4,"wire":0,"occurrence":4,"edge_orientation":false,"face_orientation":true,"vertices":[6,7],"input_sha256":"d590efb671deb93ffc187001a068045d7301ff7ce93a791984dbbcc4754bf34c"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.6600016703424378]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873739,"z":-0.8908708063747458,"w":0.0},"y":{"x":-4.5425676257949785,"y":-7.093087569365973,"z":9.960238411119947,"w":0.0},"z":{"x":0.29167889974076266,"y":0.711267458521669,"z":0.6395482952007175,"w":0.0},"w":{"x":16.390818326419705,"y":28.33169233219575,"z":89.99999999999999,"w":1.0}},"orientation":true}},[{"x":7.48211026267222,"y":32.78604636406949,"z":89.10912919362524},{"x":12.659963464915183,"y":20.869982609186717,"z":99.99999999999999}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,2.058739134764596]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873741,"z":-0.8908708063747484,"w":0.0},"y":{"x":-4.542567625794981,"y":-8.735706972682658,"z":1.7471413945365315,"w":0.0},"z":{"x":0.0,"y":0.19611613513818404,"z":0.9805806756909202,"w":0.0},"w":{"x":16.390818326419705,"y":9.93431955018822,"z":-1.986863910037644,"w":1.0}},"orientation":true}},"vector":{"x":0.0,"y":27.692307692307693,"z":138.46153846153845}}}]"#,
            ),
            (
                r#"{"case":"G4","edge":11,"face":5,"wire":0,"occurrence":1,"edge_orientation":true,"face_orientation":true,"vertices":[6,7],"input_sha256":"d590efb671deb93ffc187001a068045d7301ff7ce93a791984dbbcc4754bf34c"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.6600016703424378]},"transform":{"x":{"x":-8.908708063747484,"y":4.454354031873739,"z":-0.8908708063747458,"w":0.0},"y":{"x":-4.5425676257949785,"y":-7.093087569365973,"z":9.960238411119947,"w":0.0},"z":{"x":0.29167889974076266,"y":0.711267458521669,"z":0.6395482952007175,"w":0.0},"w":{"x":16.390818326419705,"y":28.33169233219575,"z":89.99999999999999,"w":1.0}},"orientation":true}},[{"x":7.48211026267222,"y":32.78604636406949,"z":89.10912919362524},{"x":12.659963464915183,"y":20.869982609186717,"z":99.99999999999999}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.6600016703424387]},"transform":{"x":{"x":-8.908708063747476,"y":4.45435403187374,"z":-0.8908708063747497,"w":0.0},"y":{"x":-0.7968190728895965,"y":0.3984095364448004,"z":9.960238411119944,"w":0.0},"z":{"x":0.44721359549995804,"y":0.8944271909999159,"z":-1.850988594463041e-16,"w":0.0},"w":{"x":7.305683074829738,"y":10.161421829015834,"z":89.99999999999999,"w":1.0}},"orientation":true}},"vector":{"x":116.33806231708238,"y":232.67612463416478,"z":0.0}}}]"#,
            ),
            (
                r#"{"case":"G8","edge":8,"face":4,"wire":0,"occurrence":3,"edge_orientation":false,"face_orientation":true,"vertices":[5,1],"input_sha256":"ded999b86b0bc1ea1cb25e10f9c5f0b01eb7500379ed6f181fa6830dc102d576"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.784406903277099]},"transform":{"x":{"x":-4.788403000283334e-16,"y":-5.099019513592787,"z":2.775557561562892e-16,"w":0.0},"y":{"x":-5.000000000000001,"y":4.883232067478193e-16,"z":-2.6580976310493685e-32,"w":0.0},"z":{"x":0.0,"y":-5.443316218272764e-17,"z":-1.0,"w":0.0},"w":{"x":6.160353002686564,"y":5.099019513592786,"z":-4.440892098500627e-16,"w":1.0}},"orientation":true}},[{"x":6.160353002686563,"y":0.0,"z":0.0},{"x":1.273993597583394,"y":6.179959583497775,"z":-4.440892098500626e-16}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
            (
                r#"{"case":"G8","edge":10,"face":4,"wire":0,"occurrence":1,"edge_orientation":true,"face_orientation":true,"vertices":[7,2],"input_sha256":"ded999b86b0bc1ea1cb25e10f9c5f0b01eb7500379ed6f181fa6830dc102d576"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,0.4110061363320847]},"transform":{"x":{"x":-4.902903378454593,"y":-1.000000000000001,"z":5.4433162182727695e-17,"w":0.0},"y":{"x":-0.9805806756909209,"y":4.999999999999994,"z":-2.7216581091363785e-16,"w":0.0},"z":{"x":0.0,"y":-5.443316218272764e-17,"z":-1.0,"w":0.0},"w":{"x":6.160353002686556,"y":25.099019513592786,"z":100.0,"w":1.0}},"orientation":true}},[{"x":1.2574496242319624,"y":24.099019513592786,"z":100.0},{"x":1.273993597583394,"y":26.179959583497777,"z":99.99999999999999}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
            (
                r#"{"case":"G8","edge":11,"face":4,"wire":0,"occurrence":0,"edge_orientation":true,"face_orientation":true,"vertices":[6,7],"input_sha256":"ded999b86b0bc1ea1cb25e10f9c5f0b01eb7500379ed6f181fa6830dc102d576"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.3734007669450161]},"transform":{"x":{"x":1.4486352951323498e-15,"y":-4.9029033784546,"z":0.9805806756909189,"w":0.0},"y":{"x":-4.999999999999999,"y":0.9805806756909186,"z":4.9029033784545994,"w":0.0},"z":{"x":-0.7071067811865476,"y":-0.13867504905630734,"z":-0.6933752452815365,"w":0.0},"w":{"x":6.160353002686561,"y":24.099019513592786,"z":95.0,"w":1.0}},"orientation":true}},[{"x":6.160353002686563,"y":19.196116135138187,"z":95.98058067569092},{"x":1.2574496242319624,"y":24.099019513592786,"z":100.0}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.7844069032771008]},"transform":{"x":{"x":1.7763568394002505e-15,"y":-4.9029033784546,"z":0.9805806756909203,"w":0.0},"y":{"x":-5.000000000000001,"y":-1.7268358604088909e-15,"z":4.2349888516142927e-16,"w":0.0},"z":{"x":-1.5322889600906654e-17,"y":-0.19611613513818413,"z":-0.9805806756909203,"w":0.0},"w":{"x":6.160353002686561,"y":4.9029033784546,"z":-0.9805806756909203,"w":1.0}},"orientation":false}},"vector":{"x":0.0,"y":20.237690753211453,"z":101.18845376605725}}}]"#,
            ),
            (
                r#"{"case":"G8","edge":11,"face":5,"wire":0,"occurrence":2,"edge_orientation":false,"face_orientation":true,"vertices":[6,7],"input_sha256":"ded999b86b0bc1ea1cb25e10f9c5f0b01eb7500379ed6f181fa6830dc102d576"}"#,
                r#"[{"Conic":{"entity":{"curve":null,"range":[0.0,1.3734007669450161]},"transform":{"x":{"x":1.4486352951323498e-15,"y":-4.9029033784546,"z":0.9805806756909189,"w":0.0},"y":{"x":-4.999999999999999,"y":0.9805806756909186,"z":4.9029033784545994,"w":0.0},"z":{"x":-0.7071067811865476,"y":-0.13867504905630734,"z":-0.6933752452815365,"w":0.0},"w":{"x":6.160353002686561,"y":24.099019513592786,"z":95.0,"w":1.0}},"orientation":true}},[{"x":6.160353002686563,"y":19.196116135138187,"z":95.98058067569092},{"x":1.2574496242319624,"y":24.099019513592786,"z":100.0}],{"Extruded":{"curve":{"Conic":{"entity":{"curve":null,"range":[0.0,1.373400766945024]},"transform":{"x":{"x":0.0,"y":-4.902903378454582,"z":0.9805806756908879,"w":0.0},"y":{"x":0.0,"y":0.9805806756908879,"z":4.902903378454582,"w":0.0},"z":{"x":-1.0,"y":0.0,"z":-0.0,"w":0.0},"w":{"x":1.1603530026865627,"y":24.099019513592765,"z":95.00000000000003,"w":1.0}},"orientation":false}},"vector":{"x":98.83964699731344,"y":0.0,"z":0.0}}}]"#,
            ),
        ];
        for (index, (binding, data)) in fixtures.into_iter().enumerate() {
            println!("ASSOCIATION {} {}", index, binding);
            let case = "all-sides";
            let (curve, vertices, surface): (
                truck_modeling::Curve,
                [Point3; 2],
                truck_modeling::Surface,
            ) = serde_json::from_str(data).unwrap();
            let ellipse = curve.step_ellipse().unwrap();
            let vertices = vertices.map(|p| Point3::from(p.step_point_coordinates().unwrap()));
            let vertices = if curve.same_sense() {
                vertices
            } else {
                [vertices[1], vertices[0]]
            };
            println!(
                "ORIGINAL {case} range={:?} precanonical={:?} shift={:?}",
                ellipse.range, ellipse.trim_parameters, ellipse.trim_shift
            );
            let trim = ellipse.bounded_trim(vertices).unwrap();
            let cylinder = surface.step_cylinder().unwrap();
            let candidate =
                EllipsePcurve::candidate_on_domain(&ellipse, &cylinder, trim.domain).unwrap();
            println!(
                "ORIGINAL bound={:?} mapping={:?} trim={:?}",
                candidate.stored, ellipse.source_mapping_error, trim
            );
            let ledger = EllipsePositionLedger::compute(
                &candidate,
                &ellipse,
                &cylinder,
                trim.domain,
                trim.source_trim_error,
            )
            .unwrap();
            for invalid_charge in [-1.0, f64::NAN, f64::INFINITY, 5e-8] {
                assert!(EllipsePositionLedger::compute(
                    &candidate,
                    &ellipse,
                    &cylinder,
                    trim.domain,
                    invalid_charge
                )
                .is_none());
            }
            assert_eq!(
                EllipsePcurve::certified_on_domain(
                    &ellipse,
                    &cylinder,
                    trim.domain,
                    trim.source_trim_error
                )
                .is_some(),
                ellipse_runtime::eligible()
            );
            println!(
                "TYPED {} {:?} {:?} {:?} {:?} {:?} {:?}",
                index,
                ledger.evaluation.error,
                ledger.evaluation.hull.map(|h| [h.lo, h.hi]),
                ledger.stored,
                ledger.cylinder,
                ledger.ellipse,
                ledger.total
            );
            let co = [cylinder.origin.x, cylinder.origin.y, cylinder.origin.z];
            let o = [ellipse.center.x, ellipse.center.y, ellipse.center.z];
            let radius = cylinder.radius;
            let radii = ellipse.reader_radii;
            let norms = cylinder.reader_frame.map(|axis| axis.map(|q| q.norm()));
            let en = ellipse.reader_frame.map(|axis| axis.map(|q| q.norm()));
            println!("LEDGER {{\"case\":\"{}\",\"index\":{},\"domain\":{:?},\"residual\":{:?},\"stored\":{:?},\"origin\":{:?},\"frame\":{:?},\"radius\":{:?},\"ellipse_origin\":{:?},\"ellipse_frame\":{:?},\"radii\":{:?}}}",
                case, index, [trim.domain.0,trim.domain.1], (EllipseInterval::point(candidate.residual) + EllipseInterval::point(trim.source_trim_error)).hi,
                [candidate.stored.interpolation,candidate.stored.control_u,candidate.stored.control_v], co,norms,radius,o,en,radii);
            println!(
                "BEGIN_CURVE {}\n{}END_CURVE",
                index,
                StepDataDisplay::new(&candidate.curve, 1)
            );
        }
    }
}
