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

fn selection(
    input: &Solid,
    inverse: Matrix4,
    a: f64,
    b: f64,
    tangent: f64,
    scale: f64,
) -> Vec<(EdgeID, [f64; 2])> {
    let shell = &input.boundaries()[0];
    let mut seen = std::collections::HashSet::new();
    input
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .map(|edge| {
            let sides: Vec<_> = shell
                .iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .collect();
            let axis_edge = [edge.front().point(), edge.back().point()]
                .into_iter()
                .all(|p| {
                    let p = inverse.transform_point(p);
                    p.y.abs() < 1e-8 && p.z.abs() < 1e-8
                });
            let cylinder = sides.iter().any(|f| {
                matches!(
                    f.surface().elementary(),
                    Some((Elementary::Cylinder { .. }, _))
                )
            });
            let d = std::array::from_fn(|i| {
                let surface = sides[i].surface();
                let value =
                    if matches!(surface.elementary(), Some((Elementary::Cylinder { .. }, _))) {
                        if matches!(edge.curve(), Curve::Line(_)) {
                            tangent
                        } else {
                            a
                        }
                    } else {
                        let n = inverse.transform_vector(surface.normal(0., 0.)).normalize();
                        if n.x.abs() > 0.9 || (axis_edge && n.y.abs() > 0.9) || cylinder {
                            b
                        } else {
                            a
                        }
                    };
                value * scale
            });
            (edge.id(), d)
        })
        .collect()
}

fn check(a: f64, t: f64, volume: f64, scale: f64, reorder: bool, swap: bool) {
    let permutation = if swap {
        Matrix4::from_cols(
            Vector4::unit_x(),
            Vector4::unit_z(),
            Vector4::unit_y(),
            Vector4::unit_w(),
        )
    } else {
        Matrix4::identity()
    };
    let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.37))
        * Matrix4::from_scale(scale)
        * permutation;
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&quarter(), transform * permutation);
    if reorder {
        input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
    }
    let mut selected = selection(&input, inverse, a, 0.2, t, scale);
    if reorder {
        selected.reverse();
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result =
        try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    common::assert_solid(&result.solid, volume * scale.powi(3), &[0], 0.0001 * scale);
    common::blend::assert_step(&result.solid, volume * scale.powi(3), 0.0001 * scale);
    let shell = &result.solid.boundaries()[0];
    let b = 0.2;
    let cy = t / 1_f64.hypot(t);
    let cz = 1. / 1_f64.hypot(t);
    let rz = (0.8_f64.powi(2) - b * b).sqrt();
    for far in [false, true] {
        let map = |mut p: Point3| {
            if far {
                p.x = 1. - p.x;
            }
            transform.transform_point(p)
        };
        let triangle = [
            Point3::new(0., b, b),
            Point3::new(a, a, 0.),
            Point3::new(a, 0., b),
        ]
        .map(map);
        let mut expected = triangle.to_vec();
        for exchange in [false, true] {
            expected.extend(
                [
                    Point3::new(a, cy, cz),
                    Point3::new(a, 0., 0.8),
                    Point3::new(0., b, rz),
                ]
                .map(|p| {
                    map(if exchange {
                        Point3::new(p.x, p.z, p.y)
                    } else {
                        p
                    })
                }),
            );
        }
        for p in expected {
            let v = shell
                .vertex_iter()
                .find(|v| v.point().distance(p) < 1e-8 * scale)
                .expect("missing independent contact");
            assert_eq!(
                shell
                    .iter()
                    .filter(|f| f.vertex_iter().any(|w| w == v))
                    .count(),
                4
            );
        }
        let face = shell
            .iter()
            .find(|f| {
                triangle.iter().all(|p| {
                    f.vertex_iter()
                        .any(|v| v.point().distance(*p) < 1e-8 * scale)
                })
            })
            .unwrap();
        assert_eq!(face.edge_iter().count(), 3);
        for weights in [
            [1. / 3.; 3],
            [0.6, 0.2, 0.2],
            [0.2, 0.6, 0.2],
            [0.2, 0.2, 0.6],
        ] {
            let p = Point3::from_vec((0..3).map(|i| triangle[i].to_vec() * weights[i]).sum());
            let surface = face.oriented_surface();
            let (u, v) = surface.search_parameter(p, None, 100).unwrap();
            assert!(surface.subs(u, v).distance(p) < 1e-8 * scale);
            let mut inward = Vector3::new(b * b, a * b, a * a);
            if far {
                inward.x = -inward.x;
            }
            assert!(surface.normal(u, v).dot(transform.transform_vector(inward)) < 0.);
        }
    }
    let mut uses = std::collections::HashMap::<_, Vec<bool>>::new();
    for edge in shell.edge_iter() {
        uses.entry(edge.id()).or_default().push(edge.orientation());
    }
    assert!(uses.values().all(|v| v.len() == 2 && v[0] != v[1]));
    for v in shell.vertex_iter() {
        let p = inverse.transform_point(v.point());
        assert!(
            p.x >= -1e-8
                && p.x <= 1. + 1e-8
                && p.y >= -1e-8
                && p.z >= -1e-8
                && p.y.hypot(p.z) <= 1. + 1e-8
        );
    }
}

#[test]
fn translated_triangle_family_and_perturbation() {
    for shift in [0., 4., 1e12] {
        for perturbed in [false, true] {
            let transform = Matrix4::from_translation(Vector3::new(shift, shift, shift));
            let input = builder::transformed(&quarter(), transform);
            let mut selected = selection(
                &input,
                transform.invert().unwrap(),
                0.1,
                0.2,
                (2. * (0.05_f64).asin()).tan(),
                1.,
            );
            if perturbed {
                let id = input
                    .edge_iter()
                    .find(|e| {
                        [e.front().point(), e.back().point()].into_iter().all(|p| {
                            let p = p - Vector3::new(shift, shift, shift);
                            p.y.abs() < 1e-8 && p.z.abs() < 1e-8
                        })
                    })
                    .unwrap()
                    .id();
                let d = &mut selected.iter_mut().find(|(i, _)| *i == id).unwrap().1;
                for value in d {
                    if (*value - 0.2).abs() < 1e-8 {
                        *value = 0.199;
                    }
                }
            }
            let before = serde_json::to_string(&input.compress()).unwrap();
            let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            if perturbed || shift == 1e12 {
                assert!(
                    result.is_err(),
                    "translation {shift}, perturbed {perturbed}"
                );
            } else {
                let result = result.unwrap();
                let local = builder::transformed(&result.solid, transform.invert().unwrap());
                common::assert_solid(&local, 0.6980521376877056, &[0], 0.0001);
                common::blend::assert_step(&local, 0.6980521376877056, 0.0001);
            }
        }
    }
}

#[test]
fn unequal_triangle_distance_distance() {
    check(
        0.1,
        (2. * (0.05_f64).asin()).tan(),
        0.6980521376877056,
        1.,
        false,
        false,
    );
}
#[test]
fn unequal_triangle_distance_angle() {
    check(
        0.2 / 3_f64.sqrt(),
        0.11255363778941033,
        0.6858063469848357,
        1.,
        false,
        false,
    );
}
#[test]
fn unequal_triangle_transform_scale_order() {
    for (scale, swap) in [(0.1, false), (10., false), (1., true)] {
        check(
            0.1,
            (2. * (0.05_f64).asin()).tan(),
            0.6980521376877056,
            scale,
            true,
            swap,
        );
    }
}

#[test]
fn outside_triangle_family_is_immutable() {
    for variant in 0..4 {
        let input = quarter();
        let mut selected = selection(
            &input,
            Matrix4::identity(),
            0.1,
            0.2,
            (2. * (0.05_f64).asin()).tan(),
            1.,
        );
        match variant {
            0 => {
                selected[0].1[0] *= 1.01;
            }
            1 => {
                for (_, d) in &mut selected {
                    d[0] *= 8.;
                    d[1] *= 8.;
                }
            }
            2 => {
                for (_, d) in &mut selected {
                    if (d[0] - 0.1).abs() < 1e-8 {
                        d[0] = 0.5;
                    }
                    if (d[1] - 0.1).abs() < 1e-8 {
                        d[1] = 0.5;
                    }
                }
            }
            _ => {
                let id = input
                    .edge_iter()
                    .find(|e| {
                        [e.front().point(), e.back().point()]
                            .into_iter()
                            .all(|p| p.y.abs() < 1e-8 && p.z.abs() < 1e-8)
                    })
                    .unwrap()
                    .id();
                let d = &mut selected.iter_mut().find(|(i, _)| *i == id).unwrap().1;
                d[0] *= 1.01;
                d[1] *= 1.01;
            }
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err(),
            "variant {variant}"
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
