mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input() -> Solid {
    let table = Table::from_step(include_str!(
        "fixtures/separate-contours/cylinder-cone-stack.step"
    ))
    .unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(1e-5);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn contour(solid: &Solid, inverse: Matrix4, radius: f64, height: f64) -> Wire {
    let mut seen = HashSet::new();
    let mut edges: Vec<_> = solid
        .edge_iter()
        .filter(|edge| seen.insert(edge.id()))
        .filter(|edge| {
            let c = edge.curve();
            let (a, b) = c.range_tuple();
            (0..=4).all(|i| {
                let p = inverse.transform_point(c.subs(a + (b - a) * i as f64 / 4.));
                (p.z - height).abs() < 1e-8 && (p.x.hypot(p.y) - radius).abs() < 1e-8
            })
        })
        .collect();
    let mut wire = Wire::from(vec![edges.remove(0)]);
    while !edges.is_empty() {
        let v = wire.back_vertex().unwrap().clone();
        let i = edges
            .iter()
            .position(|e| e.front() == &v || e.back() == &v)
            .unwrap();
        let e = edges.remove(i);
        wire.push_back(if e.front() == &v { e } else { e.inverse() });
    }
    assert!(wire.is_cyclic());
    wire
}

fn added_volume(radius: f64, support_radius: f64, slope: f64) -> f64 {
    let norm = 1.0_f64.hypot(slope);
    let center = support_radius + radius * (norm - slope);
    let contact = center - radius / norm;
    let height = radius - radius * slope / norm;
    let a = (-slope).atan2(-1.);
    let b = -std::f64::consts::FRAC_PI_2;
    // Revolve the signed profile integral rho^2 dz around the stack axis.
    let primitive = |t: f64| {
        radius
            * (center * center * t.sin()
                + center * radius * (t + (2. * t).sin() / 2.)
                + radius * radius * (t.sin() - t.sin().powi(3) / 3.))
    };
    -std::f64::consts::PI
        * (height
            * (support_radius * support_radius + support_radius * contact + contact * contact)
            / 3.
            + primitive(b)
            - primitive(a))
}

#[test]
fn separate_curved_contours_preserve_material_normals_history_and_step() {
    let volume =
        std::f64::consts::PI * 56.25 + added_volume(0.3, 2., 1. / 6.) + added_volume(0.3, 1., 0.);
    assert!((volume - 177.01105983494742).abs() < 1e-12);
    for (scale, moved) in [(1., false), (1., true), (0.1, true)] {
        let matrix = if moved {
            Matrix4::from_translation(Vector3::new(2., -3., 5.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let solid = builder::transformed(&input(), matrix);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let wires = [
            contour(&solid, inverse, 2., 5.),
            contour(&solid, inverse, 1., 8.),
        ];
        let selection: Vec<_> = wires.iter().flatten().map(|e| e.id()).collect();
        let mut reference = None;
        for order in [
            selection.clone(),
            selection.iter().rev().copied().collect(),
            vec![selection[2], selection[0], selection[3], selection[1]],
        ] {
            let result = try_fillet_solid_edges(&solid, &order, 0.3 * scale, 1e-6).unwrap();
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), 4);
            assert_eq!(result.modified_faces.len(), 6);
            let original: Vec<_> = solid.face_iter().collect();
            let output: Vec<_> = result.solid.face_iter().collect();
            let expected: Vec<_> = original
                .iter()
                .zip(&output)
                .filter(|(a, b)| a.id() != b.id())
                .map(|(a, b)| (a.id(), b.id()))
                .collect();
            assert_eq!(
                result
                    .modified_faces
                    .iter()
                    .map(|(a, b)| (*a, b.id()))
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                result
                    .generated_faces
                    .iter()
                    .map(|f| f.id())
                    .collect::<Vec<_>>(),
                output
                    .iter()
                    .skip(original.len())
                    .map(|f| f.id())
                    .collect::<Vec<_>>()
            );
            let compressed = serde_json::to_string(&result.solid.compress()).unwrap();
            if let Some(expected) = &reference {
                assert_eq!(&compressed, expected);
            } else {
                reference = Some(compressed);
            }
            for face in &result.generated_faces {
                let surface = face.oriented_surface();
                let (u, v) = surface.try_range_tuple();
                let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                for i in 0..=8 {
                    for j in 0..=8 {
                        let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 8.);
                        let p = inverse.transform_point(surface.subs(u, v));
                        let (support, height, slope) = if p.z < 7. {
                            (2., 5., 1. / 6.)
                        } else {
                            (1., 8., 0.)
                        };
                        let center = support + 0.3 * (1.0_f64.hypot(slope) - slope);
                        let rho = p.x.hypot(p.y);
                        let radial = Vector3::new(
                            p.x * (1. - center / rho),
                            p.y * (1. - center / rho),
                            p.z - height - 0.3,
                        );
                        assert!((radial.magnitude() - 0.3).abs() * scale < 1e-6, "{p:?}");
                        let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                        assert!(
                            normal.dot(radial.normalize()) < -1. + 1e-5,
                            "{p:?} {normal:?}"
                        );
                    }
                }
            }
            common::assert_solid(&result.solid, volume * scale.powi(3), &[0], 0.001);
            common::blend::assert_step(&result.solid, volume * scale.powi(3), 0.001);
        }
    }
}

#[test]
fn separate_contours_fail_atomically_when_a_later_round_does_not_fit() {
    let solid = input();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let first = contour(&solid, Matrix4::identity(), 2., 5.);
    let second = contour(&solid, Matrix4::identity(), 1., 8.);
    assert!(try_fillet_solid_along_wire(&solid, &first, 0.8, 1e-6).is_ok());
    assert!(try_fillet_solid_along_wire(&solid, &second, 0.8, 1e-6).is_err());
    let edges: Vec<_> = first.iter().chain(&second).map(|e| e.id()).collect();
    for order in [edges.clone(), edges.into_iter().rev().collect()] {
        assert!(try_fillet_solid_edges(&solid, &order, 0.8, 1e-6).is_err());
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}

#[test]
fn contours_sharing_support_faces_remain_unsupported() {
    let solid = input();
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let a = contour(&solid, Matrix4::identity(), 2., 5.);
    let b = contour(&solid, Matrix4::identity(), 1.5, 8.);
    let edges: Vec<_> = a.iter().chain(&b).map(|e| e.id()).collect();
    assert!(try_fillet_solid_edges(&solid, &edges, 0.3, 1e-6).is_err());
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}
