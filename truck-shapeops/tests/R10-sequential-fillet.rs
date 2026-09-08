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
                        if ratio < 1.0 {
                            let result = result.expect("regular smaller-radius second fillet");
                            assert_history(&solid, &result, 3);
                            common::assert_solid(
                                &result.solid,
                                expected_volume(size, first_radius, radius),
                                &[0],
                                0.001,
                            );
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
