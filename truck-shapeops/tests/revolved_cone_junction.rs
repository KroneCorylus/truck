mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges_with_radii;

fn fixture() -> (Solid, Vec<EdgeID>) {
    let vertices = [
        Point3::new(0., 0., 0.),
        Point3::new(10., 0., 0.),
        Point3::new(5., 0., 5.),
    ]
    .map(Vertex::new);
    let profile = builder::try_attach_plane(&[wire![
        builder::line(&vertices[0], &vertices[1]),
        builder::line(&vertices[1], &vertices[2]),
        builder::line(&vertices[2], &vertices[0]),
    ]])
    .unwrap();
    let input: Solid = builder::rsweep(
        &profile.inverse(),
        Point3::new(-20., 0., 0.),
        Vector3::unit_z(),
        Rad(std::f64::consts::FRAC_PI_2),
        1,
    );
    let endpoints = [
        (Point3::new(0., 0., 0.), Point3::new(-20., 20., 0.)),
        (Point3::new(10., 0., 0.), Point3::new(5., 0., 5.)),
        (Point3::new(0., 0., 0.), Point3::new(10., 0., 0.)),
    ];
    let mut seen = HashSet::new();
    let edges: Vec<_> = input.edge_iter().filter(|e| seen.insert(e.id())).collect();
    assert_eq!(edges.len(), 9);
    let selected = endpoints
        .map(|(a, b)| {
            let matches: Vec<_> = edges
                .iter()
                .filter(|e| {
                    (e.front().point().distance(a) < 1e-10 && e.back().point().distance(b) < 1e-10)
                        || (e.front().point().distance(b) < 1e-10
                            && e.back().point().distance(a) < 1e-10)
                })
                .collect();
            assert_eq!(matches.len(), 1);
            matches[0].id()
        })
        .to_vec();
    (input, selected)
}

#[test]
fn procedural_source_is_closed_quarter_revolved_triangle() {
    let (input, _) = fixture();
    assert_eq!(input.face_iter().count(), 5);
    assert!(input.is_geometric_consistent());
    // Integral pi/4 * ((30-z)^2-(20+z)^2), z=0..5.
    let volume = 312.5 * std::f64::consts::PI;
    common::assert_solid(&input, volume, &[0], 0.001);
    common::blend::assert_step(&input, volume, 0.001);
}

#[test]
fn three_selected_cone_junction_preserves_residual_edges() {
    // Independent regularized section integral, not native output or mesh volume.
    // Its 1e-8 numerical allowance is separate from geometric chord error.
    const ORACLE_ALLOWANCE: f64 = 1e-8;
    for (radius, volume) in [
        (0.5, 970.934499167348),
        (1.0, 937.7409690789835),
        (1.5, 881.0404265627494),
    ] {
        let (input, selected) = fixture();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for reverse in [false, true] {
            let mut assignments: Vec<_> = selected.iter().map(|id| (*id, radius)).collect();
            if reverse {
                assignments.reverse();
            }
            let result = try_fillet_solid_edges_with_radii(&input, &assignments, 0.001);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            let result = result.expect("I2 bounded three-support construction").solid;
            assert_eq!(result.face_iter().count(), 8);
            let edges: HashSet<_> = result.edge_iter().map(|e| e.id()).collect();
            let vertices: HashSet<_> = result.vertex_iter().map(|v| v.id()).collect();
            assert_eq!(edges.len(), 18, "DC and AS must remain distinct edges");
            assert_eq!(vertices.len(), 12);
            assert!(result.is_geometric_consistent());
            common::assert_solid(&result, volume, &[0], 0.001);
            common::assert_volume(&result, volume - ORACLE_ALLOWANCE, 0.001);
            common::assert_volume(&result, volume + ORACLE_ALLOWANCE, 0.001);
            common::blend::assert_step(&result, volume, 0.001);
        }
    }
}

#[path = "../src/fillet/i2.rs"]
mod private_candidate;

#[test]
fn private_candidate_material_and_step() {
    for (radius, volume) in [
        (0.5, 970.934499167348),
        (1.0, 937.7409690789835),
        (1.5, 881.0404265627494),
    ] {
        let (input, selected) = fixture();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let selected = selected.into_iter().collect();
        let (shell, ledgers) = private_candidate::build_candidate_with_ledgers(
            &input.boundaries()[0],
            &selected,
            radius,
        )
        .unwrap();
        assert_eq!(ledgers["full_admission"], false);
        assert_eq!(ledgers["physical_endpoints"].as_array().unwrap().len(), 7);
        for row in ledgers["physical_endpoints"].as_array().unwrap() {
            for end in row["ends"].as_array().unwrap() {
                let correction = end["vertex_correction"].as_f64().unwrap();
                let sliver = end["sliver"].as_f64().unwrap();
                assert!(correction.is_finite() && sliver.is_finite());
                assert!(correction + sliver <= 1e-10);
            }
        }
        let separation = &ledgers["separation"];
        assert!(separation["zero_containing_boxes"].as_u64().unwrap() > 0);
        assert!(separation["stationary_partial_lower"].as_f64().unwrap() > 0.);
        let charge = separation["charge"].as_f64().unwrap();
        assert!(charge > 0.);
        for gap in separation["gaps"].as_array().unwrap() {
            assert!(gap[0].as_f64().unwrap() > charge);
        }
        let result = Solid::try_new(vec![shell]).unwrap();
        if let Some(directory) = std::env::var_os("I2_CANDIDATE_EVIDENCE") {
            let path =
                std::path::PathBuf::from(directory).join(format!("candidate-r{radius}.json"));
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap();
            serde_json::to_writer_pretty(
                file,
                &serde_json::json!({
                    "radius": radius, "input": input.compress(), "candidate": result.compress(),
                    "ledgers": ledgers, "builder_source": include_str!("../src/fillet/i2.rs"), "full_admission": false
                }),
            )
            .unwrap();
        }
        let reordered: Shell = input.boundaries()[0].iter().rev().cloned().collect();
        let reordered = Solid::try_new(vec![private_candidate::build_candidate(
            &reordered, &selected, radius,
        )
        .unwrap()])
        .unwrap();
        assert_eq!(
            serde_json::to_string(&result.compress()).unwrap(),
            serde_json::to_string(&reordered.compress()).unwrap()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        assert!(result.is_geometric_consistent());
        let retained: Vec<_> = [0, 4, 5, 6, 7]
            .map(|i| serde_json::to_string(&result.boundaries()[0][i].oriented_surface()).unwrap())
            .into();
        for face in input.face_iter() {
            let original = serde_json::to_string(&face.oriented_surface()).unwrap();
            assert_eq!(
                retained.iter().filter(|s| **s == original).count(),
                1,
                "original support retained exactly once"
            );
        }
        common::assert_solid(&result, volume, &[0], 0.001);
        common::assert_volume(&result, volume - 1e-8, 0.001);
        common::assert_volume(&result, volume + 1e-8, 0.001);
        common::blend::assert_step(&result, volume, 0.001);
    }
}

#[test]
fn private_candidate_rejects_incomplete_selection_and_invalid_radius() {
    let (input, ids) = fixture();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let selected = ids.iter().copied().collect();
    for r in [0., -1., f64::NAN, f64::INFINITY, 5.] {
        assert!(private_candidate::build_candidate(&input.boundaries()[0], &selected, r).is_none());
    }
    for omit in 0..3 {
        let partial = ids
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != omit)
            .map(|(_, id)| *id)
            .collect();
        assert!(private_candidate::build_candidate(&input.boundaries()[0], &partial, 1.).is_none());
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn public_route_validates_original_assignments_without_mutation() {
    use truck_base::diagnostics::Code;
    let (input, ids) = fixture();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let (_, foreign) = fixture();
    let extra = input
        .edge_iter()
        .find(|e| !ids.contains(&e.id()))
        .unwrap()
        .id();
    let baseline: Vec<_> = ids.iter().map(|id| (*id, 1.)).collect();
    let mut cases = vec![];
    for omit in 0..3 {
        let partial: Vec<_> = baseline
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != omit)
            .map(|(_, v)| *v)
            .collect();
        let selected = partial.iter().map(|(id, _)| *id).collect();
        assert!(
            private_candidate::build_candidate(&input.boundaries()[0], &selected, 1.).is_none()
        );
        // Missing I2 incidences can still describe an ordinary supported fillet.
        if let Ok(result) = try_fillet_solid_edges_with_radii(&input, &partial, 0.001) {
            assert_ne!(result.solid.face_iter().count(), 8);
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
    let mut duplicate = baseline.clone();
    duplicate.push((ids[0], 1.));
    cases.push((duplicate, Some(Code::DuplicateSelection)));
    let mut conflict = baseline.clone();
    conflict.push((ids[0], 2.));
    cases.push((conflict, Some(Code::DuplicateSelection)));
    let mut outside = baseline.clone();
    outside[0].0 = foreign[0];
    cases.push((outside, Some(Code::UnknownEdge)));
    let mut more = baseline.clone();
    more.push((extra, 1.));
    cases.push((more, None));
    let mut unequal = baseline.clone();
    unequal[1].1 = 1.1;
    cases.push((unequal, None));
    for bad in [0., -1., f64::NAN, f64::INFINITY] {
        let mut values = baseline.clone();
        values[0].1 = bad;
        cases.push((values, None));
    }
    for (assignments, code) in cases {
        let error = try_fillet_solid_edges_with_radii(&input, &assignments, 0.001).unwrap_err();
        if let Some(code) = code {
            assert_eq!(error.code, code);
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn public_i2_rejects_additional_shell_without_discarding_it() {
    let (input, ids) = fixture();
    let other = primitive::cuboid(BoundingBox::from_iter([
        Point3::new(100., 100., 100.),
        Point3::new(101., 101., 101.),
    ]));
    for cavity in [false, true] {
        let nested = primitive::cuboid(BoundingBox::from_iter([
            Point3::new(-2.4, 17.6, 1.9),
            Point3::new(-2.3, 17.7, 2.0),
        ]));
        // Every corner has x+20,y>0 and 20+z < hypot(x+20,y) < 30-z.
        for v in nested.vertex_iter() {
            let p = v.point();
            let radial = (p.x + 20.).hypot(p.y);
            assert!(p.x > -20. && p.y > 0. && p.z > 0. && radial > 20. + p.z && radial < 30. - p.z);
        }
        let mut shell = if cavity {
            nested.boundaries()[0].clone()
        } else {
            other.boundaries()[0].clone()
        };
        if cavity {
            shell.invert();
        }
        let multi = Solid::try_new(vec![input.boundaries()[0].clone(), shell]).unwrap();
        let before = serde_json::to_string(&multi.compress()).unwrap();
        assert!(truck_shapeops::fillet::try_fillet_solid_edges(&multi, &ids, 1., 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&multi.compress()).unwrap());
    }
}

#[cfg(feature = "i2-reviewed-glibc")]
#[test]
fn public_i2_history_follows_reordered_source_faces() {
    let (input, ids) = fixture();
    let mut faces: Vec<_> = input.boundaries()[0].iter().cloned().collect();
    faces.rotate_left(2);
    faces.reverse();
    let input = Solid::try_new(vec![faces.into()]).unwrap();
    let result = truck_shapeops::fillet::try_fillet_solid_edges(&input, &ids, 1., 0.001).unwrap();
    let private = private_candidate::build_for_solid(
        &input.boundaries()[0],
        &ids.iter().copied().collect(),
        1.,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_string(&private.compress()).unwrap(),
        serde_json::to_string(&result.solid.boundaries()[0].compress()).unwrap()
    );
    assert_eq!(result.modified_faces.len(), 5);
    assert_eq!(result.generated_faces.len(), 3);
    for ((old_id, new), old) in result
        .modified_faces
        .iter()
        .zip(input.boundaries()[0].iter())
    {
        assert_eq!(*old_id, old.id());
        assert_eq!(
            serde_json::to_string(&new.oriented_surface()).unwrap(),
            serde_json::to_string(&old.oriented_surface()).unwrap()
        );
    }
}

#[cfg(feature = "i2-reviewed-glibc")]
#[test]
fn public_transforms_preserve_material_and_scale_budget() {
    let (source, ids) = fixture();
    for scale in [0.25, 1., 3.] {
        let matrix = Matrix4::from_translation(Vector3::new(2., -3., 1.))
            * Matrix4::from_angle_x(Rad(0.2))
            * Matrix4::from_angle_z(Rad(0.1))
            * Matrix4::from_scale(scale);
        let input = builder::transformed(&source, matrix);
        let endpoints: Vec<_> = source
            .edge_iter()
            .filter(|e| ids.contains(&e.id()))
            .map(|e| {
                (
                    matrix.transform_point(e.front().point()),
                    matrix.transform_point(e.back().point()),
                )
            })
            .collect();
        let mut seen = HashSet::new();
        let selected: Vec<_> = input
            .edge_iter()
            .filter(|e| seen.insert(e.id()))
            .filter(|e| {
                endpoints.iter().any(|(a, b)| {
                    (e.front().point().distance(*a) < 1e-9 && e.back().point().distance(*b) < 1e-9)
                        || (e.front().point().distance(*b) < 1e-9
                            && e.back().point().distance(*a) < 1e-9)
                })
            })
            .map(|e| e.id())
            .collect();
        assert_eq!(selected.len(), 3);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for reverse in [false, true] {
            let mut assignments: Vec<_> = selected.iter().map(|id| (*id, scale)).collect();
            if reverse {
                assignments.reverse();
            }
            let result = try_fillet_solid_edges_with_radii(&input, &assignments, 0.001);
            if scale == 3. {
                assert!(result.is_err());
            } else {
                let result = result.unwrap().solid;
                let volume = 937.7409690789835 * scale.powi(3);
                common::assert_solid(&result, volume, &[0], 0.001);
                common::blend::assert_step(&result, volume, 0.001);
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[cfg(feature = "i2-reviewed-glibc")]
#[test]
fn public_dc_plane_charts_preserve_material() {
    for (u, v) in [(30., 30.), (-60., 15.)] {
        let (input, ids) = fixture();
        let face = input
            .face_iter()
            .find(|f| f.vertex_iter().all(|v| v.point().z.abs() < 1e-10))
            .unwrap();
        let sign = face.surface().normal(0., 0.).z.signum();
        let o = Point3::new(10., 30., 0.);
        face.set_surface(
            Plane::new(
                o,
                o + Vector3::unit_x() * u,
                o + Vector3::unit_y() * v * sign * u.signum(),
            )
            .into(),
        );
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = truck_shapeops::fillet::try_fillet_solid_edges(&input, &ids, 1., 0.001)
            .unwrap()
            .solid;
        common::assert_solid(&result, 937.7409690789835, &[0], 0.001);
        common::blend::assert_step(&result, 937.7409690789835, 0.001);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
