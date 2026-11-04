mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

fn quarter() -> Solid {
    let start = Vertex::new(Point3::new(0., 1., 0.));
    let arc = builder::rsweep(
        &start,
        Point3::origin(),
        Vector3::unit_x(),
        Rad(std::f64::consts::FRAC_PI_2),
        1,
    );
    let center = Vertex::new(Point3::origin());
    let mut wire = arc.clone();
    wire.extend([
        builder::line(arc.back_vertex().unwrap(), &center),
        builder::line(&center, &start),
    ]);
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_x(),
    )
}

fn selection(input: &Solid, scale: f64, unequal: bool) -> Vec<(EdgeID, [f64; 2])> {
    let shell = &input.boundaries()[0];
    let mut seen = std::collections::HashSet::new();
    input
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .map(|edge| {
            let mut d = [0.2 * scale; 2];
            let sides: Vec<_> = shell
                .iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .collect();
            if matches!(edge.curve(), Curve::Line(_)) {
                for (i, face) in sides.iter().enumerate() {
                    if matches!(
                        face.surface().elementary(),
                        Some((Elementary::Cylinder { .. }, _))
                    ) {
                        // Public two-distance setbacks use tangent projection; native equal distance is a chord.
                        d[i] = scale * (2. * (0.2_f64 / 2.).asin()).tan();
                    }
                }
                if unequal
                    && sides
                        .iter()
                        .all(|f| matches!(f.surface(), Surface::Plane(_)))
                {
                    d = [0.1 * scale; 2];
                }
            }
            (edge.id(), d)
        })
        .collect()
}

fn constraints() -> Vec<(Vector3, f64)> {
    let d = 0.2_f64;
    let cy = d * (1. - d * d / 4.).sqrt();
    let cz = 1. - d * d / 2.;
    let p = Point3::new(d, cy, cz);
    let q = Point3::new(d, 0., 1. - d);
    let r = Point3::new(0., d, ((1. - d).powi(2) - d * d).sqrt());
    let n = (r - p).cross(q - p);
    let offset = n.dot(p.to_vec());
    let mut planes = Vec::new();
    for far in [false, true] {
        for swap in [false, true] {
            let mut m = if swap { Vector3::new(n.x, n.z, n.y) } else { n };
            let mut c = offset;
            if far {
                c -= m.x;
                m.x = -m.x;
            }
            planes.push((m, c));
        }
    }
    let slope = (cz - (1. - d)) / cy;
    planes.extend([
        (Vector3::new(0., -slope, 1.), 1. - d),
        (Vector3::new(0., 1., -slope), 1. - d),
        (Vector3::new(-1., -1., 0.), -d),
        (Vector3::new(-1., 0., -1.), -d),
        (Vector3::new(1., -1., 0.), 1. - d),
        (Vector3::new(1., 0., -1.), 1. - d),
        (Vector3::new(0., -1., -1.), -d),
        (Vector3::new(-1., -1., -1.), -2. * d),
        (Vector3::new(1., -1., -1.), 1. - 2. * d),
    ]);
    planes
}

fn check(scale: f64, reverse: bool) {
    let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.37))
        * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&quarter(), transform);
    if reverse {
        input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
    }
    let mut selected = selection(&input, scale, false);
    if reverse {
        selected.reverse();
    }
    assert_eq!(selected.len(), 9);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result =
        try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    // Independent integration of the circle/cone sections clipped by the half-spaces
    // below: exact circle/line section integrals, equal at quadrature tolerances 1e-8 and 1e-10.
    let volume = 0.6180855963196404 * scale.powi(3);
    common::assert_solid(&result.solid, volume, &[0], 0.0001 * scale);
    common::blend::assert_step(&result.solid, volume, 0.0001 * scale);
    let planes = constraints();
    for edge in result.solid.edge_iter() {
        let curve = edge.curve();
        let (a, b) = curve.range_tuple();
        for i in 0..=32 {
            let p = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 32.));
            assert!(p.x >= -1e-8 && p.x <= 1. + 1e-8 && p.y >= -1e-8 && p.z >= -1e-8);
            let radius = 0.8 + p.x.min(1. - p.x).min(0.2);
            assert!(p.y.hypot(p.z) <= radius + 1e-8);
            assert!(
                planes.iter().all(|&(n, c)| n.dot(p.to_vec()) <= c + 1e-8),
                "material excursion at {p:?}"
            );
        }
    }
}

#[test]
fn captured_a7_network() { check(1., false); }

#[test]
fn curved_network_scaled_and_reordered() {
    check(0.1, true);
    check(10., false);
}

#[test]
fn unequal_network_remains_immutable_rejection() {
    let input = quarter();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(
        &input,
        &selection(&input, 1., true),
        &[],
        0.001
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn overlapping_end_contacts_remain_immutable_rejection() {
    for length in [0.3, 0.4] {
        let input =
            builder::transformed(&quarter(), Matrix4::from_nonuniform_scale(length, 1., 1.));
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(try_chamfer_solid_edges_with_distances(
            &input,
            &selection(&input, 1., false),
            &[],
            0.001
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn partial_network_remains_immutable_rejection() {
    let input = quarter();
    let mut selected = selection(&input, 1., false);
    selected.pop();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn unequal_planar_corner_guard_isolates_a6_network() {
    for (scale, reverse) in [(1., false), (0.1, true), (10., true)] {
        let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.37))
            * Matrix4::from_scale(scale);
        let inverse = transform.invert().unwrap();
        let mut input = builder::transformed(&quarter(), transform);
        if reverse {
            input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
        }
        let mut selected = selection(&input, scale, false);
        if reverse {
            selected.reverse();
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale).is_ok()
        );
        // Change only the axis edge between the two radial planes. All curved-corner
        // axial setbacks remain equal; the planar three-edge corners become unequal.
        let axis_edge = input
            .edge_iter()
            .find(|edge| {
                [edge.front().point(), edge.back().point()]
                    .into_iter()
                    .all(|p| {
                        let q = inverse.transform_point(p);
                        q.y.abs() < 1e-9 && q.z.abs() < 1e-9
                    })
            })
            .unwrap()
            .id();
        let mut changed = 0;
        for (id, distances) in &mut selected {
            if *id == axis_edge {
                *distances = [0.1 * scale; 2];
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale).is_err()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

fn preference_case(
    input: &Solid,
    choices: [bool; 2],
) -> (
    Vec<(EdgeID, [f64; 2])>,
    Vec<truck_shapeops::fillet::ChamferCornerPreference>,
) {
    use truck_shapeops::fillet::ChamferCornerPreference;
    let mut selected = selection(input, 1., false);
    let axis = input
        .edge_iter()
        .find(|e| {
            [e.front().point(), e.back().point()]
                .iter()
                .all(|p| p.y.abs() < 1e-9 && p.z.abs() < 1e-9)
        })
        .unwrap();
    selected
        .iter_mut()
        .find(|(id, _)| *id == axis.id())
        .unwrap()
        .1 = [0.1; 2];
    let preferences = choices
        .into_iter()
        .enumerate()
        .map(|(cap, swap)| {
            let point = Point3::new(cap as f64, 0., 0.);
            let vertex = [axis.front(), axis.back()]
                .into_iter()
                .find(|v| v.point().near(&point))
                .unwrap();
            let edge = input
                .edge_iter()
                .find(|e| {
                    e.id() != axis.id()
                        && [e.front().id(), e.back().id()].contains(&vertex.id())
                        && {
                            let delta = e.back().point() - e.front().point();
                            if swap {
                                delta.z.abs() > 0.9
                            } else {
                                delta.y.abs() > 0.9
                            }
                        }
                })
                .unwrap();
            ChamferCornerPreference {
                vertex: vertex.id(),
                edge: edge.id(),
            }
        })
        .collect();
    (selected, preferences)
}

#[test]
fn explicit_cap_preferences_four_combinations() {
    for choices in [[false, false], [false, true], [true, false], [true, true]] {
        let input = quarter();
        let (mut selected, mut preferences) = preference_case(&input, choices);
        for reverse in [false, true] {
            if reverse {
                selected.reverse();
                preferences.reverse();
            }
            let result =
                try_chamfer_solid_edges_with_distances(&input, &selected, &preferences, 0.001)
                    .unwrap();
            common::assert_solid(&result.solid, 0.6287522637884021, &[0], 0.0001);
            common::blend::assert_step(&result.solid, 0.6287522637884021, 0.0001);
            use truck_meshalgo::prelude::*;
            let mesh = result.solid.robust_triangulation(0.0001).to_polygon();
            let inside = |point: Point3| {
                let winding: f64 = mesh
                    .faces()
                    .triangle_iter()
                    .map(|tri| {
                        let [a, b, c] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos] - point);
                        2. * a.dot(b.cross(c)).atan2(
                            a.magnitude() * b.magnitude() * c.magnitude()
                                + a.dot(b) * c.magnitude()
                                + b.dot(c) * a.magnitude()
                                + c.dot(a) * b.magnitude(),
                        )
                    })
                    .sum();
                winding.abs() > 6.
            };
            for (cap, swap) in choices.into_iter().enumerate() {
                let map = |mut p: Point3| {
                    if swap {
                        std::mem::swap(&mut p.y, &mut p.z);
                    }
                    if cap == 1 {
                        p.x = 1. - p.x;
                    }
                    p
                };
                let points = [
                    Point3::new(0., 0.2, 0.2),
                    Point3::new(0.2, 0., 0.2),
                    Point3::new(0.2, 0., 0.1),
                    Point3::new(0.2, 0.1, 0.),
                ]
                .map(map);
                for point in points {
                    assert!(
                        result.solid.vertex_iter().any(|v| v.point().near(&point)),
                        "missing {point:?}"
                    );
                }
                for (point, normal) in [
                    (Point3::new(0.1, 0.1, 0.4), Vector3::new(1., 1., 0.)),
                    (Point3::new(0.1, 0.4, 0.1), Vector3::new(1., 0., 1.)),
                    (Point3::new(0.3, 0., 0.15), Vector3::unit_y()),
                    (Point3::new(0.4, 0.05, 0.05), Vector3::new(0., 1., 1.)),
                    (Point3::new(0., 0.4, 0.4), Vector3::unit_x()),
                ] {
                    let normal = normal.normalize();
                    assert!(
                        inside(map(point + 0.001 * normal)),
                        "neighbor lost material {point:?}"
                    );
                    assert!(
                        !inside(map(point - 0.001 * normal)),
                        "neighbor extra material {point:?}"
                    );
                }
                for u in [0.2, 0.5, 0.8] {
                    for v in [0.2, 0.5, 0.8] {
                        let [a, b, c, d] = points.map(|p| p.to_vec());
                        let point = Point3::from_vec(
                            (1. - u) * (1. - v) * a
                                + u * (1. - v) * b
                                + u * v * c
                                + (1. - u) * v * d,
                        );
                        let mut inward = ((1. - v) * (b - a) + v * (c - d))
                            .cross((1. - u) * (d - a) + u * (c - b))
                            .normalize();
                        let center = map(Point3::new(0.5, 0.5, 0.5));
                        if inward.dot(center - point) < 0. {
                            inward = -inward;
                        }
                        assert!(inside(point + 0.001 * inward), "lost material {point:?}");
                        assert!(!inside(point - 0.001 * inward), "extra material {point:?}");
                        assert!(
                            result.generated_faces.iter().any(|f| {
                                let s = f.surface();
                                s.search_parameter(point, None, 100)
                                    .is_some_and(|(u, v)| s.subs(u, v).near(&point))
                            }),
                            "missing patch sample {point:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn invalid_preferences_reject_without_mutation() {
    use truck_base::diagnostics::Code;
    let input = quarter();
    let (selected, preferences) = preference_case(&input, [false, false]);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let mut axis = preferences[0];
    axis.edge = input
        .edge_iter()
        .find(|e| {
            [e.front().id(), e.back().id()].contains(&preferences[0].vertex)
                && [e.front().id(), e.back().id()].contains(&preferences[1].vertex)
        })
        .unwrap()
        .id();
    let other = quarter();
    let (_, stale) = preference_case(&other, [false, false]);
    let mut nonincident = preferences.clone();
    nonincident[0].edge = preferences[1].edge;
    for invalid in [
        vec![preferences[0], preferences[0]],
        vec![axis, preferences[1]],
        stale,
        nonincident,
    ] {
        let error =
            try_chamfer_solid_edges_with_distances(&input, &selected, &invalid, 0.001).unwrap_err();
        assert_eq!(error.code, Code::InvalidParameter);
    }
    assert!(
        try_chamfer_solid_edges_with_distances(&input, &selected, &preferences[..1], 0.001)
            .is_err()
    );
    for selection in [
        Vec::new(),
        selection(&input, 1., false),
        selected[1..].to_vec(),
    ] {
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selection, &preferences, 0.001)
                .is_err()
        );
    }
    let axis_id = axis.edge;
    for h in [0.10001, 0.09999, 0.3] {
        let mut changed = selected.clone();
        changed.iter_mut().find(|(id, _)| *id == axis_id).unwrap().1 = [h; 2];
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &changed, &preferences, 0.001).is_err()
        );
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn preferences_transport_with_rigid_motion_scale_and_face_order() {
    for (scale, swap, reflect) in [
        (0.1, false, false),
        (1., false, false),
        (10., false, false),
        (1., true, false),
        (1., false, true),
        (1., true, true),
    ] {
        let radial = if swap {
            Matrix4::from_cols(
                Vector4::unit_x(),
                Vector4::unit_z(),
                Vector4::unit_y(),
                Vector4::unit_w(),
            )
        } else {
            Matrix4::identity()
        };
        let cap = if reflect {
            Matrix4::from_translation(Vector3::unit_x())
                * Matrix4::from_nonuniform_scale(-1., 1., 1.)
        } else {
            Matrix4::identity()
        };
        let transform = Matrix4::from_translation(Vector3::new(7., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
            * Matrix4::from_scale(scale)
            * radial
            * cap;
        let source = quarter();
        let (selected, preferences) = preference_case(&source, [false, true]);
        let transformed = builder::transformed(&source, transform);
        let input = Solid::new(vec![transformed.boundaries()[0]
            .iter()
            .rev()
            .map(|f| {
                if swap != reflect {
                    f.inverse()
                } else {
                    f.clone()
                }
            })
            .collect()]);
        let find_vertex = |id| {
            let point = transform
                .transform_point(source.vertex_iter().find(|v| v.id() == id).unwrap().point());
            input
                .vertex_iter()
                .find(|v| v.point().near(&point))
                .unwrap()
                .id()
        };
        let find_edge = |id| {
            let old = source.edge_iter().find(|e| e.id() == id).unwrap();
            let ends = [find_vertex(old.front().id()), find_vertex(old.back().id())];
            input
                .edge_iter()
                .find(|e| ends.contains(&e.front().id()) && ends.contains(&e.back().id()))
                .unwrap()
                .id()
        };
        let selected: Vec<_> = selected
            .into_iter()
            .rev()
            .map(|(id, mut d)| {
                let old_faces: Vec<_> = source
                    .face_iter()
                    .filter(|f| f.edge_iter().any(|e| e.id() == id))
                    .collect();
                let edge = find_edge(id);
                let first = input
                    .face_iter()
                    .find(|f| f.edge_iter().any(|e| e.id() == edge))
                    .unwrap();
                if matches!(
                    first.surface().elementary(),
                    Some((Elementary::Cylinder { .. }, _))
                ) != matches!(
                    old_faces[0].surface().elementary(),
                    Some((Elementary::Cylinder { .. }, _))
                ) {
                    d.reverse();
                }
                (edge, d.map(|v| v * scale))
            })
            .collect();
        let preferences: Vec<_> = preferences
            .into_iter()
            .rev()
            .map(|p| truck_shapeops::fillet::ChamferCornerPreference {
                vertex: find_vertex(p.vertex),
                edge: find_edge(p.edge),
            })
            .collect();
        let result =
            try_chamfer_solid_edges_with_distances(&input, &selected, &preferences, 0.001 * scale)
                .unwrap();
        for p in [
            Point3::new(0.15, 0.075, 0.125),
            Point3::new(0.85, 0.125, 0.075),
        ] {
            let p = transform.transform_point(p);
            assert!(result.generated_faces.iter().any(|f| {
                let surface = f.surface();
                surface
                    .search_parameter(p, None, 100)
                    .is_some_and(|(u, v)| surface.subs(u, v).near(&p))
            }));
        }
        common::assert_solid(
            &result.solid,
            0.6287522637884021 * scale.powi(3),
            &[0],
            0.0001 * scale,
        );
    }
}

#[test]
fn translated_preferences_preserve_local_material_or_reject_immutably() {
    use truck_meshalgo::prelude::*;
    for magnitude in [0., 1e3, 1e6, 1e12] {
        let source = quarter();
        let (selected, preferences) = preference_case(&source, [false, true]);
        let translation = Vector3::new(magnitude, -magnitude, magnitude * 0.5);
        let input = builder::translated(&source, translation);
        let vertex = |id| {
            let point = source.vertex_iter().find(|v| v.id() == id).unwrap().point() + translation;
            input
                .vertex_iter()
                .find(|v| v.point() == point)
                .unwrap()
                .id()
        };
        let edge = |id| {
            let old = source.edge_iter().find(|e| e.id() == id).unwrap();
            let ends = [vertex(old.front().id()), vertex(old.back().id())];
            input
                .edge_iter()
                .find(|e| ends.contains(&e.front().id()) && ends.contains(&e.back().id()))
                .unwrap()
                .id()
        };
        let selected: Vec<_> = selected.into_iter().map(|(id, d)| (edge(id), d)).collect();
        let preferences: Vec<_> = preferences
            .into_iter()
            .map(|p| truck_shapeops::fillet::ChamferCornerPreference {
                vertex: vertex(p.vertex),
                edge: edge(p.edge),
            })
            .collect();
        let axis = input
            .edge_iter()
            .find(|e| {
                [e.front().id(), e.back().id()].contains(&preferences[0].vertex)
                    && [e.front().id(), e.back().id()].contains(&preferences[1].vertex)
            })
            .unwrap()
            .id();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for h in [0.1, 0.09999, 0.10001] {
            let mut changed = selected.clone();
            changed.iter_mut().find(|(id, _)| *id == axis).unwrap().1 = [h; 2];
            let result =
                try_chamfer_solid_edges_with_distances(&input, &changed, &preferences, 0.001);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            if h != 0.1 {
                assert!(
                    result.is_err(),
                    "admitted perturbed relation at {magnitude}: {h}"
                );
                continue;
            }
            if magnitude == 0. {
                assert!(result.is_ok(), "origin control must construct");
            }
            let Ok(result) = result else { continue };
            let local = builder::translated(&result.solid, -translation);
            common::assert_solid(&local, 0.6287522637884021, &[0], 0.0001);
            let mesh = local.robust_triangulation(0.0001).to_polygon();
            let inside = |point: Point3| {
                let winding: f64 = mesh
                    .faces()
                    .triangle_iter()
                    .map(|tri| {
                        let [a, b, c] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos] - point);
                        2. * a.dot(b.cross(c)).atan2(
                            a.magnitude() * b.magnitude() * c.magnitude()
                                + a.dot(b) * c.magnitude()
                                + b.dot(c) * a.magnitude()
                                + c.dot(a) * b.magnitude(),
                        )
                    })
                    .sum();
                winding.abs() > 6.
            };
            for cap in [0, 1] {
                let map = |mut p: Point3| {
                    if cap == 1 {
                        std::mem::swap(&mut p.y, &mut p.z);
                        p.x = 1. - p.x;
                    }
                    p
                };
                let points = [
                    Point3::new(0., 0.2, 0.2),
                    Point3::new(0.2, 0., 0.2),
                    Point3::new(0.2, 0., 0.1),
                    Point3::new(0.2, 0.1, 0.),
                ]
                .map(map);
                for point in points {
                    assert!(
                        local.vertex_iter().any(|v| v.point().near(&point)),
                        "wrong local contact at translation {magnitude}: {point:?}"
                    );
                }
                for u in [0.2, 0.5, 0.8] {
                    for v in [0.2, 0.5, 0.8] {
                        let [a, b, c, d] = points.map(|p| p.to_vec());
                        let point = Point3::from_vec(
                            (1. - u) * (1. - v) * a
                                + u * (1. - v) * b
                                + u * v * c
                                + (1. - u) * v * d,
                        );
                        let mut inward = ((1. - v) * (b - a) + v * (c - d))
                            .cross((1. - u) * (d - a) + u * (c - b))
                            .normalize();
                        if inward.dot(map(Point3::new(0.5, 0.5, 0.5)) - point) < 0. {
                            inward = -inward;
                        }
                        assert!(
                            inside(point + 0.001 * inward),
                            "lost material at {magnitude}"
                        );
                        assert!(
                            !inside(point - 0.001 * inward),
                            "extra material at {magnitude}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn incident_selection_permutations_keep_explicit_pivot() {
    let input = quarter();
    let (selected, preferences) = preference_case(&input, [false, true]);
    let mut seen = std::collections::HashSet::new();
    let incident: Vec<_> = input
        .edge_iter()
        .filter(|e| {
            [e.front().id(), e.back().id()].contains(&preferences[0].vertex) && seen.insert(e.id())
        })
        .map(|e| e.id())
        .collect();
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut selected = selected.clone();
        selected.sort_by_key(|(id, _)| order.iter().position(|&i| incident[i] == *id).unwrap_or(3));
        let result =
            try_chamfer_solid_edges_with_distances(&input, &selected, &preferences, 0.001).unwrap();
        common::assert_topology(&result.solid, &[0]);
        for p in [
            Point3::new(0.15, 0.075, 0.125),
            Point3::new(0.85, 0.125, 0.075),
        ] {
            assert!(result.generated_faces.iter().any(|f| {
                let s = f.surface();
                s.search_parameter(p, None, 100)
                    .is_some_and(|(u, v)| s.subs(u, v).near(&p))
            }));
        }
    }
}
