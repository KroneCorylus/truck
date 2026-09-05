use crate::alternative::Alternative;

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
fn smooth_leader<S: ShapeOpsSurface>(
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
fn classify_unknown<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    unknown: AltCurveShell<C, S>,
    other: &Shell<Point3, PolylineCurve<Point3>, Option<PolygonMesh>>,
    and: &mut AltCurveShell<C, S>,
    or: &mut AltCurveShell<C, S>,
    tol: f64,
) -> Option<()> {
    unknown.into_iter().try_for_each(|face| {
        let pt = interior_point(&face, tol)?;
        let dir = hash::take_one_unit(pt);
        let count = other.iter().try_fold(0, |count, face| {
            let poly = face.surface()?;
            Some(count + poly.signed_crossing_faces(pt, dir))
        })?;
        if count >= 1 {
            and.push(face);
        } else {
            or.push(face);
        }
        Some(())
    })
}

fn process_one_pair_of_shells<C: ShapeOpsCurve<S>, S: ShapeOpsSurface>(
    shell0: &Shell<Point3, C, S>,
    shell1: &Shell<Point3, C, S>,
    tol: f64,
) -> Option<[Shell<Point3, C, S>; 2]> {
    nonpositive_tolerance!(tol);
    let poly_shell0 = shell0.triangulation(tol);
    let poly_shell1 = shell1.triangulation(tol);
    let altshell0: AltCurveShell<C, S> =
        shell0.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let altshell1: AltCurveShell<C, S> =
        shell1.mapped(|x| *x, |c| Alternative::FirstType(c.clone()), Clone::clone);
    let loops_store::LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = loops_store::create_loops_stores(&altshell0, &poly_shell0, &altshell1, &poly_shell1)?;
    let mut cls0 = divide_face::divide_faces(&altshell0, &loops_store0, tol)?;
    cls0.integrate_by_component();
    let mut cls1 = divide_face::divide_faces(&altshell1, &loops_store1, tol)?;
    cls1.integrate_by_component();
    let [mut and0, mut or0, unknown0] = cls0.and_or_unknown();
    classify_unknown(unknown0, &poly_shell1, &mut and0, &mut or0, tol)?;
    let [mut and1, mut or1, unknown1] = cls1.and_or_unknown();
    classify_unknown(unknown1, &poly_shell0, &mut and1, &mut or1, tol)?;
    and0.append(&mut and1);
    or0.append(&mut or1);
    Some([altshell_to_shell(&and0)?, altshell_to_shell(&or0)?])
}

/// AND operation between two solids.
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

/// OR operation between two solids.
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
