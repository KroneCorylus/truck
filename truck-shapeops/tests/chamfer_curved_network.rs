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

fn selection(input: &Solid, scale: f64, unequal: bool) -> Vec<(EdgeID, [f64; 2])> {
    let shell = &input.boundaries()[0];
    let mut seen = std::collections::HashSet::new();
    input
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .map(|edge| {
            let mut d = [0.2 * scale; 2];
            let sides: Vec<_> = shell
                .iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .collect();
            if matches!(edge.curve(), Curve::Line(_)) {
                for (i, face) in sides.iter().enumerate() {
                    if matches!(
                        face.surface().elementary(),
                        Some((Elementary::Cylinder { .. }, _))
                    ) {
                        // Public two-distance setbacks use tangent projection; native equal distance is a chord.
                        d[i] = scale * (2. * (0.2_f64 / 2.).asin()).tan();
                    }
                }
                if unequal
                    && sides
                        .iter()
                        .all(|f| matches!(f.surface(), Surface::Plane(_)))
                {
                    d = [0.1 * scale; 2];
                }
            }
            (edge.id(), d)
        })
        .collect()
}

fn constraints() -> Vec<(Vector3, f64)> {
    let d = 0.2_f64;
    let cy = d * (1. - d * d / 4.).sqrt();
    let cz = 1. - d * d / 2.;
    let p = Point3::new(d, cy, cz);
    let q = Point3::new(d, 0., 1. - d);
    let r = Point3::new(0., d, ((1. - d).powi(2) - d * d).sqrt());
    let n = (r - p).cross(q - p);
    let offset = n.dot(p.to_vec());
    let mut planes = Vec::new();
    for far in [false, true] {
        for swap in [false, true] {
            let mut m = if swap { Vector3::new(n.x, n.z, n.y) } else { n };
            let mut c = offset;
            if far {
                c -= m.x;
                m.x = -m.x;
            }
            planes.push((m, c));
        }
    }
    let slope = (cz - (1. - d)) / cy;
    planes.extend([
        (Vector3::new(0., -slope, 1.), 1. - d),
        (Vector3::new(0., 1., -slope), 1. - d),
        (Vector3::new(-1., -1., 0.), -d),
        (Vector3::new(-1., 0., -1.), -d),
        (Vector3::new(1., -1., 0.), 1. - d),
        (Vector3::new(1., 0., -1.), 1. - d),
        (Vector3::new(0., -1., -1.), -d),
        (Vector3::new(-1., -1., -1.), -2. * d),
        (Vector3::new(1., -1., -1.), 1. - 2. * d),
    ]);
    planes
}

fn check(scale: f64, reverse: bool) {
    let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.37))
        * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&quarter(), transform);
    if reverse {
        input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
    }
    let mut selected = selection(&input, scale, false);
    if reverse {
        selected.reverse();
    }
    assert_eq!(selected.len(), 9);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, 0.001 * scale).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    // Independent integration of the circle/cone sections clipped by the half-spaces
    // below: exact circle/line section integrals, equal at quadrature tolerances 1e-8 and 1e-10.
    let volume = 0.6180855963196404 * scale.powi(3);
    common::assert_solid(&result.solid, volume, &[0], 0.0001 * scale);
    common::blend::assert_step(&result.solid, volume, 0.0001 * scale);
    let planes = constraints();
    for edge in result.solid.edge_iter() {
        let curve = edge.curve();
        let (a, b) = curve.range_tuple();
        for i in 0..=32 {
            let p = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 32.));
            assert!(p.x >= -1e-8 && p.x <= 1. + 1e-8 && p.y >= -1e-8 && p.z >= -1e-8);
            let radius = 0.8 + p.x.min(1. - p.x).min(0.2);
            assert!(p.y.hypot(p.z) <= radius + 1e-8);
            assert!(
                planes.iter().all(|&(n, c)| n.dot(p.to_vec()) <= c + 1e-8),
                "material excursion at {p:?}"
            );
        }
    }
}

#[test]
fn captured_a7_network() { check(1., false); }

#[test]
fn curved_network_scaled_and_reordered() {
    check(0.1, true);
    check(10., false);
}

#[test]
fn unequal_network_remains_immutable_rejection() {
    let input = quarter();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(
        try_chamfer_solid_edges_with_distances(&input, &selection(&input, 1., true), 0.001)
            .is_err()
    );
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn overlapping_end_contacts_remain_immutable_rejection() {
    for length in [0.3, 0.4] {
        let input =
            builder::transformed(&quarter(), Matrix4::from_nonuniform_scale(length, 1., 1.));
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(try_chamfer_solid_edges_with_distances(
            &input,
            &selection(&input, 1., false),
            0.001
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn partial_network_remains_immutable_rejection() {
    let input = quarter();
    let mut selected = selection(&input, 1., false);
    selected.pop();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn unequal_planar_corner_guard_isolates_a6_network() {
    for (scale, reverse) in [(1., false), (0.1, true), (10., true)] {
        let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.37))
            * Matrix4::from_scale(scale);
        let inverse = transform.invert().unwrap();
        let mut input = builder::transformed(&quarter(), transform);
        if reverse {
            input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
        }
        let mut selected = selection(&input, scale, false);
        if reverse {
            selected.reverse();
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(try_chamfer_solid_edges_with_distances(&input, &selected, 0.001 * scale).is_ok());
        // Change only the axis edge between the two radial planes. All curved-corner
        // axial setbacks remain equal; the planar three-edge corners become unequal.
        let axis_edge = input
            .edge_iter()
            .find(|edge| {
                [edge.front().point(), edge.back().point()]
                    .into_iter()
                    .all(|p| {
                        let q = inverse.transform_point(p);
                        q.y.abs() < 1e-9 && q.z.abs() < 1e-9
                    })
            })
            .unwrap()
            .id();
        let mut changed = 0;
        for (id, distances) in &mut selected {
            if *id == axis_edge {
                *distances = [0.1 * scale; 2];
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        assert!(try_chamfer_solid_edges_with_distances(&input, &selected, 0.001 * scale).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
