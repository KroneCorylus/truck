mod common;
use std::collections::HashSet;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn sector(bottom: f64, top: f64, height: f64, concave: bool) -> (Solid, Vec<EdgeID>) {
    let sign = if concave { -1. } else { 1. };
    let lower = builder::vertices([
        Point3::origin(),
        Point3::new(bottom, 0., 0.),
        Point3::new(0., sign * bottom, 0.),
    ]);
    let upper = builder::vertices([
        Point3::new(0., 0., height),
        Point3::new(top, 0., height),
        Point3::new(0., sign * top, height),
    ]);
    let wire = |points: &[Vertex], radius: f64, z: f64| {
        vec![
            builder::line(&points[0], &points[1]),
            builder::circle_arc(
                &points[1],
                &points[2],
                Point3::new(sign * radius / 2_f64.sqrt(), radius / 2_f64.sqrt(), z),
            ),
            builder::line(&points[2], &points[0]),
        ]
    };
    let low = wire(&lower, bottom, 0.);
    let high = wire(&upper, top, height);
    let vertical: Vec<_> = (0..3)
        .map(|i| builder::line(&lower[i], &upper[i]))
        .collect();
    let mut shell = Shell::new();
    shell.push(
        builder::try_attach_plane(&[low.iter().cloned().collect()])
            .unwrap()
            .inverse(),
    );
    shell.push(builder::try_attach_plane(&[high.iter().cloned().collect()]).unwrap());
    for i in 0..3 {
        let j = (i + 1) % 3;
        let surface: Surface = if i == 1 {
            Surface::RevolutedCurve(Processor::new(RevolutedCurve::by_revolution(
                Curve::Line(Line(lower[1].point(), upper[1].point())),
                Point3::origin(),
                Vector3::unit_z(),
            )))
            .inverse()
        } else {
            Plane::new(lower[i].point(), lower[j].point(), upper[j].point()).into()
        };
        shell.push(Face::new(
            vec![vec![
                low[i].clone(),
                vertical[j].clone(),
                high[i].inverse(),
                vertical[i].inverse(),
            ]
            .into()],
            surface,
        ));
    }
    (
        Solid::new(vec![shell]),
        vec![vertical[1].id(), high[0].id(), high[1].id()],
    )
}
// Integrate horizontal circular sections with the exact rolling-sphere corner.
fn volume(r: f64, concave: bool) -> f64 {
    let alpha = if concave { 3. * PI / 2. } else { PI / 2. };
    let a = 1. - r;
    let theta = (r / a).asin();
    let x = (a * a - r * r).sqrt();
    let area = |rho: f64| {
        let b = r - rho;
        let outer = a + rho;
        0.5 * (-b * x + x * rho * (theta.sin() + 1.) - r * rho * theta.cos()
            + rho * rho * (theta + PI / 2.)
            + outer * outer * (alpha - theta))
    };
    let n = 4096;
    let d = PI / 2. / n as f64;
    let f = |t: f64| area(r * t.cos()) * r * t.cos();
    (2. - r) * area(r)
        + d / 3.
            * (f(0.)
                + f(PI / 2.)
                + (1..n)
                    .map(|i| (if i % 2 == 0 { 2. } else { 4. }) * f(i as f64 * d))
                    .sum::<f64>())
}
#[test]
fn cylindrical_rim_corners_preserve_radius_tangency_and_material() {
    for (bottom, top, height, concave) in [(1., 1., 2., false), (1., 1., 2., true)] {
        let (original, ids) = sector(bottom, top, height, concave);
        common::assert_solid(
            &original,
            (if concave { 3. } else { 1. })
                * PI
                * height
                * (bottom * bottom + bottom * top + top * top)
                / 12.,
            &[0],
            0.001,
        );
        for radius in [0.1, 0.2, 0.45] {
            for moved in [false, true] {
                let scale = if moved { 2.5 } else { 1. };
                let transform = if moved {
                    Matrix4::from_translation(Vector3::new(7., -11., 13.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                        * Matrix4::from_scale(scale)
                } else {
                    Matrix4::identity()
                };
                let inverse = transform.invert().unwrap();
                let solid = builder::transformed(&original, transform);
                let mut selected: Vec<_> = ids
                    .iter()
                    .map(|id| {
                        let edge = original.edge_iter().find(|e| e.id() == *id).unwrap();
                        let c = edge.curve();
                        let (a, b) = c.range_tuple();
                        common::blend::edge_through(
                            &solid.boundaries()[0],
                            transform.transform_point(c.subs((a + b) / 2.)),
                        )
                        .id()
                    })
                    .collect();
                let before = serde_json::to_string(&solid.compress()).unwrap();
                for _ in 0..2 {
                    let result =
                        try_fillet_solid_edges(&solid, &selected, radius * scale, 0.001 * scale)
                            .expect("cylinder and two-plane fillet corner");
                    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
                    assert_eq!(result.generated_faces.len(), 4);
                    let mut cylinders = 0;
                    let mut spheres = 0;
                    let mut tori = 0;
                    for face in &result.generated_faces {
                        match face.surface().elementary().unwrap().0 {
                            Elementary::Cylinder { radius: r, .. } => {
                                cylinders += 1;
                                assert!((r - radius * scale).abs() < 1e-8);
                            }
                            Elementary::Sphere { radius: r, .. } => {
                                spheres += 1;
                                assert!((r - radius * scale).abs() < 1e-8);
                            }
                            Elementary::Torus { minor, major, .. } => {
                                tori += 1;
                                assert!((minor - radius * scale).abs() < 1e-8);
                                assert!((major - (1. - radius) * scale).abs() < 1e-8);
                            }
                            _ => panic!("constant-radius fillet surfaces"),
                        }
                    }
                    assert_eq!((cylinders, spheres, tori), (2, 1, 1));
                    assert!(result.solid.is_geometric_consistent());
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
                                .find(|other| {
                                    other.id() != face.id()
                                        && other.edge_iter().any(|e| e.id() == edge.id())
                                })
                                .unwrap();
                            let tangent = generated.contains(&neighbour.id())
                                || match neighbour.oriented_surface().elementary() {
                                    Some((Elementary::Plane(p), _)) => {
                                        let n = inverse.transform_vector(p.normal()).normalize();
                                        n.dot(Vector3::unit_z()) > 0.9
                                            || n.dot(-Vector3::unit_y()) > 0.9
                                    }
                                    Some((Elementary::Cylinder { radius: r, .. }, _)) => {
                                        (r - scale).abs() < 1e-8
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
                    let expected = volume(radius, concave) * scale.powi(3);
                    common::assert_solid(&result.solid, expected, &[0], 0.0003 * scale);
                    common::blend::assert_step(&result.solid, expected, 0.0003 * scale);
                    selected.reverse();
                }
            }
        }
    }
}

#[test]
fn cylindrical_corner_collisions_leave_the_input_unchanged() {
    for concave in [false, true] {
        let (solid, selected) = sector(1., 1., 2., concave);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        for radius in [0.51, 1.] {
            assert!(try_fillet_solid_edges(&solid, &selected, radius, 0.001).is_err());
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
