//! Original R10 input and sequential fillets around a shared cube corner.
mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::{fillet_solid_along_wire, fillet_solid_edges, BlendResult};

fn input() -> Solid {
    let compressed = serde_json::from_str(include_str!("R10-first-fillet-solid.json")).unwrap();
    Solid::extract(compressed).unwrap()
}

fn second_wire(solid: &Solid) -> Wire { second_wire_at(solid, 70.0, false) }

fn second_wire_at(solid: &Solid, size: f64, swapped: bool) -> Wire {
    let canonical = |p: Point3| {
        if swapped {
            Point3::new(size - p.z, p.y, size - p.x)
        } else {
            p
        }
    };
    let top = solid
        .face_iter()
        .find(|face| {
            face.vertex_iter()
                .all(|v| (canonical(v.point()).z - size).abs() < 1e-6)
        })
        .unwrap();
    let seed = top
        .edge_iter()
        .find(|edge| {
            let (a, b) = (
                canonical(edge.front().point()),
                canonical(edge.back().point()),
            );
            a.y.abs() < 1e-6 && b.y.abs() < 1e-6 && (a.x - b.x).abs() > size / 2.0
        })
        .unwrap();

    // Match the application's tangent-chain expansion, including seed orientation.
    let mut unique = HashSet::new();
    let edges: Vec<_> = solid
        .edge_iter()
        .filter(|e| unique.insert(e.id()))
        .collect();
    let mut wire: Wire = vec![seed].into();
    for reverse in [false, true] {
        if reverse {
            wire.invert();
        }
        while !wire.is_cyclic() {
            let last = wire.back().unwrap();
            let vertex = last.back();
            let curve = last.oriented_curve();
            let tangent = curve.der(curve.range_tuple().1).normalize();
            let candidates: Vec<_> = edges
                .iter()
                .filter(|edge| !wire.iter().any(|old| old.id() == edge.id()))
                .filter_map(|edge| {
                    let edge = if edge.front() == vertex {
                        edge.clone()
                    } else if edge.back() == vertex {
                        edge.inverse()
                    } else {
                        return None;
                    };
                    let curve = edge.oriented_curve();
                    let next = curve.der(curve.range_tuple().0).normalize();
                    (tangent.dot(next) > 1.0 - 1e-8).then_some(edge)
                })
                .collect();
            match candidates.as_slice() {
                [] => break,
                [edge] => wire.push_back(edge.clone()),
                _ => panic!("ambiguous tangent chain"),
            }
        }
        if reverse {
            wire.invert();
        }
    }
    assert_eq!(wire.len(), 3);
    assert!(!wire.is_cyclic());
    wire
}

#[test]
fn first_fillet_input_and_second_wire_control() {
    let solid = input();
    assert_eq!(solid.face_iter().count(), 7);
    assert!(Solid::try_new(solid.boundaries().clone()).is_ok());
    second_wire(&solid);
}

#[test]
fn adjacent_second_fillet_must_not_panic_or_mutate_input() {
    let solid = input();
    let wire = second_wire(&solid);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fillet_solid_along_wire(&solid, &wire, 1.0, 0.001)
    }));
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    let result = result.expect("second fillet panicked in the kernel");
    assert!(
        result.is_none(),
        "equal radii require a collapsed contact curve"
    );
}

fn assert_history(input: &Solid, result: &BlendResult, generated: usize) {
    assert_eq!(result.generated_faces.len(), generated);
    let mut accounted = HashSet::new();
    for original in input.face_iter() {
        let replacements: Vec<_> = result
            .modified_faces
            .iter()
            .filter(|(id, _)| *id == original.id())
            .collect();
        assert!(replacements.len() <= 1);
        let face = replacements.first().map_or(original, |(_, face)| face);
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        assert!(accounted.insert(face.id()));
    }
    for (id, _) in &result.modified_faces {
        assert!(input.face_iter().any(|face| face.id() == *id));
    }
    for face in &result.generated_faces {
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        assert!(accounted.insert(face.id()));
    }
    assert_eq!(accounted.len(), result.solid.face_iter().count());
}

fn expected_volume(size: f64, first: f64, second: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let first_removed = first * first * (1.0 - pi / 4.0) * size;
    let area = second * second * (1.0 - pi / 4.0);
    let centroid = second * (10.0 - 3.0 * pi) / (12.0 - 3.0 * pi);
    // Pappus: two straight runs plus the quarter-circle swept by the section centroid.
    let length = 2.0 * (size - first) + pi / 2.0 * (first - centroid);
    size.powi(3) - first_removed - area * length
}

#[test]
fn smaller_second_radius_on_original_fixture_preserves_geometry_and_history() {
    let solid = input();
    let wire = second_wire(&solid);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    for radius in [0.2, 0.5, 0.8] {
        let result = fillet_solid_along_wire(&solid, &wire, radius, 0.001).unwrap();
        assert_history(&solid, &result, 3);
        common::assert_solid(
            &result.solid,
            expected_volume(70.0, 1.0, radius),
            &[0],
            0.001,
        );
    }
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}

#[test]
fn sequential_radii_sizes_and_edge_orders() {
    for size in [10.0, 70.0] {
        for first_radius in [1.0, 2.0] {
            for swapped in [false, true] {
                let vertices = [(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)]
                    .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)));
                let wire = (0..4)
                    .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
                    .collect();
                let face: Face = builder::try_attach_plane(vec![wire]).unwrap();
                let cube: Solid = builder::tsweep(&face, Vector3::unit_z() * size);
                let edge = cube
                    .edge_iter()
                    .find(|edge| {
                        [edge.front().point(), edge.back().point()].iter().all(|p| {
                            p.y.abs() < 1e-6
                                && if swapped {
                                    (p.z - size).abs() < 1e-6
                                } else {
                                    p.x.abs() < 1e-6
                                }
                        })
                    })
                    .unwrap();
                let cube_before = serde_json::to_string(&cube.compress()).unwrap();
                let first = fillet_solid_edges(&cube, &[edge.id()], first_radius, 0.001).unwrap();
                assert_history(&cube, &first, 1);
                assert_eq!(
                    cube_before,
                    serde_json::to_string(&cube.compress()).unwrap()
                );
                let solid = first.solid;
                let before = serde_json::to_string(&solid.compress()).unwrap();
                let wire = second_wire_at(&solid, size, swapped);
                for ratio in [0.5, 1.0, 1.2] {
                    let radius = ratio * first_radius;
                    for wire in [wire.clone(), wire.inverse()] {
                        let result = fillet_solid_along_wire(&solid, &wire, radius, 0.001);
                        if ratio <= 1.0 {
                            let result =
                                result.expect("regular or collapsed-contact second fillet");
                            assert_history(&solid, &result, 3);
                            common::assert_solid(
                                &result.solid,
                                expected_volume(size, first_radius, radius),
                                &[0],
                                0.001,
                            );
                            if ratio == 1.0 {
                                common::blend::assert_step(
                                    &result.solid,
                                    expected_volume(size, first_radius, radius),
                                    0.001,
                                );
                                assert_spherical_corner(&result, radius);
                            }
                        } else {
                            assert!(result.is_none(), "size={size}, first={first_radius}, second={radius}, swapped={swapped}");
                        }
                        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
                    }
                }
            }
        }
    }
}

fn assert_spherical_corner(result: &BlendResult, radius: f64) {
    let spheres: Vec<_> = result
        .generated_faces
        .iter()
        .filter_map(|face| match face.oriented_surface().elementary() {
            Some((
                Elementary::Sphere {
                    center,
                    radius: actual,
                },
                true,
            )) => Some((face, center, actual)),
            _ => None,
        })
        .collect();
    assert_eq!(spheres.len(), 1);
    for (face, center, actual) in spheres {
        assert!((actual - radius).abs() < 1e-8);
        for edge in face.edge_iter() {
            let curve = edge.oriented_curve();
            let (a, b) = curve.range_tuple();
            for i in 0..=16 {
                assert!(
                    ((curve.subs(a + (b - a) * i as f64 / 16.) - center).magnitude() - radius)
                        .abs()
                        < 1e-7
                );
            }
        }
    }
}

#[test]
fn collapsed_corner_survives_similarity_transforms() {
    let cube = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(10., 10., 10.),
    ]));
    let edge = cube
        .edge_iter()
        .find(|e| {
            [e.front().point(), e.back().point()]
                .iter()
                .all(|p| p.x.abs() < 1e-8 && p.y.abs() < 1e-8)
        })
        .unwrap();
    let solid = fillet_solid_edges(&cube, &[edge.id()], 1., 0.001)
        .unwrap()
        .solid;
    let wire = second_wire_at(&solid, 10., false);
    for scale in [0.2, 7.] {
        let matrix = Matrix4::from_translation(Vector3::new(7., -11., 17.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let input = builder::transformed(&solid, matrix);
        let transformed: Wire = wire
            .iter()
            .map(|edge| {
                let a = matrix.transform_point(edge.front().point());
                let b = matrix.transform_point(edge.back().point());
                input
                    .edge_iter()
                    .find_map(|e| {
                        if (e.front().point() - a).magnitude() < 1e-7
                            && (e.back().point() - b).magnitude() < 1e-7
                        {
                            Some(e.clone())
                        } else if (e.front().point() - b).magnitude() < 1e-7
                            && (e.back().point() - a).magnitude() < 1e-7
                        {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for selection in [transformed.clone(), transformed.inverse()] {
            let result = fillet_solid_along_wire(&input, &selection, scale, 0.001).unwrap();
            assert_history(&input, &result, 3);
            assert_spherical_corner(&result, scale);
            let expected = expected_volume(10., 1., 1.) * scale.powi(3);
            common::assert_solid(&result.solid, expected, &[0], 0.001);
            common::blend::assert_step(&result.solid, expected, 0.001);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

fn rounded_rectangle(height: f64) -> Solid {
    let cube = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(10., 8., height),
    ]));
    let mut seen = HashSet::new();
    let edges: Vec<_> = cube
        .edge_iter()
        .filter(|e| {
            let v = e.back().point() - e.front().point();
            v.x.abs() < 1e-8 && v.y.abs() < 1e-8 && seen.insert(e.id())
        })
        .map(|e| e.id())
        .collect();
    fillet_solid_edges(&cube, &edges, 1., 0.001).unwrap().solid
}
fn top_boundary(solid: &Solid, height: f64) -> Wire {
    solid
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| (v.point().z - height).abs() < 1e-8))
        .unwrap()
        .boundaries()[0]
        .clone()
}
#[test]
fn closed_collapsed_corners_merge_contacts_across_the_wire_seam() {
    let solid = rounded_rectangle(6.);
    let wire = top_boundary(&solid, 6.);
    assert_eq!(wire.len(), 8);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let pi = std::f64::consts::PI;
    let area = 1. - pi / 4.;
    let centroid = (10. - 3. * pi) / (12. - 3. * pi);
    let expected = (80. - 4. * area) * 6. - area * (28. + 2. * pi * (1. - centroid));
    for offset in 0..wire.len() {
        let selected: Wire = (0..wire.len())
            .map(|i| wire[(i + offset) % wire.len()].clone())
            .collect();
        for selected in [selected.clone(), selected.inverse()] {
            let result = fillet_solid_along_wire(&solid, &selected, 1., 0.001).unwrap();
            assert_history(&solid, &result, 8);
            assert_eq!(
                result
                    .generated_faces
                    .iter()
                    .filter(|f| matches!(
                        f.oriented_surface().elementary(),
                        Some((Elementary::Sphere { .. }, true))
                    ))
                    .count(),
                4
            );
            common::assert_solid(&result.solid, expected, &[0], 0.001);
            common::blend::assert_step(&result.solid, expected, 0.001);
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
#[test]
fn collapsed_contacts_reject_cap_consumption_and_short_walls() {
    let cylinder = common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 1., 5.);
    for (solid, height) in [(cylinder, 5.), (rounded_rectangle(0.5), 0.5)] {
        let wire = top_boundary(&solid, height);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        for selected in [wire.clone(), wire.inverse()] {
            assert!(fillet_solid_along_wire(&solid, &selected, 1., 0.001).is_none());
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
