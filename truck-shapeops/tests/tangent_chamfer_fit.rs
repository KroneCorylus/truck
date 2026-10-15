mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::{simple_chamfer, try_chamfer_solid_along_wire};

fn rounded_prism() -> Solid {
    let vertices = builder::vertices(
        [[0., 0.], [6., 0.], [10., 4.], [10., 20.], [0., 20.]].map(|[x, y]| Point3::new(x, y, 0.)),
    );
    let middle = Point3::new(6. + 4. / 2_f64.sqrt(), 4. - 4. / 2_f64.sqrt(), 0.);
    let wire: Wire = vec![
        builder::line(&vertices[0], &vertices[1]),
        builder::circle_arc(&vertices[1], &vertices[2], middle),
        builder::line(&vertices[2], &vertices[3]),
        builder::line(&vertices[3], &vertices[4]),
        builder::line(&vertices[4], &vertices[0]),
    ]
    .into();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::new(0., 0., 30.),
    )
}

fn top_chain(input: &Solid, inverse: Matrix4) -> Wire {
    let top = input
        .face_iter()
        .find(|face| {
            inverse
                .transform_vector(face.oriented_surface().normal(0., 0.))
                .normalize()
                .z
                > 0.9
        })
        .unwrap();
    let mut edges: Vec<_> = top.boundaries()[0]
        .iter()
        .filter(|edge| {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            let point = inverse.transform_point(curve.subs((a + b) / 2.));
            point.x.abs() > 1e-8 && (point.y - 20.).abs() > 1e-8
        })
        .cloned()
        .collect();
    while edges.last().unwrap().back() == edges.first().unwrap().front() {
        edges.rotate_left(1);
    }
    let wire: Wire = edges.into();
    assert_eq!(wire.len(), 3);
    assert!(wire.is_continuous());
    wire
}

fn material() -> f64 {
    let original = (200. - 16. + 4. * PI) * 30.;
    // Integrate the inward offset strip along the two straight sides and quarter circle.
    let removed = (22. + 2. * PI) / 2. - PI / 12.;
    original - removed
}

fn check_chamfer(scale: f64, moved: bool, reversed: bool, shifted: bool) {
    let original = rounded_prism();
    if shifted {
        let mut seen = std::collections::HashSet::new();
        for edge in original.edge_iter() {
            if seen.insert(edge.id()) {
                let mut curve = NurbsCurve::<Vector4>::try_from(&edge.curve()).unwrap();
                curve.parameter_transform(1., 2.);
                edge.set_curve(curve.into());
            }
        }
    }
    let transform = if moved {
        Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let inverse = transform.invert().unwrap();
    let input = builder::transformed(&original, transform);
    let mut wire = top_chain(&input, inverse);
    if reversed {
        wire.invert();
    }
    assert!(input.is_geometric_consistent());
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_along_wire(&input, &wire, scale, scale, 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert!(result.solid.is_geometric_consistent());
    assert_eq!(result.generated_faces.len(), 3);
    common::assert_solid(
        &result.solid,
        material() * scale.powi(3),
        &[0],
        1e-4 * scale,
    );
    common::blend::assert_step(&result.solid, material() * scale.powi(3), 1e-4 * scale);
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (urange, vrange) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (urange.unwrap(), vrange.unwrap());
        for i in 1..16 {
            for v in [0.25, 0.5, 0.75] {
                let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * v);
                let p = inverse.transform_point(surface.subs(u, v));
                let n = inverse.transform_vector(surface.normal(u, v)).normalize();
                let (error, expected) = if p.x < 6. - 1e-6 {
                    (p.y - p.z + 29., Vector3::new(0., -1., 1.).normalize())
                } else if p.y > 4. + 1e-6 {
                    (p.x + p.z - 39., Vector3::new(1., 0., 1.).normalize())
                } else {
                    let r = Vector2::new(p.x - 6., p.y - 4.);
                    (
                        r.magnitude() + p.z - 33.,
                        Vector3::new(r.x / r.magnitude(), r.y / r.magnitude(), 1.).normalize(),
                    )
                };
                assert!(
                    error.abs() * scale < 5e-7,
                    "contact error {}",
                    error * scale
                );
                assert!(
                    n.distance(expected) < 1e-4,
                    "normal {n:?}, expected {expected:?}"
                );
            }
        }
    }
}

#[test]
fn tangent_chamfers_preserve_material_contacts_and_step_across_scales() {
    for (scale, moved, reversed) in [
        (0.025, false, false),
        (0.025, true, true),
        (1., false, true),
    ] {
        check_chamfer(scale, moved, reversed, false);
    }
}

#[test]
fn shifted_curve_domains_preserve_bevel_orientation_and_material() {
    for reversed in [false, true] {
        check_chamfer(0.025, true, reversed, true);
    }
}

#[test]
fn simple_chamfer_keeps_contact_curves_on_its_ruled_surface() {
    let vertices = builder::vertices([Point3::new(0., -0.1, 0.), Point3::new(0., 0.1, 0.)]);
    let arc = builder::circle_arc(&vertices[0], &vertices[1], Point3::new(0.1, 0., 0.));
    let middle = builder::vertex(Point3::origin());
    let boundary = wire![
        arc,
        builder::line(&vertices[1], &middle),
        builder::line(&middle, &vertices[0])
    ];
    let input: Solid = builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::new(0., 0., 0.75),
    );
    let edge = common::blend::edge_through(&input.boundaries()[0], Point3::new(0.1, 0., 0.75));
    let faces: Vec<_> = input
        .face_iter()
        .filter(|face| face.edge_iter().any(|e| e.id() == edge.id()))
        .collect();
    assert_eq!(faces.len(), 2);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = simple_chamfer(faces[0], faces[1], edge.id(), 0.025, 0.025, 0.001).unwrap();
    assert!(result.fillet.is_geometric_consistent());
    assert!(result.face0.is_geometric_consistent());
    assert!(result.face1.is_geometric_consistent());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
