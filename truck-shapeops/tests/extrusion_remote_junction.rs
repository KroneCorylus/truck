mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn body(reversed: bool, depth: f64) -> (Solid, [Point3; 3]) {
    let a = Vertex::new(Point3::new(0., -100., 50.));
    let b = Vertex::new(Point3::new(0., 100., 50.));
    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![a.point(), Point3::new(0., 0., -50.), b.point()],
    );
    let arc = Edge::new(&a, &b, Curve::BSplineCurve(curve));
    let wire = wire![arc, builder::line(&b, &a)];
    let face = builder::try_attach_plane(&[wire]).unwrap();
    let solid: Solid = builder::tsweep(&face, Vector3::new(depth, 0., 5. * depth));
    // tsweep stores polynomial sweeps as tensor-product splines. Keep the
    // exact geometry in the explicit extrusion representation under test.
    let mut converted = 0;
    for face in solid.face_iter() {
        let Surface::BSplineSurface(surface) = face.surface() else {
            continue;
        };
        let controls = surface.control_points();
        assert_eq!(controls.len(), 3);
        assert!(controls.iter().all(|row| row.len() == 2));
        let vector = controls[0][1] - controls[0][0];
        let curve = BSplineCurve::new(
            surface.uknot_vec().clone(),
            controls.iter().map(|row| row[0]).collect(),
        );
        let extrusion: Surface = ExtrudedCurve::by_extrusion(curve.into(), vector).into();
        for u in [0., 0.25, 0.5, 0.75, 1.] {
            for v in [0., 0.25, 0.5, 0.75, 1.] {
                assert!(surface.subs(u, v).near(&extrusion.subs(u, v)));
                assert!(surface.normal(u, v).near(&extrusion.normal(u, v)));
            }
        }
        face.set_surface(extrusion);
        converted += 1;
    }
    assert_eq!(converted, 1);
    assert!(solid.is_geometric_consistent());
    if reversed {
        let mut changed = 0;
        for face in solid.face_iter() {
            let original = face.oriented_surface();
            let Surface::Extruded(extrusion) = &original else {
                continue;
            };
            if !matches!(extrusion.entity_curve(), Curve::BSplineCurve(_)) {
                continue;
            }
            let vector = extrusion.extruding_vector();
            let mut curve = extrusion.entity_curve().clone();
            curve.transform_by(Matrix4::from_translation(vector));
            curve.invert();
            let reversed: Surface = ExtrudedCurve::by_extrusion(curve, -vector).into();
            for u in [0., 0.25, 0.5, 0.75, 1.] {
                for v in [0., 0.25, 0.5, 0.75, 1.] {
                    assert!(original.subs(u, v).near(&reversed.subs(1. - u, 1. - v)));
                }
            }
            let mut stored = reversed;
            if !face.orientation() {
                stored.invert();
            }
            face.set_surface(stored);
            let effective = face.oriented_surface();
            assert!((effective.vder(0.5, 0.5) + original.vder(0.5, 0.5)).magnitude() < 1e-10);
            changed += 1;
        }
        assert_eq!(changed, 1);
        assert!(solid.is_geometric_consistent());
    }
    (
        solid,
        [
            Point3::new(depth, 0., 5. * depth),
            Point3::new(depth, 0., 50. + 5. * depth),
            Point3::new(depth / 2., 100., 50. + 2.5 * depth),
        ],
    )
}

#[test]
fn quadratic_remote_construction_preserves_all_similarity_and_reversal_cases() {
    for radius in [0.3, 0.5, 0.6] {
        for (moved, reversed) in [(false, false), (true, false), (false, true), (true, true)] {
            let (original, witnesses) = body(reversed, 20.);
            let factor = if moved { 2.5 } else { 1. };
            let matrix = if moved {
                Matrix4::from_translation(Vector3::new(7., -11., 13.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(factor)
            } else {
                Matrix4::identity()
            };
            let input = builder::transformed(&original, matrix);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut selected: Vec<_> = witnesses
                .into_iter()
                .map(|p| {
                    common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p))
                        .id()
                })
                .collect();
            for _ in 0..2 {
                let result =
                    try_fillet_solid_edges(&input, &selected, radius * factor, 1e-5 * factor);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                let result = result.unwrap_or_else(|error| {
                    panic!("radius={radius}, moved={moved}, reversed={reversed}: {error:?}")
                });
                assert!(result.solid.is_geometric_consistent());
                selected.reverse();
            }
        }
    }
}

fn selected_center(radius: f64) -> Point3 {
    let mut low = 0_f64;
    let mut high = 1_f64;
    for _ in 0..80 {
        let s = (low + high) / 2.;
        let residual = 50. * (s * s - 1.) + radius * (26. / (26. + s * s).sqrt() + 26_f64.sqrt());
        if residual < 0. {
            low = s;
        } else {
            high = s;
        }
    }
    let s = (low + high) / 2.;
    let length = (26. + s * s).sqrt();
    Point3::new(
        20. - radius,
        100. * s - radius * s / length,
        50. * s * s + 100. - 5. * radius + 26. * radius / length,
    )
}

#[test]
fn quadratic_extrusion_remote_miter_preserves_material_contacts_and_step() {
    // Independent analytic flux oracle and its convergence controls are stored
    // in audits/fillet-chamfer/oracles/g2_material.py and G2_MATERIAL.txt.
    for (radius, volume) in [
        (0.3, 133098.1329516965),
        (0.5, 132681.8707211064),
        (0.6, 132396.5321309278),
    ] {
        for (moved, reversed) in [(false, false), (true, false), (false, true), (true, true)] {
            let (original, witnesses) = body(reversed, 20.);
            let factor = if moved { 2.5 } else { 1. };
            let matrix = if moved {
                Matrix4::from_translation(Vector3::new(7., -11., 13.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(factor)
            } else {
                Matrix4::identity()
            };
            let inverse = matrix.invert().unwrap();
            let input = builder::transformed(&original, matrix);
            let oriented = input.face_iter().map(|f| f.oriented_surface()).find(|s| {
                matches!(s, Surface::Extruded(e) if matches!(e.entity_curve(), Curve::BSplineCurve(_)))
            }).unwrap();
            let expected_axis = Vector3::new(20., 0., 100.) * if reversed { -1. } else { 1. };
            assert!(
                (inverse.transform_vector(oriented.vder(0.5, 0.5)) - expected_axis).magnitude()
                    < 1e-8
            );
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut selected: Vec<_> = witnesses
                .into_iter()
                .map(|p| {
                    common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p))
                        .id()
                })
                .collect();
            assert_eq!(selected.iter().copied().collect::<HashSet<_>>().len(), 3);
            for _ in 0..2 {
                let result =
                    try_fillet_solid_edges(&input, &selected, radius * factor, 1e-5 * factor)
                        .unwrap();
                let solid = &result.solid;
                assert!(solid.is_geometric_consistent());
                assert_eq!(solid.face_iter().count(), 8);
                assert_eq!(result.generated_faces.len(), 4);
                let mut fitted = 0;
                let mut spheres = 0;
                let mut cylinders = 0;
                let mut worst_radius = 0_f64;
                let mut sphere_ids = HashSet::new();
                for face in &result.generated_faces {
                    if let Surface::Fillet(fit) = face.surface() {
                        fitted += 1;
                        let af = fit.entity();
                        let (side, plane) = [af.surface0(), af.surface1()]
                            .into_iter()
                            .enumerate()
                            .find_map(|(i, s)| match s.as_ref() {
                                Surface::Plane(p) => Some((i, p)),
                                _ => None,
                            })
                            .unwrap();
                        let normal = fit.transform().transform_vector(plane.normal()).normalize();
                        let (a, b) = af.range_tuple().1;
                        for j in 0..=32 {
                            let v = a + (b - a) * j as f64 / 32.;
                            let contact = fit.transform().transform_point(af.subs(side as f64, v));
                            let center = contact - radius * factor * normal;
                            for i in 0..=16 {
                                let p = fit.transform().transform_point(af.subs(i as f64 / 16., v));
                                let residual = (p.distance(center) / factor - radius).abs();
                                assert!(residual.is_finite(), "non-finite radius residual");
                                worst_radius = worst_radius.max(residual);
                            }
                        }
                    } else if let Some((Elementary::Sphere { center, radius: r }, _)) =
                        face.oriented_surface().elementary()
                    {
                        spheres += 1;
                        sphere_ids.insert(face.id());
                        assert!(
                            inverse
                                .transform_point(center)
                                .distance(selected_center(radius))
                                < 1e-8
                        );
                        assert!((r / factor - radius).abs() < 1e-8);
                    } else {
                        cylinders += 1;
                        let center = Point3::new(
                            20. - radius,
                            0.,
                            150. - 5. * radius - radius * 26_f64.sqrt(),
                        );
                        let surface = face.surface();
                        let (ur, vr) = surface.try_range_tuple();
                        let ((u0, u1), (v0, v1)) = (ur.unwrap(), vr.unwrap());
                        for i in 0..=16 {
                            for j in 0..=16 {
                                let p = inverse.transform_point(surface.subs(
                                    u0 + (u1 - u0) * i as f64 / 16.,
                                    v0 + (v1 - v0) * j as f64 / 16.,
                                ));
                                let delta = p - center;
                                assert!(
                                    ((delta.x * delta.x + delta.z * delta.z).sqrt() - radius).abs()
                                        < 1e-8
                                );
                            }
                        }
                    }
                }
                assert_eq!((fitted, spheres, cylinders), (2, 1, 1));
                assert!(worst_radius < 1e-5, "radius residual {worst_radius}");
                let mut checked = HashSet::new();
                let mut tangent = 0;
                let mut remote_sharp = 0;
                let mut bottom_sharp = 0;
                for face in &result.generated_faces {
                    for edge in face.edge_iter() {
                        if !checked.insert(edge.id()) {
                            continue;
                        }
                        let other = solid
                            .face_iter()
                            .find(|f| {
                                f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id())
                            })
                            .unwrap();
                        let surfaces = [face.oriented_surface(), other.oriented_surface()];
                        let curve = edge.oriented_curve();
                        let (a, b) = curve.range_tuple();
                        let mut dot = 1_f64;
                        let mut on_bottom = true;
                        let mut at_remote = true;
                        for i in 0..=16 {
                            let p = curve.subs(a + (b - a) * i as f64 / 16.);
                            let original_p = inverse.transform_point(p);
                            on_bottom &= original_p.x.abs() < 1e-7;
                            at_remote &= original_p.y < 0.;
                            let normals = surfaces.each_ref().map(|s| {
                                let (u, v) = s.search_parameter(p, None, 100).unwrap();
                                s.normal(u, v).normalize()
                            });
                            let normal_dot = normals[0].dot(normals[1]);
                            assert!(normal_dot.is_finite(), "non-finite contact normal dot");
                            dot = dot.min(normal_dot);
                        }
                        if dot > 1. - 1e-7 {
                            tangent += 1;
                        } else if on_bottom {
                            bottom_sharp += 1;
                        } else {
                            assert!(
                                at_remote
                                    && !sphere_ids.contains(&face.id())
                                    && !sphere_ids.contains(&other.id())
                            );
                            remote_sharp += 1;
                        }
                    }
                }
                assert_eq!((tangent, remote_sharp, bottom_sharp), (9, 2, 1));
                common::assert_solid(solid, volume * factor.powi(3), &[0], 0.0003 * factor);
                common::blend::assert_step(solid, volume * factor.powi(3), 0.0003 * factor);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                selected.reverse();
            }
        }
    }
}

#[test]
fn remote_round_cannot_cross_unselected_bottom_cap() {
    for (depth, radii) in [(20., vec![11., 20.]), (5., vec![3.])] {
        let (input, witnesses) = body(false, depth);
        let mut selected: Vec<_> = witnesses
            .into_iter()
            .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
            .collect();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for radius in radii {
            // Exact R is beyond the unchanged bottom cap. For the shallower
            // body the two cap offsets still meet, isolating the extent guard.
            assert!(depth - radius * (1. + 5. / 26_f64.sqrt()) < 0.);
            if depth == 5. {
                assert!(-50. + 2. * radius * 26_f64.sqrt() < 0.);
            }
            for _ in 0..2 {
                assert!(try_fillet_solid_edges(&input, &selected, radius, 1e-5).is_err());
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                selected.reverse();
            }
        }
    }
}

fn imported_g2() -> (Solid, Vec<EdgeID>) {
    use truck_shapeops::SplitClosedEdgesAndFaces;
    use truck_stepio::r#in::Table;

    let table =
        Table::from_step(include_str!("fixtures/conic-import/parabolic-prism.step")).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(1e-5);
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
    assert!(input.is_geometric_consistent());
    let selected: Vec<_> = [
        Point3::new(200., 0., 1000.),
        Point3::new(200., 0., 1500.),
        Point3::new(100., 1000., 1000.),
    ]
    .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
    .into();
    (input, selected)
}

#[test]
fn imported_g2_remote_miter_preserves_material_at_replay_tolerance() {
    let (input, selected) = imported_g2();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_fillet_solid_edges(&input, &selected, 10., 0.001).unwrap();
    assert!(result.solid.is_geometric_consistent());
    common::assert_solid(&result.solid, 130744686.35830233, &[0], 0.003);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn imported_g2_step_remote_edges_stay_on_exact_cylinder() {
    let (input, selected) = imported_g2();
    let result = try_fillet_solid_edges(&input, &selected, 10., 0.001).unwrap();
    let compressed = result.solid.compress();
    let prepared = truck_stepio::out::prepare_for_step(&compressed, 0.00005).unwrap();
    let center_z = 1450. - 10. * 26_f64.sqrt();
    let mut checked = 0;
    let mut worst_exported = 0_f64;
    for (raw, exported) in compressed.boundaries[0]
        .edges
        .iter()
        .zip(&prepared.boundaries[0].edges)
    {
        let Curve::IntersectionCurve(intersection) = &raw.curve else {
            continue;
        };
        let (a, b) = raw.curve.range_tuple();
        if raw.curve.subs((a + b) / 2.).y >= 0. {
            continue;
        }
        checked += 1;
        let mut raw_error = 0_f64;
        let mut exported_error = 0_f64;
        // Include guide-segment interiors: the 64 guide nodes are exact by construction.
        for i in 0..=512 {
            let t = a + (b - a) * i as f64 / 512.;
            let residual = |p: Point3| ((p.x - 190.).hypot(p.z - center_z) - 10.).abs();
            raw_error = raw_error.max(residual(raw.curve.subs(t)));
            exported_error = exported_error.max(residual(exported.curve.subs(t)));
        }
        assert!(raw_error < 1e-6, "raw cylinder residual {raw_error}");
        let node_error = (0..=64)
            .map(|i| {
                let t = a + (b - a) * i as f64 / 64.;
                raw.curve.subs(t).distance(intersection.leader().subs(t))
            })
            .fold(0_f64, f64::max);
        eprintln!("remote edge {checked}: raw {raw_error}, STEP {exported_error}, guide-node error {node_error}");
        worst_exported = worst_exported.max(exported_error);
    }
    assert_eq!(checked, 2);
    assert!(
        worst_exported < 0.00005,
        "STEP cylinder residual {worst_exported}"
    );
}
