mod common;
use std::collections::HashSet;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
fn sector(concave: bool) -> (Solid, Vec<EdgeID>) {
    let radius = 1.5;
    let h = radius * 3_f64.sqrt() / 2.;
    let rho = radius / 2.;
    let sign = if concave { -1. } else { 1. };
    let low = builder::vertices([
        Point3::new(0., 0., -h),
        Point3::new(rho, 0., -h),
        Point3::new(0., sign * rho, -h),
    ]);
    let high = builder::vertices([
        Point3::new(0., 0., h),
        Point3::new(rho, 0., h),
        Point3::new(0., sign * rho, h),
    ]);
    let rim = |v: &[Vertex], z: f64| {
        vec![
            builder::line(&v[0], &v[1]),
            builder::circle_arc(
                &v[1],
                &v[2],
                Point3::new(sign * rho / 2_f64.sqrt(), rho / 2_f64.sqrt(), z),
            ),
            builder::line(&v[2], &v[0]),
        ]
    };
    let lower = rim(&low, -h);
    let upper = rim(&high, h);
    let vertical = [
        builder::line(&low[0], &high[0]),
        builder::circle_arc(&low[1], &high[1], Point3::new(radius, 0., 0.)),
        builder::circle_arc(&low[2], &high[2], Point3::new(0., sign * radius, 0.)),
    ];
    let mut shell = Shell::new();
    shell.push(
        builder::try_attach_plane(&[lower.iter().cloned().collect()])
            .unwrap()
            .inverse(),
    );
    shell.push(builder::try_attach_plane(&[upper.iter().cloned().collect()]).unwrap());
    for i in 0..3 {
        let j = (i + 1) % 3;
        let boundary = wire![
            lower[i].clone(),
            vertical[j].clone(),
            upper[i].inverse(),
            vertical[i].inverse()
        ];
        let face = if i == 1 {
            Face::new(
                vec![boundary],
                Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), radius))),
            )
        } else {
            builder::try_attach_plane(&[boundary]).unwrap()
        };
        shell.push(face);
    }
    (
        Solid::new(vec![shell]),
        vec![vertical[1].id(), upper[0].id(), upper[1].id()],
    )
}
fn polar_sector(radius: f64) -> (Solid, Vec<EdgeID>) {
    let z = radius * 3_f64.sqrt() / 2.;
    let pole = builder::vertex(Point3::new(0., 0., -radius));
    let axis = builder::vertex(Point3::new(0., 0., z));
    let x = builder::vertex(Point3::new(radius / 2., 0., z));
    let y = builder::vertex(Point3::new(0., radius / 2., z));
    let axial = builder::line(&pole, &axis);
    let radial_x = builder::line(&axis, &x);
    let radial_y = builder::line(&axis, &y);
    let top_arc = builder::circle_arc(
        &x,
        &y,
        Point3::new(radius / 8_f64.sqrt(), radius / 8_f64.sqrt(), z),
    );
    let meridian_x = builder::circle_arc(&pole, &x, Point3::new(radius, 0., 0.));
    let meridian_y = builder::circle_arc(&pole, &y, Point3::new(0., radius, 0.));
    let cap =
        builder::try_attach_plane(&[wire![radial_x.clone(), top_arc.clone(), radial_y.inverse()]])
            .unwrap();
    let side_x = builder::try_attach_plane(&[wire![
        meridian_x.clone(),
        radial_x.inverse(),
        axial.inverse()
    ]])
    .unwrap();
    let side_y =
        builder::try_attach_plane(&[wire![axial.clone(), radial_y.clone(), meridian_y.inverse()]])
            .unwrap();
    let selected = vec![axial.id(), meridian_x.id(), meridian_y.id()];
    let sphere = Face::new(
        vec![wire![meridian_x.inverse(), meridian_y, top_arc.inverse()]],
        Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), radius))),
    );
    (
        Solid::new(vec![vec![cap, side_x, side_y, sphere].into()]),
        selected,
    )
}

fn extra_half(round: f64) -> f64 {
    let r = 1.5;
    let h = r * 3_f64.sqrt() / 2.;
    let q = r - round;
    let cap = h - round;
    let contact = cap * r / q;
    let rho = (q * q - cap * cap).sqrt();
    let n = 4096;
    let a = ((contact - cap) / round).asin();
    let b = PI / 2.;
    let f = |t: f64| (rho + round * t.cos()).powi(2) * round * t.cos();
    PI / 2.
        * (r * r * (contact + h) - (contact.powi(3) + h.powi(3)) / 3.
            + (b - a) / n as f64 / 3.
                * (f(a)
                    + f(b)
                    + (1..n)
                        .map(|i| {
                            if i % 2 == 0 {
                                2. * f(a + (b - a) * i as f64 / n as f64)
                            } else {
                                4. * f(a + (b - a) * i as f64 / n as f64)
                            }
                        })
                        .sum::<f64>()))
}
#[test]
fn spherical_support_corners_preserve_radius_tangency_and_material() {
    for kind in 0..3 {
        let concave = kind == 1;
        let polar = kind == 2;
        let (original, ids) = if polar {
            polar_sector(1.5)
        } else {
            sector(concave)
        };
        let h = 1.5 * 3_f64.sqrt() / 2.;
        let input_volume = if polar {
            PI / 4. * (2.25 * (h + 1.5) - (h.powi(3) + 1.5_f64.powi(3)) / 3.)
        } else {
            (if concave { 3. } else { 1. }) * PI / 4. * (2. * 2.25 * h - 2. * h.powi(3) / 3.)
        };
        common::assert_solid(&original, input_volume, &[0], 0.001);
        // Independently integrate distance to the inward ball intersected with two half-spaces.
        for (radius, quarter, polar_volume) in [
            (0.05, 3.4411919452738977, 3.4829260368093546),
            (0.1, 3.4342246313954865, 3.464310439158544),
            (0.15, 3.4217834421245579, 3.4314225430002887),
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
                        let edge = original.edge_iter().find(|e| e.id() == *id).unwrap();
                        let curve = edge.curve();
                        let (a, b) = curve.range_tuple();
                        common::blend::edge_through(
                            &input.boundaries()[0],
                            matrix.transform_point(curve.subs((a + b) / 2.)),
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
                    let mut counts = [0, 0, 0];
                    for face in &result.generated_faces {
                        match face.surface().elementary().unwrap().0 {
                            Elementary::Cylinder { radius: r, .. } => {
                                counts[0] += 1;
                                assert!((r - radius * scale).abs() < 1e-8);
                            }
                            Elementary::Sphere { center, radius: r } => {
                                counts[1] += 1;
                                assert!((r - radius * scale).abs() < 1e-8);
                                let center = inverse.transform_point(center);
                                assert!(
                                    (center.to_vec().magnitude() - (1.5 - radius)).abs() < 1e-8
                                );
                                assert!((center.y - radius).abs() < 1e-8);
                                if polar {
                                    assert!((center.x - radius).abs() < 1e-8);
                                    assert!(center.z < 0.);
                                } else {
                                    assert!((center.z - (h - radius)).abs() < 1e-8);
                                }
                            }
                            Elementary::Torus { minor, .. } => {
                                counts[2] += 1;
                                assert!((minor - radius * scale).abs() < 1e-8);
                            }
                            _ => panic!("constant radius analytic surfaces"),
                        }
                    }
                    assert_eq!(counts, [1, 1, 2]);
                    let generated: HashSet<_> =
                        result.generated_faces.iter().map(|f| f.id()).collect();
                    let mut checked = HashSet::new();
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
                                    Some((Elementary::Plane(p), _)) => {
                                        let n = inverse.transform_vector(p.normal()).normalize();
                                        n.y < -0.9 || if polar { n.x < -0.9 } else { n.z > 0.9 }
                                    }
                                    Some((Elementary::Sphere { radius: r, .. }, _)) => {
                                        (r - 1.5 * scale).abs() < 1e-8
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
                                assert!(normal(face).dot(normal(neighbour)) > 1. - 1e-7);
                            }
                        }
                    }
                    assert_eq!(checked.len(), 9);
                    let expected = ((if polar { polar_volume } else { quarter })
                        + if concave { extra_half(radius) } else { 0. })
                        * scale.powi(3);
                    common::assert_solid(&result.solid, expected, &[0], 0.0003 * scale);
                    common::blend::assert_step(&result.solid, expected, 0.0003 * scale);
                    selected.reverse();
                }
            }
        }
    }
}
#[test]
fn spherical_corner_collisions_preserve_input() {
    for kind in 0..3 {
        let (input, selected) = if kind == 2 {
            polar_sector(1.5)
        } else {
            sector(kind == 1)
        };
        let before = serde_json::to_string(&input.compress()).unwrap();
        for radius in [0.2, 0.4, 1.6] {
            assert!(try_fillet_solid_edges(&input, &selected, radius, 0.001).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
