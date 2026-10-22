mod common;
use std::collections::HashSet;
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
fn mixed_torus_corner_preserves_contacts_material_and_step() {
    for (radius, expected) in [
        (0.03, 1.5395408738884995),
        (0.06, 1.5383527170561335),
        (0.1, 1.535_440_006_371_804),
    ] {
        for moved in [false, true] {
            let (original, ids) = sector(3);
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
                    try_fillet_solid_edges(&input, &selected, radius * factor, 1e-5 * factor)
                        .unwrap();
                let solid = &result.solid;
                let r = &radius;
                assert_eq!(result.generated_faces.len(), 4);
                assert!(solid.is_geometric_consistent());
                let mut worst_radius = 0_f64;
                let mut fits = 0;
                for face in solid.face_iter() {
                    if let Surface::Fillet(fit) = face.surface() {
                        fits += 1;
                        let af = fit.entity();
                        let wall=[af.surface0(),af.surface1()].into_iter().any(|s|matches!(s.as_ref(),Surface::Plane(p) if inverse.transform_vector(fit.transform().transform_vector(p.normal())).normalize().y.abs()>0.9));
                        for j in 0..=32 {
                            let (a, b) = af.range_tuple().1;
                            let v = a + (b - a) * j as f64 / 32.;
                            let p = [0., 1.].map(|u| {
                                inverse
                                    .transform_point(fit.transform().transform_point(af.subs(u, v)))
                            });
                            let center = if wall {
                                let q = p.iter().find(|p| p.y.abs() < 1e-7).expect("wall contact");
                                *q + Vector3::unit_y() * *r
                            } else {
                                let h = 0.5 * 160_f64.to_radians().sin();
                                let q = p
                                    .iter()
                                    .find(|p| (p.z - h).abs() < 1e-7)
                                    .expect("concave cap contact");
                                *q + Vector3::unit_z() * *r
                            };
                            for i in 0..=16 {
                                let p = inverse.transform_point(
                                    fit.transform().transform_point(af.subs(i as f64 / 16., v)),
                                );
                                worst_radius = worst_radius.max((p.distance(center) - r).abs());
                            }
                        }
                    }
                }
                let mut seen = HashSet::new();
                let mut worst_dot = 1_f64;
                for face in solid
                    .face_iter()
                    .filter(|f| matches!(f.surface(), Surface::Fillet(_)))
                {
                    for edge in face.edge_iter() {
                        if seen.contains(&edge.id()) {
                            continue;
                        }
                        let other = solid
                            .face_iter()
                            .find(|f| {
                                f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id())
                            })
                            .unwrap();
                        if let Surface::Plane(p) = other.oriented_surface() {
                            let n = inverse.transform_vector(p.normal()).normalize();
                            if n.x.abs() > 0.9 || n.z < -0.9 {
                                continue;
                            }
                        }
                        seen.insert(edge.id());
                        let c = edge.oriented_curve();
                        let (a, b) = c.range_tuple();
                        let surfaces = [face.oriented_surface(), other.oriented_surface()];
                        for i in 0..=16 {
                            let p = c.subs(a + (b - a) * i as f64 / 16.);
                            let normals = surfaces.each_ref().map(|s| {
                                let (u, v) = s.search_parameter(p, None, 100).unwrap();
                                s.normal(u, v).normalize()
                            });
                            worst_dot = worst_dot.min(normals[0].dot(normals[1]));
                        }
                    }
                }
                println!("quality fits {fits} radius_error {worst_radius} g1_edges {} minimum_dot {worst_dot}",seen.len());

                assert_eq!(fits, 4);
                assert_eq!(seen.len(), 10);
                assert!(worst_radius < 1e-5, "radius residual {worst_radius}");
                assert!(worst_dot > 1. - 1e-7, "normal dot {worst_dot}");
                common::assert_solid(solid, expected * factor.powi(3), &[0], 0.0003 * factor);
                common::blend::assert_step(solid, expected * factor.powi(3), 0.0003 * factor);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                selected.reverse();
            }
        }
    }
}
#[test]
fn oversized_mixed_torus_corner_preserves_input() {
    let (input, mut selected) = sector(3);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [1., 1.6] {
        for _ in 0..2 {
            assert!(try_fillet_solid_edges(&input, &selected, radius, 1e-5).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            selected.reverse();
        }
    }
}
