mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges_with_radii;

fn body() -> (Solid, Vec<EdgeID>) { body_variant(0., 0.) }

fn body_variant(perturb: f64, oblique: f64) -> (Solid, Vec<EdgeID>) {
    body_with_hole(perturb, oblique, false)
}
fn body_with_hole(perturb: f64, oblique: f64, hole: bool) -> (Solid, Vec<EdgeID>) {
    let controls = vec![
        Point3::new(0., 0., 0.),
        Point3::new(20., perturb, 0.),
        Point3::new(20., 5., 0.),
        Point3::new(25., 10., 0.),
        Point3::new(10., 20., 0.),
    ];
    let a = Vertex::new(controls[0]);
    let b = Vertex::new(controls[4]);
    let curve = BSplineCurve::new(KnotVec::bezier_knot(4), controls);
    let edge = Edge::new(&a, &b, Curve::BSplineCurve(curve));
    let mut boundaries = vec![wire![edge, builder::line(&b, &a)]];
    if hole {
        let v = [
            Point3::new(4., 1., 0.),
            Point3::new(5., 1., 0.),
            Point3::new(5., 2., 0.),
            Point3::new(4., 2., 0.),
        ]
        .map(Vertex::new);
        boundaries.push(
            (0..4)
                .map(|i| builder::line(&v[i], &v[(i + 1) % 4]))
                .collect::<Wire>()
                .inverse(),
        );
    }
    let face = builder::try_attach_plane(boundaries.as_slice()).unwrap();
    let input: Solid = builder::tsweep(&face, Vector3::new(oblique, 2., 10.));
    for face in input.face_iter() {
        if let Surface::BSplineSurface(s) = face.surface() {
            let p = s.control_points();
            let v = p[0][1] - p[0][0];
            let c = BSplineCurve::new(s.uknot_vec().clone(), p.iter().map(|r| r[0]).collect());
            face.set_surface(ExtrudedCurve::by_extrusion(c.into(), v).into());
        }
    }
    assert_eq!(input.face_iter().count(), if hole { 8 } else { 4 });
    let mut seen = std::collections::HashSet::new();
    let edges: Vec<_> = input.boundaries()[0]
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .collect();
    assert_eq!(edges.len(), if hole { 18 } else { 6 });
    let selected = [
        Point3::new(oblique / 2., 1., 5.),
        Point3::new(5. + oblique, 12., 10.),
    ]
    .map(|p| {
        let matches: Vec<_> = edges
            .iter()
            .filter(|e| {
                let c = e.curve();
                let (a, b) = c.range_tuple();
                c.subs((a + b) / 2.).distance(p) < 1e-8
            })
            .collect();
        assert_eq!(matches.len(), 1);
        matches[0].id()
    });
    assert_ne!(selected[0], selected[1]);
    (input, selected.to_vec())
}

fn positive(r: f64, volume: f64) {
    let (input, selected) = body();
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, 2300., &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for reverse in [false, true] {
        let mut assignments: Vec<_> = selected.iter().map(|id| (*id, r)).collect();
        if reverse {
            assignments.reverse();
        }
        let result = try_fillet_solid_edges_with_radii(&input, &assignments, 1e-6);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        let result = result.expect("bounded G4 direct two-straight-edge construction");
        assert_eq!(result.modified_faces.len(), 4);
        assert_eq!(result.generated_faces.len(), 2);
        let mut mapped = std::collections::HashSet::new();
        for (original, (id, replacement)) in input.face_iter().zip(&result.modified_faces) {
            assert_eq!(original.id(), *id);
            assert_eq!(
                serde_json::to_value(original.oriented_surface()).unwrap(),
                serde_json::to_value(replacement.oriented_surface()).unwrap(),
            );
            assert!(mapped.insert(replacement.id()));
            assert!(result.solid.face_iter().any(|f| f.id() == replacement.id()));
        }
        for generated in &result.generated_faces {
            assert!(mapped.insert(generated.id()));
            assert!(result.solid.face_iter().any(|f| f.id() == generated.id()));
        }
        assert_eq!(mapped.len(), result.solid.face_iter().count());
        let output = result.solid;
        assert!(output.is_geometric_consistent());
        assert_eq!(output.face_iter().count(), 6);
        common::assert_solid(&output, volume, &[0], 0.001);
        common::blend::assert_step(&output, volume, 0.001);
    }
}

#[test]
fn bounded_g4_half() { positive(0.5, 2296.987818795686); }
#[test]
fn bounded_g4_one() { positive(1., 2288.022716418066); }
#[test]
fn bounded_g4_one_and_half() { positive(1.5, 2273.201866106094); }

fn structure(input: &Solid, output: &Solid) {
    for (a, b) in input.face_iter().zip(output.face_iter()) {
        assert_eq!(
            serde_json::to_string(&a.oriented_surface()).unwrap(),
            serde_json::to_string(&b.oriented_surface()).unwrap()
        );
    }
    let mut uses = std::collections::HashMap::<_, Vec<_>>::new();
    for face in output.face_iter() {
        for e in face.edge_iter() {
            uses.entry(e.id()).or_default().push(e.orientation());
        }
    }
    assert!(uses.values().all(|v| v.len() == 2 && v[0] != v[1]));
}

#[test]
fn bounded_g4_transform_and_radius_guards() {
    let (input, ids) = body();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [1.5_f64.next_down(), 1.5, 1.5_f64.next_up()] {
        let result = try_fillet_solid_edges_with_radii(
            &input,
            &ids.iter().map(|id| (*id, radius)).collect::<Vec<_>>(),
            1e-6,
        );
        assert_eq!(
            result.is_ok(),
            radius <= 1.5,
            "radius admission at {radius:?}"
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if let Ok(result) = result {
            structure(&input, &result.solid);
        }
    }
    for radii in [[0.5, 0.25], [1.5001, 1.5001], [1e-9, 1e-9], [10., 10.]] {
        assert!(try_fillet_solid_edges_with_radii(
            &input,
            &[(ids[0], radii[0]), (ids[1], radii[1])],
            1e-6
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
    for scale in [0.5, 2.] {
        let m = Matrix4::from_translation(Vector3::new(12., -7., 3.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
            * Matrix4::from_scale(scale);
        let transformed = builder::transformed(&input, m);
        let ids = [Point3::new(0., 1., 5.), Point3::new(5., 12., 10.)].map(|p| {
            common::blend::edge_through(&transformed.boundaries()[0], m.transform_point(p)).id()
        });
        let out = try_fillet_solid_edges_with_radii(
            &transformed,
            &ids.map(|id| (id, 0.5 * scale)),
            1e-6 * scale,
        )
        .unwrap()
        .solid;
        structure(&transformed, &out);
        common::assert_solid(&out, 2296.987818795686 * scale.powi(3), &[0], 0.001 * scale);
    }
}

#[test]
fn bounded_g4_profile_pair_and_numeric_guards() {
    for (perturb, oblique) in [(0.001, 0.), (-0.001, 0.), (0., 0.1)] {
        let (input, ids) = body_variant(perturb, oblique);
        assert!(input.is_geometric_consistent());
        common::assert_topology(&input, &[0]);
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(try_fillet_solid_edges_with_radii(
            &input,
            &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
            1e-6
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
    let (input, ids) = body();
    let bottom = common::blend::edge_through(&input.boundaries()[0], Point3::new(5., 10., 0.)).id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(
        try_fillet_solid_edges_with_radii(&input, &[(ids[0], 0.5), (bottom, 0.5)], 1e-6).is_err()
    );
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    for offset in [1e3, 1e12] {
        let shift = Vector3::new(offset, -offset, offset);
        let moved = builder::translated(&input, shift);
        let ids = [Point3::new(0., 1., 5.), Point3::new(5., 12., 10.)]
            .map(|p| common::blend::edge_through(&moved.boundaries()[0], p + shift).id());
        let before = serde_json::to_string(&moved.compress()).unwrap();
        let result = try_fillet_solid_edges_with_radii(&moved, &ids.map(|id| (id, 0.5)), 1e-6);
        assert_eq!(before, serde_json::to_string(&moved.compress()).unwrap());
        if let Ok(output) = result {
            common::assert_solid(
                &builder::translated(&output.solid, -shift),
                2296.987818795686,
                &[0],
                0.001,
            );
        } else {
            assert!(offset > 1e6);
        }
    }
}

#[test]
fn bounded_g4_parameterization_and_rational_gate() {
    for rational in [false, true] {
        let (input, ids) = body();
        for face in input.face_iter() {
            if let Surface::Extruded(e) = face.surface() {
                let Curve::BSplineCurve(c) = e.entity_curve() else {
                    panic!("quartic basis")
                };
                let c = BSplineCurve::new(
                    vec![2., 2., 2., 2., 2., 7., 7., 7., 7., 7.].into(),
                    c.control_points().to_vec(),
                );
                let mut curve: Curve = if rational {
                    NurbsCurve::new(BSplineCurve::new(
                        c.knot_vec().clone(),
                        c.control_points()
                            .iter()
                            .map(|p| p.to_homogeneous())
                            .collect(),
                    ))
                    .into()
                } else {
                    c.into()
                };
                let v = e.extruding_vector();
                curve.transform_by(Matrix4::from_translation(v));
                curve.invert();
                face.set_surface(ExtrudedCurve::by_extrusion(curve, -v).into());
            }
        }
        assert!(input.is_geometric_consistent());
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_edges_with_radii(
            &input,
            &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
            1e-6,
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if rational {
            assert!(result.is_err());
        } else {
            common::assert_solid(&result.unwrap().solid, 2296.987818795686, &[0], 0.001);
        }
    }
}

#[test]
fn bounded_g4_hole_rejects_immutably() {
    let (input, ids) = body_with_hole(0., 0., true);
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, 2290., &[1], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
        1e-6
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn bounded_g4_split_trim_rejects_immutably() {
    let (input, ids) = body();
    let split = input.boundaries()[0]
        .edge_iter()
        .find(|e| matches!(e.curve(), Curve::BSplineCurve(_)))
        .unwrap()
        .absolute_clone();
    let curve = split.curve();
    let (a, b) = curve.range_tuple();
    let t = (a + b) / 2.;
    let vertex = Vertex::new(curve.subs(t));
    let (a, b) = split.cut_with_parameter(&vertex, t).unwrap();
    let pieces: Wire = vec![a, b].into();
    let faces = input
        .face_iter()
        .map(|f| {
            let wire = f.boundaries()[0]
                .iter()
                .flat_map(|e| {
                    if e.id() != split.id() {
                        vec![e.clone()]
                    } else if e.orientation() {
                        pieces.iter().cloned().collect()
                    } else {
                        pieces.inverse().iter().cloned().collect()
                    }
                })
                .collect();
            Face::new(vec![wire], f.oriented_surface())
        })
        .collect();
    let input = Solid::new(vec![faces]);
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, 2300., &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
        1e-6
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn bounded_g4_material_seams_and_remote_trim() {
    use truck_meshalgo::prelude::*;
    let (input, ids) = body();
    let output = try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 1.)).collect::<Vec<_>>(),
        1e-6,
    )
    .unwrap()
    .solid;
    structure(&input, &output);
    let frame = |p: Point3| {
        Vector3::new(
            (p.y + 5. * p.z) / 26_f64.sqrt(),
            (13. * p.x + 25. * p.y - 5. * p.z) / 819_f64.sqrt(),
            (10. * p.x - 5. * p.y + p.z) / 126_f64.sqrt(),
        )
    };
    let alpha = (26_f64 / 819.).sqrt();
    let beta = -(26_f64 / 126.).sqrt() / 5.;
    let k = (1. + alpha * alpha).sqrt();
    let xc = 1.6471103743616902;
    let c = 9.087383771951112;
    let dc = 1.0890870806374748;
    let dt = 1.4688097654734333;
    let length = 2. * 26_f64.sqrt();
    for (index, face) in output.face_iter().skip(4).enumerate() {
        let surface = face.oriented_surface();
        let Surface::Extruded(extrusion) = &surface else {
            panic!("exact cylinder support")
        };
        let (lo, hi) = extrusion.entity_curve().range_tuple();
        for i in 0..=32 {
            for v in [0.25, 0.75] {
                let t = lo + (hi - lo) * i as f64 / 32.;
                let p = frame(surface.subs(t, v));
                let radial = if index == 0 {
                    Vector3::new(0., p.y - xc, p.z - 1.)
                } else {
                    let a = (p.x - alpha * p.y - c) / (k * k);
                    Vector3::new(a, -alpha * a, p.z - 1.)
                };
                assert!((radial.magnitude() - 1.).abs() < 1e-8);
                let normal = frame(Point3::from_vec(surface.normal(t, v))).normalize();
                assert!(normal.dot(radial.normalize()) > 1. - 1e-8);
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    let edges: Vec<_> = output.boundaries()[0]
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .collect();
    assert_eq!(edges.len(), 12);
    let mut crease = false;
    let mut residual = false;
    let mut remote = false;
    for e in edges {
        let a = frame(e.front().point());
        let b = frame(e.back().point());
        let curve = e.curve();
        let (lo, hi) = curve.range_tuple();
        let seam =
            a.y < 2. && b.y < 2. && a.z.min(b.z).abs() < 1e-8 && (a.z.max(b.z) - dc).abs() < 1e-8;
        let cap = a.y < 2.
            && b.y < 2.
            && (a.z.min(b.z) - dc).abs() < 1e-8
            && (a.z.max(b.z) - dt).abs() < 1e-8;
        let far = a.y > 20. && b.y > 20. && (a.z.max(b.z) - dc).abs() < 1e-8;
        if !(seam || cap || far) {
            continue;
        }
        let mut min = f64::INFINITY;
        let mut min_index = 0;
        for i in 0..=1024 {
            let p = frame(curve.subs(lo + (hi - lo) * i as f64 / 1024.));
            if seam || cap {
                assert!(((p.y - xc).powi(2) + (p.z - 1.).powi(2) - 1.).abs() < 1e-8);
            }
            if seam || far {
                assert!(
                    ((p.x - alpha * p.y - c).powi(2) / (k * k) + (p.z - 1.).powi(2) - 1.).abs()
                        < 1e-7
                );
            }
            if cap {
                assert!((p.x - alpha * p.y - beta * p.z - length).abs() < 1e-8);
                assert!(p.z > 1.);
            }
            if seam && p.y < min {
                min = p.y;
                min_index = i;
            }
            if far {
                let (mut l, mut h) = (0.5, 1.);
                for _ in 0..60 {
                    let t = (l + h) / 2.;
                    let x = (1040. * t - 810. * t * t + 800. * t * t * t - 400. * t * t * t * t)
                        / 819_f64.sqrt();
                    if x < p.y {
                        l = t;
                    } else {
                        h = t;
                    }
                }
                let t = (l + h) / 2.;
                let d = (800. * t - 1350. * t * t + 1100. * t * t * t - 550. * t * t * t * t)
                    / 126_f64.sqrt();
                assert!((d - p.z).abs() < 1e-7);
            }
        }
        if seam {
            assert!(min_index > 0 && min_index < 1024);
            crease = true;
        }
        residual |= cap;
        remote |= far;
    }
    assert!(crease && residual && remote);
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/g4_witnesses.json"
    ))
    .unwrap();
    let witnesses = oracle["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), 806);
    let mesh = output.triangulation(0.000005).to_polygon();
    let source = input.triangulation(0.000005).to_polygon();
    for (i, w) in witnesses.iter().enumerate() {
        let u = w["frame"][0].as_f64().unwrap();
        let x = w["frame"][1].as_f64().unwrap();
        let d = w["frame"][2].as_f64().unwrap();
        let p = Point3::new(
            13. * x / 819_f64.sqrt() + 10. * d / 126_f64.sqrt(),
            u / 26_f64.sqrt() + 25. * x / 819_f64.sqrt() - 5. * d / 126_f64.sqrt(),
            5. * u / 26_f64.sqrt() - 5. * x / 819_f64.sqrt() + d / 126_f64.sqrt(),
        );
        let inside = w["inside"].as_bool().unwrap();
        assert_eq!(mesh.inside(p), inside, "material witness {i}");
        assert_eq!(
            source.inside(p),
            w["source_inside"].as_bool().unwrap(),
            "source witness {i}"
        );
    }
}

#[test]
fn bounded_g4_imported_representation() {
    use truck_shapeops::SplitClosedEdgesAndFaces;
    use truck_stepio::r#in::Table;
    let text = include_str!("../../audits/fillet-chamfer/oracles/g4_imported.step");
    for perturb in [false, true] {
        let text = if perturb {
            text.replace("-0.980580675691", "-0.980580665691")
        } else {
            text.to_owned()
        };
        let table = Table::from_step(&text).unwrap();
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
        for scale in [0.5, 1., 2.] {
            let m = Matrix4::from_translation(Vector3::new(2., -3., 4.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.4))
                * Matrix4::from_scale(scale);
            let input = builder::transformed(&input, m);
            let ids = [Point3::new(0., 10., 50.), Point3::new(50., 120., 100.)].map(|p| {
                common::blend::edge_through(&input.boundaries()[0], m.transform_point(p)).id()
            });
            let before = serde_json::to_string(&input.compress()).unwrap();
            let result = try_fillet_solid_edges_with_radii(
                &input,
                &ids.map(|id| (id, 10. * scale)),
                0.001 * scale,
            );
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            if perturb {
                assert!(result.is_err());
            } else {
                let output = result.expect("imported represented G4").solid;
                structure(&input, &output);
                assert!(output.is_geometric_consistent());
                common::assert_solid(
                    &output,
                    2288022.716418066 * scale.powi(3),
                    &[0],
                    0.01 * scale,
                );
                common::blend::assert_step(
                    &output,
                    2288022.716418066 * scale.powi(3),
                    0.01 * scale,
                );
            }
        }
    }
}
