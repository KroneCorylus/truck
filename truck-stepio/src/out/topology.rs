use super::{Result, *};
use truck_geometry::prelude::*;
use truck_topology::compress::*;

fn enclosed_pole<C, S>(face: &CompressedFace<S>, edges: &[CompressedEdge<C>]) -> Option<Point3>
where
    C: ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
    if face.boundaries.len() != 1 {
        return None;
    }
    let (u, v) = face.surface.try_range_tuple();
    let ranges = [u?, v?];
    let periods = [face.surface.u_period(), face.surface.v_period()];
    for axis in 0..2 {
        let Some(period) = periods[axis].filter(|p| p.is_finite() && *p > 0.) else {
            continue;
        };
        let poles = [ranges[1 - axis].0, ranges[1 - axis].1].map(|end| {
            let mut uv = [ranges[0].0, ranges[1].0];
            uv[1 - axis] = end;
            let pole = face.surface.subs(uv[0], uv[1]);
            (0..4)
                .all(|i| {
                    uv[axis] = ranges[axis].0 + i as f64 * period / 4.;
                    face.surface.subs(uv[0], uv[1]).near(&pole)
                })
                .then_some(pole)
        });
        if poles.iter().all(Option::is_none) {
            continue;
        }
        let mut previous: Option<(f64, f64)> = None;
        let mut winding = 0.;
        let mut touched = [false; 2];
        for edge in &face.boundaries[0] {
            let curve = &edges.get(edge.index)?.curve;
            let (_, mut points) = curve.parameter_division(curve.range_tuple(), TOLERANCE);
            if !edge.orientation {
                points.reverse();
            }
            for point in points {
                for (pole, touched) in poles.iter().zip(&mut touched) {
                    *touched |= pole.is_some_and(|pole| pole.near(&point));
                }
                let uv = face
                    .surface
                    .search_nearest_parameter(point, previous, 50)
                    .or_else(|| face.surface.search_nearest_parameter(point, None, 50))?;
                if !uv.0.is_finite() || !uv.1.is_finite() {
                    return None;
                }
                if let Some(previous) = previous {
                    let step = [uv.0 - previous.0, uv.1 - previous.1][axis];
                    winding += step - period * (step / period).round();
                }
                previous = Some(uv);
            }
        }
        if (winding.abs() - period).abs() <= TOLERANCE {
            // The left side of the oriented loop closes at the corresponding chart pole.
            let end = usize::from((axis == 0) == (winding > 0.));
            if !touched[end] {
                return poles[end];
            }
        }
    }
    None
}

#[derive(Clone, Debug)]
struct StepShellIndices {
    face_indices: Vec<usize>,
    ep_edges: usize,
    ep_vertices: usize,
    surface_indices: Vec<usize>,
    curve_indices: Vec<usize>,
    ep_points: usize,
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct StepShell<'a, P, C, S> {
    entity: &'a CompressedShell<P, C, S>,
    indices: StepShellIndices,
    is_open: bool,
    pole_bounds: Vec<Option<Point3>>,
}

impl<'a, P, C, S> StepShell<'a, P, C, S>
where
    P: Copy,
    C: StepLength + ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: StepLength + ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    fn new(shell: &'a CompressedShell<P, C, S>, is_open: bool) -> Self {
        let faces = &shell.faces;
        let edges = &shell.edges;
        let vertices = &shell.vertices;
        let pole_bounds: Vec<_> = faces
            .iter()
            .map(|face| enclosed_pole(face, edges))
            .collect();
        let mut cursor = 1;
        let face_indices = faces
            .iter()
            .zip(&pole_bounds)
            .map(|(f, pole)| {
                let res = cursor;
                cursor += match f.boundaries.is_empty() {
                    true => 5,
                    false => 1 + f.boundaries.iter().map(|b| 2 + b.len()).sum::<usize>(),
                };
                cursor += 4 * usize::from(pole.is_some());
                res
            })
            .collect::<Vec<_>>();
        let ep_edges = cursor;
        let ep_vertices = ep_edges + edges.len();
        cursor = ep_vertices + vertices.len();
        let surface_indices = faces
            .iter()
            .map(|f| {
                let res = cursor;
                cursor += f.surface.step_length();
                res
            })
            .collect::<Vec<_>>();
        let curve_indices = edges
            .iter()
            .map(|e| {
                let res = cursor;
                cursor += e.curve.step_length();
                res
            })
            .collect::<Vec<_>>();
        let ep_points = cursor;
        StepShell {
            entity: shell,
            indices: StepShellIndices {
                face_indices,
                ep_edges,
                ep_vertices,
                surface_indices,
                curve_indices,
                ep_points,
            },
            is_open,
            pole_bounds,
        }
    }
}

impl StepShellIndices {
    fn shift(mut self, idx: usize) -> Self {
        let StepShellIndices {
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
        } = &mut self;
        face_indices.iter_mut().for_each(|i| *i += idx);
        *ep_edges += idx;
        *ep_vertices += idx;
        surface_indices.iter_mut().for_each(|i| *i += idx);
        curve_indices.iter_mut().for_each(|i| *i += idx);
        *ep_points += idx;
        self
    }
}

impl<P, C, S> DisplayByStep for StepShell<'_, P, C, S>
where
    P: DisplayByStep + Copy,
    C: DisplayByStep + StepCurve,
    S: DisplayByStep + StepSurface,
{
    fn fmt(&self, idx: usize, formatter: &mut Formatter<'_>) -> Result {
        let StepShell {
            entity,
            indices,
            is_open,
            pole_bounds,
        } = self;
        let StepShellIndices {
            face_indices,
            ep_edges,
            ep_vertices,
            surface_indices,
            curve_indices,
            ep_points,
        } = indices.clone().shift(idx);
        let faces = &entity.faces;
        let edges = &entity.edges;
        let vertices = &entity.vertices;
        let shell_kind = match is_open {
            true => "OPEN_SHELL",
            false => "CLOSED_SHELL",
        };
        formatter.write_fmt(format_args!(
            "#{idx} = {shell_kind}('', {face_indices});\n",
            face_indices = IndexSliceDisplay(face_indices.iter().copied()),
        ))?;
        faces.iter().enumerate().try_for_each(|(i, f)| {
            let idx = face_indices[i];
            let mut cursor = idx + 1;
            let face_geometry = surface_indices[i];
            let mut face_bounds = match f.boundaries.is_empty() {
                true => vec![cursor],
                false => {
                    let closure = |b: &Vec<CompressedEdgeIndex>| {
                        let res = cursor;
                        cursor += 2 + b.len();
                        res
                    };
                    f.boundaries.iter().map(closure).collect()
                }
            };
            if pole_bounds[i].is_some() {
                face_bounds.push(cursor);
            }
            formatter.write_fmt(format_args!(
                "#{idx} = FACE_SURFACE('', {face_bound}, #{face_geometry}, {same_sense});\n",
                same_sense = BooleanDisplay(f.orientation == f.surface.same_sense()),
                face_bound = IndexSliceDisplay(face_bounds.iter().copied()),
            ))?;
            cursor = idx + 1;
            if f.boundaries.is_empty() {
                let face_bound_idx = cursor;
                let vertex_loop_idx = cursor + 1;
                let vertex_idx = cursor + 2;
                let vertex_geometry = cursor + 3;
                formatter.write_fmt(format_args!(
                    "#{face_bound_idx} = FACE_BOUND('', #{vertex_loop_idx}, .T.);
#{vertex_loop_idx} = VERTEX_LOOP('', #{vertex_idx});
#{vertex_idx} = VERTEX_POINT('', #{vertex_geometry});
#{vertex_geometry} = POINT_ON_SURFACE('', #{face_geometry}, 0.0, 0.0);\n"
                ))?;
            }
            f.boundaries.iter().try_for_each(|b| {
                let face_bound_idx = cursor;
                let edge_loop_idx = cursor + 1;
                let ep_oriented_edges = cursor + 2;
                cursor += 2 + b.len();
                formatter.write_fmt(format_args!(
                    "#{face_bound_idx} = FACE_BOUND('', #{edge_loop_idx}, {orientation});
#{edge_loop_idx} = EDGE_LOOP('', {oriented_edge_indices});\n",
                    orientation = BooleanDisplay(f.orientation),
                    oriented_edge_indices =
                        IndexSliceDisplay(ep_oriented_edges..ep_oriented_edges + b.len()),
                ))?;
                b.iter().enumerate().try_for_each(|(j, ce)| {
                    formatter.write_fmt(format_args!(
                        "#{idx} = ORIENTED_EDGE('', *, *, #{edge_element}, {orientation});\n",
                        idx = ep_oriented_edges + j,
                        edge_element = ep_edges + ce.index,
                        orientation = if ce.orientation { ".T." } else { ".F." },
                    ))
                })
            })?;
            if let Some(pole) = pole_bounds[i] {
                formatter.write_fmt(format_args!(
                    "#{cursor} = FACE_BOUND('', #{vertex_loop}, .T.);\n\
#{vertex_loop} = VERTEX_LOOP('', #{vertex});\n\
#{vertex} = VERTEX_POINT('', #{point});\n",
                    vertex_loop = cursor + 1,
                    vertex = cursor + 2,
                    point = cursor + 3,
                ))?;
                Display::fmt(&StepDataDisplay::new(pole, cursor + 3), formatter)?;
            }
            Ok(())
        })?;
        edges.iter().enumerate().try_for_each(|(i, e)| {
            let same_sense = if e.curve.same_sense() { ".T." } else { ".F." };
            formatter.write_fmt(format_args!(
                "#{idx} = EDGE_CURVE('', #{edge_start}, #{edge_end}, #{edge_geometry}, {same_sense});\n",
                idx = ep_edges + i,
                edge_start = ep_vertices + e.vertices.0,
                edge_end = ep_vertices + e.vertices.1,
                edge_geometry = curve_indices[i],
            ))
        })?;
        (0..vertices.len()).try_for_each(|i| {
            formatter.write_fmt(format_args!(
                "#{idx} = VERTEX_POINT('', #{vertex_geometry});\n",
                idx = ep_vertices + i,
                vertex_geometry = ep_points + i,
            ))
        })?;
        faces.iter().zip(surface_indices).try_for_each(|(f, idx)| {
            Display::fmt(&StepDataDisplay::new(&f.surface, idx), formatter)
        })?;
        edges.iter().zip(curve_indices).try_for_each(|(e, idx)| {
            Display::fmt(&StepDataDisplay::new(&e.curve, idx), formatter)
        })?;
        vertices.iter().enumerate().try_for_each(|(i, v)| {
            Display::fmt(&StepDataDisplay::new(*v, ep_points + i), formatter)
        })
    }
}

impl<P, C, S> StepLength for StepShell<'_, P, C, S> {
    fn step_length(&self) -> usize { self.indices.ep_points + self.entity.vertices.len() }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct StepSolid<'a, P, C, S> {
    boundaries: Vec<StepShell<'a, P, C, S>>,
}

impl<'a, P, C, S> StepSolid<'a, P, C, S>
where
    P: Copy,
    C: StepLength + ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: StepLength + ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    fn new(solid: &'a CompressedSolid<P, C, S>) -> Self {
        let boundaries = solid
            .boundaries
            .iter()
            .map(|shell| StepShell::new(shell, false))
            .collect::<Vec<_>>();
        StepSolid { boundaries }
    }
}

impl<P, C, S> DisplayByStep for StepSolid<'_, P, C, S>
where
    P: DisplayByStep + Copy,
    C: DisplayByStep + StepLength + StepCurve,
    S: DisplayByStep + StepLength + StepSurface,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        let StepSolid { boundaries } = self;
        match boundaries.len() {
            0 => {
                f.pad("empty solid!")?;
                Err(std::fmt::Error)
            }
            1 => {
                let shell_idx = idx + 1;
                let step_shell = &boundaries[0];
                f.write_fmt(format_args!(
                    "#{idx} = MANIFOLD_SOLID_BREP('', #{shell_idx});\n"
                ))?;
                DisplayByStep::fmt(step_shell, shell_idx, f)
            }
            _ => {
                let first_shell_idx = idx + 1;
                let mut cursor = first_shell_idx;
                let other_shells_indices = boundaries[..boundaries.len() - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, step_shell)| {
                        let oriented_shell_length = match i {
                            0 => 0,
                            _ => 1,
                        };
                        cursor += step_shell.step_length() + oriented_shell_length;
                        cursor
                    })
                    .collect::<Vec<usize>>();
                f.write_fmt(format_args!(
                    "#{idx} = BREP_WITH_VOIDS('', #{first_shell_idx}, {});\n",
                    IndexSliceDisplay(other_shells_indices.iter().copied()),
                ))?;
                DisplayByStep::fmt(&boundaries[0], first_shell_idx, f)?;
                boundaries[1..]
                    .iter()
                    .zip(&other_shells_indices)
                    .try_for_each(|(step_shell, oriented_shell_idx)| {
                        let shell_idx = oriented_shell_idx + 1;
                        f.write_fmt(format_args!(
                            "#{oriented_shell_idx} = ORIENTED_CLOSED_SHELL('', *, #{shell_idx}, .T.);\n",
                        ))?;
                        DisplayByStep::fmt(step_shell, shell_idx, f)
                    })
            }
        }
    }
}

impl<P, C, S> StepLength for StepSolid<'_, P, C, S> {
    fn step_length(&self) -> usize {
        let b = &self.boundaries;
        match b.len() {
            0 => 0,
            1 => 1 + b[0].step_length(),
            _ => b.len() + b.iter().map(StepLength::step_length).sum::<usize>(),
        }
    }
}

impl<'a, P, C, S> From<&'a CompressedShell<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength + ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: StepLength + ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    fn from(shell: &'a CompressedShell<P, C, S>) -> Self {
        Self::Shells(vec![StepShell::new(shell, true)])
    }
}

impl<'a, P, C, S> From<&'a CompressedSolid<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength + ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: StepLength + ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    fn from(solid: &'a CompressedSolid<P, C, S>) -> Self { Self::Solid(StepSolid::new(solid)) }
}

impl<'a, P, C, S> FromIterator<&'a CompressedShell<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy,
    C: StepLength + ParametricCurve3D + BoundedCurve + ParameterDivision1D<Point = Point3>,
    S: StepLength + ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    fn from_iter<T: IntoIterator<Item = &'a CompressedShell<P, C, S>>>(iter: T) -> Self {
        Self::Shells(
            iter.into_iter()
                .map(|shell| StepShell::new(shell, true))
                .collect(),
        )
    }
}

impl<P, C, S> DisplayByStep for StepModel<'_, P, C, S>
where
    P: DisplayByStep + Copy,
    C: DisplayByStep + StepLength + StepCurve,
    S: DisplayByStep + StepLength + StepSurface,
{
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Shells(shells) => {
                let mut cursor = idx + 1;
                let shell_indices = shells
                    .iter()
                    .map(|shell| {
                        let res = cursor;
                        cursor += shell.step_length();
                        res
                    })
                    .collect::<Vec<_>>();
                f.write_fmt(format_args!(
                    "#{idx} = SHELL_BASED_SURFACE_MODEL('', {});\n",
                    IndexSliceDisplay(shell_indices.iter().copied()),
                ))?;
                shells
                    .iter()
                    .zip(shell_indices)
                    .try_for_each(|(shell, idx)| DisplayByStep::fmt(shell, idx, f))
            }
            Self::Solid(x) => DisplayByStep::fmt(x, idx, f),
        }
    }
}

impl<P, C, S> StepLength for StepModel<'_, P, C, S> {
    fn step_length(&self) -> usize {
        match self {
            Self::Shells(x) => 1 + x.iter().map(|x| x.step_length()).sum::<usize>(),
            Self::Solid(x) => x.step_length(),
        }
    }
}
