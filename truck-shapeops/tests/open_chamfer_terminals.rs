mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_along_wire;
fn lens_area(radius: f64) -> f64 {
    let (other, distance) = (2_f64, 3_f64);
    radius
        * radius
        * ((distance * distance + radius * radius - other * other) / (2. * distance * radius))
            .acos()
        + other
            * other
            * ((distance * distance + other * other - radius * radius) / (2. * distance * other))
                .acos()
        - 0.5
            * ((-distance + radius + other)
                * (distance + radius - other)
                * (distance - radius + other)
                * (distance + radius + other))
                .sqrt()
}
fn lens() -> (Solid, Edge) {
    let x = 7. / 3.;
    let y = (9_f64 - x * x).sqrt();
    let vertices = builder::vertices([Point3::new(x, -y, 0.), Point3::new(x, y, 0.)]);
    let outer = builder::circle_arc(&vertices[0], &vertices[1], Point3::new(3., 0., 0.));
    let terminal = builder::circle_arc(&vertices[1], &vertices[0], Point3::new(1., 0., 0.));
    let face = builder::try_attach_plane(&[vec![outer.clone(), terminal].into()]).unwrap();
    (builder::tsweep(&face, Vector3::unit_z() * 2.), outer)
}
fn volume(radial: f64, axial: f64, n: usize) -> f64 {
    let f = |i| lens_area(3. - radial + radial * i as f64 / n as f64);
    (2. - axial) * lens_area(3.)
        + axial / n as f64 / 3.
            * (f(0)
                + f(n)
                + (1..n)
                    .map(|i| if i % 2 == 0 { 2. * f(i) } else { 4. * f(i) })
                    .sum::<f64>())
}
#[test]
fn open_chamfer_contacts_continue_to_curved_terminals() {
    for moved in [false, true] {
        let scale = if moved { 2.5 } else { 1. };
        let matrix = if moved {
            Matrix4::from_translation(Vector3::new(7., -4., 11.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.61))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let (base, _) = lens();
        let input = builder::transformed(&base, matrix);
        let before = serde_json::to_string(&input.compress()).unwrap();
        common::assert_solid(
            &input,
            2. * lens_area(3.) * scale.powi(3),
            &[0],
            0.001 * scale,
        );
        for top in [false, true] {
            let z = if top { 2. } else { 0. };
            let edge = common::blend::edge_through(
                &input.boundaries()[0],
                matrix.transform_point(Point3::new(3., 0., z)),
            );
            for (radial, axial) in [(0.1, 0.1), (0.2, 0.1), (0.1, 0.2)] {
                let expected = volume(radial, axial, 4096);
                assert!((expected - volume(radial, axial, 1024)).abs() < 1e-10);
                let mut wire: Wire = vec![edge.clone()].into();
                for _ in 0..2 {
                    let cap = input
                        .face_iter()
                        .find(|f| {
                            matches!(
                                f.oriented_surface().elementary(),
                                Some((Elementary::Plane(_), _))
                            ) && f.edge_iter().any(|e| e.id() == wire[0].id())
                        })
                        .unwrap();
                    let same = cap
                        .edge_iter()
                        .find(|e| e.id() == wire[0].id())
                        .unwrap()
                        .orientation()
                        == wire[0].orientation();
                    let (d0, d1) = if same {
                        (radial, axial)
                    } else {
                        (axial, radial)
                    };
                    let result = try_chamfer_solid_along_wire(
                        &input,
                        &wire,
                        d0 * scale,
                        d1 * scale,
                        0.001 * scale,
                    )
                    .unwrap();
                    assert!(result.solid.is_geometric_consistent());
                    common::assert_solid(
                        &result.solid,
                        expected * scale.powi(3),
                        &[0],
                        0.001 * scale,
                    );
                    common::blend::assert_step(
                        &result.solid,
                        expected * scale.powi(3),
                        0.001 * scale,
                    );
                    assert_eq!(result.generated_faces.len(), 1);
                    for vertex in result.generated_faces[0].vertex_iter() {
                        let p = inverse.transform_point(vertex.point());
                        let rho = p.x.hypot(p.y);
                        assert!(((p.x - 3.).powi(2) + p.y * p.y - 4.).abs() < 1e-6);
                        assert!(
                            ((rho - 3.).abs() < 1e-6
                                && (p.z - (if top { 2. - axial } else { axial })).abs() < 1e-6)
                                || ((rho - (3. - radial)).abs() < 1e-6 && (p.z - z).abs() < 1e-6)
                        );
                    }
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                    wire.invert();
                }
            }
        }
    }
}

#[test]
fn open_chamfers_reject_consumed_supports_without_mutation() {
    let (input, edge) = lens();
    let wire: Wire = vec![edge].into();
    let cap = input
        .face_iter()
        .find(|f| {
            matches!(
                f.oriented_surface().elementary(),
                Some((Elementary::Plane(_), _))
            ) && f.edge_iter().any(|e| e.id() == wire[0].id())
        })
        .unwrap();
    let same = cap
        .edge_iter()
        .find(|e| e.id() == wire[0].id())
        .unwrap()
        .orientation()
        == wire[0].orientation();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for (radial, axial) in [(3.1, 0.1), (0.1, 2.1)] {
        let (d0, d1) = if same {
            (radial, axial)
        } else {
            (axial, radial)
        };
        assert!(try_chamfer_solid_along_wire(&input, &wire, d0, d1, 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
