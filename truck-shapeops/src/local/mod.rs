//! Local operations: editing a solid face by face, leaving the rest untouched.

mod delete;
mod draft;
mod intersect;
mod replace;
mod shell;
pub use delete::{delete_face, try_delete_face};
pub use draft::{draft, try_draft};
pub use intersect::{intersect_surfaces, parameter_domain, try_intersect_surfaces, Domain};
pub use replace::{replace_surfaces, try_replace_surfaces};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub use shell::{shell, thicken, try_shell, try_thicken};
use std::{fmt, result::Result};
use truck_base::diagnostics::{Code, Diagnostic};
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::*;

/// Why a local operation could not be done. The face ids are those of the input solid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalOpError<S> {
    /// `face` is not a face of the solid
    UnknownFace {
        /// the id given
        face: FaceID<S>,
    },
    /// the operation on `face` needs its boundary re-intersected with its neighbours, which
    /// this operation does not do
    Unsupported {
        /// the face
        face: FaceID<S>,
    },
    /// the moved boundary of `face` leaves the region of `neighbour`, or crosses its other loops
    OutsideNeighbour {
        /// the moved face
        face: FaceID<S>,
        /// the neighbour it leaves
        neighbour: FaceID<S>,
    },
    /// the new surface of `face` does not meet `neighbour`, or the edges between them meet no
    /// vertex
    NoIntersection {
        /// the replaced face
        face: FaceID<S>,
        /// the neighbour
        neighbour: FaceID<S>,
    },
    /// the surface of `face` has no offset of its own kind by the distance asked
    NoOffset {
        /// the face
        face: FaceID<S>,
    },
    /// the edge between `face` and `neighbour` is concave, which an offset cannot cross yet
    Concave {
        /// one face at the edge
        face: FaceID<S>,
        /// the other
        neighbour: FaceID<S>,
    },
    /// a shell or thickening asked for outward, or with no thickness
    NotInward,
}

impl<S> fmt::Display for LocalOpError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFace { face } => write!(f, "{face:?} is not a face of the solid"),
            Self::Unsupported { face } => write!(
                f,
                "the operation does not support the geometry or boundary reconstruction at {face:?}"
            ),
            Self::OutsideNeighbour { face, neighbour } => {
                write!(f, "the moved boundary of {face:?} leaves {neighbour:?}")
            }
            Self::NoIntersection { face, neighbour } => {
                write!(f, "the new surface of {face:?} does not meet {neighbour:?}")
            }
            Self::NoOffset { face } => {
                write!(
                    f,
                    "the surface of {face:?} has no offset of its own kind by that distance"
                )
            }
            Self::Concave { face, neighbour } => {
                write!(f, "the edge between {face:?} and {neighbour:?} is concave")
            }
            Self::NotInward => write!(f, "the thickness must be positive, going inward"),
        }
    }
}

impl<S: fmt::Debug> std::error::Error for LocalOpError<S> {}

/// Points along `curve`, in parameter order.
const SAMPLES: usize = 16;

/// Moves `faces` of `solid` by `translation` and returns the moved solid; `solid` is not
/// modified. When the faces can move as a whole, every loop of a neighbouring face that touches
/// them consisting of moved edges only, being an inner loop of that neighbour, and after the
/// move still lying on the neighbour's surface inside its region and clear of its other loops,
/// they move rigidly with their edges and vertices: that is a hole through a plate moved within
/// the plate, and it is exact. Otherwise the faces' surfaces are translated and
/// [`replace_surfaces`] re-intersects them with their neighbours, as for the top of a box moved
/// up, with that operation's limits and errors.
/// # Examples
/// ```
/// use truck_modeling::*;
/// use truck_shapeops::local::move_faces;
/// // a plate with a round hole, the hole's three faces moved along the plate
/// let outer: Wire = {
///     let v: Vec<Vertex> = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]
///         .iter()
///         .map(|&(x, y)| builder::vertex(Point3::new(x, y, 0.0)))
///         .collect();
///     (0..4).map(|i| builder::line(&v[i], &v[(i + 1) % 4])).collect()
/// };
/// let center = Point3::new(3.0, 3.0, 0.0);
/// let vertex = builder::vertex(center + Vector3::unit_x());
/// let hole: Wire = builder::rsweep(&vertex, center, -Vector3::unit_z(), Rad(7.0), 3);
/// let face: Face = builder::try_attach_plane(&[outer, hole]).unwrap();
/// let plate: Solid = builder::tsweep(&face, Vector3::unit_z());
/// let faces: Vec<_> = plate
///     .face_iter()
///     .filter(|face| matches!(face.surface(), Surface::Extruded(_)))
///     .map(|face| face.id())
///     .collect();
/// let moved = move_faces(&plate, &faces, Vector3::new(5.0, 0.0, 0.0)).unwrap();
/// assert_eq!(moved.face_iter().count(), plate.face_iter().count());
/// assert!(move_faces(&plate, &faces, Vector3::new(8.0, 0.0, 0.0)).is_err());
/// ```
pub fn move_faces(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    translation: Vector3,
) -> Result<truck_modeling::Solid, LocalOpError<truck_modeling::Surface>> {
    move_faces_impl(solid, faces, translation).map_err(|e| e.legacy)
}

/// Diagnostic variant of [`move_faces`], with input-relative locations and retained causes.
pub fn try_move_faces(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    translation: Vector3,
) -> Result<truck_modeling::Solid, Diagnostic> {
    validate_solid(solid, "move_faces")?;
    if !translation.x.is_finite() || !translation.y.is_finite() || !translation.z.is_finite() {
        return Err(
            Diagnostic::new(Code::InvalidParameter, "move_faces", "validate_input")
                .parameter("translation", format!("{translation:?}")),
        );
    }
    move_faces_impl(solid, faces, translation)
        .map_err(|e| locate_failure(solid.face_iter(), faces, "move_faces", e))
}

pub(super) fn move_faces_impl(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    translation: Vector3,
) -> Result<truck_modeling::Solid, Failure> {
    match move_faces_rigidly(solid, faces, translation) {
        Err(LocalOpError::Unsupported { .. }) => {
            let matrix = Matrix4::from_translation(translation);
            let replacements = faces
                .iter()
                .map(|&id| {
                    let face = solid
                        .face_iter()
                        .find(|face| face.id() == id)
                        .ok_or(LocalOpError::UnknownFace { face: id })?;
                    Ok((id, face.surface().transformed(matrix)))
                })
                .collect::<Result<Vec<_>, Failure>>()?;
            replace::replace_surfaces_impl(solid, &replacements)
        }
        other => other.map_err(Failure::from),
    }
}

/// Offsets `faces` of `solid` by `distance` along their outward normals, through
/// [`Surface::offset`](truck_modeling::Surface::offset) and [`replace_surfaces`]; a positive
/// distance moves a face outward, so a boss grows and a hole shrinks. A face whose surface has
/// no offset of its kind by that distance is [`LocalOpError::NoOffset`].
pub fn offset_faces(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    distance: f64,
) -> Result<truck_modeling::Solid, LocalOpError<truck_modeling::Surface>> {
    offset_faces_impl(solid, faces, distance).map_err(|e| e.legacy)
}

/// Diagnostic variant of [`offset_faces`], with input-relative locations and retained causes.
pub fn try_offset_faces(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    distance: f64,
) -> Result<truck_modeling::Solid, Diagnostic> {
    validate_solid(solid, "offset_faces")?;
    if !distance.is_finite() {
        return Err(
            Diagnostic::new(Code::InvalidParameter, "offset_faces", "validate_input")
                .parameter("distance", distance),
        );
    }
    offset_faces_impl(solid, faces, distance)
        .map_err(|e| locate_failure(solid.face_iter(), faces, "offset_faces", e))
}

pub(super) fn offset_faces_impl(
    solid: &truck_modeling::Solid,
    faces: &[truck_modeling::FaceID],
    distance: f64,
) -> Result<truck_modeling::Solid, Failure> {
    let replacements = faces
        .iter()
        .map(|&id| {
            let face = solid
                .face_iter()
                .find(|face| face.id() == id)
                .ok_or(LocalOpError::UnknownFace { face: id })?;
            let surface = face.oriented_surface().offset(distance).map_err(|e| {
                Failure::new(
                    LocalOpError::NoOffset { face: id },
                    Code::NoOffset,
                    "offset_surface",
                )
                .source(e)
            })?;
            Ok((id, surface))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    replace::replace_surfaces_impl(solid, &replacements)
}

/// The rigid move of [`move_faces`].
fn move_faces_rigidly<C, S>(
    solid: &Solid<Point3, C, S>,
    faces: &[FaceID<S>],
    translation: Vector3,
) -> Result<Solid<Point3, C, S>, LocalOpError<S>>
where
    C: ParametricCurve3D + BoundedCurve + Transformed<Matrix4> + Invertible + Clone,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + Transformed<Matrix4>
        + Invertible
        + Clone,
{
    // the solid is copied, and faces are matched between the two by position
    let copy = solid.mapped(|p| *p, |c| c.clone(), |s| s.clone());
    let mut original_id = HashMap::default();
    let mut moving = HashSet::default();
    for (shell, shell_copy) in solid.boundaries().iter().zip(copy.boundaries()) {
        for (face, face_copy) in shell.iter().zip(shell_copy.iter()) {
            original_id.insert(face_copy.id(), face.id());
            if faces.contains(&face.id()) {
                moving.insert(face_copy.id());
            }
        }
    }
    if let Some(&face) = faces
        .iter()
        .find(|id| !original_id.values().any(|v| v == *id))
    {
        return Err(LocalOpError::UnknownFace { face });
    }
    let moving_edges: HashSet<EdgeID<C>> = copy
        .face_iter()
        .filter(|face| moving.contains(&face.id()))
        .flat_map(|face| face.edge_iter().map(|edge| edge.id()))
        .collect();
    let matrix = Matrix4::from_translation(translation);

    for neighbour in copy.face_iter().filter(|face| !moving.contains(&face.id())) {
        let surface = neighbour.oriented_surface();
        let loops = neighbour.boundaries();
        let touched: Vec<bool> = loops
            .iter()
            .map(|wire| wire.iter().any(|edge| moving_edges.contains(&edge.id())))
            .collect();
        if !touched.iter().any(|&t| t) {
            continue;
        }
        let moved_face = |wire: &Wire<Point3, C>| {
            let edge = wire
                .iter()
                .find(|e| moving_edges.contains(&e.id()))
                .unwrap();
            let face = copy
                .face_iter()
                .find(|f| moving.contains(&f.id()) && f.edge_iter().any(|e| e.id() == edge.id()))
                .unwrap();
            original_id[&face.id()]
        };
        for (wire, &is_touched) in loops.iter().zip(&touched) {
            if !is_touched {
                continue;
            }
            let face = moved_face(wire);
            let unsupported = || LocalOpError::Unsupported { face };
            if !wire.iter().all(|edge| moving_edges.contains(&edge.id())) {
                return Err(unsupported());
            }
            let moved = parameter_polyline(&surface, wire, Some(matrix)).ok_or_else(unsupported)?;
            let fixed: Vec<PolylineCurve<Point2>> = loops
                .iter()
                .zip(&touched)
                .filter(|(_, &t)| !t)
                .map(|(wire, _)| parameter_polyline(&surface, wire, None).ok_or_else(unsupported))
                .collect::<Result<_, _>>()?;
            let outside = LocalOpError::OutsideNeighbour {
                face,
                neighbour: original_id[&neighbour.id()],
            };
            if signed_area(&moved) >= 0.0 {
                return Err(unsupported());
            }
            let inside = moved.iter().all(|&p| polyline_curve::include(&fixed, p));
            if !inside || fixed.iter().any(|other| crosses(&moved, other)) {
                return Err(outside);
            }
        }
    }

    // shared edges and vertices come up once per face; each is moved once
    let mut done_edges = HashSet::default();
    let mut done_vertices = HashSet::default();
    for face in copy.face_iter().filter(|face| moving.contains(&face.id())) {
        face.set_surface(face.surface().transformed(matrix));
        for edge in face.edge_iter() {
            if !done_edges.insert(edge.id()) {
                continue;
            }
            edge.set_curve(edge.curve().transformed(matrix));
            for vertex in [edge.front(), edge.back()] {
                if done_vertices.insert(vertex.id()) {
                    vertex.set_point(vertex.point() + translation);
                }
            }
        }
    }
    Ok(copy)
}

/// The loop `wire`, moved by `matrix` if given, sampled in the parameter space of `surface`;
/// `None` when a sample is not on the surface.
fn parameter_polyline<C, S>(
    surface: &S,
    wire: &Wire<Point3, C>,
    matrix: Option<Matrix4>,
) -> Option<PolylineCurve<Point2>>
where
    C: ParametricCurve3D + BoundedCurve + Invertible + Clone,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    let mut polyline = Vec::new();
    let mut hint = None;
    for edge in wire.iter() {
        let curve = edge.oriented_curve();
        let (t0, t1) = curve.range_tuple();
        for i in 0..SAMPLES {
            let mut p = curve.subs(t0 + (t1 - t0) * i as f64 / SAMPLES as f64);
            if let Some(matrix) = matrix {
                p = matrix.transform_point(p);
            }
            let (u, v) = surface.search_parameter(p, hint, 100)?;
            if !surface.subs(u, v).near(&p) {
                return None;
            }
            hint = Some((u, v));
            polyline.push(Point2::new(u, v));
        }
    }
    Some(PolylineCurve(polyline))
}

fn signed_area(polyline: &PolylineCurve<Point2>) -> f64 {
    let n = polyline.len();
    (0..n)
        .map(|i| {
            let (p, q) = (polyline[i], polyline[(i + 1) % n]);
            p.x * q.y - q.x * p.y
        })
        .sum::<f64>()
        / 2.0
}

/// Whether a segment of one closed polyline crosses a segment of the other.
fn crosses(a: &PolylineCurve<Point2>, b: &PolylineCurve<Point2>) -> bool {
    let segments = |p: &PolylineCurve<Point2>| {
        (0..p.len())
            .map(|i| (p[i], p[(i + 1) % p.len()]))
            .collect::<Vec<_>>()
    };
    let side =
        |p: Point2, q: Point2, r: Point2| (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x);
    segments(a).iter().any(|&(p0, p1)| {
        segments(b).iter().any(|&(q0, q1)| {
            side(p0, p1, q0) * side(p0, p1, q1) < 0.0 && side(q0, q1, p0) * side(q0, q1, p1) < 0.0
        })
    })
}

impl<S: fmt::Debug> truck_base::diagnostics::CodedError for LocalOpError<S> {
    fn code(&self) -> &'static str { local_code(self).as_str() }
}
fn local_code<S>(error: &LocalOpError<S>) -> Code {
    match error {
        LocalOpError::UnknownFace { .. } => Code::UnknownFace,
        LocalOpError::Unsupported { .. } => Code::UnsupportedGeometry,
        LocalOpError::OutsideNeighbour { .. } => Code::OutsideNeighbour,
        LocalOpError::NoIntersection { .. } => Code::NoIntersection,
        LocalOpError::NoOffset { .. } => Code::NoOffset,
        LocalOpError::Concave { .. } => Code::ConcaveEdge,
        LocalOpError::NotInward => Code::InvalidParameter,
    }
}

#[derive(Debug)]
pub(super) struct Failure {
    legacy: LocalOpError<truck_modeling::Surface>,
    diagnostic: Diagnostic,
}
impl Failure {
    fn new(legacy: LocalOpError<truck_modeling::Surface>, code: Code, stage: &'static str) -> Self {
        Self {
            legacy,
            diagnostic: Diagnostic::new(code, "local", stage),
        }
    }
    fn source(mut self, error: impl truck_base::diagnostics::CodedError + 'static) -> Self {
        self.diagnostic = self.diagnostic.with_coded_source(error);
        self
    }
}
impl From<LocalOpError<truck_modeling::Surface>> for Failure {
    fn from(error: LocalOpError<truck_modeling::Surface>) -> Self {
        let code = local_code(&error);
        let stage = match code {
            Code::UnknownFace | Code::InvalidParameter => "validate_input",
            Code::NoOffset => "offset_surface",
            Code::NoIntersection => "intersect_surfaces",
            Code::OutsideNeighbour => "validate_boundary",
            _ => "rebuild_boundaries",
        };
        let mut result = Self::new(error.clone(), code, stage);
        result.diagnostic = result.diagnostic.with_coded_source(error);
        result.diagnostic.message = code.message().into();
        result
    }
}
fn locate_failure<'a>(
    faces: impl Iterator<Item = &'a truck_modeling::Face>,
    selection: &[truck_modeling::FaceID],
    operation: &'static str,
    failure: Failure,
) -> Diagnostic {
    let Failure {
        legacy,
        mut diagnostic,
    } = failure;
    let (face, neighbour) = match &legacy {
        LocalOpError::UnknownFace { face }
        | LocalOpError::Unsupported { face }
        | LocalOpError::NoOffset { face } => (Some(*face), None),
        LocalOpError::OutsideNeighbour { face, neighbour }
        | LocalOpError::NoIntersection { face, neighbour }
        | LocalOpError::Concave { face, neighbour } => (Some(*face), Some(*neighbour)),
        LocalOpError::NotInward => (None, None),
    };
    diagnostic.operation = operation;
    diagnostic.context.selection_index = selection.iter().position(|id| Some(*id) == face);
    for (i, f) in faces.enumerate() {
        if Some(f.id()) == face && diagnostic.context.face_index.is_none() {
            diagnostic.context.face_index = Some(i);
        }
        if Some(f.id()) == neighbour {
            diagnostic.context.related_index = Some(i);
        }
    }
    diagnostic
}

fn validate_solid(
    solid: &truck_modeling::Solid,
    operation: &'static str,
) -> Result<(), Diagnostic> {
    truck_modeling::Solid::try_new(solid.boundaries().clone())
        .map(|_| ())
        .map_err(|e| {
            Diagnostic::new(Code::InvalidInputTopology, operation, "validate_input")
                .with_coded_source(e)
        })
}
