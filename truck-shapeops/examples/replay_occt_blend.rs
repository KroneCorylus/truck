//! Replays a captured physical selection; successful construction is not a parity verdict.
use serde_json::{json, Value};
use std::cell::Cell;
use std::collections::HashSet;
use std::path::Path;
use std::result::Result;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::{out::*, r#in::Table};

const TOL: f64 = 0.001;

fn import(path: &str) -> Result<Solid, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let table = Table::from_step(&text).ok_or("STEP parse failed")?;
    if table.manifold_solid_brep.len() != 1 {
        return Err("Replay currently requires one manifold solid".into());
    }
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(entity)
        .map_err(|e| format!("{e:?}"))?;
    if !skipped.is_empty() {
        return Err(format!("STEP skipped entities: {skipped:?}"));
    }
    compressed.split_closed_edges_and_faces(TOL / 100.);
    let modeling = compressed
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .ok_or("Unsupported imported geometry")?;
    Solid::extract(modeling).map_err(|e| format!("{e:?}"))
}

fn segment_distance(point: Point3, a: Point3, b: Point3) -> f64 {
    let delta = b - a;
    let u = if delta.magnitude2() > 0. {
        ((point - a).dot(delta) / delta.magnitude2()).clamp(0., 1.)
    } else {
        0.
    };
    point.distance(a + u * delta)
}

type SelectionStation = (Point3, f64, usize);

fn nearest_selection_station(point: Point3, points: &[Point3]) -> SelectionStation {
    let (index, distance) = points
        .windows(2)
        .enumerate()
        .map(|(i, p)| (i, segment_distance(point, p[0], p[1])))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    (point, distance, index)
}

fn selection_covers_chord(
    a: SelectionStation,
    b: SelectionStation,
    points: &[Point3],
    tol: f64,
) -> bool {
    let mut pending = vec![(a, b, 0)];
    while let Some((a, b, depth)) = pending.pop() {
        if !a.1.is_finite() || !b.1.is_finite() || a.1 > tol || b.1 > tol {
            return false;
        }
        // A segment capsule is convex. Otherwise the distance to the complete
        // polyline is 1-Lipschitz, giving a bound from both endpoint distances.
        if segment_distance(b.0, points[a.2], points[a.2 + 1]) <= tol
            || segment_distance(a.0, points[b.2], points[b.2 + 1]) <= tol
            || a.1 + b.1 + a.0.distance(b.0) <= 2. * tol
        {
            continue;
        }
        let middle = a.0 + (b.0 - a.0) * 0.5;
        if depth == 48 || middle == a.0 || middle == b.0 {
            return false;
        }
        let middle = nearest_selection_station(middle, points);
        pending.push((a, middle, depth + 1));
        pending.push((middle, b, depth + 1));
    }
    true
}

fn match_selection(solid: &Solid, selection: &Value) -> Result<Vec<Edge>, String> {
    let points = selection["points"]
        .as_array()
        .ok_or("Missing selection points")?
        .iter()
        .map(|p| {
            Ok(Point3::new(
                p[0].as_f64().ok_or("invalid x")?,
                p[1].as_f64().ok_or("invalid y")?,
                p[2].as_f64().ok_or("invalid z")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    match_edges(solid.edge_iter().collect(), &points)
}

fn match_edges(edges: Vec<Edge>, points: &[Point3]) -> Result<Vec<Edge>, String> {
    if points.len() < 2 || !points.iter().all(|p| p.to_vec().is_finite()) {
        return Err("Insufficient or nonfinite selection points".into());
    }
    let mut seen = HashSet::new();
    let candidates: Vec<_> = edges
        .into_iter()
        .filter(|e| seen.insert(e.id()))
        .filter(|e| {
            let curve = e.oriented_curve();
            let (a, b) = curve.range_tuple();
            let (_, samples) = curve.parameter_division((a, b), TOL / 100.);
            let stations: Vec<_> = samples
                .into_iter()
                .map(|p| nearest_selection_station(p, points))
                .collect();
            stations
                .windows(2)
                .all(|p| selection_covers_chord(p[0], p[1], points, TOL * 0.09))
        })
        .collect();
    if candidates.is_empty() {
        return Err("No matching physical edge".into());
    }
    if !points.iter().all(|p| {
        candidates.iter().any(|e| {
            let curve = e.oriented_curve();
            let (a, b) = curve.range_tuple();
            curve
                .search_nearest_parameter(*p, None, 100)
                .is_some_and(|t| curve.subs(t.clamp(a, b)).distance(*p) <= TOL / 10.)
        })
    }) {
        return Err("Imported edges do not cover the complete physical selection".into());
    }
    for (i, edge) in candidates.iter().enumerate() {
        let curve = edge.oriented_curve();
        let (a, b) = curve.range_tuple();
        for other in candidates.iter().skip(i + 1) {
            let other = other.oriented_curve();
            let (c, d) = other.range_tuple();
            for station in 1..4 {
                let p = curve.subs(a + (b - a) * station as f64 / 4.);
                if other
                    .search_nearest_parameter(p, None, 100)
                    .is_some_and(|t| t > c && t < d && other.subs(t).distance(p) <= TOL / 10.)
                {
                    return Err("Ambiguous overlapping physical edges".into());
                }
            }
        }
    }
    Ok(candidates)
}

fn as_wire(edges: &[Edge]) -> Option<Wire> {
    let mut remaining = edges.to_vec();
    let start = remaining
        .iter()
        .position(|e| {
            [e.front(), e.back()].iter().any(|v| {
                remaining
                    .iter()
                    .filter(|e| e.front() == *v || e.back() == *v)
                    .count()
                    == 1
            })
        })
        .unwrap_or(0);
    let mut edge = remaining.get(start)?.clone();
    let degree = |v: &Vertex| {
        remaining
            .iter()
            .filter(|e| e.front() == v || e.back() == v)
            .count()
    };
    if degree(edge.front()) != 1 && degree(edge.back()) == 1 {
        edge.invert();
    }
    remaining.remove(start);
    let mut wire = wire![edge];
    while !remaining.is_empty() {
        let last = wire.back_vertex()?;
        let candidates: Vec<_> = remaining
            .iter()
            .enumerate()
            .filter(|(_, e)| e.front() == last || e.back() == last)
            .map(|(i, _)| i)
            .collect();
        if candidates.len() != 1 {
            return None;
        }
        let mut edge = remaining.remove(candidates[0]);
        if edge.front() != last {
            edge.invert();
        }
        wire.push_back(edge);
    }
    Some(wire)
}

fn replay(prefix: &str, stage: &Cell<&'static str>) -> Result<Value, String> {
    let task: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{prefix}.input.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if task["operation"] != "fillet" {
        return Ok(json!({"status":"unhandled-intent","reason":"Only fillets are captured"}));
    }
    stage.set("import-and-preprocess");
    let input = match import(&format!("{prefix}.input.step")) {
        Ok(solid) => solid,
        Err(error) => return Ok(json!({"status":"input-import-failed","error":error})),
    };
    stage.set("input-consistency");
    if !input.is_geometric_consistent() {
        return Ok(json!({"status":"input-inconsistent"}));
    }
    let before = serde_json::to_string(&input.compress()).map_err(|e| e.to_string())?;
    let selection = task["selection"].as_array().ok_or("Missing selection")?;
    let radius = selection
        .first()
        .and_then(|s| s["radius"].as_f64())
        .ok_or("Missing radius")?;
    if !radius.is_finite() || radius <= 0. {
        return Err("Invalid captured radius".into());
    }
    if !selection
        .iter()
        .all(|s| s["radius"].as_f64() == Some(radius))
    {
        return Ok(
            json!({"status":"unhandled-intent","reason":"This replay route currently handles equal radii only"}),
        );
    }
    stage.set("physical-selection");
    let mut edges = Vec::new();
    let mut seen = HashSet::new();
    for chosen in selection {
        match match_selection(&input, chosen) {
            Ok(matched) => edges.extend(matched.into_iter().filter(|e| seen.insert(e.id()))),
            Err(error) => return Ok(json!({"status":"selection-unmatched","error":error})),
        }
    }
    let ids: Vec<_> = edges.iter().map(|e| e.id()).collect();
    stage.set("blend");
    let result = try_fillet_solid_edges(&input, &ids, radius, TOL).or_else(|error| {
        if let Some(wire) = as_wire(&edges) {
            try_fillet_solid_along_wire(&input, &wire, radius, TOL)
        } else {
            Err(error)
        }
    });
    let unchanged =
        before == serde_json::to_string(&input.compress()).map_err(|e| e.to_string())?;
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            return Ok(
                json!({"status":"operation-failed","code":format!("{:?}",error.code),"error":error.to_string(),"input_unchanged":unchanged,"selected_edges":ids.len()}),
            )
        }
    };
    stage.set("mesh");
    let mut mesh = result.solid.triangulation(TOL).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    stage.set("step-export");
    let prepared = prepare_for_step(&result.solid.compress(), TOL / 20.);
    if let Some(prepared) = &prepared {
        let design = StepDesign::from_model(StepModel::from(prepared));
        std::fs::write(
            format!("{prefix}.truck.step"),
            StepDisplay::new(Default::default(), design).to_string(),
        )
        .map_err(|e| e.to_string())?;
    }
    stage.set("output-consistency");
    Ok(
        json!({"status":"operation-succeeded","input_unchanged":unchanged,
        "geometry_consistent":result.solid.is_geometric_consistent(),
        "mesh_closed":mesh.shell_condition()==ShellCondition::Closed,
        "mesh_volume":mesh.volume(),"step_prepared":prepared.is_some(),"selected_edges":ids.len()}),
    )
}

fn main() {
    let prefix = std::env::args()
        .nth(1)
        .expect("usage: replay_occt_blend captured-prefix");
    let stage = Cell::new("read-intent");
    let result =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| replay(&prefix, &stage))) {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => json!({"status":"harness-error","error":error}),
            Err(_) => json!({"status":"panic"}),
        };
    let report = json!({"schema_version":1,"tolerance":TOL,"comparison":"unverified","stage":stage.get(),"result":result});
    std::fs::write(
        Path::new(&format!("{prefix}.truck.json")),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_selection_accepts_split_edges_and_ignores_order() {
        let vertices: Vec<_> = (0..=2)
            .map(|x| builder::vertex(Point3::new(x as f64, 0., 0.)))
            .collect();
        let first = builder::line(&vertices[0], &vertices[1]);
        let second = builder::line(&vertices[1], &vertices[2]);
        let selected = match_edges(
            vec![second.inverse(), first.clone(), first.inverse()],
            &[vertices[0].point(), vertices[2].point()],
        )
        .unwrap();
        assert_eq!(selected.len(), 2);
        let wire = as_wire(&selected).unwrap();
        assert!(wire.is_continuous());
        assert_eq!(wire.len(), 2);
    }

    #[test]
    fn physical_selection_rejects_incomplete_and_ambiguous_matches() {
        let a = builder::vertex(Point3::origin());
        let b = builder::vertex(Point3::new(2., 0., 0.));
        let middle = builder::vertex(Point3::new(1., 0., 0.));
        assert!(
            match_edges(vec![builder::line(&a, &middle)], &[a.point(), b.point()])
                .unwrap_err()
                .contains("complete")
        );
        assert!(match_edges(
            vec![builder::line(&a, &b), builder::line(&a, &b)],
            &[a.point(), b.point()],
        )
        .unwrap_err()
        .contains("Ambiguous"));
        assert!(match_edges(
            vec![builder::line(&a, &b)],
            &[a.point(), Point3::new(f64::NAN, 0., 0.)]
        )
        .is_err());
    }

    #[test]
    fn wire_route_rejects_branches_and_disconnected_selection() {
        let center = builder::vertex(Point3::origin());
        let tips: Vec<_> = [Vector3::unit_x(), Vector3::unit_y(), Vector3::unit_z()]
            .into_iter()
            .map(|v| builder::vertex(Point3::from_vec(v)))
            .collect();
        let star: Vec<_> = tips.iter().map(|v| builder::line(&center, v)).collect();
        assert!(as_wire(&star).is_none());
        let detached = builder::line(
            &builder::vertex(Point3::new(3., 0., 0.)),
            &builder::vertex(Point3::new(4., 0., 0.)),
        );
        assert!(as_wire(&[star[0].clone(), detached]).is_none());
    }
}

#[cfg(test)]
mod curved_selection_tests {
    use super::*;

    #[test]
    fn curved_selection_excludes_the_straight_chord_between_its_endpoints() {
        let curve = Curve::BSplineCurve(BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point3::new(1., -2., 0.),
                Point3::new(-1., 0., 0.),
                Point3::new(1., 2., 0.),
            ],
        ));
        let a = builder::vertex(curve.front());
        let b = builder::vertex(curve.back());
        let points = curve.parameter_division(curve.range_tuple(), TOL / 1000.).1;
        let arc = Edge::new(&a, &b, curve);
        let selected = match_edges(vec![builder::line(&a, &b), arc.inverse()], &points).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id(), arc.id());
        assert!(!as_wire(&selected).unwrap().is_cyclic());
    }

    #[test]
    fn matching_endpoints_and_midpoint_do_not_establish_physical_coverage() {
        let points = vec![
            Point3::new(0., 0., 0.),
            Point3::new(2., 1., 0.),
            Point3::new(5., 0., 0.),
            Point3::new(8., -1., 0.),
            Point3::new(10., 0., 0.),
        ];
        let curve = Curve::BSplineCurve(BSplineCurve::new(
            KnotVec::uniform_knot(1, 4),
            points.clone(),
        ));
        let a = builder::vertex(curve.front());
        let b = builder::vertex(curve.back());
        let bent = Edge::new(&a, &b, curve);
        let selected = match_edges(vec![builder::line(&a, &b), bent.clone()], &points).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id(), bent.id());
    }
}
