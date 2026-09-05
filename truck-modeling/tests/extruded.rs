use std::f64::consts::PI;
use truck_modeling::*;

/// Flux of the field `(x, y, 0) / 2` through a face, integrated on the exact surface over its
/// full parameter rectangle by composite Simpson.
///
/// The field has unit divergence, so the fluxes through the boundary of a solid sum to its
/// volume. Faces with normal `±z` contribute nothing, so the planar caps of a cylinder along `z`
/// need no integration at all and the side faces can be integrated without trimming.
fn side_flux(face: &Face, n: usize) -> f64 {
    let surface = face.oriented_surface();
    let (Some((u0, u1)), Some((v0, v1))) = surface.try_range_tuple() else {
        panic!("side face has no parameter rectangle");
    };
    let weight = |i: usize| match (i == 0 || i == n, i % 2) {
        (true, _) => 1.0,
        (false, 1) => 4.0,
        (false, _) => 2.0,
    };
    let mut sum = 0.0;
    for i in 0..=n {
        let u = u0 + (u1 - u0) * i as f64 / n as f64;
        for j in 0..=n {
            let v = v0 + (v1 - v0) * j as f64 / n as f64;
            let p = surface.subs(u, v);
            let da = surface.uder(u, v).cross(surface.vder(u, v));
            sum += weight(i) * weight(j) * (p.x * da.x + p.y * da.y);
        }
    }
    sum * (u1 - u0) * (v1 - v0) / (9.0 * n as f64 * n as f64) / 2.0
}

#[test]
fn tsweep_of_disk_keeps_extruded_conic() {
    let center = Point3::new(1.0, -2.0, 0.5);
    let (radius, height) = (1.5, 2.0);
    let vertex = builder::vertex(center + Vector3::unit_x() * radius);
    let circle: Wire = builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(7.0), 3);
    let disk: Face = builder::try_attach_plane(&[circle]).unwrap();
    let cylinder: Solid = builder::tsweep(&disk, Vector3::unit_z() * height);
    assert!(cylinder.is_geometric_consistent());

    let mut planes = 0;
    let mut volume = 0.0;
    for face in cylinder.boundaries()[0].iter() {
        match face.surface() {
            Surface::Plane(_) => planes += 1,
            Surface::Extruded(surface) => {
                assert!(
                    matches!(surface.entity_curve(), Curve::Conic(_)),
                    "extruded curve is not the arc: {:?}",
                    surface.entity_curve()
                );
                assert!(surface.extruding_vector().near(&(Vector3::unit_z() * height)));
                volume += side_flux(face, 256);
            }
            surface => panic!("unexpected surface: {surface:?}"),
        }
    }
    assert_eq!(planes, 2);
    let expected = PI * radius * radius * height;
    assert!(
        (volume - expected).abs() < 1e-9,
        "volume {volume}, expected {expected}"
    );
}
