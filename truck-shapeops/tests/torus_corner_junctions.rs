mod common;
use std::collections::HashSet;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
fn sector(kind: usize) -> (Solid, Vec<EdgeID>) {
    let (major, minor, end) = if kind < 2 {
        (1., 1., std::f64::consts::FRAC_PI_4)
    } else {
        (2., 0.5, 160_f64.to_radians())
    };
    let bottom = major + minor;
    let top = major + minor * end.cos();
    let height = minor * end.sin();
    let low = builder::vertices([
        Point3::origin(),
        Point3::new(bottom, 0., 0.),
        Point3::new(0., bottom, 0.),
    ]);
    let high = builder::vertices([
        Point3::new(0., 0., height),
        Point3::new(top, 0., height),
        Point3::new(0., top, height),
    ]);
    let rim = |v: &[Vertex], r: f64, z: f64| {
        vec![
            builder::line(&v[0], &v[1]),
            builder::circle_arc(
                &v[1],
                &v[2],
                Point3::new(r / 2_f64.sqrt(), r / 2_f64.sqrt(), z),
            ),
            builder::line(&v[2], &v[0]),
        ]
    };
    let lower = rim(&low, bottom, 0.);
    let upper = rim(&high, top, height);
    let middle = major + minor * (end / 2.).cos();
    let z = minor * (end / 2.).sin();
    let vertical = [
        builder::line(&low[0], &high[0]),
        builder::circle_arc(&low[1], &high[1], Point3::new(middle, 0., z)),
        builder::circle_arc(&low[2], &high[2], Point3::new(0., middle, z)),
    ];
    let torus: Surface = Processor::new(RevolutedCurve::by_revolution(
        vertical[1].curve(),
        Point3::origin(),
        Vector3::unit_z(),
    ))
    .into();
    let torus = torus.inverse();
    let mut shell = Shell::new();
    shell.push(
        builder::try_attach_plane(&[lower.iter().cloned().collect()])
            .unwrap()
            .inverse(),
    );
    shell.push(builder::try_attach_plane(&[upper.iter().cloned().collect()]).unwrap());
    for i in 0..3 {
        let j = (i + 1) % 3;
        let wire = wire![
            lower[i].clone(),
            vertical[j].clone(),
            upper[i].inverse(),
            vertical[i].inverse()
        ];
        shell.push(if i == 1 {
            Face::new(vec![wire], torus.clone())
        } else {
            builder::try_attach_plane(&[wire]).unwrap()
        });
    }
    let ids = if kind == 1 || kind == 3 {
        vec![upper[0].id(), upper[1].id(), vertical[1].id()]
    } else {
        vec![lower[0].id(), lower[1].id(), vertical[1].id()]
    };
    (Solid::new(vec![shell]), ids)
}
#[test]
fn torus_corners_preserve_radius_tangency_and_material() {
    for kind in 0..3 {
        let (original, ids) = sector(kind);
        let (major, minor, end) = if kind < 2 {
            (1., 1., std::f64::consts::FRAC_PI_4)
        } else {
            (2., 0.5, 160_f64.to_radians())
        };
        let h = minor * end.sin();
        let input_volume = PI / 4.
            * (minor * major * major * end.sin()
                + major * minor * minor * (end + end.sin() * end.cos())
                + minor.powi(3) * (end.sin() - end.sin().powi(3) / 3.));
        common::assert_solid(&original, input_volume, &[0], 0.001);
        // Independent offset-solid projection; the nonconvex profile subtracts its rounded cavity.
        for (radius, volumes) in [
            (
                0.03,
                [2.0265388031152822, 2.027173877045031, 1.538346428764247],
            ),
            (
                0.06,
                [2.022909211218013, 2.025553492505257, 1.5332824447959748],
            ),
            (
                0.1,
                [2.013894323167084, 2.0216724399060886, 1.5198480862125514],
            ),
        ] {
            for moved in [false, true] {
                let scale = if moved { 2.5 } else { 1. };
                let matrix = if moved {
                    Matrix4::from_translation(Vector3::new(7., -11., 13.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                        * Matrix4::from_scale(scale)
                } else {
                    Matrix4::identity()
                };
                let inverse = matrix.invert().unwrap();
                let input = builder::transformed(&original, matrix);
                let before = serde_json::to_string(&input.compress()).unwrap();
                let mut selected: Vec<_> = ids
                    .iter()
                    .map(|id| {
                        let e = original.edge_iter().find(|e| e.id() == *id).unwrap();
                        let c = e.curve();
                        let (a, b) = c.range_tuple();
                        common::blend::edge_through(
                            &input.boundaries()[0],
                            matrix.transform_point(c.subs((a + b) / 2.)),
                        )
                        .id()
                    })
                    .collect();
                for _ in 0..2 {
                    let result =
                        try_fillet_solid_edges(&input, &selected, radius * scale, 0.001 * scale)
                            .unwrap();
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                    assert!(result.solid.is_geometric_consistent());
                    assert_eq!(result.generated_faces.len(), 4);
                    let mut counts = [0, 0, 0, 0];
                    let mut worst_radius = 0_f64;
                    for face in &result.generated_faces {
                        if let Surface::Fillet(fit) = face.surface() {
                            counts[3] += 1;
                            let s = fit.entity();
                            for j in 0..=32 {
                                let v = j as f64 / 32.;
                                let points = [0., 1.].map(|u| {
                                    inverse.transform_point(
                                        fit.transform().transform_point(s.subs(u, v)),
                                    )
                                });
                                let q = points
                                    .into_iter()
                                    .find(|p| p.x.abs() < 1e-8 || p.y.abs() < 1e-8)
                                    .unwrap();
                                let n = if q.y.abs() < 1e-8 {
                                    -Vector3::unit_y()
                                } else {
                                    -Vector3::unit_x()
                                };
                                let c = q - radius * n;
                                for i in 0..=16 {
                                    let p = inverse.transform_point(
                                        fit.transform().transform_point(s.subs(i as f64 / 16., v)),
                                    );
                                    worst_radius = worst_radius.max((p.distance(c) - radius).abs());
                                }
                            }
                        } else {
                            match face.surface().elementary().unwrap().0 {
                                Elementary::Cylinder { radius: r, .. } => {
                                    counts[0] += 1;
                                    assert!((r - radius * scale).abs() < 1e-8);
                                }
                                Elementary::Sphere { center, radius: r } => {
                                    counts[1] += 1;
                                    assert!((r - radius * scale).abs() < 1e-8);
                                    let c = inverse.transform_point(center);
                                    assert!(
                                        ((c.x.hypot(c.y) - major).hypot(c.z) - (minor - radius))
                                            .abs()
                                            < 1e-8
                                    );
                                    assert!((c.y - radius).abs() < 1e-8);
                                    assert!(
                                        (c.z - if kind == 1 { h - radius } else { radius }).abs()
                                            < 1e-8
                                    );
                                }
                                Elementary::Torus { minor, .. } => {
                                    counts[2] += 1;
                                    assert!((minor - radius * scale).abs() < 1e-8);
                                }
                                _ => panic!("expected round surfaces"),
                            }
                        }
                    }
                    assert_eq!(counts, [1, 1, 1, 1]);
                    assert!(worst_radius < 0.00001);
                    let generated: HashSet<_> =
                        result.generated_faces.iter().map(|f| f.id()).collect();
                    let mut checked = HashSet::new();
                    let mut worst_dot = 1_f64;
                    for face in &result.generated_faces {
                        for edge in face.edge_iter() {
                            if checked.contains(&edge.id()) {
                                continue;
                            }
                            let neighbour = result
                                .solid
                                .face_iter()
                                .find(|f| {
                                    f.id() != face.id()
                                        && f.edge_iter().any(|e| e.id() == edge.id())
                                })
                                .unwrap();
                            let tangent = generated.contains(&neighbour.id())
                                || match neighbour.oriented_surface().elementary() {
                                    Some((
                                        Elementary::Torus {
                                            major: r, minor: m, ..
                                        },
                                        _,
                                    )) => {
                                        (r - major * scale).abs() < 1e-8
                                            && (m - minor * scale).abs() < 1e-8
                                    }
                                    Some((Elementary::Plane(p), _)) => {
                                        let n = inverse.transform_vector(p.normal()).normalize();
                                        n.y < -0.9 || if kind == 1 { n.z > 0.9 } else { n.z < -0.9 }
                                    }
                                    _ => false,
                                };
                            if !tangent {
                                continue;
                            }
                            checked.insert(edge.id());
                            let curve = edge.oriented_curve();
                            let (a, b) = curve.range_tuple();
                            for i in 0..=16 {
                                let point = curve.subs(a + (b - a) * i as f64 / 16.);
                                let normal = |f: &Face| {
                                    let s = f.oriented_surface();
                                    let (u, v) = s.search_parameter(point, None, 100).unwrap();
                                    s.normal(u, v).normalize()
                                };
                                worst_dot = worst_dot.min(normal(face).dot(normal(neighbour)));
                            }
                        }
                    }
                    assert_eq!(checked.len(), 9);
                    assert!(worst_dot > 1. - 1e-7);
                    let expected = volumes[kind] * scale.powi(3);
                    common::assert_solid(&result.solid, expected, &[0], 0.0003 * scale);
                    common::blend::assert_step(&result.solid, expected, 0.0003 * scale);
                    selected.reverse();
                }
            }
        }
    }
}

#[test]
fn torus_corner_collisions_preserve_input() {
    for kind in 0..3 {
        let (input, selected) = sector(kind);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for radius in [1.0, 1.6] {
            assert!(try_fillet_solid_edges(&input, &selected, radius, 0.001).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
    let (input, selected) = sector(3);
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges(&input, &selected, 0.1, 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
