mod components;
pub use components::{solid_components, try_solid_components};
use std::result::Result;
use truck_base::diagnostics::{Code, Diagnostic};

use crate::alternative::Alternative;
use crate::profile::{self, Stage};

use super::*;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::*;

/// Only solids consisting of faces whose surface is implemented this trait can be used for set operations.
pub trait ShapeOpsSurface:
    ParametricSurface3D
    + ParameterDivision2D
    + SearchParameter<D2, Point = Point3>
    + SearchNearestParameter<D2, Point = Point3>
    + Invertible
    + Send
    + Sync {
}
impl<S> ShapeOpsSurface for S where S: ParametricSurface3D
        + ParameterDivision2D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>
        + Invertible
        + Send
        + Sync
{
}

/// Only solids consisting of edges whose curve is implemented this trait can be used for set operations.
pub trait ShapeOpsCurve<S: ShapeOpsSurface>:
    ParametricCurve3D
    + ParameterDivision1D<Point = Point3>
    + Cut
    + Invertible
    + From<IntersectionCurve<BSplineCurve<Point3>, S, S>>
    + SearchParameter<D1, Point = Point3>
    + SearchNearestParameter<D1, Point = Point3>
    + Send
    + Sync {
}
impl<C, S: ShapeOpsSurface> ShapeOpsCurve<S> for C where C: ParametricCurve3D
        + ParameterDivision1D<Point = Point3>
        + Cut
        + Invertible
        + From<IntersectionCurve<BSplineCurve<Point3>, S, S>>
        + SearchParameter<D1, Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + Send
        + Sync
{
}

type AltCurveShell<C, S> =
    Shell<Point3, Alternative<C, IntersectionCurve<PolylineCurve<Point3>, S, S>>, S>;
type AltCurveFace<C, S> =
    Face<Point3, Alternative<C, IntersectionCurve<PolylineCurve<Point3>, S, S>>, S>;

fn altshell_to_shell<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    altshell: &AltCurveShell<C, S>,
) -> Result<Shell<Point3, C, S>, Diagnostic> {
    let mut failure = None;
    let result = altshell.try_mapped(
        |p| Some(*p),
        |c| match c {
            Alternative::FirstType(c) => Some(c.clone()),
            Alternative::SecondType(ic) => {
                let leader = match try_smooth_leader(ic) {
                    Ok(leader) => leader,
                    Err(error) => {
                        failure = Some(error);
                        return None;
                    }
                };
                Some(
                    IntersectionCurve::new(ic.surface0().clone(), ic.surface1().clone(), leader)
                        .into(),
                )
            }
        },
        |s| Some(s.clone()),
    );
    result.ok_or_else(|| {
        failure
            .unwrap_or_else(|| Diagnostic::new(Code::CurveFittingFailed, "boolean", "fit_curves"))
    })
}

/// A cubic B-spline through the exact intersection points at the vertices of the leading
/// polyline of `ic`, parametrized by chord length.
///
/// The final curve projects this leader onto the intersection inside the plane normal to the
/// leader's tangent, so the leader must follow the intersection smoothly: a wiggle of the size
/// of the tolerance makes the projected points zigzag, or the projection fail.
pub(crate) fn try_smooth_leader<S: ShapeOpsSurface>(
    ic: &IntersectionCurve<PolylineCurve<Point3>, S, S>,
) -> Result<BSplineCurve<Point3>, Diagnostic> {
    let (t0, t1) = ic.range_tuple();
    let mut params = vec![t0];
    let mut t = t0.floor() + 1.0;
    while t < t1 && !t.near(&t1) {
        if !t.near(&t0) {
            params.push(t);
        }
        t += 1.0;
    }
    params.push(t1);
    let mut points: Vec<Point3> = Vec::with_capacity(params.len());
    for t in params {
        let (p, _, _) = ic.search_triple(t, 100).ok_or_else(|| {
            let mut e = Diagnostic::new(Code::ProjectionFailed, "boolean", "fit_curves");
            e.context.station = Some(t);
            e
        })?;
        if points.last().is_none_or(|q| !q.near(&p)) {
            points.push(p);
        }
    }
    let n = points.len();
    if n < 2 {
        return Err(Diagnostic::new(
            Code::DegenerateCurve,
            "boolean",
            "fit_curves",
        ));
    }
    let mut chord = vec![0.0];
    for w in points.windows(2) {
        chord.push(chord[chord.len() - 1] + w[0].distance(w[1]));
    }
    let degree = 3.min(n - 1);
    let mut knots = vec![chord[0]; degree + 1];
    knots.extend((1..n - degree).map(|j| chord[j..j + degree].iter().sum::<f64>() / degree as f64));
    knots.extend(vec![chord[n - 1]; degree + 1]);
    let parameter_points: Vec<_> = chord.into_iter().zip(points).collect();
    BSplineCurve::try_interpole(KnotVec::from(knots), parameter_points).map_err(|e| {
        Diagnostic::new(Code::CurveFittingFailed, "boolean", "fit_curves").with_coded_source(e)
    })
}

/// A point inside `face`, away from its boundary: the centroid of a triangle of its mesh.
///
/// Boundary vertices are unusable for inside-outside tests since they lie on the other shell
/// whenever a cut ran through them.
fn interior_point<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    face: &AltCurveFace<C, S>,
    tol: f64,
) -> Option<Point3> {
    let shell: AltCurveShell<C, S> = vec![face.clone()].into();
    let mesh = shell.triangulation(tol)[0].surface()?;
    let positions = mesh.positions();
    let triangle = mesh.faces().tri_faces().first()?;
    let sum = triangle
        .iter()
        .fold(Vector3::zero(), |sum, v| sum + positions[v.pos].to_vec());
    Some(Point3::from_vec(sum / 3.0))
}

/// Sorts the faces whose status the cuts left undecided into `and` (inside the other solid) or
/// `or` (outside), by ray casting from an interior point against the other solid's mesh.
///
/// The signed crossing count of a ray from a point inside a closed mesh is 1. An inverted mesh,
/// the complement of a solid, has its interior where the count is 0, and −1 inside the
/// original solid. The nesting of the operand boundaries determines which case applies.
fn classify_unknown<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    unknown: AltCurveShell<C, S>,
    other: &Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>,
    and: &mut AltCurveShell<C, S>,
    or: &mut AltCurveShell<C, S>,
    tol: f64,
    inverted: bool,
) -> Option<()> {
    let mesh = other.to_polygon();
    let inside_count = if inverted { 0 } else { 1 };
    unknown.into_iter().try_for_each(|face| {
        let pt = interior_point(&face, tol)?;
        let dir = hash::take_one_unit(pt);
        if mesh.signed_crossing_faces(pt, dir) >= inside_count {
            and.push(face);
        } else {
            or.push(face);
        }
        Some(())
    })
}

/// Cuts the complete boundaries `shell0` and `shell1` against each other and sorts the pieces into those inside the
/// other solid (`[0]`) and those outside (`[1]`).
///
/// `tol` does two jobs: both shells are triangulated at `tol`, which seeds the face pairing
/// and the interference polylines, and the edges are sampled at `tol` when the faces are
/// divided and the leftover pieces classified. Coincidence of points, vertex snapping and the
/// bounding-box slack in `loops_store` and `polyline_construction` use the global `TOLERANCE`.
fn process_boundaries<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell0: &Shell<Point3, C, S>,
    shell1: &Shell<Point3, C, S>,
    tol: f64,
    poly_shells: [&Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>; 2],
    inverted: [bool; 2],
    intersection: bool,
) -> Result<(Shell<Point3, C, S>, bool), Diagnostic> {
    let [poly_shell0, poly_shell1] = poly_shells;
    let altshell0: AltCurveShell<C, S> =
        shell0.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let altshell1: AltCurveShell<C, S> =
        shell1.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let start = profile::now();
    let quadruple = loops_store::try_create_loops_stores(
        &altshell0,
        poly_shell0,
        &altshell1,
        poly_shell1,
        tol,
    );
    profile::lap(Stage::LoopsStore, start);
    let loops_store::LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = quadruple?;
    let start = profile::now();
    let mut cls0 =
        divide_face::try_divide_faces(&altshell0, &loops_store0, tol).map_err(|e| e.operand(0))?;
    cls0.integrate_by_component();
    let mut cls1 =
        divide_face::try_divide_faces(&altshell1, &loops_store1, tol).map_err(|e| e.operand(1))?;
    cls1.integrate_by_component();
    profile::lap(Stage::Division, start);
    let start = profile::now();
    let [mut and0, mut or0, unknown0] = cls0.and_or_unknown();
    let outside0 = or0.len();
    classify_unknown(unknown0, poly_shell1, &mut and0, &mut or0, tol, inverted[1]).ok_or_else(
        || Diagnostic::new(Code::ClassificationFailed, "boolean", "classify_faces").operand(0),
    )?;
    let [mut and1, mut or1, unknown1] = cls1.and_or_unknown();
    classify_unknown(unknown1, poly_shell0, &mut and1, &mut or1, tol, inverted[0]).ok_or_else(
        || Diagnostic::new(Code::ClassificationFailed, "boolean", "classify_faces").operand(1),
    )?;
    profile::lap(Stage::Classification, start);
    // Against a complemented cutter, excluded target faces and cutter faces inside
    // the target witness material removal. A same-normal contact is `Both` on the
    // target and its duplicate is `Neither` on the cutter, so neither counts here.
    let removed_material =
        cls0.excludes_first_material() || or0.len() > outside0 || !and1.is_empty();
    let shell = if intersection {
        and0.append(&mut and1);
        and0
    } else {
        or0.append(&mut or1);
        or0
    };
    let start = profile::now();
    let shell = altshell_to_shell(&shell)?;
    profile::lap(Stage::Fitting, start);
    Ok((shell, removed_material))
}

/// Intersection of two solids.
///
/// Each operand is a set of oriented, embedded boundary shells. Disconnected exteriors point
/// outward, cavity boundaries point inward, and orientation alternates at each nesting level.
/// Shell order is immaterial. Inverting every shell represents the complement of a nonempty
/// bounded solid. See [`solid_components`] for the supported separation between shells and for
/// grouping a result into bodies with their cavities before STEP export.
///
/// Zero shells always represent the empty set. Intersection with empty is empty, and union
/// with empty is the other operand. The whole space has no representation: an operation that
/// would produce it returns `None`. Use [`subtract`] for subtraction with possibly empty cutters.
///
/// Returns `None` for detected invalid shell nesting or topology, failed tessellation, or
/// unsupported division/fitting. In particular, an invalid result is not constructed as a
/// successful `Solid`. Inputs must satisfy the geometric contracts of their curves and
/// surfaces; tessellation is not an exact check for arbitrary self-intersections.
///
/// `tol` is the chord error of the tessellation that seeds the operation, and does two jobs:
/// the shells are meshed at `tol` to find which faces meet and where, and the edges are sampled
/// at `tol` for dividing the faces and for the inside tests. The cut edges themselves are exact,
/// interpolated through points on both surfaces, so `tol` does not enter the geometry of the
/// result. Whether two points, or a point and a surface, coincide is decided by the global
/// [`TOLERANCE`](truck_base::tolerance::TOLERANCE), the identity every geometry routine of
/// truck uses; it is not a parameter of this function.
///
/// Loosening `tol` makes the operation faster and leaves the result unchanged as long as `tol`
/// stays below the features of the faces: a `tol` above the radius of a circular edge collapses
/// each of its arcs to one chord, the mesh degenerates, and the operation returns `None`.
/// Tightening `tol` only adds triangles and time. `tol` must be finite and positive; invalid
/// tolerances return `None`. It must also resolve gaps used to classify nested shells.
pub fn and<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Solid<Point3, C, S>> {
    try_and(solid0, solid1, tol).ok()
}

/// Union of two solids.
///
/// Uses the same shell representation, failure contract, and tolerance as [`and`].
pub fn or<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Solid<Point3, C, S>> {
    try_or(solid0, solid1, tol).ok()
}

fn boolean<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
    intersection: bool,
) -> Result<(Solid<Point3, C, S>, bool), Diagnostic> {
    let start = profile::now();
    let (poly0, nesting0) =
        components::triangulate_boundaries(solid0, tol).map_err(|e| e.operand(0))?;
    let (poly1, nesting1) =
        components::triangulate_boundaries(solid1, tol).map_err(|e| e.operand(1))?;
    profile::lap(Stage::Triangulation, start);
    if solid0.boundaries().is_empty() || solid1.boundaries().is_empty() {
        return Ok((
            if intersection {
                Solid::new(Vec::new())
            } else if solid0.boundaries().is_empty() {
                solid1.clone()
            } else {
                solid0.clone()
            },
            false,
        ));
    }
    let shell0 = solid0.face_iter().cloned().collect();
    let shell1 = solid1.face_iter().cloned().collect();
    let (shell, removed_material) = process_boundaries(
        &shell0,
        &shell1,
        tol,
        [&poly0, &poly1],
        [nesting0.inverted, nesting1.inverted],
        intersection,
    )?;
    let unbounded = if intersection {
        nesting0.inverted && nesting1.inverted
    } else {
        nesting0.inverted || nesting1.inverted
    };
    if shell.is_empty() && unbounded {
        return Err(Diagnostic::new(
            Code::UnboundedResult,
            "boolean",
            "validate_output",
        ));
    }
    Ok((
        Solid::try_new(shell.connected_components()).map_err(|e| {
            Diagnostic::new(Code::InvalidOutputTopology, "boolean", "validate_output")
                .with_coded_source(e)
        })?,
        removed_material,
    ))
}

/// Subtracts `solid1` from `solid0`, including empty operands.
///
/// Uses the same shell contract and tolerance as [`and`]. Prefer this over manually inverting
/// the cutter: `Solid::not` cannot distinguish the complement of an empty solid from empty.
pub fn subtract<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Solid<Point3, C, S>> {
    try_subtract(solid0, solid1, tol).ok()
}

/// A subtraction and whether the cutter removed any material.
#[derive(Clone, Debug)]
pub struct SubtractionResult<C, S> {
    /// The target after subtraction.
    pub solid: Solid<Point3, C, S>,
    /// True for partial cuts, complete consumption, and newly created cavities.
    /// False for disjoint or contact-only cutters and either empty operand.
    pub removed_material: bool,
}

/// Subtracts and reports material removal using the same face classification.
///
/// Uses the same tolerance, nonmutation, and failure contract as [`subtract`]. No
/// second boolean or mesh-volume comparison is performed. Shells may be disconnected
/// or nested, including cavity boundaries, as described in [`and`].
pub fn subtract_with_effect<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<SubtractionResult<C, S>> {
    try_subtract_with_effect(solid0, solid1, tol).ok()
}

/// Intersection with stable diagnostics. Empty intersections are successful solids.
pub fn try_and<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<Solid<Point3, C, S>, Diagnostic> {
    boolean(solid0, solid1, tol, true)
        .map(|(solid, _)| solid)
        .map_err(|e| e.operation("and"))
}

/// Union with stable diagnostics.
pub fn try_or<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<Solid<Point3, C, S>, Diagnostic> {
    boolean(solid0, solid1, tol, false)
        .map(|(solid, _)| solid)
        .map_err(|e| e.operation("or"))
}

/// Subtraction with stable diagnostics, including empty operands.
pub fn try_subtract<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<Solid<Point3, C, S>, Diagnostic> {
    try_subtract_with_effect(solid0, solid1, tol)
        .map(|result| result.solid)
        .map_err(|e| e.operation("subtract"))
}

/// Subtraction with diagnostics and material-removal information from the same operation.
pub fn try_subtract_with_effect<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<SubtractionResult<C, S>, Diagnostic> {
    subtraction(solid0, solid1, tol).map_err(|e| e.operation("subtract_with_effect"))
}

fn subtraction<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Result<SubtractionResult<C, S>, Diagnostic> {
    if solid1.boundaries().is_empty() {
        return Ok(SubtractionResult {
            solid: try_or(solid0, solid1, tol)?,
            removed_material: false,
        });
    }
    let mut complement = solid1.clone();
    complement.not();
    let (solid, removed_material) = boolean(solid0, &complement, tol, true)?;
    Ok(SubtractionResult {
        solid,
        removed_material,
    })
}
