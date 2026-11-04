mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

fn rounded_rectangle(hole: bool) -> Solid { rounded_rectangle_shift(hole, 0., 0.) }

fn rounded_rectangle_shift(hole: bool, shift: f64, slant: f64) -> Solid {
    let p = [
        (-1., -1.),
        (0.1 + slant, -1.),
        (0.1, 0.),
        (0., 0.1),
        (-1., 0.1),
    ]
    .map(|(x, y)| Vertex::new(Point3::new(x, y, 0.)));
    let arc = builder::rsweep(
        &p[2],
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::FRAC_PI_2),
        1,
    );
    let edge = Edge::new(&p[2], &p[3], arc[0].curve());
    let wire = Wire::from(vec![
        builder::line(&p[0], &p[1]),
        builder::line(&p[1], &p[2]),
        edge,
        builder::line(&p[3], &p[4]),
        builder::line(&p[4], &p[0]),
    ]);
    let mut loops = vec![wire];
    if hole {
        let p = [(-0.7, -0.7), (-0.7, -0.5), (-0.5, -0.5), (-0.5, -0.7)]
            .map(|(x, y)| Vertex::new(Point3::new(x + shift, y + shift, 0.)));
        loops.push(
            (0..4)
                .map(|i| builder::line(&p[i], &p[(i + 1) % 4]))
                .collect(),
        );
    }
    builder::tsweep(
        &builder::try_attach_plane(loops).unwrap(),
        Vector3::unit_z(),
    )
}

fn selection(
    input: &Solid,
    params: [f64; 6],
    both: bool,
    inverse: Matrix4,
    scale: f64,
) -> Vec<(EdgeID, [f64; 2])> {
    let [hx, hy, by, ay, bx, ax] = params;
    let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
    let mut seen = std::collections::HashSet::new();
    input
        .edge_iter()
        .filter_map(|edge| {
            if !seen.insert(edge.id()) {
                return None;
            }
            let p = inverse.transform_point(edge.front().point());
            let q = inverse.transform_point(edge.back().point());
            let vertical = near(p.x, -1.) && near(q.x, -1.) && near(p.y, -1.) && near(q.y, -1.);
            let cap = near(p.z, q.z)
                && (near(p.z, 1.) || both && near(p.z, 0.))
                && (p.x >= 0. || p.y >= 0. || near(p.x, -1.) || near(p.y, -1.));
            if !vertical && !cap {
                return None;
            }
            let faces: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .collect();
            Some((
                edge.id(),
                std::array::from_fn::<_, 2, _>(|i| {
                    let normal = match faces[i].oriented_surface() {
                        Surface::Plane(plane) => {
                            inverse.transform_vector(plane.normal()).normalize()
                        }
                        _ => Vector3::zero(),
                    };
                    if vertical {
                        if normal.x.abs() > 0.9 {
                            hy
                        } else {
                            hx
                        }
                    } else if near(p.x, -1.) && near(q.x, -1.) {
                        if normal.z.abs() > 0.9 {
                            bx
                        } else {
                            ax
                        }
                    } else if near(p.y, -1.) && near(q.y, -1.) {
                        if normal.z.abs() > 0.9 {
                            by
                        } else {
                            ay
                        }
                    } else {
                        0.03
                    }
                }),
            ))
        })
        .map(|(id, d)| (id, d.map(|v| v * scale)))
        .collect()
}

fn check(params: [f64; 6], expected: f64, hole: bool, both: bool) {
    let input = rounded_rectangle(hole);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let selected = selection(&input, params, both, Matrix4::identity(), 1.);
    assert_eq!(selected.len(), if both { 11 } else { 6 });
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    let output = result.expect("A5 certified planar corner with tangent cap must construct");
    let [hx, hy, by, ay, bx, ax] = params;
    let [a, b, c, d] = [
        Point3::new(0., hy, ax),
        Point3::new(bx, hy, 0.),
        Point3::new(hx, by, 0.),
        Point3::new(hx, 0., ay),
    ];
    for bottom in [false, true].into_iter().filter(|bottom| !bottom || both) {
        for u in [0.2, 0.5, 0.8] {
            for v in [0.2, 0.5, 0.8] {
                let p = (1. - u) * (1. - v) * a.to_vec()
                    + u * (1. - v) * b.to_vec()
                    + u * v * c.to_vec()
                    + (1. - u) * v * d.to_vec();
                let n = ((1. - v) * (b - a) + v * (c - d))
                    .cross((1. - u) * (d - a) + u * (c - b))
                    .normalize();
                let point = Point3::new(p.x - 1., p.y - 1., if bottom { p.z } else { 1. - p.z });
                let outward = Vector3::new(n.x, n.y, if bottom { n.z } else { -n.z });
                assert!(output.solid.face_iter().any(|face| {
                    let surface = face.oriented_surface();
                    matches!(surface, Surface::BSplineSurface(_))
                        && surface
                            .search_parameter(point, None, 100)
                            .is_some_and(|uv| {
                                surface.subs(uv.0, uv.1).distance(point) < 1e-8
                                    && surface.normal(uv.0, uv.1).dot(outward) > 1. - 1e-8
                            })
                }));
            }
        }
    }
    if hole {
        for edge in input.edge_iter().filter(|edge| {
            [edge.front(), edge.back()].iter().all(|v| {
                let p = v.point();
                p.x >= -0.7 && p.x <= -0.5 && p.y >= -0.7 && p.y <= -0.5
            })
        }) {
            assert!(output.solid.edge_iter().any(|e| e.id() == edge.id()));
        }
    }

    common::blend::assert_step(
        &output.solid,
        expected - if hole { 0.04 } else { 0. },
        0.001,
    );
    common::assert_solid(
        &output.solid,
        expected - if hole { 0.04 } else { 0. },
        &[usize::from(hole)],
        0.001,
    );
}

#[test]
fn a5_procedural_equal_volume_and_hole() {
    for hole in [false, true] {
        check(
            [0.05, 0.05, 0.03, 0.03, 0.02, 0.02],
            1.2049581977160724,
            hole,
            false,
        );
        check(
            [0.05, 0.05, 0.03, 0.03, 0.02, 0.02],
            1.2033124137981706,
            hole,
            true,
        );
    }
}

#[test]
fn a5_procedural_dd_volume_and_hole() {
    for hole in [false, true] {
        check(
            [0.04, 0.05, 0.03, 0.03, 0.04, 0.02],
            1.2049970310494058,
            hole,
            false,
        );
        check(
            [0.04, 0.05, 0.03, 0.03, 0.04, 0.02],
            1.2031400804648373,
            hole,
            true,
        );
    }
}

#[test]
fn a5_procedural_transform_and_selection_order() {
    for scale in [0.1, 1., 10.] {
        let transform = Matrix4::from_translation(Vector3::new(3., -2., 4.))
            * Matrix4::from_angle_x(Rad(0.7))
            * Matrix4::from_scale(scale);
        let input = builder::transformed(&rounded_rectangle(true), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected = selection(
            &input,
            [0.05, 0.05, 0.03, 0.03, 0.02, 0.02],
            true,
            transform.invert().unwrap(),
            scale,
        );
        for _ in 0..2 {
            let result =
                try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).unwrap();
            common::assert_solid(
                &result.solid,
                (1.2033124137981706 - 0.04) * scale.powi(3),
                &[1],
                0.001,
            );
            selected.reverse();
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn a5_rejects_ambiguous_da_exhausted_span_and_hole_interference() {
    for (params, shift) in [
        (
            [
                0.05 / 3_f64.sqrt(),
                0.05,
                0.03,
                0.03,
                0.02 * 3_f64.sqrt(),
                0.02,
            ],
            0.,
        ),
        ([0.05, 0.05, 0.03, 0.6, 0.02, 0.6], 0.),
        ([0.05, 0.05, 0.03, 0.03, 0.02, 0.02], -0.28),
    ] {
        let input = rounded_rectangle_shift(true, shift, 0.);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let selected = selection(&input, params, true, Matrix4::identity(), 1.);
        assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn a5_explicit_section_preference_is_order_independent() {
    use truck_shapeops::fillet::ChamferCornerPreference;
    for (factor, expected) in [
        (0.98, 1.1636087971315043),
        (1., 1.163594247131504),
        (1.02, 1.163579697131504),
    ] {
        for scale in [0.1, 1., 10.] {
            let transform = Matrix4::from_translation(Vector3::new(3., -2., 4.))
                * Matrix4::from_angle_x(Rad(0.7))
                * Matrix4::from_scale(scale);
            let inverse = transform.invert().unwrap();
            let input = builder::transformed(&rounded_rectangle(true), transform);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut selected = selection(
                &input,
                [0.05 * factor, 0.03, 0.03, 0.03, 0.02, 0.03],
                true,
                inverse,
                scale,
            );
            let edge = input
                .edge_iter()
                .find(|e| {
                    [e.front(), e.back()].iter().all(|v| {
                        let p = inverse.transform_point(v.point());
                        (p.x + 1.).abs() < 1e-8 && (p.y + 1.).abs() < 1e-8
                    })
                })
                .unwrap();
            let mut preferences: Vec<_> = [edge.front(), edge.back()]
                .into_iter()
                .map(|v| ChamferCornerPreference {
                    vertex: v.id(),
                    edge: edge.id(),
                })
                .collect();
            assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
            for order in 0..6 {
                for _ in 0..2 {
                    let result = try_chamfer_solid_edges_with_distances(
                        &input,
                        &selected,
                        &preferences,
                        0.001,
                    )
                    .unwrap();
                    common::assert_solid(&result.solid, expected * scale.powi(3), &[1], 0.001);
                    common::blend::assert_step(&result.solid, expected * scale.powi(3), 0.001);
                    preferences.reverse();
                }
                if order % 2 == 0 {
                    selected.reverse();
                } else {
                    selected.rotate_left(1);
                }
            }
            let first = preferences[0];
            let unsupported = input
                .edge_iter()
                .find(|e| {
                    e.id() != edge.id()
                        && selected.iter().any(|(id, _)| *id == e.id())
                        && [e.front().id(), e.back().id()].contains(&first.vertex)
                })
                .unwrap();
            let nonincident = input
                .edge_iter()
                .find(|e| {
                    selected.iter().any(|(id, _)| *id == e.id())
                        && ![e.front().id(), e.back().id()].contains(&first.vertex)
                })
                .unwrap();
            let unselected = input
                .edge_iter()
                .find(|e| !selected.iter().any(|(id, _)| *id == e.id()))
                .unwrap();
            let stale = Vertex::new(Point3::origin());
            for bad in [
                vec![first, first],
                vec![first],
                vec![ChamferCornerPreference {
                    vertex: first.vertex,
                    edge: nonincident.id(),
                }],
                vec![ChamferCornerPreference {
                    vertex: first.vertex,
                    edge: unselected.id(),
                }],
                vec![
                    ChamferCornerPreference {
                        vertex: first.vertex,
                        edge: unsupported.id(),
                    },
                    preferences[1],
                ],
                vec![ChamferCornerPreference {
                    vertex: stale.id(),
                    edge: edge.id(),
                }],
            ] {
                assert!(
                    try_chamfer_solid_edges_with_distances(&input, &selected, &bad, 0.001).is_err()
                );
            }
            let regular = selection(
                &input,
                [0.05, 0.05, 0.03, 0.03, 0.02, 0.02],
                true,
                inverse,
                scale,
            );
            assert!(
                try_chamfer_solid_edges_with_distances(&input, &regular, &preferences, 0.001)
                    .is_err()
            );
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[cfg(feature = "step-test")]
#[test]
fn a5_whole_input_flux_hole_support_order_and_preference() {
    use truck_shapeops::fillet::ChamferCornerPreference;
    use truck_stepio::r#in::Table;
    let table = Table::from_step(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_assembly_input.step.txt"
    ))
    .unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (compressed, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty());
    let compressed = compressed
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap();
    let original = Solid::extract(compressed).unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_assembly_input.json"
    ))
    .unwrap();
    let preferred: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_assembly_preference.json"
    ))
    .unwrap();
    let point = |value: &serde_json::Value| {
        Point3::new(
            value[0].as_f64().unwrap(),
            value[1].as_f64().unwrap(),
            value[2].as_f64().unwrap(),
        )
    };
    // Exact all-face flux from a5_assembly_material.py, including the inner wire.
    for (mode, volume) in [
        ("equal_dist", 0.5589185890511593),
        ("dist_dist", 0.5586484905043578),
    ] {
        for reverse in [false, true] {
            let input = if reverse {
                Solid::new(vec![original.boundaries()[0]
                    .iter()
                    .rev()
                    .cloned()
                    .collect()])
            } else {
                original.clone()
            };
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut selected = Vec::new();
            for edge in fixture["modes"][mode]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| !e["distances"].is_null())
            {
                let a = point(&edge["ends"][0]);
                let b = point(&edge["ends"][1]);
                let actual = input
                    .edge_iter()
                    .find(|e| {
                        let p = e.front().point();
                        let q = e.back().point();
                        (p.distance(a) < 1e-8 && q.distance(b) < 1e-8)
                            || (p.distance(b) < 1e-8 && q.distance(a) < 1e-8)
                    })
                    .unwrap();
                let adjacent: Vec<_> = input
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == actual.id()))
                    .collect();
                let distances = std::array::from_fn(|side| {
                    let normal = match adjacent[side].oriented_surface() {
                        Surface::Plane(p) => Some(p.normal()),
                        _ => None,
                    };
                    let i = (0..2)
                        .find(|&i| {
                            let expected = &fixture["faces"]
                                [edge["faces"][i].as_u64().unwrap() as usize]["normal"];
                            match normal {
                                Some(n) => {
                                    !expected.is_null()
                                        && n.dot(point(expected).to_vec()) > 1. - 1e-8
                                }
                                None => expected.is_null(),
                            }
                        })
                        .unwrap();
                    edge["distances"][i].as_f64().unwrap()
                });
                selected.push((actual.id(), distances));
            }
            assert_eq!(selected.len(), 21);
            if reverse {
                selected.reverse();
            }
            let v = input
                .vertex_iter()
                .find(|v| v.point().distance(point(&preferred["vertex"])) < 1e-8)
                .unwrap();
            let e = input
                .edge_iter()
                .find(|e| {
                    let a = point(&preferred["edge"]["points"][0]);
                    let b = point(&preferred["edge"]["points"][1]);
                    (e.front().point().distance(a) < 1e-8 && e.back().point().distance(b) < 1e-8)
                        || (e.front().point().distance(b) < 1e-8
                            && e.back().point().distance(a) < 1e-8)
                })
                .unwrap();
            let prefs = if mode == "dist_dist" {
                assert!(
                    try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err()
                );
                vec![ChamferCornerPreference {
                    vertex: v.id(),
                    edge: e.id(),
                }]
            } else {
                vec![]
            };
            if mode == "dist_dist" {
                let q = point(&fixture["modes"][mode][22]["ends"][0]);
                let r = point(&fixture["modes"][mode][22]["ends"][1]);
                let triangle = input
                    .edge_iter()
                    .find(|edge| {
                        (edge.front().point().distance(q) < 1e-8
                            && edge.back().point().distance(r) < 1e-8)
                            || (edge.front().point().distance(r) < 1e-8
                                && edge.back().point().distance(q) < 1e-8)
                    })
                    .unwrap();
                let unsupported = [ChamferCornerPreference {
                    vertex: v.id(),
                    edge: triangle.id(),
                }];
                let error =
                    try_chamfer_solid_edges_with_distances(&input, &selected, &unsupported, 0.001)
                        .unwrap_err();
                assert_eq!(error.stage, "a5_corner_preference_domain");
            }
            let output =
                try_chamfer_solid_edges_with_distances(&input, &selected, &prefs, 0.001).unwrap();
            common::assert_solid(&output.solid, volume, &[0], 0.001);
            common::blend::assert_step(&output.solid, volume, 0.001);
            for id in [0, 1, 2, 3] {
                let a = point(&fixture["modes"][mode][id]["ends"][0]);
                let b = point(&fixture["modes"][mode][id]["ends"][1]);
                let edge = input
                    .edge_iter()
                    .find(|e| {
                        (e.front().point().distance(a) < 1e-8
                            && e.back().point().distance(b) < 1e-8)
                            || (e.front().point().distance(b) < 1e-8
                                && e.back().point().distance(a) < 1e-8)
                    })
                    .unwrap();
                assert!(output.solid.edge_iter().any(|e| e.id() == edge.id()));
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn a5_da_whole_input_explicit_signed_material() {
    use truck_shapeops::fillet::ChamferCornerPreference;
    use truck_stepio::r#in::Table;
    let table = Table::from_step(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_assembly_input.step.txt"
    ))
    .unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (compressed, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty());
    let compressed = compressed
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap();
    let original = Solid::extract(compressed).unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_da_assembly_input.json"
    ))
    .unwrap();
    let preferred: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/a5_da_assembly_preference.json"
    ))
    .unwrap();
    let point = |value: &serde_json::Value| {
        Point3::new(
            value[0].as_f64().unwrap(),
            value[1].as_f64().unwrap(),
            value[2].as_f64().unwrap(),
        )
    };
    // Exact all-face flux from a5_assembly_material.py, including the inner wire.
    for (mode, volume) in [("dist_angle", 0.5589738415716731)] {
        for reverse in [false, true] {
            let input = if reverse {
                Solid::new(vec![original.boundaries()[0]
                    .iter()
                    .rev()
                    .cloned()
                    .collect()])
            } else {
                original.clone()
            };
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut selected = Vec::new();
            for edge in fixture["modes"][mode]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| !e["distances"].is_null())
            {
                let a = point(&edge["ends"][0]);
                let b = point(&edge["ends"][1]);
                let actual = input
                    .edge_iter()
                    .find(|e| {
                        let p = e.front().point();
                        let q = e.back().point();
                        (p.distance(a) < 1e-8 && q.distance(b) < 1e-8)
                            || (p.distance(b) < 1e-8 && q.distance(a) < 1e-8)
                    })
                    .unwrap();
                let adjacent: Vec<_> = input
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == actual.id()))
                    .collect();
                let distances = std::array::from_fn(|side| {
                    let normal = match adjacent[side].oriented_surface() {
                        Surface::Plane(p) => Some(p.normal()),
                        _ => None,
                    };
                    let i = (0..2)
                        .find(|&i| {
                            let expected = &fixture["faces"]
                                [edge["faces"][i].as_u64().unwrap() as usize]["normal"];
                            match normal {
                                Some(n) => {
                                    !expected.is_null()
                                        && n.dot(point(expected).to_vec()) > 1. - 1e-8
                                }
                                None => expected.is_null(),
                            }
                        })
                        .unwrap();
                    edge["distances"][i].as_f64().unwrap()
                });
                selected.push((actual.id(), distances));
            }
            assert_eq!(selected.len(), 21);
            if reverse {
                selected.reverse();
            }
            assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
            let mut prefs: Vec<_> = preferred
                .as_array()
                .unwrap()
                .iter()
                .map(|preference| {
                    let v = input
                        .vertex_iter()
                        .find(|v| v.point().distance(point(&preference["vertex"])) < 1e-8)
                        .unwrap();
                    let a = point(&preference["edge"]["points"][0]);
                    let b = point(&preference["edge"]["points"][1]);
                    let edge = input
                        .edge_iter()
                        .find(|e| {
                            (e.front().point().distance(a) < 1e-8
                                && e.back().point().distance(b) < 1e-8)
                                || (e.front().point().distance(b) < 1e-8
                                    && e.back().point().distance(a) < 1e-8)
                        })
                        .unwrap();
                    ChamferCornerPreference {
                        vertex: v.id(),
                        edge: edge.id(),
                    }
                })
                .collect();
            if reverse {
                prefs.reverse();
            }
            for missing in 0..prefs.len() {
                let mut incomplete = prefs.clone();
                incomplete.remove(missing);
                assert!(try_chamfer_solid_edges_with_distances(
                    &input,
                    &selected,
                    &incomplete,
                    0.001
                )
                .is_err());
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
            let output =
                try_chamfer_solid_edges_with_distances(&input, &selected, &prefs, 0.001).unwrap();
            common::assert_solid(&output.solid, volume, &[0], 0.001);
            common::blend::assert_step(&output.solid, volume, 0.001);
            for id in [0, 1, 2, 3] {
                let a = point(&fixture["modes"][mode][id]["ends"][0]);
                let b = point(&fixture["modes"][mode][id]["ends"][1]);
                let edge = input
                    .edge_iter()
                    .find(|e| {
                        (e.front().point().distance(a) < 1e-8
                            && e.back().point().distance(b) < 1e-8)
                            || (e.front().point().distance(b) < 1e-8
                                && e.back().point().distance(a) < 1e-8)
                    })
                    .unwrap();
                assert!(output.solid.edge_iter().any(|e| e.id() == edge.id()));
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn a5_da_procedural_signed_preference_material() {
    use truck_shapeops::fillet::ChamferCornerPreference;
    for scale in [0.1, 1., 10.] {
        let transform = Matrix4::from_translation(Vector3::new(3., -2., 4.))
            * Matrix4::from_angle_x(Rad(0.7))
            * Matrix4::from_scale(scale);
        let inverse = transform.invert().unwrap();
        let input = builder::transformed(&rounded_rectangle(true), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected = selection(
            &input,
            [
                0.05 / 3_f64.sqrt(),
                0.05,
                0.03,
                0.03,
                0.02 * 3_f64.sqrt(),
                0.02,
            ],
            true,
            inverse,
            scale,
        );
        let edge = input
            .edge_iter()
            .find(|e| {
                [e.front(), e.back()].iter().all(|v| {
                    let p = inverse.transform_point(v.point());
                    (p.x + 1.).abs() < 1e-8 && (p.y + 1.).abs() < 1e-8
                })
            })
            .unwrap();
        let mut preferences: Vec<_> = [edge.front(), edge.back()]
            .into_iter()
            .map(|v| ChamferCornerPreference {
                vertex: v.id(),
                edge: edge.id(),
            })
            .collect();
        for order in 0..6 {
            let output =
                try_chamfer_solid_edges_with_distances(&input, &selected, &preferences, 0.001)
                    .unwrap();
            common::assert_solid(
                &output.solid,
                1.1635246505147215 * scale.powi(3),
                &[1],
                0.001,
            );
            common::blend::assert_step(&output.solid, 1.1635246505147215 * scale.powi(3), 0.001);
            for missing in 0..2 {
                assert!(try_chamfer_solid_edges_with_distances(
                    &input,
                    &selected,
                    &preferences[missing..missing + 1],
                    0.001
                )
                .is_err());
            }
            assert!(try_chamfer_solid_edges_with_distances(
                &input,
                &selected,
                &[preferences[0], preferences[0]],
                0.001
            )
            .is_err());
            preferences.reverse();
            if order % 2 == 0 {
                selected.reverse();
            } else {
                selected.rotate_left(1);
            }
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
