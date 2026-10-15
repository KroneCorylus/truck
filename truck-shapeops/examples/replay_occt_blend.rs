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
const CHAMFER_STEP_TOL: f64 = TOLERANCE / 10.;

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

fn captured_vector(value: &Value) -> Option<Vector3> {
    let values = value.as_array()?;
    if values.len() != 3 {
        return None;
    }
    let v = Vector3::new(
        values[0].as_f64()?,
        values[1].as_f64()?,
        values[2].as_f64()?,
    );
    v.is_finite().then_some(v)
}

fn chamfer_selection(
    input: &Solid,
    selection: &[Value],
    native_contours: bool,
) -> Result<Vec<(Edge, [f64; 2])>, Value> {
    let invalid = |reason: &str| json!({"status":"harness-error","error":reason});
    let unsupported = |reason: &str| json!({"status":"unhandled-intent","reason":reason});
    if selection.is_empty() {
        return Err(invalid("Missing chamfer selections"));
    }
    let mut selected = Vec::new();
    let mut assignments = std::collections::HashMap::new();
    for chosen in selection {
        let contour = if native_contours {
            let applied = chosen["native_applied"]
                .as_bool()
                .ok_or_else(|| invalid("Missing native chamfer assignment state"))?;
            let contour = chosen["contour_edges"]
                .as_array()
                .ok_or_else(|| invalid("Missing native chamfer contour scope"))?;
            if !applied {
                if !contour.is_empty()
                    || !matches!(
                        chosen["ignored_reason"].as_str(),
                        Some("already-selected" | "invalid-parameters" | "not-added")
                    )
                {
                    return Err(invalid("Contradictory ignored native chamfer assignment"));
                }
                if chosen["ignored_reason"] == "already-selected" {
                    let seed = match_selection(input, chosen)
                        .map_err(|error| json!({"status":"selection-unmatched","error":error}))?;
                    if seed.iter().any(|e| !assignments.contains_key(&e.id())) {
                        return Err(invalid(
                            "Ignored seed is not covered by an earlier native assignment",
                        ));
                    }
                }
                continue;
            }
            if contour.is_empty() || !chosen["ignored_reason"].is_null() {
                return Err(invalid(
                    "Empty or contradictory active native chamfer contour",
                ));
            }
            Some(contour)
        } else {
            None
        };
        let mode = chosen["mode"].as_str().unwrap_or("");
        let positive = |v: &Value| {
            v.as_f64()
                .filter(|v| v.is_finite() && *v > 0.)
                .ok_or_else(|| invalid("Invalid captured chamfer distance"))
        };
        let mut distances = match mode {
            "equal-distance" => [positive(&chosen["distance"])?; 2],
            "two-distances" => [
                positive(&chosen["distances"][0])?,
                positive(&chosen["distances"][1])?,
            ],
            "distance-angle" => {
                let d = positive(&chosen["distance"])?;
                let angle = chosen["angle_degrees"]
                    .as_f64()
                    .filter(|a| a.is_finite() && *a > 0. && *a < 90.)
                    .ok_or_else(|| invalid("Invalid captured chamfer angle"))?;
                [d, d * angle.to_radians().tan()]
            }
            _ => return Err(unsupported("Unsupported chamfer parameter mode")),
        };
        if distances.iter().any(|d| !d.is_finite() || *d <= 0.) {
            return Err(invalid("Invalid derived chamfer distance"));
        }
        let reference = if mode == "equal-distance" {
            None
        } else {
            let p = captured_vector(&chosen["reference_plane"]["origin"]);
            let n = captured_vector(&chosen["reference_plane"]["normal"]);
            let (Some(p), Some(n)) = (p, n) else {
                return Err(unsupported(
                    "Face-dependent chamfers require a captured reference plane",
                ));
            };
            if n.magnitude2() < TOLERANCE * TOLERANCE {
                return Err(invalid("Invalid reference plane normal"));
            }
            Some((Point3::from_vec(p), n.normalize()))
        };
        let seed = match_selection(input, chosen)
            .map_err(|error| json!({"status":"selection-unmatched","error":error}))?;
        let matched = if let Some(contour) = contour {
            let mut edges = Vec::new();
            let mut seen = HashSet::new();
            for member in contour {
                for edge in match_selection(input, member)
                    .map_err(|error| json!({"status":"selection-unmatched","error":error}))?
                {
                    if seen.insert(edge.id()) {
                        edges.push(edge);
                    }
                }
            }
            if seed.iter().any(|e| !seen.contains(&e.id())) {
                return Err(invalid("Native chamfer contour does not contain its seed"));
            }
            if as_wire(&edges).is_none() {
                return Err(
                    json!({"status":"selection-unmatched","error":"Native contour does not match one connected unbranched wire"}),
                );
            }
            edges
        } else {
            seed
        };
        let requested = distances;
        for edge in matched {
            distances = requested;
            if let Some((origin, normal)) = reference {
                let adjacent: Vec<_> = input
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                    .collect();
                let planes: Option<Vec<_>> = adjacent
                    .iter()
                    .map(|f| match f.oriented_surface().elementary() {
                        Some((Elementary::Plane(p), forward)) => {
                            let normal = p.normal() * if forward { 1. } else { -1. };
                            Some((p, normal))
                        }
                        _ => None,
                    })
                    .collect();
                let planes = planes.filter(|p| p.len() == 2).ok_or_else(|| {
                    unsupported("Face-dependent chamfers currently require two planar supports")
                })?;
                let sides: Vec<_> = planes
                    .iter()
                    .enumerate()
                    .filter(|(_, (p, _))| {
                        Plane::normal(p).cross(normal).magnitude() < 1e-8
                            && (p.origin() - origin).dot(normal).abs() < TOL / 10.
                    })
                    .map(|(i, _)| i)
                    .collect();
                let [side] = sides.as_slice() else {
                    return Err(
                        json!({"status":"selection-unmatched","error":"Reference plane does not uniquely match an adjacent face"}),
                    );
                };
                if mode == "distance-angle" {
                    let cos_beta = -planes[0].1.dot(planes[1].1);
                    let sin_beta = planes[0].1.cross(planes[1].1).magnitude();
                    let tangent = requested[1] / requested[0];
                    // Sine rule in the corner section: d1 = d0 sin(alpha) / sin(beta + alpha).
                    let denominator = sin_beta + cos_beta * tangent;
                    if sin_beta <= 1e-12 || denominator <= 1e-12 {
                        return Err(invalid(
                            "Chamfer angle has no finite positive setback on these supports",
                        ));
                    }
                    distances[1] = requested[1] / denominator;
                    if !distances[1].is_finite() || distances[1] <= 0. {
                        return Err(invalid("Invalid derived chamfer distance"));
                    }
                }
                if *side == 1 {
                    distances.swap(0, 1);
                }
            }
            if let Some(previous) = assignments.insert(edge.id(), distances) {
                if previous != distances {
                    return Err(invalid(
                        "Conflicting chamfer distances for the same physical edge",
                    ));
                }
            } else {
                selected.push((edge, distances));
            }
        }
    }
    if selected.is_empty() {
        return Err(unsupported("No native-effective chamfer selections"));
    }
    Ok(selected)
}

fn replay(prefix: &str, stage: &Cell<&'static str>) -> Result<Value, String> {
    let task: Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{prefix}.input.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if task["operation"] != "fillet" && task["operation"] != "chamfer" {
        return Ok(json!({"status":"unhandled-intent","reason":"Unsupported captured operation"}));
    }
    let schema = match task.get("schema_version") {
        None => 1,
        Some(value) => value.as_u64().ok_or("Invalid capture schema version")?,
    };
    let scope = match (task["operation"].as_str(), schema) {
        (Some("chamfer"), 2) => "native-contours",
        (Some("fillet" | "chamfer"), 1) => "seed-only",
        _ => {
            return Ok(
                json!({"status":"harness-error","error":"Unsupported capture schema version"}),
            )
        }
    };
    let step_tol = if task["operation"] == "chamfer" {
        CHAMFER_STEP_TOL
    } else {
        TOL / 20.
    };
    let mut report = replay_task(prefix, stage, &task, schema, step_tol)?;
    report["selection_scope"] = json!(scope);
    report["step_fitting_tolerance"] = json!(step_tol);
    Ok(report)
}

fn replay_task(
    prefix: &str,
    stage: &Cell<&'static str>,
    task: &Value,
    schema: u64,
    step_tol: f64,
) -> Result<Value, String> {
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
    let (ids, result) = if task["operation"] == "chamfer" {
        stage.set("physical-selection");
        let selected = match chamfer_selection(&input, selection, schema == 2) {
            Ok(selected) => selected,
            Err(report) => return Ok(report),
        };
        let assignments: Vec<_> = selected.iter().map(|(e, d)| (e.id(), *d)).collect();
        let edges: Vec<_> = selected.into_iter().map(|(e, _)| e).collect();
        let ids: Vec<_> = edges.iter().map(|e| e.id()).collect();
        let distance = assignments[0].1[0];
        let equal = assignments.iter().all(|(_, d)| *d == [distance; 2]);
        stage.set("blend");
        let result =
            try_chamfer_solid_edges_with_distances(&input, &assignments, TOL).or_else(|error| {
                if equal {
                    if let Some(wire) = as_wire(&edges) {
                        return try_chamfer_solid_along_wire(
                            &input, &wire, distance, distance, TOL,
                        );
                    }
                }
                Err(error)
            });
        (ids, result)
    } else {
        let radius = selection
            .first()
            .and_then(|s| s["radius"].as_f64())
            .ok_or("Missing radius")?;
        if !radius.is_finite() || radius <= 0. {
            return Err("Invalid captured radius".into());
        }
        let mut edges = Vec::new();
        let mut selected = Vec::new();
        let mut radii = std::collections::HashMap::new();
        stage.set("physical-selection");
        for chosen in selection {
            let chosen_radius = chosen["radius"].as_f64().ok_or("Missing radius")?;
            if !chosen_radius.is_finite() || chosen_radius <= 0. {
                return Err("Invalid captured radius".into());
            }
            match match_selection(&input, chosen) {
                Ok(matched) => {
                    for edge in matched {
                        if let Some(previous) = radii.insert(edge.id(), chosen_radius) {
                            if previous != chosen_radius {
                                return Err("Conflicting radii for the same physical edge".into());
                            }
                        } else {
                            selected.push((edge.id(), chosen_radius));
                            edges.push(edge);
                        }
                    }
                }
                Err(error) => return Ok(json!({"status":"selection-unmatched","error":error})),
            }
        }
        let ids: Vec<_> = edges.iter().map(|e| e.id()).collect();
        let equal = selected.iter().all(|&(_, r)| r == radius);
        stage.set("blend");
        let result = try_fillet_solid_edges_with_radii(&input, &selected, TOL).or_else(|error| {
            if equal {
                if let Some(wire) = as_wire(&edges) {
                    return try_fillet_solid_along_wire(&input, &wire, radius, TOL);
                }
            }
            Err(error)
        });
        (ids, result)
    };
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
    let prepared = prepare_for_step(&result.solid.compress(), step_tol);
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

#[cfg(test)]
#[path = "../tests/fixtures/unequal-radii/model.rs"]
mod unequal_fixture;

#[cfg(test)]
mod unequal_radius_tests {
    use super::*;

    fn check(name: &str) {
        let case = unequal_fixture::case(name);
        let directory = std::env::temp_dir().join(format!(
            "truck-unequal-replay-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        std::fs::write(format!("{prefix}.input.step"), case.step).unwrap();
        let selection: Vec<_> = case
            .edges
            .iter()
            .map(|(a, b, r)| json!({"radius":r,"points":[[a.x,a.y,a.z],[b.x,b.y,b.z]]}))
            .collect();
        let task = json!({"operation":"fillet","selection":selection});
        std::fs::write(format!("{prefix}.input.json"), task.to_string()).unwrap();
        let result = replay(prefix, &Cell::new("start")).unwrap();
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        for field in [
            "input_unchanged",
            "geometry_consistent",
            "mesh_closed",
            "step_prepared",
        ] {
            assert_eq!(result[field], true, "{field}: {result}");
        }
        assert!(
            (result["mesh_volume"].as_f64().unwrap() - case.volume).abs() < 0.001 * case.volume
        );
        assert_eq!(
            std::fs::read_to_string(format!("{prefix}.input.step")).unwrap(),
            case.step
        );
        let mut duplicate = task.clone();
        let same = duplicate["selection"][0].clone();
        duplicate["selection"].as_array_mut().unwrap().push(same);
        std::fs::write(format!("{prefix}.input.json"), duplicate.to_string()).unwrap();
        assert_eq!(replay(prefix, &Cell::new("start")).unwrap(), result);
        duplicate["selection"][2]["radius"] = json!(case.edges[0].2 + 1.);
        std::fs::write(format!("{prefix}.input.json"), duplicate.to_string()).unwrap();
        assert!(replay(prefix, &Cell::new("start"))
            .unwrap_err()
            .contains("Conflicting radii"));
        let mut invalid = task;
        invalid["selection"][1]["radius"] = json!(0.);
        std::fs::write(format!("{prefix}.input.json"), invalid.to_string()).unwrap();
        assert!(replay(prefix, &Cell::new("start"))
            .unwrap_err()
            .contains("Invalid captured radius"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unequal_box_radii_reach_the_solid_solver() { check("A2"); }
    #[test]
    fn unequal_wedge_radii_reach_the_solid_solver() { check("B5"); }
    #[test]
    fn unequal_cylindrical_lip_radii_reach_the_solid_solver() { check("C4"); }
}

#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod replay_common;

#[cfg(test)]
mod chamfer_replay_tests {
    use super::*;

    fn check(name: &str, parameters: Value, d0: f64, d1: f64) {
        let directory = std::env::temp_dir().join(format!(
            "truck-chamfer-replay-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        let step = unequal_fixture::case("A2").step;
        std::fs::write(format!("{prefix}.input.step"), step).unwrap();
        let mut chosen = parameters;
        chosen["points"] = json!([[0., 0., 30.], [10., 0., 30.]]);
        let task = json!({"operation":"chamfer","selection":[chosen.clone()]});
        std::fs::write(format!("{prefix}.input.json"), task.to_string()).unwrap();
        let result = replay(prefix, &Cell::new("start")).unwrap();
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        for field in [
            "input_unchanged",
            "geometry_consistent",
            "mesh_closed",
            "step_prepared",
        ] {
            assert_eq!(result[field], true, "{field}: {result}");
        }
        assert!((result["mesh_volume"].as_f64().unwrap() - (6000. - 5. * d0 * d1)).abs() < 1e-6);
        let output = import(&format!("{prefix}.truck.step")).unwrap();
        replay_common::assert_solid(&output, 6000. - 5. * d0 * d1, &[0], TOL);
        replay_common::blend::assert_step(&output, 6000. - 5. * d0 * d1, TOL);
        let a = Point3::new(5., 0., 30. - d0);
        let b = Point3::new(5., d1, 30.);
        let expected = Vector3::new(0., -d0, d1).normalize();
        assert!(output.face_iter().any(|f| {
            let s = f.oriented_surface();
            let pa = s.search_parameter(a, None, 20);
            let pb = s.search_parameter(b, None, 20);
            pa.zip(pb)
                .is_some_and(|((u, v), _)| s.normal(u, v).dot(expected) > 1. - 1e-6)
        }));
        assert_eq!(
            std::fs::read_to_string(format!("{prefix}.input.step")).unwrap(),
            step
        );
        let mut duplicate = task.clone();
        duplicate["selection"].as_array_mut().unwrap().push(chosen);
        std::fs::write(format!("{prefix}.input.json"), duplicate.to_string()).unwrap();
        assert_eq!(replay(prefix, &Cell::new("start")).unwrap(), result);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn equal_distance_chamfers_use_physical_edges() {
        check(
            "equal",
            json!({"mode":"equal-distance","distance":2.}),
            2.,
            2.,
        );
    }

    #[test]
    fn unequal_chamfer_distances_preserve_the_reference_face() {
        for normal in [[0., -1., 0.], [0., 1., 0.]] {
            check(
                if normal[1] < 0. {
                    "unequal"
                } else {
                    "inverted-plane"
                },
                json!({"mode":"two-distances","distances":[2.,3.],
                    "reference_plane":{"origin":[0.,0.,0.],"normal":normal}}),
                2.,
                3.,
            );
        }
        check(
            "other-side",
            json!({"mode":"two-distances","distances":[2.,3.],
            "reference_plane":{"origin":[0.,0.,30.],"normal":[0.,0.,1.]}}),
            3.,
            2.,
        );
    }

    #[test]
    fn perpendicular_distance_angle_chamfers_preserve_the_angle() {
        check(
            "angle",
            json!({"mode":"distance-angle","distance":2.,"angle_degrees":30.,
            "reference_plane":{"origin":[0.,0.,0.],"normal":[0.,-1.,0.]}}),
            2.,
            2. * (std::f64::consts::PI / 6.).tan(),
        );
    }
    fn check_wedge_angle(acute: bool) {
        let directory = std::env::temp_dir().join(format!(
            "truck-chamfer-wedge-{}-{acute}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        std::fs::write(
            format!("{prefix}.input.step"),
            unequal_fixture::case("B5").step,
        )
        .unwrap();
        let base = import(&format!("{prefix}.input.step")).unwrap();
        let (corner, rays, angle): (Point3, [Vector3; 2], f64) = if acute {
            (
                Point3::new(5., 1., 0.),
                [-Vector3::unit_x(), Vector3::new(-4., -1., 0.).normalize()],
                30.,
            )
        } else {
            (
                Point3::new(1., 0., 0.),
                [Vector3::new(4., 1., 0.).normalize(), -Vector3::unit_x()],
                5.,
            )
        };
        let beta = rays[0].dot(rays[1]).acos();
        let alpha = angle.to_radians();
        let d = 0.1;
        let other = d * alpha.sin() / (beta + alpha).sin();
        let volume = 15. - 2.5 * d * other * beta.sin();
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(12., -9., 7.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
        ] {
            let input = builder::transformed(&base, transform);
            let prepared = prepare_for_step(&input.compress(), TOL / 20.).unwrap();
            let step = StepDisplay::new(
                Default::default(),
                StepDesign::from_model(StepModel::from(&prepared)),
            )
            .to_string();
            std::fs::write(format!("{prefix}.input.step"), &step).unwrap();
            for side in 0..2 {
                for sign in [-1., 1.] {
                    let point = transform.transform_point(corner);
                    let end = transform.transform_point(corner + 5. * Vector3::unit_z());
                    let normal =
                        transform.transform_vector(rays[side].cross(Vector3::unit_z()) * sign);
                    let chosen = json!({"mode":"distance-angle", "distance":d, "angle_degrees":angle,
                        "reference_plane":{"origin":[point.x,point.y,point.z], "normal":[normal.x,normal.y,normal.z]},
                        "points":[[point.x,point.y,point.z],[end.x,end.y,end.z]]});
                    std::fs::write(
                        format!("{prefix}.input.json"),
                        json!({"operation":"chamfer", "selection":[chosen]}).to_string(),
                    )
                    .unwrap();
                    let result = replay(prefix, &Cell::new("start")).unwrap();
                    assert_eq!(result["status"], "operation-succeeded", "{result}");
                    for field in [
                        "input_unchanged",
                        "geometry_consistent",
                        "mesh_closed",
                        "step_prepared",
                    ] {
                        assert_eq!(result[field], true, "{field}: {result}");
                    }
                    assert!((result["mesh_volume"].as_f64().unwrap() - volume).abs() < 1e-6);
                    let output = import(&format!("{prefix}.truck.step")).unwrap();
                    replay_common::assert_solid(&output, volume, &[0], TOL);
                    replay_common::blend::assert_step(&output, volume, TOL);
                    let a = transform
                        .transform_point(corner + d * rays[side] + 2.5 * Vector3::unit_z());
                    let b = transform
                        .transform_point(corner + other * rays[1 - side] + 2.5 * Vector3::unit_z());
                    let mut expected = (b - a)
                        .cross(transform.transform_vector(Vector3::unit_z()))
                        .normalize();
                    if expected.dot(point + 2.5 * transform.transform_vector(Vector3::unit_z()) - a)
                        < 0.
                    {
                        expected = -expected;
                    }
                    assert!(output.face_iter().any(|f| {
                        let surface = f.oriented_surface();
                        surface
                            .search_parameter(a, None, 20)
                            .zip(surface.search_parameter(b, None, 20))
                            .is_some_and(|((u, v), _)| {
                                surface.normal(u, v).dot(expected) > 1. - 1e-6
                            })
                    }));
                    assert_eq!(
                        std::fs::read_to_string(format!("{prefix}.input.step")).unwrap(),
                        step
                    );
                }
            }
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn acute_distance_angle_chamfers_preserve_the_angle() { check_wedge_angle(true); }

    #[test]
    fn obtuse_distance_angle_chamfers_preserve_the_angle() { check_wedge_angle(false); }

    #[test]
    fn impossible_distance_angle_chamfers_do_not_construct() {
        let directory =
            std::env::temp_dir().join(format!("truck-chamfer-impossible-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        std::fs::write(
            format!("{prefix}.input.step"),
            unequal_fixture::case("B5").step,
        )
        .unwrap();
        for angle in [0.25_f64.atan().to_degrees(), 30.] {
            let chosen = json!({"mode":"distance-angle", "distance":0.1, "angle_degrees":angle,
                "reference_plane":{"origin":[0.,0.,0.], "normal":[0.,-1.,0.]},
                "points":[[1.,0.,0.],[1.,0.,5.]]});
            std::fs::write(
                format!("{prefix}.input.json"),
                json!({"operation":"chamfer", "selection":[chosen]}).to_string(),
            )
            .unwrap();
            let result = replay(prefix, &Cell::new("start")).unwrap();
            assert_eq!(result["status"], "harness-error", "{result}");
            assert!(!Path::new(&format!("{prefix}.truck.step")).exists());
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_ambiguous_and_unsupported_chamfer_intents_do_not_construct() {
        let directory =
            std::env::temp_dir().join(format!("truck-chamfer-invalid-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        std::fs::write(
            format!("{prefix}.input.step"),
            unequal_fixture::case("A2").step,
        )
        .unwrap();
        let equal =
            json!({"mode":"equal-distance","distance":2.,"points":[[0.,0.,30.],[10.,0.,30.]]});
        let mut invalid = equal.clone();
        invalid["distance"] = json!(-1.);
        let mut conflict = equal.clone();
        conflict["distance"] = json!(3.);
        let mut missing = equal.clone();
        missing["mode"] = json!("two-distances");
        missing["distances"] = json!([2., 3.]);
        let mut wrong = missing.clone();
        wrong["reference_plane"] = json!({"origin":[0.,20.,0.],"normal":[0.,1.,0.]});
        let mut angle = equal.clone();
        angle["mode"] = json!("distance-angle");
        angle["angle_degrees"] = json!(90.);
        for (selection, status) in [
            (json!([invalid]), "harness-error"),
            (json!([equal, conflict]), "harness-error"),
            (json!([missing]), "unhandled-intent"),
            (json!([wrong]), "selection-unmatched"),
            (json!([angle]), "harness-error"),
        ] {
            std::fs::write(
                format!("{prefix}.input.json"),
                json!({"operation":"chamfer","selection":selection}).to_string(),
            )
            .unwrap();
            let result = replay(prefix, &Cell::new("start")).unwrap();
            assert_eq!(result["status"], status, "{result}");
            assert!(!Path::new(&format!("{prefix}.truck.step")).exists());
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod native_contour_tests {
    use super::*;

    fn task() -> Value {
        let mut original: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/chamfer-contours/A4-seed.json"
        ))
        .unwrap();
        let expanded: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/chamfer-contours/A4-expanded.json"
        ))
        .unwrap();
        original["schema_version"] = json!(2);
        original["selection"][0]["native_applied"] = json!(true);
        original["selection"][0]["ignored_reason"] = Value::Null;
        original["selection"][0]["contour_edges"] = expanded["selection"].clone();
        original
    }

    fn run(name: &str, task: &Value) -> Value {
        let directory = std::env::temp_dir().join(format!(
            "truck-native-contour-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        let step = include_str!("../tests/fixtures/chamfer-contours/A4.step");
        std::fs::write(format!("{prefix}.input.step"), step).unwrap();
        std::fs::write(format!("{prefix}.input.json"), task.to_string()).unwrap();
        let result = replay(prefix, &Cell::new("start")).unwrap();
        assert_eq!(
            std::fs::read_to_string(format!("{prefix}.input.step")).unwrap(),
            step
        );
        if result["status"] == "operation-succeeded" && result["selected_edges"] == 3 {
            let output = import(&format!("{prefix}.truck.step")).unwrap();
            replay_common::assert_solid(&output, 0.5630665232959563, &[0], TOL);
            replay_common::blend::assert_step(&output, 0.5630665232959563, TOL);
        }
        std::fs::remove_dir_all(directory).unwrap();
        result
    }

    #[test]
    fn explicit_complete_contour_preserves_the_step_round_trip() {
        let expanded: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/chamfer-contours/A4-expanded.json"
        ))
        .unwrap();
        let result = run("explicit-round-trip", &expanded);
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        assert_eq!(result["selected_edges"], 3, "{result}");
    }

    #[test]
    fn native_contour_scope_expands_the_seed_to_all_three_edges() {
        let result = run("complete", &task());
        assert_eq!(result["selection_scope"], "native-contours", "{result}");
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        assert_eq!(result["selected_edges"], 3, "{result}");
        for field in [
            "input_unchanged",
            "geometry_consistent",
            "mesh_closed",
            "step_prepared",
        ] {
            assert_eq!(result[field], true, "{result}");
        }
    }

    #[test]
    fn native_contour_scope_cannot_fall_back_to_a_seed() {
        for field in ["native_applied", "contour_edges"] {
            let mut task = task();
            task["selection"][0].as_object_mut().unwrap().remove(field);
            assert_eq!(run(field, &task)["status"], "harness-error");
        }
        let mut task = task();
        task["selection"][0]["contour_edges"] = json!([]);
        assert_eq!(run("empty", &task)["status"], "harness-error");
    }

    #[test]
    fn native_contour_scope_requires_seed_membership() {
        let mut task = task();
        task["selection"][0]["contour_edges"]
            .as_array_mut()
            .unwrap()
            .pop();
        let result = run("missing-seed", &task);
        assert_eq!(result["status"], "harness-error", "{result}");
    }

    #[test]
    fn native_first_assignment_ignores_later_contour_parameters() {
        let mut task = task();
        let mut ignored = task["selection"][0].clone();
        ignored["distance"] = json!(-2.);
        ignored["native_applied"] = json!(false);
        ignored["ignored_reason"] = json!("already-selected");
        ignored["contour_edges"] = json!([]);
        task["selection"].as_array_mut().unwrap().push(ignored);
        let result = run("ignored-duplicate", &task);
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        assert_eq!(result["selected_edges"], 3, "{result}");
    }

    #[test]
    fn native_contour_assignments_deduplicate_and_reject_conflicts() {
        let mut task = task();
        let member = task["selection"][0]["contour_edges"][0].clone();
        task["selection"][0]["contour_edges"]
            .as_array_mut()
            .unwrap()
            .push(member);
        let duplicate = task["selection"][0].clone();
        task["selection"].as_array_mut().unwrap().push(duplicate);
        let result = run("active-duplicate", &task);
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        assert_eq!(result["selected_edges"], 3, "{result}");
        task["selection"][1]["distance"] = json!(0.02);
        let result = run("active-conflict", &task);
        assert_eq!(result["status"], "harness-error", "{result}");
    }

    #[test]
    fn ignored_native_assignments_require_consistent_scope() {
        for reason in [
            Value::Null,
            json!("invented-reason"),
            json!("already-selected"),
        ] {
            let mut task = task();
            task["selection"][0]["native_applied"] = json!(false);
            task["selection"][0]["ignored_reason"] = reason;
            let result = run("contradictory-ignored", &task);
            assert_eq!(result["status"], "harness-error", "{result}");
        }
        let mut task = task();
        task["selection"][0]["native_applied"] = json!(false);
        task["selection"][0]["ignored_reason"] = json!("invalid-parameters");
        task["selection"][0]["contour_edges"] = json!([]);
        task["selection"][0]["distance"] = json!(0.);
        let result = run("no-effective-selection", &task);
        assert_eq!(result["status"], "unhandled-intent", "{result}");
    }

    #[test]
    fn unknown_capture_schemas_do_not_replay() {
        let mut task = task();
        task["schema_version"] = json!(3);
        let result = run("unknown-schema", &task);
        assert_eq!(result["status"], "harness-error", "{result}");
    }

    #[test]
    fn legacy_native_seeds_are_explicitly_labeled() {
        let mut task = task();
        task["schema_version"] = json!(1);
        for field in ["native_applied", "ignored_reason", "contour_edges"] {
            task["selection"][0].as_object_mut().unwrap().remove(field);
        }
        let result = run("legacy", &task);
        assert_eq!(result["selection_scope"], "seed-only", "{result}");
        assert_ne!(
            result["selected_edges"], 3,
            "Legacy replay cannot silently expand its seed"
        );
    }
}

#[cfg(test)]
mod fillet_step_tolerance_tests {
    use super::*;

    #[test]
    fn fillet_replay_preserves_its_supported_step_fitting_budget() {
        let directory =
            std::env::temp_dir().join(format!("truck-fillet-export-budget-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let prefix = directory.join("001");
        let prefix = prefix.to_str().unwrap();
        let step = include_str!("../tests/fixtures/replay-step-tolerances/D4.step");
        let task = include_str!("../tests/fixtures/replay-step-tolerances/D4.json");
        std::fs::write(format!("{prefix}.input.step"), step).unwrap();
        std::fs::write(format!("{prefix}.input.json"), task).unwrap();
        let result = replay(prefix, &Cell::new("start")).unwrap();
        assert_eq!(result["status"], "operation-succeeded", "{result}");
        assert_eq!(result["selection_scope"], "seed-only", "{result}");
        assert_eq!(result["selected_edges"], 1, "{result}");
        for field in [
            "input_unchanged",
            "geometry_consistent",
            "mesh_closed",
            "step_prepared",
        ] {
            assert_eq!(result[field], true, "{field}: {result}");
        }
        assert_eq!(
            std::fs::read_to_string(format!("{prefix}.input.step")).unwrap(),
            step
        );
        let output = import(&format!("{prefix}.truck.step")).unwrap();
        replay_common::assert_topology(&output, &[0]);
        replay_common::blend::assert_step(&output, 3435781.0157892024, TOL);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
