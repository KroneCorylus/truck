mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{simple_chamfer, simple_fillet};

fn half_cylinder() -> Solid {
    let vertices = builder::vertices([Point3::new(0., -0.1, 0.), Point3::new(0., 0.1, 0.)]);
    let arc = builder::circle_arc(&vertices[0], &vertices[1], Point3::new(0.1, 0., 0.));
    let boundary = wire![arc, builder::line(&vertices[1], &vertices[0])];
    builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::new(0., 0., 0.75),
    )
}

fn check_trim(chamfer: bool, encoded_inverse: bool, swapped: bool, nurbs: bool) {
    let input = half_cylinder();
    if nurbs {
        let mut seen = std::collections::HashSet::new();
        for edge in input.edge_iter() {
            if seen.insert(edge.id()) {
                let mut curve = NurbsCurve::<Vector4>::try_from(&edge.curve()).unwrap();
                curve.parameter_transform(2., 3.);
                edge.set_curve(curve.into());
            }
        }
    }
    let transform = Matrix4::from_translation(Vector3::new(2., -3., 5.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    let inverse = transform.invert().unwrap();
    let input = builder::transformed(&input, transform);
    assert!(input.is_geometric_consistent());
    let edge = common::blend::edge_through(
        &input.boundaries()[0],
        transform.transform_point(Point3::new(0.1, 0., 0.75)),
    );
    let mut faces: Vec<_> = input
        .face_iter()
        .filter(|face| face.edge_iter().any(|e| e.id() == edge.id()))
        .map(|face| {
            if encoded_inverse {
                Face::new(
                    face.boundaries().iter().map(Wire::inverse).collect(),
                    face.oriented_surface().inverse(),
                )
                .inverse()
            } else {
                Face::new(face.boundaries(), face.oriented_surface())
            }
        })
        .collect();
    assert_eq!(faces.len(), 2);
    if swapped {
        faces.swap(0, 1);
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = if chamfer {
        simple_chamfer(&faces[0], &faces[1], edge.id(), 0.025, 0.025, 0.001)
    } else {
        simple_fillet(&faces[0], &faces[1], edge.id(), 0.025, 1e-7)
    }
    .unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    for face in [&result.face0, &result.face1, &result.fillet] {
        assert!(face.is_geometric_consistent());
        assert!(face.boundaries().iter().all(|wire| wire.is_closed()));
    }
    for (source, trimmed) in faces.iter().zip([&result.face0, &result.face1]) {
        assert_eq!(source.orientation(), trimmed.orientation());
        assert_eq!(source.boundaries()[0].len(), trimmed.boundaries()[0].len());
        assert!(!trimmed.edge_iter().any(|e| e.id() == edge.id()));
        let contacts: Vec<_> = trimmed
            .edge_iter()
            .filter(|e| result.fillet.edge_iter().any(|f| f.id() == e.id()))
            .collect();
        assert_eq!(contacts.len(), 1);
        let shared = result
            .fillet
            .edge_iter()
            .find(|e| e.id() == contacts[0].id())
            .unwrap();
        assert_ne!(shared.orientation(), contacts[0].orientation());
        let top = source.boundaries()[0].len() == 2;
        let curve = contacts[0].curve();
        let (a, b) = curve.range_tuple();
        for i in 0..=32 {
            let p = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 32.));
            let radius = p.x.hypot(p.y);
            assert!(p.x >= -1e-6, "{p:?}");
            assert!(
                (radius - if top { 0.075 } else { 0.1 }).abs() < 1e-6,
                "{p:?}"
            );
            assert!((p.z - if top { 0.75 } else { 0.725 }).abs() < 1e-6, "{p:?}");
        }
        if top {
            let chord = trimmed
                .edge_iter()
                .find(|e| e.id() != contacts[0].id())
                .unwrap();
            let (a, b) = chord.ends();
            let mut ends = [a.point(), b.point()].map(|p| inverse.transform_point(p));
            ends.sort_by(|a, b| a.y.total_cmp(&b.y));
            assert!(ends[0].distance(Point3::new(0., -0.075, 0.75)) < 1e-6);
            assert!(ends[1].distance(Point3::new(0., 0.075, 0.75)) < 1e-6);
        }
    }
    let surface = result.fillet.oriented_surface();
    let (u, v) = surface.try_range_tuple();
    let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
    for i in 1..16 {
        for j in 1..8 {
            let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 8.);
            let p = inverse.transform_point(surface.subs(u, v));
            let radial = p.x.hypot(p.y);
            let error = if chamfer {
                radial + p.z - 0.825
            } else {
                (radial - 0.075).hypot(p.z - 0.725) - 0.025
            };
            assert!(error.abs() < 1e-6, "surface error {error} at {p:?}");
        }
    }
}

fn check_variants(chamfer: bool) {
    for encoded_inverse in [false, true] {
        for swapped in [false, true] {
            for nurbs in [false, true] {
                check_trim(chamfer, encoded_inverse, swapped, nurbs);
            }
        }
    }
}

#[test]
fn chamfer_trims_both_ends_of_one_neighbor() { check_variants(true); }

#[test]
fn fillet_trims_both_ends_of_one_neighbor() { check_variants(false); }
