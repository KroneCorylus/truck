mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
const TOL: f64 = 0.001;
fn bracket() -> Solid {
    let vertices = builder::vertices(
        [
            (0., 0.),
            (20., 0.),
            (20., 10.),
            (10., 10.),
            (10., 20.),
            (0., 20.),
        ]
        .map(|(x, y)| Point3::new(x, y, 0.)),
    );
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}
fn expected(radius: f64, ends: usize) -> f64 {
    let k = 1. - PI / 4.;
    3000.
        + 10. * k * radius * radius
        + ends as f64
            * (-20. * k * radius * radius
                + (2. * k * k - PI / 4. * (5. / 3. - PI / 2.)) * radius.powi(3))
}
#[test]
fn mixed_three_edge_fillets_have_exact_tangent_torus_corners() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for ends in [1, 2] {
            for radius in [0.2, 1., 3.] {
                let mut points = vec![Point3::new(10., 10., 5.)];
                for z in [10., 0.].into_iter().take(ends) {
                    points.extend([Point3::new(15., 10., z), Point3::new(10., 15., z)]);
                }
                let mut ids: Vec<_> = points
                    .into_iter()
                    .map(|p| {
                        common::blend::edge_through(
                            &input.boundaries()[0],
                            transform.transform_point(p),
                        )
                        .id()
                    })
                    .collect();
                let result = try_fillet_solid_edges(&input, &ids, radius, TOL).unwrap();
                assert_eq!(result.generated_faces.len(), 1 + 3 * ends);
                assert_eq!(
                    result
                        .generated_faces
                        .iter()
                        .filter(|f| matches!(
                            f.surface().elementary(),
                            Some((truck_modeling::geometry::Elementary::Torus { .. }, _))
                        ))
                        .count(),
                    ends
                );
                for torus in result.generated_faces.iter().filter(|f| {
                    matches!(
                        f.surface().elementary(),
                        Some((truck_modeling::geometry::Elementary::Torus { .. }, _))
                    )
                }) {
                    for edge in torus.edge_iter() {
                        let neighbor = result
                            .solid
                            .face_iter()
                            .find(|f| {
                                f.id() != torus.id() && f.edge_iter().any(|e| e.id() == edge.id())
                            })
                            .unwrap();
                        let curve = edge.oriented_curve();
                        let (a, b) = curve.range_tuple();
                        for fraction in [0.2, 0.5, 0.8] {
                            let p = curve.subs(a + fraction * (b - a));
                            let normal = |face: &Face| {
                                let surface = face.oriented_surface();
                                let (u, v) = surface.search_parameter(p, None, 100).unwrap();
                                surface.normal(u, v).normalize()
                            };
                            assert!(
                                normal(torus).dot(normal(neighbor)) > 1. - 1e-8,
                                "corner must meet each neighboring face tangentially"
                            );
                        }
                    }
                }
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, expected(radius, ends), &[0], TOL);
                common::blend::assert_step(&result.solid, expected(radius, ends), TOL);
                ids.reverse();
                let reversed = try_fillet_solid_edges(&input, &ids, radius, TOL).unwrap();
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
fn local_fillets_preserve_unrelated_curves_and_support_two_edge_miters() {
    let mut cutter =
        common::modeling::cylinder(Point3::new(5., 5., -1.), Vector3::unit_z(), 1., 12.);
    cutter.not();
    let input = truck_shapeops::and(&bracket(), &cutter, TOL).unwrap();
    let curved: Vec<_> = input
        .face_iter()
        .filter(|f| !matches!(f.surface(), Surface::Plane(_)))
        .map(|f| f.id())
        .collect();
    let curves: Vec<_> = input
        .edge_iter()
        .filter(|e| !matches!(e.curve(), Curve::Line(_)))
        .map(|e| e.id())
        .collect();
    let points = [
        Point3::new(10., 10., 5.),
        Point3::new(15., 10., 10.),
        Point3::new(10., 15., 10.),
    ];
    let all = points.map(|p| common::blend::edge_through(&input.boundaries()[0], p).id());
    let r = 0.5_f64;
    let k = 1. - PI / 4.;
    for (ids, volume) in [
        (vec![all[0]], 3000. + 10. * k * r * r),
        (vec![all[1]], 3000. - 10. * k * r * r),
        (
            vec![all[1], all[2]],
            3000. - 20. * k * r * r - (5. / 3. - PI / 2.) * r.powi(3),
        ),
        (all.to_vec(), expected(r, 1)),
    ] {
        let result = try_fillet_solid_edges(&input, &ids, r, TOL).unwrap();
        let volume = volume - PI * 10.;
        common::assert_solid(&result.solid, volume, &[1], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
        assert!(result.solid.is_geometric_consistent());
        assert!(curved
            .iter()
            .all(|&id| result.solid.face_iter().any(|f| f.id() == id)));
        assert!(curves
            .iter()
            .all(|&id| result.solid.edge_iter().any(|e| e.id() == id)));
    }
}

#[test]
fn local_fillets_reject_collisions_and_collapsed_contacts_without_mutation() {
    for hole in [false, true] {
        let input = if hole {
            let mut cutter =
                common::modeling::cylinder(Point3::new(8., 8., -1.), Vector3::unit_z(), 0.5, 12.);
            cutter.not();
            truck_shapeops::and(&bracket(), &cutter, TOL).unwrap()
        } else {
            bracket()
        };
        let before = serde_json::to_string(&input.compress()).unwrap();
        let ids = [
            Point3::new(10., 10., 5.),
            Point3::new(15., 10., 10.),
            Point3::new(10., 15., 10.),
        ]
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id());
        let radius = if hole { 4. } else { 10. };
        assert_eq!(
            try_fillet_solid_edges(&input, &ids, radius, TOL)
                .unwrap_err()
                .code,
            truck_base::diagnostics::Code::OutsideNeighbour
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn local_spherical_corners_work_on_nonconvex_bodies_and_cavity_shells() {
    for cavity in [false, true] {
        let input = if cavity {
            let outer =
                common::modeling::cuboid(Point3::new(-5., -5., -5.), Point3::new(15., 15., 15.));
            let mut inner = common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.));
            inner.not();
            Solid::new(vec![
                outer.boundaries()[0].clone(),
                inner.boundaries()[0].clone(),
            ])
        } else {
            bracket()
        };
        let shell = &input.boundaries()[usize::from(cavity)];
        let points = if cavity {
            [
                Point3::new(5., 0., 0.),
                Point3::new(0., 5., 0.),
                Point3::new(0., 0., 5.),
            ]
        } else {
            [
                Point3::new(15., 0., 10.),
                Point3::new(20., 5., 10.),
                Point3::new(20., 0., 5.),
            ]
        };
        let ids = points.map(|p| common::blend::edge_through(shell, p).id());
        for count in [1, 2, 3] {
            let radius = 0.5_f64;
            let overlap = match count {
                1 => 0.,
                2 => 5. / 3. - PI / 2.,
                _ => 2. - 7. * PI / 12.,
            };
            let length = 10. * count as f64 + if cavity { 0. } else { 10. };
            let loss = length * (1. - PI / 4.) * radius * radius - overlap * radius.powi(3);
            let expected = if cavity { 7000. + loss } else { 3000. - loss };
            let result = try_fillet_solid_edges(&input, &ids[..count], radius, TOL).unwrap();
            common::assert_solid(
                &result.solid,
                expected,
                if cavity { &[0, 0] } else { &[0] },
                TOL,
            );
            common::blend::assert_step(&result.solid, expected, TOL);
            assert!(result.solid.is_geometric_consistent());
        }
    }
}

#[test]
fn local_fillet_extreme_radii_return_diagnostics() {
    let input = bracket();
    let ids = [
        Point3::new(10., 10., 5.),
        Point3::new(15., 10., 10.),
        Point3::new(10., 15., 10.),
    ]
    .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id());
    for radius in [1e-12, 1e20, 1e200] {
        assert_eq!(
            try_fillet_solid_edges(&input, &ids, radius, TOL)
                .unwrap_err()
                .code,
            truck_base::diagnostics::Code::OutsideNeighbour
        );
    }
}
