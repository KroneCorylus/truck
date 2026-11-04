mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

const R: f64 = 18.75;
const CY: f64 = 2.39999985694885;
const CX: f64 = 19.0998036818876;
const LEFT: f64 = 16.8180108173766;
const RIGHT: f64 = 25.443289939164;
const LOW: f64 = 0.794433603322526;
const HIGH: f64 = 4.05306587080493;
const LENGTH: f64 = 10.75;

fn fixture() -> Solid {
    let d = CX.hypot(CY);
    let p = [
        (LEFT, LOW),
        (LEFT, HIGH),
        ((R * R - HIGH * HIGH).sqrt(), HIGH),
        (R * CX / d, R * CY / d),
        (CX, CY - 0.5),
        (RIGHT, CY - 0.5),
        (RIGHT, LOW),
    ]
    .map(|(x, y)| Vertex::new(Point3::new(x, y, 0.)));
    let angle = CY.atan2(CX);
    let outer_angle = (angle + (HIGH / R).asin()) / 2.;
    let inner_angle = (angle - std::f64::consts::PI - std::f64::consts::FRAC_PI_2) / 2.;
    let edges: Vec<Edge> = (0..7)
        .map(|i| match i {
            2 => builder::circle_arc(
                &p[2],
                &p[3],
                Point3::new(R * outer_angle.cos(), R * outer_angle.sin(), 0.),
            ),
            3 => builder::circle_arc(
                &p[3],
                &p[4],
                Point3::new(
                    CX + 0.5 * inner_angle.cos(),
                    CY + 0.5 * inner_angle.sin(),
                    0.,
                ),
            ),
            _ => builder::line(&p[i], &p[(i + 1) % 7]),
        })
        .collect();
    let mut face = builder::try_attach_plane(vec![Wire::from(edges)]).unwrap();
    face.invert();
    builder::tsweep(&face, Vector3::unit_z() * LENGTH)
}

fn selections(input: &Solid) -> Vec<(truck_topology::EdgeID<Curve>, [f64; 2])> {
    let mut seen = HashSet::new();
    input
        .edge_iter()
        .filter_map(|edge| {
            let p = edge.front().point();
            let q = edge.back().point();
            if (p.z - q.z).abs() > 1e-9 || !seen.insert(edge.id()) {
                return None;
            }
            let top = p.z > LENGTH / 2.;
            let d = if (p.x - LEFT).abs() < 1e-9 && (q.x - LEFT).abs() < 1e-9 {
                0.2
            } else if (p.x - RIGHT).abs() < 1e-9 && (q.x - RIGHT).abs() < 1e-9 {
                if top {
                    0.2
                } else {
                    0.4
                }
            } else if (p.y - LOW).abs() < 1e-9 && (q.y - LOW).abs() < 1e-9 {
                0.3
            } else if (p.y - HIGH).abs() < 1e-9 && (q.y - HIGH).abs() < 1e-9 {
                0.6
            } else if top {
                0.5
            } else {
                0.4
            };
            Some((edge.id(), [d; 2]))
        })
        .collect()
}

#[test]
fn procedural_c2_source() {
    let source = fixture();
    common::assert_counts(&source, 14, 21, 9);
    common::assert_solid(&source, 141.91202608870574, &[0], 0.001);
    assert_eq!(selections(&source).len(), 14);
}

#[test]
fn procedural_c2_both_caps() {
    let source = fixture();
    let before = serde_json::to_string(&source.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(&source, &selections(&source), &[], 0.001);
    assert_eq!(before, serde_json::to_string(&source.compress()).unwrap());
    let result = result.expect("bounded equal-distance C2 extrusion must construct");
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/c2_extrusion_fixture.json"
    ))
    .unwrap();
    let volume = oracle["volume_midpoint"].as_f64().unwrap();
    assert!(
        oracle["volume_upper"].as_f64().unwrap() - oracle["volume_lower"].as_f64().unwrap()
            < 0.0001
    );
    for entry in oracle["contacts"].as_array().unwrap() {
        for point in &entry["points"].as_array().unwrap()[2..5] {
            let p = Point3::new(
                point[0].as_f64().unwrap(),
                point[1].as_f64().unwrap(),
                point[2].as_f64().unwrap(),
            );
            assert!(
                result.solid.edge_iter().any(|edge| {
                    let curve = edge.curve();
                    let (lo, hi) = curve.range_tuple();
                    curve
                        .search_nearest_parameter(p, None, 100)
                        .is_some_and(|t| {
                            lo - 1e-10 <= t && t <= hi + 1e-10 && curve.subs(t).distance(p) < 1e-8
                        })
                }),
                "missing section contact {p:?}"
            );
        }
    }
    for original in source.face_iter() {
        let support = serde_json::to_string(&original.oriented_surface()).unwrap();
        assert!(result.solid.face_iter().any(|face| serde_json::to_string(
            &face.oriented_surface()
        )
        .unwrap()
            == support));
    }
    let mut uses = std::collections::HashMap::new();
    for edge in result.solid.edge_iter() {
        uses.entry(edge.id())
            .or_insert_with(Vec::new)
            .push(edge.orientation());
    }
    assert!(uses.values().all(|u| u.len() == 2 && u[0] != u[1]));
    common::assert_solid(&result.solid, volume, &[0], 0.001);
    common::blend::assert_step(&result.solid, volume, 0.001);
}

#[test]
fn c2_conics_and_history() {
    let input = fixture();
    let result =
        try_chamfer_solid_edges_with_distances(&input, &selections(&input), &[], 0.001).unwrap();
    assert_eq!(result.modified_faces.len(), 9);
    assert_eq!(result.generated_faces.len(), 14);
    for cap in 0..2 {
        let chain: f64 = if cap == 0 { 0.4 } else { 0.5 };
        for t in [0.075, 0.25, 0.35, (chain + 0.6) / 2.] {
            let ro = R - (chain - t).max(0.);
            let h = HIGH - (0.6 - t).max(0.);
            let q = Point3::new(
                (ro * ro - h * h).sqrt(),
                h,
                if cap == 0 { t } else { LENGTH - t },
            );
            assert!(
                result.solid.edge_iter().any(|e| {
                    let c = e.curve();
                    let (a, b) = c.range_tuple();
                    c.search_parameter(q, None, 100)
                        .is_some_and(|u| u >= a && u <= b && c.subs(u).distance(q) < 1e-8)
                }),
                "missing true conic interior {q:?}"
            );
        }
    }
}

#[test]
fn c2_transform_order_and_scale() {
    let original = fixture();
    let source_selections = selections(&original);
    for scale in [0.25, 1., 4.] {
        let transform = Matrix4::from_translation(Vector3::new(7., -9., 13.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.43))
            * Matrix4::from_scale(scale);
        let input = builder::transformed(&original, transform);
        let mut selected = Vec::new();
        for (id, d) in &source_selections {
            let e = original.edge_iter().find(|e| e.id() == *id).unwrap();
            let a = transform.transform_point(e.front().point());
            let b = transform.transform_point(e.back().point());
            let new = input
                .edge_iter()
                .find(|e| {
                    e.front().point().distance(a) < 1e-8 && e.back().point().distance(b) < 1e-8
                })
                .unwrap();
            selected.push((new.id(), d.map(|d| d * scale)));
        }
        selected.reverse();
        let mut shell = input.boundaries()[0].clone();
        shell.reverse();
        let input = Solid::new(vec![shell]);
        let result =
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale).unwrap();
        common::assert_solid(
            &result.solid,
            138.66070708439852 * scale.powi(3),
            &[0],
            0.001 * scale,
        );
    }
}

#[test]
fn c2_invalid_setbacks_preserve_input() {
    let input = fixture();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for mode in 0..4 {
        let mut selected = selections(&input);
        match mode {
            0 => selected[0].1[1] *= 1.1,
            1 => {
                for (_, d) in &mut selected {
                    *d = [6.; 2];
                }
            }
            2 => {
                for (_, d) in &mut selected {
                    if d[0] == 0.6 {
                        *d = [3.; 2];
                    }
                }
            }
            _ => {
                let (_, d) = selected.iter_mut().find(|(_, d)| d[0] == 0.5).unwrap();
                *d = [0.51; 2];
            }
        }
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err(),
            "mode {mode}"
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn c2_explicit_preferences_are_not_ignored() {
    use truck_shapeops::fillet::ChamferCornerPreference;
    let input = fixture();
    let selected = selections(&input);
    let edge = input.edge_iter().find(|e| e.id() == selected[0].0).unwrap();
    let preference = ChamferCornerPreference {
        vertex: edge.front().id(),
        edge: edge.id(),
    };
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(
        try_chamfer_solid_edges_with_distances(&input, &selected, &[preference], 0.001).is_err()
    );
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn c2_large_coordinate_budget_rejects() {
    let input = builder::translated(&fixture(), Vector3::new(1e12, 0., 0.));
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| (e.front().point().z - e.back().point().z).abs() < 1e-9)
        .map(|e| (e.id(), [0.4; 2]))
        .collect::<std::collections::HashMap<_, _>>()
        .into_iter()
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[cfg(feature = "step-test")]
#[test]
fn imported_c2_both_caps() {
    use truck_stepio::r#in::Table;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/c2_extrusion_fixture.json"
    ))
    .unwrap();
    let table = Table::from_step(fixture["source_step"].as_str().unwrap()).unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (compressed, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty());
    let input = Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result =
        try_chamfer_solid_edges_with_distances(&input, &selections(&input), &[], 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    common::assert_solid(
        &result.solid,
        fixture["volume_midpoint"].as_f64().unwrap(),
        &[0],
        0.001,
    );
    common::blend::assert_step(
        &result.solid,
        fixture["volume_midpoint"].as_f64().unwrap(),
        0.001,
    );
}

#[test]
fn c2_reversed_stored_surfaces_and_invalid_supports() {
    for mode in 0..3 {
        let original = fixture();
        let selected = selections(&original);
        let shell: Shell = original.boundaries()[0]
            .iter()
            .map(|face| {
                let mut surface = face.oriented_surface();
                if mode == 0 {
                    surface.invert();
                    return Face::new(
                        face.boundaries().iter().map(Wire::inverse).collect(),
                        surface,
                    )
                    .inverse();
                }
                if matches!(
                    surface.elementary(),
                    Some((Elementary::Cylinder { .. }, false))
                ) {
                    if mode == 1 {
                        surface.invert();
                    } else {
                        surface.transform_by(Matrix4::from_angle_x(Rad(0.01)));
                    }
                    Face::new(face.boundaries(), surface)
                } else {
                    face.clone()
                }
            })
            .collect();
        let input = Solid::new(vec![shell]);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if mode == 0 {
            common::assert_solid(&result.unwrap().solid, 138.66070708439852, &[0], 0.001);
        } else {
            assert!(result.is_err(), "invalid support mode {mode}");
        }
    }
}
