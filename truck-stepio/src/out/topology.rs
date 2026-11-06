use super::geometry::{EllipsePcurve, EllipseTrim};
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
    pcurves: Vec<Option<EdgePcurves>>,
    certificate_failed: bool,
}

impl<'a, P, C, S> StepShell<'a, P, C, S>
where
    P: Copy + DisplayByStep + 'static,
    C: StepCurve
        + 'static
        + StepLength
        + ParametricCurve3D
        + BoundedCurve
        + ParameterDivision1D<Point = Point3>,
    S: StepSurface
        + 'static
        + StepLength
        + ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>,
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
        cursor += vertices.len();
        let mut pcurves = vec![None; edges.len()];
        if ellipse_runtime::eligible()
            && std::any::TypeId::of::<P>() == std::any::TypeId::of::<Point3>()
            && std::any::TypeId::of::<C>() == std::any::TypeId::of::<truck_modeling::Curve>()
            && std::any::TypeId::of::<S>() == std::any::TypeId::of::<truck_modeling::Surface>()
        {
            let cylinders: Vec<_> = faces
                .iter()
                .map(|face| face.surface.step_cylinder())
                .collect();
            for (edge_index, edge) in edges.iter().enumerate() {
                let Some(ellipse) = edge.curve.step_ellipse() else {
                    continue;
                };
                if edge.vertices.0 == edge.vertices.1 {
                    continue;
                }
                let (first, last) = if edge.curve.same_sense() {
                    edge.vertices
                } else {
                    (edge.vertices.1, edge.vertices.0)
                };
                let Some(first) = vertices[first].step_point_coordinates() else {
                    continue;
                };
                let Some(last) = vertices[last].step_point_coordinates() else {
                    continue;
                };
                let Some(trim) = ellipse.bounded_trim([Point3::from(first), Point3::from(last)])
                else {
                    continue;
                };
                if trim.source_trim_error >= 5e-8 {
                    continue;
                }
                let incidences: Vec<_> = faces
                    .iter()
                    .enumerate()
                    .filter_map(|(i, face)| {
                        (face
                            .boundaries
                            .iter()
                            .flatten()
                            .any(|edge| edge.index == edge_index))
                        .then_some(i)
                    })
                    .collect();
                if incidences.len() > 2 {
                    continue;
                }
                let mut curves = Vec::new();
                let mut declined = false;
                for face in incidences {
                    let Some(cylinder) = &cylinders[face] else {
                        continue;
                    };
                    match EllipsePcurve::certified_on_domain(
                        &ellipse,
                        cylinder,
                        trim.domain,
                        trim.source_trim_error,
                    ) {
                        Some(curve) => curves.push((surface_indices[face], curve)),
                        None => {
                            declined = true;
                            break;
                        }
                    }
                }
                if !declined && !curves.is_empty() {
                    let graph = EdgePcurves {
                        start: cursor,
                        trim,
                        curves,
                    };
                    cursor += graph.step_length();
                    pcurves[edge_index] = Some(graph);
                }
            }
        }
        let certificate_failed =
            pcurves.iter().any(Option::is_some) && !ellipse_runtime::eligible();
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
            pcurves,
            certificate_failed,
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
    fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
        if self.certificate_failed {
            return ERR;
        }
        if !self.pcurves.iter().any(Option::is_some) {
            return self.fmt_body(idx, f);
        }
        ellipse_runtime::buffered(f, |text| {
            std::fmt::write(text, format_args!("{}", ShellText(self, idx)))
        })
    }
}

struct ShellText<'s, 'a, P, C, S>(&'s StepShell<'a, P, C, S>, usize);
impl<P, C, S> Display for ShellText<'_, '_, P, C, S>
where
    P: DisplayByStep + Copy,
    C: DisplayByStep + StepCurve,
    S: DisplayByStep + StepSurface,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result { self.0.fmt_body(self.1, f) }
}

impl<P, C, S> StepShell<'_, P, C, S>
where
    P: DisplayByStep + Copy,
    C: DisplayByStep + StepCurve,
    S: DisplayByStep + StepSurface,
{
    fn fmt_body(&self, idx: usize, formatter: &mut Formatter<'_>) -> Result {
        let StepShell {
            entity,
            indices,
            is_open,
            pole_bounds,
            pcurves,
            certificate_failed: _,
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
                edge_geometry = pcurves[i].as_ref().map_or(curve_indices[i], |graph| graph.start + idx),
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
        for (edge, graph) in pcurves.iter().enumerate() {
            if let Some(graph) = graph {
                graph.fmt(idx, curve_indices[edge], formatter)?;
            }
        }
        edges.iter().zip(curve_indices).try_for_each(|(e, idx)| {
            Display::fmt(&StepDataDisplay::new(&e.curve, idx), formatter)
        })?;
        vertices.iter().enumerate().try_for_each(|(i, v)| {
            Display::fmt(&StepDataDisplay::new(*v, ep_points + i), formatter)
        })
    }
}

impl<P, C, S> StepLength for StepShell<'_, P, C, S> {
    fn step_length(&self) -> usize {
        self.indices.ep_points
            + self.entity.vertices.len()
            + self
                .pcurves
                .iter()
                .flatten()
                .map(StepLength::step_length)
                .sum::<usize>()
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct StepSolid<'a, P, C, S> {
    boundaries: Vec<StepShell<'a, P, C, S>>,
}

impl<'a, P, C, S> StepSolid<'a, P, C, S>
where
    P: Copy + DisplayByStep + 'static,
    C: StepCurve
        + 'static
        + StepLength
        + ParametricCurve3D
        + BoundedCurve
        + ParameterDivision1D<Point = Point3>,
    S: StepSurface
        + 'static
        + StepLength
        + ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>,
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
    P: Copy + DisplayByStep + 'static,
    C: StepCurve
        + 'static
        + StepLength
        + ParametricCurve3D
        + BoundedCurve
        + ParameterDivision1D<Point = Point3>,
    S: StepSurface
        + 'static
        + StepLength
        + ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn from(shell: &'a CompressedShell<P, C, S>) -> Self {
        Self::Shells(vec![StepShell::new(shell, true)])
    }
}

impl<'a, P, C, S> From<&'a CompressedSolid<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy + DisplayByStep + 'static,
    C: StepCurve
        + 'static
        + StepLength
        + ParametricCurve3D
        + BoundedCurve
        + ParameterDivision1D<Point = Point3>,
    S: StepSurface
        + 'static
        + StepLength
        + ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn from(solid: &'a CompressedSolid<P, C, S>) -> Self { Self::Solid(StepSolid::new(solid)) }
}

impl<'a, P, C, S> FromIterator<&'a CompressedShell<P, C, S>> for StepModel<'a, P, C, S>
where
    P: Copy + DisplayByStep + 'static,
    C: StepCurve
        + 'static
        + StepLength
        + ParametricCurve3D
        + BoundedCurve
        + ParameterDivision1D<Point = Point3>,
    S: StepSurface
        + 'static
        + StepLength
        + ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>,
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

#[derive(Clone, Debug)]
struct EdgePcurves {
    start: usize,
    trim: EllipseTrim,
    curves: Vec<(usize, EllipsePcurve)>,
}
impl StepLength for EdgePcurves {
    fn step_length(&self) -> usize {
        2 + self
            .curves
            .iter()
            .map(|(_, curve)| curve.reference_length())
            .sum::<usize>()
    }
}
impl EdgePcurves {
    fn fmt(&self, offset: usize, spatial: usize, f: &mut Formatter<'_>) -> Result {
        let wrapper = offset + self.start;
        let trimmed = wrapper + 1;
        let mut cursor = wrapper + 2;
        let indices: Vec<_> = self
            .curves
            .iter()
            .map(|(_, curve)| {
                let idx = cursor;
                cursor += curve.reference_length();
                idx
            })
            .collect();
        f.write_fmt(format_args!(
            "#{wrapper} = SURFACE_CURVE('', #{trimmed}, {}, .CURVE_3D.);\n",
            IndexSliceDisplay(indices.iter().copied()),
        ))?;
        f.write_fmt(format_args!(
            "#{trimmed} = TRIMMED_CURVE('', #{spatial}, (PARAMETER_VALUE({})), (PARAMETER_VALUE({})), .T., .PARAMETER.);\n",
            FloatDisplay(self.trim.parameters.0), FloatDisplay(self.trim.parameters.1),
        ))?;
        for ((surface, curve), idx) in self.curves.iter().zip(indices) {
            curve.fmt_reference(idx, offset + surface, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod ellipse_graph_tests {
    use super::*;
    use truck_modeling::{Curve, Surface};

    fn fixture() -> CompressedShell<Point3, Curve, Surface> {
        let conic = Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0.1, 2.0)),
            Matrix4::from_cols(
                Vector4::new(2.0, 0.0, 2.0, 0.0),
                Vector4::new(0.0, 2.0, 0.0, 0.0),
                Vector4::unit_z(),
                Vector4::unit_w(),
            ),
        );
        let circle = Curve::Conic(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0.0, std::f64::consts::TAU)),
            Matrix4::from_scale(2.0),
        ));
        let surface = Surface::Extruded(ExtrudedCurve::by_extrusion(circle, Vector3::unit_z()));
        CompressedShell {
            vertices: vec![conic.subs(0.1), conic.subs(2.0)],
            edges: vec![CompressedEdge {
                vertices: (0, 1),
                curve: Curve::Conic(conic),
            }],
            faces: vec![
                CompressedFace {
                    boundaries: vec![vec![(0, true).into()]],
                    orientation: true,
                    surface: surface.clone(),
                },
                CompressedFace {
                    boundaries: vec![vec![(0, false).into()]],
                    orientation: false,
                    surface,
                },
            ],
        }
    }

    #[test]
    fn staged_graph_rejects_wrong_surface_identity() {
        let shell = fixture();
        let mut model = StepShell::new(&shell, true);
        let ellipse = shell.edges[0].curve.step_ellipse().unwrap();
        let curves = shell
            .faces
            .iter()
            .zip(&model.indices.surface_indices)
            .map(|(face, &id)| {
                let cylinder = face.surface.step_cylinder().unwrap();
                (id, EllipsePcurve::candidate(&ellipse, &cylinder).unwrap())
            })
            .collect();
        // Exercise graph formatting independently of the deliberately closed gate.
        model.pcurves[0] = Some(EdgePcurves {
            start: model.indices.ep_points + shell.vertices.len(),
            trim: ellipse
                .bounded_trim([shell.vertices[0], shell.vertices[1]])
                .unwrap(),
            curves,
        });
        let text = ShellText(&model, 100).to_string();
        let valid = |text: &str| {
            text.matches("SURFACE_CURVE(").count() == 1
                && text.matches("PCURVE(").count() == 2
                && text.matches("CYLINDRICAL_SURFACE(").count() == 2
                && text.contains(".CURVE_3D.")
                && model
                    .indices
                    .surface_indices
                    .iter()
                    .all(|id| text.matches(&format!("PCURVE('', #{},", 100 + id)).count() == 1)
        };
        assert!(valid(&text));
        assert_eq!(text.matches("TRIMMED_CURVE(").count(), 1);
        assert_eq!(text.matches("PARAMETER_VALUE(").count(), 2);
        assert!(text.contains(".T., .PARAMETER."));
        let first = model.indices.surface_indices[0] + 100;
        let second = model.indices.surface_indices[1] + 100;
        assert!(!valid(&text.replace(
            &format!("PCURVE('', #{first},"),
            &format!("PCURVE('', #{second},")
        )));
        assert!(!valid(&text.replace(
            &format!("PCURVE('', #{first},"),
            "PCURVE('', #999999,"
        )));
        assert!(!valid(&text.replace(".CURVE_3D.", ".PCURVE_S1.")));
    }

    #[test]
    fn staged_original_graphs() {
        let Some(output) = std::env::var_os("ELLIPSE_GUARD_OUTPUT") else {
            return;
        };
        assert!(
            ellipse_runtime::eligible(),
            "staged artifact export requires the eligible ellipse feature build"
        );
        for case in ["G4", "G8"] {
            let input = format!(
                "{}/{case}-fresh.step.prepared.json",
                std::path::Path::new(&output).display()
            );
            let solid: CompressedSolid<Point3, Curve, Surface> =
                serde_json::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();
            let mut model = StepSolid::new(&solid);
            assert_eq!(model.boundaries.len(), 1);
            let shell = &solid.boundaries[0];
            let graph = &mut model.boundaries[0];
            let mut cursor = graph.indices.ep_points + shell.vertices.len();
            let mut mapping = Vec::new();
            for (ei, edge) in shell.edges.iter().enumerate() {
                let Some(ellipse) = edge.curve.step_ellipse() else {
                    continue;
                };
                let (a, b) = if edge.curve.same_sense() {
                    edge.vertices
                } else {
                    (edge.vertices.1, edge.vertices.0)
                };
                let trim = ellipse
                    .bounded_trim([shell.vertices[a], shell.vertices[b]])
                    .unwrap();
                let mut curves = Vec::new();
                let mut sides = Vec::new();
                let mut incidences = Vec::new();
                let mut pc = cursor + 2;
                for (fi, face) in shell.faces.iter().enumerate() {
                    for boundary in &face.boundaries {
                        for usage in boundary.iter().filter(|u| u.index == ei) {
                            incidences.push(serde_json::json!({"face":fi,"label":graph.indices.face_indices[fi],"orientation":usage.orientation,"face_orientation":face.orientation,"surface_sense":face.surface.same_sense()}));
                            if let Some(cylinder) = face.surface.step_cylinder() {
                                println!("STAGED_SIDE {case} {ei} {fi}");
                                let curve =
                                    EllipsePcurve::staged(&ellipse, &cylinder, &trim).unwrap();
                                sides.push(serde_json::json!({"face":fi,"label":pc,"face_label":graph.indices.face_indices[fi]}));
                                pc += curve.reference_length();
                                curves.push((graph.indices.surface_indices[fi], curve));
                            }
                        }
                    }
                }
                if curves.is_empty() {
                    continue;
                }
                mapping.push(serde_json::json!({"edge":ei,"label":graph.indices.ep_edges+ei,"same_sense":edge.curve.same_sense(),"vertices":[graph.indices.ep_vertices+a,graph.indices.ep_vertices+b],"points":[shell.vertices[a],shell.vertices[b]],"domain":[trim.domain.0,trim.domain.1],"incidences":incidences,"sides":sides}));
                let pcurves = EdgePcurves {
                    start: cursor,
                    trim,
                    curves,
                };
                cursor += pcurves.step_length();
                graph.pcurves[ei] = Some(pcurves);
            }
            let design = StepDesign::from_model(StepModel::Solid(model));
            let header = StepHeaderDescriptor {
                time_stamp: "2026-11-05T00:00:00".into(),
                ..Default::default()
            };
            let text = StepDisplay::new(header, design).to_string();
            let output = std::path::Path::new(&output);
            std::fs::write(output.join(format!("{case}.step")), text).unwrap();
            std::fs::write(
                output.join(format!("{case}-map.json")),
                serde_json::to_string_pretty(&mapping).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn production_original_graphs() {
        let Some(output) = std::env::var_os("ELLIPSE_PRODUCTION_OUTPUT") else {
            return;
        };
        assert!(
            ellipse_runtime::eligible(),
            "staged artifact export requires the eligible ellipse feature build"
        );
        for case in ["G4", "G8"] {
            let input = format!(
                "{}/{case}-fresh.step.prepared.json",
                std::path::Path::new(&output).display()
            );
            let solid: CompressedSolid<Point3, Curve, Surface> =
                serde_json::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();
            let model = StepSolid::new(&solid);
            assert_eq!(model.boundaries.len(), 1);
            let shell = &solid.boundaries[0];
            let graph = &model.boundaries[0];

            let mut mapping = Vec::new();
            for (ei, edge) in shell.edges.iter().enumerate() {
                let Some(ellipse) = edge.curve.step_ellipse() else {
                    continue;
                };
                let (a, b) = if edge.curve.same_sense() {
                    edge.vertices
                } else {
                    (edge.vertices.1, edge.vertices.0)
                };
                let trim = ellipse
                    .bounded_trim([shell.vertices[a], shell.vertices[b]])
                    .unwrap();
                let Some(actual) = &graph.pcurves[ei] else {
                    continue;
                };
                let cursor = actual.start;
                let mut curve_count = 0;
                let mut sides = Vec::new();
                let mut incidences = Vec::new();
                let mut pc = cursor + 2;
                for (fi, face) in shell.faces.iter().enumerate() {
                    for boundary in &face.boundaries {
                        for usage in boundary.iter().filter(|u| u.index == ei) {
                            incidences.push(serde_json::json!({"face":fi,"label":graph.indices.face_indices[fi],"orientation":usage.orientation,"face_orientation":face.orientation,"surface_sense":face.surface.same_sense()}));
                            if face.surface.step_cylinder().is_some() {
                                println!("STAGED_SIDE {case} {ei} {fi}");
                                let curve = &actual.curves[curve_count].1;
                                sides.push(serde_json::json!({"face":fi,"label":pc,"face_label":graph.indices.face_indices[fi]}));
                                pc += curve.reference_length();
                                assert_eq!(
                                    actual.curves[curve_count].0,
                                    graph.indices.surface_indices[fi]
                                );
                                curve_count += 1;
                            }
                        }
                    }
                }
                if curve_count == 0 {
                    continue;
                }
                mapping.push(serde_json::json!({"edge":ei,"label":graph.indices.ep_edges+ei,"same_sense":edge.curve.same_sense(),"vertices":[graph.indices.ep_vertices+a,graph.indices.ep_vertices+b],"points":[shell.vertices[a],shell.vertices[b]],"domain":[trim.domain.0,trim.domain.1],"incidences":incidences,"sides":sides}));
                assert_eq!(curve_count, actual.curves.len());
            }
            let design = StepDesign::from_model(StepModel::Solid(model));
            let header = StepHeaderDescriptor {
                time_stamp: "2026-11-05T00:00:00".into(),
                ..Default::default()
            };
            let text = StepDisplay::new(header, design).to_string();
            let output = std::path::Path::new(&output);
            std::fs::write(output.join(format!("{case}.step")), text).unwrap();
            std::fs::write(
                output.join(format!("{case}-map.json")),
                serde_json::to_string_pretty(&mapping).unwrap(),
            )
            .unwrap();
        }
    }

    // Deliberately red until the numerical certificate is closed. This must fail
    // on the incoming writer as well: the original graph has no SURFACE_CURVE.
    #[test]
    fn ellipse_graph_uses_each_existing_face_surface() {
        let shell = fixture();
        let model = StepShell::new(&shell, true);
        let text = StepDataDisplay::new(&model, 100).to_string();
        assert_eq!(text.matches("SURFACE_CURVE(").count(), 1);
        assert_eq!(text.matches("PCURVE(").count(), 2);
        assert_eq!(text.matches("CYLINDRICAL_SURFACE(").count(), 2);
        assert!(text.contains(".CURVE_3D."));
        for surface in model.indices.surface_indices {
            assert!(text.contains(&format!("PCURVE('', #{},", 100 + surface)));
        }
    }

    #[test]
    fn ordinary_no_metadata_graph_keeps_all_entity_references() {
        let source = fixture();
        let shell = CompressedShell {
            vertices: source.vertices.clone(),
            edges: vec![CompressedEdge {
                vertices: (0, 1),
                curve: Line(source.vertices[0], source.vertices[1]),
            }],
            faces: source
                .faces
                .into_iter()
                .map(|face| CompressedFace {
                    boundaries: face.boundaries,
                    orientation: face.orientation,
                    surface: Plane::new(
                        Point3::origin(),
                        Point3::new(1.0, 0.0, 0.0),
                        Point3::new(0.0, 1.0, 0.0),
                    ),
                })
                .collect(),
        };
        assert!(shell.edges[0].curve.step_ellipse().is_none());
        assert!(shell.faces[0].surface.step_cylinder().is_none());
        let model = StepShell::new(&shell, true);
        assert!(model.pcurves.iter().all(Option::is_none));
        let text = StepDataDisplay::new(&model, 100).to_string();
        assert_eq!(text.matches("EDGE_CURVE(").count(), 1);
        assert_eq!(text.matches("PLANE(").count(), 2);
        assert!(!text.contains("PCURVE("));
        let labels: std::collections::HashSet<usize> = text
            .lines()
            .filter_map(|line| {
                line.strip_prefix('#')
                    .and_then(|line| line.split_once(" = "))
                    .map(|(label, _)| label.parse().unwrap())
            })
            .collect();
        assert_eq!(labels.len(), model.step_length());
        for suffix in text.split('#').skip(1) {
            let label: String = suffix.chars().take_while(char::is_ascii_digit).collect();
            assert!(labels.contains(&label.parse::<usize>().unwrap()));
        }
        assert_eq!(StepDataDisplay::new(&model, 100).to_string(), text);
    }

    thread_local! {
        static CUSTOM_METADATA_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    #[derive(Clone, ParametricCurve, BoundedCurve, ParameterDivision1D)]
    struct CustomCurve(Curve);
    impl StepCurve for CustomCurve {
        fn step_ellipse(&self) -> Option<StepEllipse> {
            CUSTOM_METADATA_CALLS.with(|calls| calls.set(calls.get() + 1));
            self.0.step_ellipse()
        }
    }
    impl StepLength for CustomCurve {
        fn step_length(&self) -> usize { self.0.step_length() }
    }
    impl DisplayByStep for CustomCurve {
        fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
            DisplayByStep::fmt(
                &self
                    .0
                    .transformed(Matrix4::from_translation(Vector3::unit_x())),
                idx,
                f,
            )
        }
    }
    #[derive(Clone, ParametricSurface, SearchNearestParameterD2)]
    struct CustomSurface(Surface);
    impl StepSurface for CustomSurface {
        fn step_cylinder(&self) -> Option<StepCylinder> {
            CUSTOM_METADATA_CALLS.with(|calls| calls.set(calls.get() + 1));
            self.0.step_cylinder()
        }
    }
    impl StepLength for CustomSurface {
        fn step_length(&self) -> usize { self.0.step_length() }
    }
    impl DisplayByStep for CustomSurface {
        fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
            DisplayByStep::fmt(
                &self
                    .0
                    .transformed(Matrix4::from_translation(Vector3::unit_x())),
                idx,
                f,
            )
        }
    }
    impl ParametricSurface3D for CustomSurface {
        fn normal(&self, u: f64, v: f64) -> Vector3 { self.0.normal(u, v) }
    }
    #[test]
    fn custom_curve_and_surface_metadata_is_not_called() {
        CUSTOM_METADATA_CALLS.with(|calls| calls.set(0));
        let source = fixture();
        assert_ne!(
            StepDataDisplay::new(&CustomCurve(source.edges[0].curve.clone()), 1).to_string(),
            StepDataDisplay::new(&source.edges[0].curve, 1).to_string()
        );
        assert_ne!(
            StepDataDisplay::new(&CustomSurface(source.faces[0].surface.clone()), 1).to_string(),
            StepDataDisplay::new(&source.faces[0].surface, 1).to_string()
        );
        let curves = CompressedShell {
            vertices: source.vertices.clone(),
            edges: source
                .edges
                .iter()
                .map(|edge| CompressedEdge {
                    vertices: edge.vertices,
                    curve: CustomCurve(edge.curve.clone()),
                })
                .collect(),
            faces: source.faces.clone(),
        };
        let surfaces = CompressedShell {
            vertices: source.vertices.clone(),
            edges: source.edges.clone(),
            faces: source
                .faces
                .iter()
                .map(|face| CompressedFace {
                    boundaries: face.boundaries.clone(),
                    orientation: face.orientation,
                    surface: CustomSurface(face.surface.clone()),
                })
                .collect(),
        };
        let curves = StepShell::new(&curves, true);
        let surfaces = StepShell::new(&surfaces, true);
        CUSTOM_METADATA_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert!(curves.pcurves.iter().all(Option::is_none));
        assert!(surfaces.pcurves.iter().all(Option::is_none));
        for text in [
            StepDataDisplay::new(&curves, 100).to_string(),
            StepDataDisplay::new(&surfaces, 100).to_string(),
        ] {
            assert!(!text.contains("PCURVE("));
            assert_eq!(text.matches("EDGE_CURVE(").count(), 1);
            assert_eq!(text.matches("CYLINDRICAL_SURFACE(").count(), 2);
        }
    }

    #[test]
    fn custom_point_metadata_is_not_called() {
        #[derive(Clone, Copy)]
        struct Custom(Point3);
        impl DisplayByStep for Custom {
            fn step_point_coordinates(&self) -> Option<[f64; 3]> {
                panic!("unverified metadata must not run")
            }
            fn fmt(&self, idx: usize, f: &mut Formatter<'_>) -> Result {
                DisplayByStep::fmt(&self.0, idx, f)
            }
        }
        let source = fixture();
        let shell = CompressedShell {
            vertices: source.vertices.into_iter().map(Custom).collect(),
            edges: source.edges,
            faces: source.faces,
        };
        let model = StepShell::new(&shell, true);
        assert!(model.pcurves.iter().all(Option::is_none));
        assert!(!StepDataDisplay::new(&model, 100)
            .to_string()
            .contains("PCURVE("));
    }

    #[cfg(all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    ))]
    #[test]
    fn delayed_format_checks_controls_and_keeps_indices() {
        let shell = fixture();
        let model = StepShell::new(&shell, true);
        assert!(model.pcurves.iter().any(Option::is_some));
        let count = model.step_length();
        let expected = StepDataDisplay::new(&model, 100).to_string();
        let reject = || {
            ellipse_runtime::with_downward(|| {
                let mut text = String::new();
                let result = std::fmt::write(
                    &mut text,
                    format_args!("{}", StepDataDisplay::new(&model, 100)),
                );
                assert!(result.is_err());
                assert!(text.is_empty());
                assert_eq!(model.step_length(), count);
            })
        };
        reject();
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    assert_eq!(StepDataDisplay::new(&model, 100).to_string(), expected);
                    reject();
                    assert_eq!(StepDataDisplay::new(&model, 100).to_string(), expected);
                })
                .join()
                .unwrap();
        });
        assert_eq!(StepDataDisplay::new(&model, 100).to_string(), expected);
    }

    #[cfg(all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    ))]
    #[test]
    fn ineligible_construction_stays_uncertified_after_restoration() {
        let shell = fixture();
        let model = ellipse_runtime::with_downward(|| StepShell::new(&shell, true));
        assert!(model.pcurves.iter().all(Option::is_none));
        let count = model.step_length();
        let text = StepDataDisplay::new(&model, 100).to_string();
        assert!(!text.contains("PCURVE("));
        assert_eq!(model.step_length(), count);
    }

    #[cfg(all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    ))]
    #[test]
    fn sink_control_change_rejects_the_next_certified_shell() {
        use crate::common::PartAttrs;
        use ellipse_runtime::DownwardGuard;
        use truck_assembly::assy::{Assembly, EdgeEntity, NodeEntity};
        struct Sink {
            text: String,
            changed: Option<DownwardGuard>,
        }
        impl std::fmt::Write for Sink {
            fn write_str(&mut self, value: &str) -> Result {
                self.text.push_str(value);
                if self.changed.is_none() && value.contains("SURFACE_CURVE(") {
                    self.changed = Some(DownwardGuard::enter());
                }
                Ok(())
            }
        }
        fn check(display: impl Display) {
            let expected = display.to_string();
            assert_eq!(expected.matches("SURFACE_CURVE(").count(), 2);
            let labels: Vec<_> = expected
                .lines()
                .filter_map(|line| {
                    line.strip_prefix('#')
                        .and_then(|line| line.split_once(" = "))
                        .map(|p| p.0)
                })
                .collect();
            let unique: std::collections::HashSet<_> = labels.iter().copied().collect();
            assert_eq!(labels.len(), unique.len());
            let mut sink = Sink {
                text: String::new(),
                changed: None,
            };
            let result = std::fmt::write(&mut sink, format_args!("{display}"));
            assert!(result.is_err());
            assert!(sink.changed.is_some());
            assert_eq!(sink.text.matches("SURFACE_CURVE(").count(), 1);
            drop(sink);
            assert_eq!(display.to_string(), expected);
            struct Reject;
            impl std::fmt::Write for Reject {
                fn write_str(&mut self, _: &str) -> Result { Err(std::fmt::Error) }
            }
            assert!(std::fmt::write(&mut Reject, format_args!("{display}")).is_err());
        }
        let shells = [fixture(), fixture()];
        let model: StepModel<'_, Point3, Curve, Surface> = shells.iter().collect();
        let count = model.step_length();
        check(StepDataDisplay::new(&model, 100));
        assert_eq!(model.step_length(), count);
        let solid = CompressedSolid {
            boundaries: shells.to_vec(),
        };
        let model = StepModel::from(&solid);
        let count = model.step_length();
        check(StepDataDisplay::new(&model, 100));
        assert_eq!(model.step_length(), count);
        let attrs = PartAttrs {
            id: "test".into(),
            name: "test".into(),
            description: String::new(),
        };
        let mut assembly = Assembly::new();
        let root = assembly.create_node(NodeEntity {
            shape: None,
            attrs: attrs.clone(),
        });
        for shell in &shells {
            let part = assembly.create_node(NodeEntity {
                shape: Some(StepModel::from(shell)),
                attrs: attrs.clone(),
            });
            assembly.create_edge(
                root,
                part,
                EdgeEntity {
                    matrix: Matrix4::identity(),
                    attrs: attrs.clone(),
                },
            );
        }
        let header = StepHeaderDescriptor {
            time_stamp: "2026-11-05T00:00:00".into(),
            ..Default::default()
        };
        check(StepDisplay::new(header, StepDesign::new(assembly)));
    }
}
