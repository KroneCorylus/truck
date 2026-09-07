use crate::Curve2;
use truck_modeling::*;

type Polyline = (Vec<f64>, Vec<Point2>);

/// For each curve, the sorted parameters where it crosses another curve, excluding crossings
/// within `tol` of its ends or of each other.
pub(crate) fn split_parameters(curves: &[&Curve2], tol: f64) -> Vec<Vec<f64>> {
    let polylines: Vec<Polyline> = curves
        .iter()
        .map(|curve| curve.parameter_division(curve.range_tuple(), tol))
        .collect();
    let boxes: Vec<BoundingBox<Point2>> = polylines
        .iter()
        .map(|(_, points)| points.iter().copied().collect())
        .collect();
    let mut splits = vec![Vec::new(); curves.len()];
    for i in 0..curves.len() {
        for j in i + 1..curves.len() {
            if !overlaps(&boxes[i], &boxes[j], tol) {
                continue;
            }
            for (s, t) in crossings(curves[i], &polylines[i], curves[j], &polylines[j], tol) {
                splits[i].push(s);
                splits[j].push(t);
            }
        }
    }
    curves
        .iter()
        .zip(splits)
        .map(|(curve, splits)| interior(curve, splits, tol))
        .collect()
}

fn overlaps(a: &BoundingBox<Point2>, b: &BoundingBox<Point2>, tol: f64) -> bool {
    (0..2).all(|i| a.min()[i] - tol <= b.max()[i] && b.min()[i] - tol <= a.max()[i])
}

/// The parameter pairs where `curve0` and `curve1` cross: crossings of their polylines,
/// refined by Newton's method when it converges inside both ranges. Meetings with parallel
/// tangents, where the curves touch or coincide, are not crossings.
fn crossings(
    curve0: &Curve2,
    (params0, points0): &Polyline,
    curve1: &Curve2,
    (params1, points1): &Polyline,
    tol: f64,
) -> Vec<(f64, f64)> {
    let mut found = Vec::new();
    for (i, a) in points0.windows(2).enumerate() {
        for (j, b) in points1.windows(2).enumerate() {
            let Some((u, v)) = segments_crossing(a[0], a[1], b[0], b[1]) else {
                continue;
            };
            let seed = (
                params0[i] + (params0[i + 1] - params0[i]) * u,
                params1[j] + (params1[j + 1] - params1[j]) * v,
            );
            let refined = algo::curve::search_intersection_parameter(curve0, curve1, seed, 100)
                .filter(|(s, t)| {
                    in_range(curve0, *s)
                        && in_range(curve1, *t)
                        && curve0.subs(*s).distance(curve1.subs(*t)) < tol
                });
            let (s, t) = refined.unwrap_or(seed);
            if !parallel(curve0.der(s), curve1.der(t)) {
                found.push((s, t));
            }
        }
    }
    found
}

fn parallel(a: Vector2, b: Vector2) -> bool {
    a.so_small() || b.so_small() || a.normalize().perp_dot(b.normalize()).so_small()
}

fn in_range<C: BoundedCurve>(curve: &C, t: f64) -> bool {
    let (t0, t1) = curve.range_tuple();
    (t0..=t1).contains(&t)
}

/// The fractions along both segments of their crossing point; `None` when parallel.
fn segments_crossing(a0: Point2, a1: Point2, b0: Point2, b1: Point2) -> Option<(f64, f64)> {
    const SLACK: f64 = 1.0e-9;
    let (da, db) = (a1 - a0, b1 - b0);
    let det = da.perp_dot(db);
    if det.so_small2() {
        return None;
    }
    let diff = b0 - a0;
    let u = diff.perp_dot(db) / det;
    let v = diff.perp_dot(da) / det;
    let unit = -SLACK..=1.0 + SLACK;
    (unit.contains(&u) && unit.contains(&v)).then_some((u, v))
}

/// `splits` sorted and thinned to those at least `tol` away from the ends of `curve` and from
/// each other.
fn interior(curve: &Curve2, mut splits: Vec<f64>, tol: f64) -> Vec<f64> {
    splits.sort_by(f64::total_cmp);
    let (front, back) = (curve.front(), curve.back());
    let mut kept: Vec<(f64, Point2)> = Vec::new();
    for t in splits {
        let p = curve.subs(t);
        let far_from_ends = p.distance(front) >= tol && p.distance(back) >= tol;
        let far_from_last = kept.last().is_none_or(|(_, q)| p.distance(*q) >= tol);
        if far_from_ends && far_from_last {
            kept.push((t, p));
        }
    }
    kept.into_iter().map(|(t, _)| t).collect()
}

/// The part of `curve` between the parameters `a` and `b`.
pub(crate) fn piece(curve: &Curve2, a: f64, b: f64) -> Curve2 {
    if let Curve2::Line(line) = curve {
        return Line(line.subs(a), line.subs(b)).into();
    }
    let (t0, t1) = curve.range_tuple();
    let mut piece = curve.clone();
    if a > t0 {
        piece = piece.cut(a);
    }
    if b < t1 {
        piece.cut(b);
    }
    piece
}
