mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges_with_distances, try_fillet_solid_edges};

const TOL: f64 = 0.001;
const RADIUS: f64 = 6.;
const LENGTH: f64 = 10.;
const INPUT_VOLUME: f64 = 4000. - 180. * PI;

fn channel() -> Solid {
    let block = common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., 10.));
    let mut tool =
        common::modeling::cylinder(Point3::new(0., 0., 10.), Vector3::unit_y(), RADIUS, 11.);
    tool.not();
    truck_shapeops::and(&block, &tool, TOL).unwrap()
}

fn transforms() -> [Matrix4; 2] {
    [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ]
}

fn integral(f: impl Fn(f64) -> f64, end: f64) -> f64 {
    let n = 4096;
    let step = end / f64::from(n);
    (0..=n)
        .map(|i| {
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * f(f64::from(i) * step)
        })
        .sum::<f64>()
        * step
        / 3.
}

fn fillet_side_area(radius: f64) -> f64 {
    let center = (RADIUS * RADIUS + 2. * RADIUS * radius).sqrt();
    // Integrate the area between the original channel circle and its tangent fillet circle.
    integral(
        |theta| {
            let depth = radius * (1. - theta.cos());
            let contact = center - radius * theta.sin();
            (contact - (RADIUS * RADIUS - depth * depth).sqrt()) * radius * theta.sin()
        },
        (radius / (RADIUS + radius)).acos(),
    )
}

fn assert_contact(solid: &Solid, transform: Matrix4, point: Point3) {
    assert!(
        solid
            .vertex_iter()
            .any(|vertex| vertex.point().near(&transform.transform_point(point))),
        "missing exact contact {point:?}"
    );
}

fn chamfer_section(plane: f64, cylinder: f64) -> (f64, f64, f64) {
    let denominator = RADIUS.hypot(cylinder);
    let contact = RADIUS * RADIUS / denominator;
    let depth = RADIUS * cylinder / denominator;
    let area = depth * (RADIUS + plane + contact) / 2.
        - (depth * contact + RADIUS * RADIUS * (depth / RADIUS).asin()) / 2.;
    (area, contact, depth)
}

fn section_widths(fillet: bool, plane: f64, cylinder: f64, depth: f64) -> (f64, f64) {
    let original = (RADIUS * RADIUS - depth * depth).sqrt();
    if fillet {
        let r = plane;
        let side = if depth <= RADIUS * r / (RADIUS + r) {
            (RADIUS * RADIUS + 2. * RADIUS * r).sqrt()
                - (r * r - (r - depth).powi(2)).max(0.).sqrt()
        } else {
            original
        };
        let back = r - (r * r - (r - depth).powi(2)).max(0.).sqrt();
        (side - original, back)
    } else {
        let (_, contact, end) = chamfer_section(plane, cylinder);
        let side = if depth <= end {
            RADIUS + plane + (contact - RADIUS - plane) * depth / end
        } else {
            original
        };
        (side - original, plane * (1. - depth / cylinder))
    }
}

fn joined_volume(fillet: bool, plane: f64, cylinder: f64, sides: usize) -> f64 {
    // Each horizontal void section gains side strips and a back strip, including their overlap.
    INPUT_VOLUME
        - integral(
            |depth| {
                let original = 2. * (RADIUS * RADIUS - depth * depth).sqrt();
                let (side, back) = section_widths(fillet, plane, cylinder, depth);
                let extension = sides as f64 * side;
                LENGTH * extension + back * (original + extension)
            },
            if fillet { plane } else { cylinder },
        )
}

#[test]
fn connected_channel_rims_share_exact_miters_and_preserve_the_circular_channel() {
    let source = channel();
    for transform in transforms() {
        let input = builder::transformed(&source, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let top_normal = transform.transform_vector(Vector3::unit_z());
        let ids = [
            Point3::new(0., 0., 10.),
            Point3::new(-RADIUS, 5., 10.),
            Point3::new(RADIUS, 5., 10.),
        ]
        .map(|point| {
            common::blend::edge_through(&input.boundaries()[0], transform.transform_point(point))
                .id()
        });
        for (fillet, plane, cylinder) in [
            (true, 0.2, 0.2),
            (true, 1., 1.),
            (false, 0.2, 0.2),
            (false, 1., 1.),
            (false, 0.3, 0.7),
            (false, 0.7, 0.3),
        ] {
            for indices in [vec![0, 1], vec![0, 2], vec![0, 1, 2]] {
                let mut selected: Vec<_> = indices.iter().map(|&i| ids[i]).collect();
                let operation = |selected: &[EdgeID]| {
                    if fillet {
                        try_fillet_solid_edges(&input, selected, plane, TOL)
                    } else {
                        let distances: Vec<_> = selected
                            .iter()
                            .map(|&id| {
                                let sizes: Vec<_> = input
                                    .face_iter()
                                    .filter(|face| face.edge_iter().any(|edge| edge.id() == id))
                                    .map(|face| match face.oriented_surface() {
                                        Surface::Plane(p)
                                            if p.normal().normalize().dot(top_normal) > 0.9 =>
                                        {
                                            plane
                                        }
                                        _ => cylinder,
                                    })
                                    .collect();
                                (id, [sizes[0], sizes[1]])
                            })
                            .collect();
                        try_chamfer_solid_edges_with_distances(&input, &distances, TOL)
                    }
                };
                let result = operation(&selected).unwrap_or_else(|error| panic!(
                    "fillet {fillet}, distances [{plane}, {cylinder}], selection {indices:?}: {error:?}"));
                let volume = joined_volume(fillet, plane, cylinder, indices.len() - 1);
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, volume, &[0], TOL);
                if plane == 1. || plane == 0.3 {
                    common::blend::assert_step(&result.solid, volume, TOL);
                }
                let end = if fillet {
                    RADIUS * plane / (RADIUS + plane)
                } else {
                    chamfer_section(plane, cylinder).2
                };
                for &side in &indices[1..] {
                    let sign = if side == 1 { -1. } else { 1. };
                    for fraction in [0., 0.25, 0.5, 1.] {
                        let depth = fraction * end;
                        let (extra, back) = section_widths(fillet, plane, cylinder, depth);
                        let width = (RADIUS * RADIUS - depth * depth).sqrt() + extra;
                        let point = Point3::new(sign * width, -back, 10. - depth);
                        if fraction == 0. || fraction == 1. {
                            assert_contact(&result.solid, transform, point);
                        } else {
                            common::blend::edge_through(
                                &result.solid.boundaries()[0],
                                transform.transform_point(point),
                            );
                        }
                    }
                }
                selected.reverse();
                let reversed = operation(&selected).unwrap();
                assert_eq!(
                    serde_json::to_string(&result.solid.compress()).unwrap(),
                    serde_json::to_string(&reversed.solid.compress()).unwrap()
                );
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}

#[test]
fn cylindrical_channel_lips_chamfer_with_equal_and_unequal_distances() {
    let source = channel();
    for transform in transforms() {
        let input = builder::transformed(&source, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let ids = [-RADIUS, RADIUS].map(|x| {
            common::blend::edge_through(
                &input.boundaries()[0],
                transform.transform_point(Point3::new(x, 5., 10.)),
            )
            .id()
        });
        for [plane, cylinder] in [[0.1; 2], [0.5; 2], [0.3, 0.7], [0.7, 0.3]] {
            for indices in [vec![0], vec![1], vec![0, 1]] {
                let mut selected: Vec<_> = indices
                    .iter()
                    .map(|&i| {
                        let sizes: Vec<_> = input
                            .face_iter()
                            .filter(|face| face.edge_iter().any(|edge| edge.id() == ids[i]))
                            .map(|face| match face.surface().elementary().unwrap().0 {
                                Elementary::Plane(_) => plane,
                                Elementary::Cylinder { .. } => cylinder,
                                other => panic!("unexpected lip support {other:?}"),
                            })
                            .collect();
                        assert_eq!(sizes.len(), 2);
                        (ids[i], [sizes[0], sizes[1]])
                    })
                    .collect();
                let result = try_chamfer_solid_edges_with_distances(&input, &selected, TOL)
                    .unwrap_or_else(|error| {
                        panic!("distances [{plane}, {cylinder}], sides {indices:?}: {error:?}")
                    });
                let (area, contact, depth) = chamfer_section(plane, cylinder);
                let volume = INPUT_VOLUME - indices.len() as f64 * LENGTH * area;
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, volume, &[0], TOL);
                if plane == 0.3 {
                    common::blend::assert_step(&result.solid, volume, TOL);
                }
                for &index in &indices {
                    let sign = if index == 0 { -1. } else { 1. };
                    for y in [0., LENGTH] {
                        assert_contact(
                            &result.solid,
                            transform,
                            Point3::new(sign * (RADIUS + plane), y, 10.),
                        );
                        assert_contact(
                            &result.solid,
                            transform,
                            Point3::new(sign * contact, y, 10. - depth),
                        );
                    }
                }
                selected.reverse();
                let reversed =
                    try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
                assert_eq!(
                    serde_json::to_string(&result.solid.compress()).unwrap(),
                    serde_json::to_string(&reversed.solid.compress()).unwrap()
                );
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}

#[test]
fn channel_lip_blends_reject_collapsed_outer_walls_without_mutation() {
    let input = channel();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let ids = [-RADIUS, RADIUS]
        .map(|x| common::blend::edge_through(&input.boundaries()[0], Point3::new(x, 5., 10.)).id());
    for size in [8., 12., 20.] {
        for selected in [vec![ids[0]], ids.to_vec()] {
            assert!(try_fillet_solid_edges(&input, &selected, size, TOL).is_err());
            let chamfers: Vec<_> = selected.iter().map(|&id| (id, [size; 2])).collect();
            assert!(try_chamfer_solid_edges_with_distances(&input, &chamfers, TOL).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn cylindrical_channel_lips_round_independently_and_together() {
    let source = channel();
    for transform in transforms() {
        let input = builder::transformed(&source, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let ids = [-RADIUS, RADIUS].map(|x| {
            common::blend::edge_through(
                &input.boundaries()[0],
                transform.transform_point(Point3::new(x, 5., 10.)),
            )
            .id()
        });
        for radius in [0.1, 0.5, 1.] {
            for indices in [vec![0], vec![1], vec![0, 1]] {
                let mut selected: Vec<_> = indices.iter().map(|&i| ids[i]).collect();
                let result = try_fillet_solid_edges(&input, &selected, radius, TOL).unwrap_or_else(
                    |error| panic!("radius {radius}, sides {indices:?}: {error:?}"),
                );
                let volume =
                    INPUT_VOLUME - indices.len() as f64 * LENGTH * fillet_side_area(radius);
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, volume, &[0], TOL);
                if radius == 0.5 {
                    common::blend::assert_step(&result.solid, volume, TOL);
                }
                let center = (RADIUS * RADIUS + 2. * RADIUS * radius).sqrt();
                for &index in &indices {
                    let sign = if index == 0 { -1. } else { 1. };
                    for y in [0., LENGTH] {
                        assert_contact(
                            &result.solid,
                            transform,
                            Point3::new(sign * center, y, 10.),
                        );
                        assert_contact(
                            &result.solid,
                            transform,
                            Point3::new(
                                sign * RADIUS * center / (RADIUS + radius),
                                y,
                                10. - RADIUS * radius / (RADIUS + radius),
                            ),
                        );
                    }
                }
                selected.reverse();
                let reversed = try_fillet_solid_edges(&input, &selected, radius, TOL).unwrap();
                assert_eq!(
                    serde_json::to_string(&result.solid.compress()).unwrap(),
                    serde_json::to_string(&reversed.solid.compress()).unwrap()
                );
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}
