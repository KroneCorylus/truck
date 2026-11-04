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
        Point3::new(2., 10. + perturb, 0.),
        Point3::new(4., 6., 0.),
        Point3::new(10., 0., 0.),
    ];
    let a = Vertex::new(controls[0]);
    let b = Vertex::new(controls[3]);
    let curve = BSplineCurve::new(vec![-1., -1., -1., -1., 1., 1., 1., 1.].into(), controls);
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
                .collect(),
        );
    }
    let face = builder::try_attach_plane(boundaries.as_slice()).unwrap();
    // Reverse the clockwise profile to give the upward sweep positive volume.
    let input: Solid = builder::tsweep(&face.inverse(), Vector3::new(oblique, 2., 10.));
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
        Point3::new(5. + oblique, 2., 10.),
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
    common::assert_solid(&input, 372., &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for reverse in [false, true] {
        let mut assignments: Vec<_> = selected.iter().map(|id| (*id, r)).collect();
        if reverse {
            assignments.reverse();
        }
        let result = try_fillet_solid_edges_with_radii(&input, &assignments, 1e-6);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        let result = result.expect("bounded G8 direct two-straight-edge construction");
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
fn bounded_g8_quarter() { positive(0.25, 371.701056377750); }
#[test]
fn bounded_g8_half() { positive(0.5, 370.803729986635); }
#[test]
fn bounded_g8_three_quarters() { positive(0.75, 369.305657324348); }

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
fn bounded_g8_material_and_turning_point() {
    use truck_meshalgo::prelude::*;
    let (input, ids) = body();
    let output = try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
        1e-6,
    )
    .unwrap()
    .solid;
    structure(&input, &output);
    let k = 26_f64.sqrt();
    let frame = |p: Point3| ((p.y + 5. * p.z) / k, p.x, (5. * p.y - p.z) / k);
    let mut seen = std::collections::HashSet::new();
    let edges: Vec<_> = output.boundaries()[0]
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .collect();
    let residual = edges
        .iter()
        .find(|e| {
            let (_, x, a) = frame(e.front().point());
            let (_, y, b) = frame(e.back().point());
            x < 1.
                && y < 1.
                && (a.min(b) - 0.401941932430908).abs() < 1e-8
                && (a.max(b) - 0.6059948944128823).abs() < 1e-8
        })
        .expect("residual cap ellipse crossing radial turning point");
    let c = residual.curve();
    let (a, b) = c.range_tuple();
    let mut minimum = f64::INFINITY;
    let mut station = 0;
    for i in 0..=1000 {
        let (u, x, d) = frame(c.subs(a + (b - a) * i as f64 / 1000.));
        assert!((u - 2. * k - 0.2 * d).abs() < 1e-8);
        assert!(((x - 0.6160353002686563).powi(2) + (d - 0.5).powi(2) - 0.25).abs() < 1e-8);
        if x < minimum {
            minimum = x;
            station = i;
        }
    }
    assert!(station > 0 && station < 1000);
    assert!((minimum - 0.1160353002686563).abs() < 1e-7);
    assert!(edges.iter().any(|e| {
        let (_, x, d) = frame(e.front().point());
        let (_, y, f) = frame(e.back().point());
        (x - 10.).abs() < 1e-8 && (y - 10.).abs() < 1e-8 && d.abs() < 1e-8 && f.abs() < 1e-8
    }));
    assert!(edges
        .iter()
        .any(|e| matches!(e.curve(),Curve::BSplineCurve(ref c) if c.degree()==3)));
    for (index, face) in output.face_iter().skip(4).enumerate() {
        let surface = face.oriented_surface();
        let Surface::Extruded(e) = &surface else {
            panic!("exact cylinder extrusion")
        };
        let (a, b) = e.entity_curve().range_tuple();
        for i in 0..=32 {
            let t = a + (b - a) * i as f64 / 32.;
            for v in [0.1, 0.5, 0.9] {
                let p = surface.subs(t, v);
                let (u, x, d) = frame(p);
                let radial = if index == 0 {
                    Vector3::unit_x() * (x - 0.6160353002686563)
                        + Vector3::new(0., 5., -1.) / k * (d - 0.5)
                } else {
                    Vector3::new(0., 1., 5.) / k * (u - 9.78813707582629)
                        + Vector3::new(0., 5., -1.) / k * (d - 0.5)
                };
                assert!((radial.magnitude() - 0.5).abs() < 1e-8);
                assert!(surface.normal(t, v).normalize().dot(radial.normalize()) > 1. - 1e-8);
            }
        }
    }
    let remote = edges
        .iter()
        .find(|e| {
            let (_, x, d) = frame(e.front().point());
            let (_, y, f) = frame(e.back().point());
            x > 9. && y > 9. && d.min(f).abs() < 1e-8 && (d.max(f) - 0.401941932430908).abs() < 1e-8
        })
        .expect("descending-profile cap-cylinder termination");
    let curve = remote.curve();
    let (a, b) = curve.range_tuple();
    for i in 0..=129 {
        let (u, x, d) = frame(curve.subs(a + (b - a) * i as f64 / 129.));
        let (mut lo, mut hi) = (0., 1.);
        for _ in 0..60 {
            let t = (lo + hi) / 2.;
            if 6. * t + 4. * t * t * t < x {
                lo = t;
            } else {
                hi = t;
            }
        }
        let t = (lo + hi) / 2.;
        assert!(t > (7. - 19_f64.sqrt()) / 6.);
        assert!((d - (30. * t - 42. * t * t + 12. * t * t * t) * 5. / k).abs() < 1e-7);
        assert!(((u - 9.78813707582629).powi(2) + (d - 0.5).powi(2) - 0.25).abs() < 1e-7);
    }
    let bottom = edges
        .iter()
        .find(|e| {
            let (u, x, d) = frame(e.front().point());
            let (v, y, f) = frame(e.back().point());
            x < 1.
                && y < 1.
                && (u - 0.2 * d).abs() < 1e-8
                && (v - 0.2 * f).abs() < 1e-8
                && (d - f).abs() > 0.5
        })
        .expect("bottom cylinder-plane ellipse");
    let curve = bottom.curve();
    let (a, b) = curve.range_tuple();
    for i in 0..=129 {
        let (u, x, d) = frame(curve.subs(a + (b - a) * i as f64 / 129.));
        assert!((u - 0.2 * d).abs() < 1e-8);
        assert!(((x - 0.6160353002686563).powi(2) + (d - 0.5).powi(2) - 0.25).abs() < 1e-8);
    }
    let witnesses: serde_json::Value = serde_json::from_str(include_str!(
        "../../audits/fillet-chamfer/oracles/g8_witnesses.json"
    ))
    .unwrap();
    let witnesses = witnesses["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), 770);
    let mesh = output.triangulation(0.000005).to_polygon();
    for (i, w) in witnesses.iter().enumerate() {
        let u = w["frame"][0].as_f64().unwrap();
        let x = w["frame"][1].as_f64().unwrap();
        let d = w["frame"][2].as_f64().unwrap();
        assert_eq!(
            mesh.inside(Point3::new(x, (u + 5. * d) / k, (5. * u - d) / k)),
            w["inside"].as_bool().unwrap(),
            "independent full material witness {i}"
        );
    }
}

#[test]
fn bounded_g8_transform_and_radius_guards() {
    let (input, ids) = body();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [0.75_f64.next_down(), 0.75, 0.75_f64.next_up()] {
        let result = try_fillet_solid_edges_with_radii(
            &input,
            &ids.iter().map(|id| (*id, radius)).collect::<Vec<_>>(),
            1e-6,
        );
        assert_eq!(
            result.is_ok(),
            radius <= 0.75,
            "radius admission at {radius:?}"
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if let Ok(result) = result {
            structure(&input, &result.solid);
        }
    }
    for radii in [[0.5, 0.25], [0.75001, 0.75001], [1e-9, 1e-9], [10., 10.]] {
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
        let ids = [Point3::new(0., 1., 5.), Point3::new(5., 2., 10.)].map(|p| {
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
        common::assert_solid(&out, 370.803729986635 * scale.powi(3), &[0], 0.001 * scale);
    }
}

#[test]
fn bounded_g8_profile_pair_and_numeric_guards() {
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
    let bottom = common::blend::edge_through(&input.boundaries()[0], Point3::new(5., 0., 0.)).id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(
        try_fillet_solid_edges_with_radii(&input, &[(ids[0], 0.5), (bottom, 0.5)], 1e-6).is_err()
    );
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    for offset in [1e3, 1e12] {
        let shift = Vector3::new(offset, -offset, offset);
        let moved = builder::translated(&input, shift);
        let ids = [Point3::new(0., 1., 5.), Point3::new(5., 2., 10.)]
            .map(|p| common::blend::edge_through(&moved.boundaries()[0], p + shift).id());
        let before = serde_json::to_string(&moved.compress()).unwrap();
        let result = try_fillet_solid_edges_with_radii(&moved, &ids.map(|id| (id, 0.5)), 1e-6);
        assert_eq!(before, serde_json::to_string(&moved.compress()).unwrap());
        if let Ok(output) = result {
            common::assert_solid(
                &builder::translated(&output.solid, -shift),
                370.803729986635,
                &[0],
                0.001,
            );
        } else {
            assert!(offset > 1e6);
        }
    }
}

#[test]
fn bounded_g8_parameterization_and_rational_gate() {
    for rational in [false, true] {
        let (input, ids) = body();
        for face in input.face_iter() {
            if let Surface::Extruded(e) = face.surface() {
                let Curve::BSplineCurve(c) = e.entity_curve() else {
                    panic!("cubic basis")
                };
                let c = BSplineCurve::new(
                    vec![2., 2., 2., 2., 7., 7., 7., 7.].into(),
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
            common::assert_solid(&result.unwrap().solid, 370.803729986635, &[0], 0.001);
        }
    }
}

#[test]
fn bounded_g8_hole_rejects_immutably() {
    let (input, ids) = body_with_hole(0., 0., true);
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, 362., &[1], 0.001);
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
fn bounded_g8_split_trim_rejects_immutably() {
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
    common::assert_solid(&input, 372., &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 0.5)).collect::<Vec<_>>(),
        1e-6
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
