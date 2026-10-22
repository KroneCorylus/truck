mod common;
use std::collections::HashSet;
use std::f64::consts::{FRAC_PI_2, PI};
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
        vec![vertical[0].id(), high[0].id(), high[2].id()],
    )
}
// Integrate horizontal disk sections clipped by the rounded planar corner.
fn volume(bottom: f64, top: f64, height: f64, radius: f64, concave: bool) -> f64 {
    let k = 1. - PI / 4.;
    let slope = (top - bottom) / height;
    let h = height - radius;
    let sector = if concave { 3. } else { 1. };
    let sign = if concave { -1. } else { 1. };
    let lower = sector * PI / 4.
        * (bottom * bottom * h + bottom * slope * h * h + slope * slope * h * h * h / 3.)
        - sign * k * radius * radius * h;
    let section = |theta: f64| {
        let rho = radius * theta.cos();
        let a = radius - rho;
        let r = bottom + slope * (h + radius * theta.sin());
        let corner = if concave {
            k * (radius + a).powi(2)
        } else {
            -k * rho * rho
        };
        (sector * PI * r * r / 4. - r * r * (a / r).asin() - a * (r * r - a * a).sqrt()
            + sign * a * a
            + corner)
            * radius
            * theta.cos()
    };
    let n = 4096;
    let d = FRAC_PI_2 / n as f64;
    lower
        + d / 3.
            * (section(0.)
                + section(FRAC_PI_2)
                + (1..n)
                    .map(|i| {
                        if i % 2 == 0 {
                            2. * section(i as f64 * d)
                        } else {
                            4. * section(i as f64 * d)
                        }
                    })
                    .sum::<f64>())
}
#[test]
fn planar_corner_fillets_trim_their_cylindrical_and_conical_terminals() {
    for (bottom, top, height, concave) in [
        (1., 1., 2., false),
        (1.8, 0.8, 3., false),
        (1., 1., 2., true),
        (1.8, 0.8, 3., true),
    ] {
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
        for radius in [0.1, 0.2] {
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
                            .expect("shared planar corner with curved terminal faces");
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
                                assert!((major - 2. * radius * scale).abs() < 1e-8);
                            }
                            _ => panic!("constant-radius fillet surfaces"),
                        }
                    }
                    assert_eq!(
                        (cylinders, spheres, tori),
                        if concave { (3, 0, 1) } else { (3, 1, 0) }
                    );
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
                                        inverse
                                            .transform_vector(p.normal())
                                            .normalize()
                                            .dot(Vector3::unit_z())
                                            > -0.9
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
                                assert!(normal(face).dot(normal(&neighbour)) > 1. - 1e-7);
                            }
                        }
                    }
                    assert_eq!(checked.len(), if concave { 10 } else { 9 });
                    let expected = volume(bottom, top, height, radius, concave) * scale.powi(3);
                    common::assert_solid(&result.solid, expected, &[0], 0.0003 * scale);
                    common::blend::assert_step(&result.solid, expected, 0.0003 * scale);
                    selected.reverse();
                }
            }
        }
    }
}

#[test]
fn curved_terminals_reject_colliding_corner_fillets_without_mutation() {
    for (bottom, top, height, concave) in [
        (1., 1., 2., false),
        (1.8, 0.8, 3., false),
        (1., 1., 2., true),
        (1.8, 0.8, 3., true),
    ] {
        let (solid, selected) = sector(bottom, top, height, concave);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        for radius in [top, 2. * bottom] {
            assert!(try_fillet_solid_edges(&solid, &selected, radius, 0.001).is_err());
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
