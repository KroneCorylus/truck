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
) -> Option<Shell<Point3, C, S>> {
    altshell.try_mapped(
        |p| Some(*p),
        |c| match c {
            Alternative::FirstType(c) => Some(c.clone()),
            Alternative::SecondType(ic) => {
                let leader = smooth_leader(ic)?;
                Some(
                    IntersectionCurve::new(ic.surface0().clone(), ic.surface1().clone(), leader)
                        .into(),
                )
            }
        },
        |s| Some(s.clone()),
    )
}

/// A cubic B-spline through the exact intersection points at the vertices of the leading
/// polyline of `ic`, parametrized by chord length.
///
/// The final curve projects this leader onto the intersection inside the plane normal to the
/// leader's tangent, so the leader must follow the intersection smoothly: a wiggle of the size
/// of the tolerance makes the projected points zigzag, or the projection fail.
pub(crate) fn smooth_leader<S: ShapeOpsSurface>(
    ic: &IntersectionCurve<PolylineCurve<Point3>, S, S>,
) -> Option<BSplineCurve<Point3>> {
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
        let (p, _, _) = ic.search_triple(t, 100)?;
        if points.last().is_none_or(|q| !q.near(&p)) {
            points.push(p);
        }
    }
    let n = points.len();
    if n < 2 {
        return None;
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
    BSplineCurve::try_interpole(KnotVec::from(knots), parameter_points).ok()
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
/// original solid; its orientation shows in the sign of its volume.
fn classify_unknown<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    unknown: AltCurveShell<C, S>,
    other: &Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>,
    and: &mut AltCurveShell<C, S>,
    or: &mut AltCurveShell<C, S>,
    tol: f64,
) -> Option<()> {
    let mesh = other.to_polygon();
    let inside_count = if mesh.volume() < 0.0 { 0 } else { 1 };
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

/// Cuts `shell0` and `shell1` against each other and sorts the pieces into those inside the
/// other solid (`[0]`) and those outside (`[1]`).
///
/// `tol` does two jobs: both shells are triangulated at `tol`, which seeds the face pairing
/// and the interference polylines, and the edges are sampled at `tol` when the faces are
/// divided and the leftover pieces classified. Coincidence of points, vertex snapping and the
/// bounding-box slack in `loops_store` and `polyline_construction` use the global `TOLERANCE`.
fn process_one_pair_of_shells<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell0: &Shell<Point3, C, S>,
    shell1: &Shell<Point3, C, S>,
    tol: f64,
) -> Option<[Shell<Point3, C, S>; 2]> {
    nonpositive_tolerance!(tol);
    let start = profile::now();
    let poly_shell0 = shell0.triangulation(tol);
    let poly_shell1 = shell1.triangulation(tol);
    profile::lap(Stage::Triangulation, start);
    let altshell0: AltCurveShell<C, S> =
        shell0.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let altshell1: AltCurveShell<C, S> =
        shell1.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let start = profile::now();
    let quadruple =
        loops_store::create_loops_stores(&altshell0, &poly_shell0, &altshell1, &poly_shell1);
    profile::lap(Stage::LoopsStore, start);
    let loops_store::LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = quadruple?;
    let start = profile::now();
    let mut cls0 = divide_face::divide_faces(&altshell0, &loops_store0, tol)?;
    cls0.integrate_by_component();
    let mut cls1 = divide_face::divide_faces(&altshell1, &loops_store1, tol)?;
    cls1.integrate_by_component();
    profile::lap(Stage::Division, start);
    let start = profile::now();
    let [mut and0, mut or0, unknown0] = cls0.and_or_unknown();
    classify_unknown(unknown0, &poly_shell1, &mut and0, &mut or0, tol)?;
    let [mut and1, mut or1, unknown1] = cls1.and_or_unknown();
    classify_unknown(unknown1, &poly_shell0, &mut and1, &mut or1, tol)?;
    profile::lap(Stage::Classification, start);
    and0.append(&mut and1);
    or0.append(&mut or1);
    let start = profile::now();
    let shells = [altshell_to_shell(&and0)?, altshell_to_shell(&or0)?];
    profile::lap(Stage::Fitting, start);
    Some(shells)
}

/// Intersection of two solids.
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
/// Tightening `tol` only adds triangles and time. `tol` must be positive.
pub fn and<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Solid<Point3, C, S>> {
    let mut iter0 = solid0.boundaries().iter();
    let mut iter1 = solid1.boundaries().iter();
    let shell0 = iter0.next().unwrap();
    let shell1 = iter1.next().unwrap();
    let [mut and_shell, _] = process_one_pair_of_shells(shell0, shell1, tol)?;
    for shell in iter0 {
        let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
        and_shell = res;
    }
    for shell in iter1 {
        let [res, _] = process_one_pair_of_shells(&and_shell, shell, tol)?;
        and_shell = res;
    }
    let boundaries = and_shell.connected_components();
    Some(Solid::new(boundaries))
}

/// Union of two solids.
///
/// `tol` is the chord error of the seeding tessellation, as for [`and`].
pub fn or<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    solid0: &Solid<Point3, C, S>,
    solid1: &Solid<Point3, C, S>,
    tol: f64,
) -> Option<Solid<Point3, C, S>> {
    let mut iter0 = solid0.boundaries().iter();
    let mut iter1 = solid1.boundaries().iter();
    let shell0 = iter0.next().unwrap();
    let shell1 = iter1.next().unwrap();
    let [_, mut or_shell] = process_one_pair_of_shells(shell0, shell1, tol)?;
    for shell in iter0 {
        let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
        or_shell = res;
    }
    for shell in iter1 {
        let [_, res] = process_one_pair_of_shells(&or_shell, shell, tol)?;
        or_shell = res;
    }
    let boundaries = or_shell.connected_components();
    Some(Solid::new(boundaries))
}
