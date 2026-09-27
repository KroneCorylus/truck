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

/// An app sketch: a pentagon with a reflex vertex at `outline[4]`, extruded 20 high.
struct Pentagon {
    prism: Solid,
    outline: [Point3; 5],
    /// Interior angle at each outline vertex.
    angles: [f64; 5],
    area: f64,
    perimeter: f64,
}

impl Pentagon {
    fn new() -> Self {
        let outline = [
            (-50.11918803317464, 54.71104651125626),
            (-46.848871341936984, -6.265190155003754),
            (-1.7266402147777953, 18.012518685622943),
            (-14.189935449778307, 41.56541242084759),
            (-38.32230290663329, 37.19417041227136),
        ]
        .map(|(x, y)| Point3::new(x, y, 0.));
        let vertices = builder::vertices(outline);
        let wire: Wire = (0..5)
            .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 5]))
            .collect();
        let prism = builder::tsweep(
            &builder::try_attach_plane(&[wire]).unwrap(),
            Vector3::unit_z() * 20.,
        );
        let (mut area, mut perimeter) = (0., 0.);
        let angles = std::array::from_fn(|i| {
            let [a, b, c] = [4, 0, 1].map(|j| outline[(i + j) % 5]);
            area += (b.x * c.y - c.x * b.y) / 2.;
            perimeter += b.distance(c);
            let (u, v) = (a - b, c - b);
            let angle = (v.x * u.y - v.y * u.x).atan2(u.dot(v));
            if angle < 0. {
                angle + 2. * PI
            } else {
                angle
            }
        });
        assert!(angles[4] > PI, "the outline must be nonconvex");
        Self {
            prism,
            outline,
            angles,
            area,
            perimeter,
        }
    }
    /// A point on the rim edge from `outline[i]` to `outline[i + 1]` at height `z`.
    fn rim(&self, i: usize, z: f64) -> Point3 {
        let [a, b] = [self.outline[i], self.outline[(i + 1) % 5]];
        a + (b - a) / 2. + Vector3::unit_z() * z
    }
    /// A point on the vertical edge at `outline[i]`.
    fn vertical(&self, i: usize) -> Point3 {
        self.outline[i] + Vector3::unit_z() * 10.
    }
    fn length(&self, i: usize) -> f64 {
        self.outline[i].distance(self.outline[(i + 1) % 5])
    }
    /// Checks the fillet of the edges through `points` against `expected(radius)`.
    /// `corners` counts the generated tori and spheres.
    fn check(
        &self,
        points: &[Point3],
        radii: &[f64],
        corners: (usize, usize),
        expected: impl Fn(f64) -> f64,
    ) {
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
        ] {
            let input = builder::transformed(&self.prism, transform);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut ids: Vec<_> = points
                .iter()
                .map(|&p| {
                    common::blend::edge_through(
                        &input.boundaries()[0],
                        transform.transform_point(p),
                    )
                    .id()
                })
                .collect();
            for &radius in radii {
                let result = try_fillet_solid_edges(&input, &ids, radius, TOL).unwrap();
                let count = |torus: bool| {
                    result
                        .generated_faces
                        .iter()
                        .filter(|f| match f.surface().elementary() {
                            Some((truck_modeling::geometry::Elementary::Torus { .. }, _)) => torus,
                            Some((truck_modeling::geometry::Elementary::Sphere { .. }, _)) => {
                                !torus
                            }
                            _ => false,
                        })
                        .count()
                };
                assert_eq!((count(true), count(false)), corners);
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, expected(radius), &[0], TOL);
                common::blend::assert_step(&result.solid, expected(radius), TOL);
                ids.reverse();
                let reversed = try_fillet_solid_edges(&input, &ids, radius, TOL).unwrap();
                assert_eq!(
                    serde_json::to_string(&result.solid.compress()).unwrap(),
                    serde_json::to_string(&reversed.solid.compress()).unwrap()
                );
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

const K: f64 = 1. - PI / 4.;
/// Per unit `r³`, the removal lost where a right-angle rim round ends at inward distance
/// `t` a further `t · cot` along its edge, integrated over the profile.
const END: f64 = 5. / 6. - PI / 4.;
fn cot(x: f64) -> f64 {
    1. / x.tan()
}
/// Section area per `r²` of a round on a vertical edge of interior angle `angle`.
fn corner(angle: f64) -> f64 {
    cot(angle / 2.) - (PI - angle) / 2.
}
/// Correction per `r³` for a spherical corner with a vertical edge of interior angle `angle`.
fn sphere(angle: f64) -> f64 {
    corner(angle) / 3. + 2. * END * cot(angle / 2.)
}
/// Correction per `r³` where two convex rim rounds sweep a torus around a concave vertical
/// round turning through `turn`. Along the concave round of radius `r`, the rim round at
/// inward distance `t` runs `turn · (r + t)` instead of the `2r · tan(turn / 2)` of the
/// sharp outline.
fn torus(turn: f64) -> f64 {
    K * (2. * (turn / 2.).tan() - turn) - turn * END
}

#[test]
fn oblique_spherical_corners_work_beside_a_reflex_vertex() {
    let pentagon = Pentagon::new();
    // Every top and bottom edge, and the vertical edge at the corner between them.
    let mut points: Vec<_> = (0..5)
        .flat_map(|i| [0., 20.].map(|z| pentagon.rim(i, z)))
        .collect();
    points.push(pentagon.vertical(3));
    let Pentagon {
        area,
        perimeter,
        angles,
        ..
    } = pentagon;
    pentagon.check(&points, &[1., 5.], (0, 2), |r| {
        20. * area - (2. * perimeter * K + 20. * corner(angles[3])) * r * r
            + 2. * (sphere(angles[3]) - 2. * END * cot(angles[3] / 2.)
                + 2. * END * angles.iter().map(|&a| cot(a / 2.)).sum::<f64>())
                * r.powi(3)
    });
}

#[test]
fn oblique_toroidal_corners_wrap_a_concave_round() {
    let pentagon = Pentagon::new();
    let Pentagon {
        area,
        perimeter,
        angles,
        ..
    } = pentagon;
    let turn = angles[4] - PI;
    let concave = 20. * ((turn / 2.).tan() - turn / 2.);
    // The concave vertical edge with both top rims, which end on the neighboring walls.
    let points = [
        pentagon.vertical(4),
        pentagon.rim(3, 20.),
        pentagon.rim(4, 20.),
    ];
    pentagon.check(&points, &[2.], (1, 0), |r| {
        20. * area
            + (concave - K * (pentagon.length(3) + pentagon.length(4))) * r * r
            + (torus(turn) + END * (cot(angles[3]) + cot(angles[0]))) * r.powi(3)
    });
    // All rims and vertical edges but the one at the sharpest corner, as in the app.
    let mut points: Vec<_> = (0..5)
        .flat_map(|i| [0., 20.].map(|z| pentagon.rim(i, z)))
        .collect();
    points.extend((1..5).map(|i| pentagon.vertical(i)));
    let convex: f64 = (1..4).map(|i| corner(angles[i])).sum();
    let corners = (1..4).map(|i| sphere(angles[i])).sum::<f64>()
        + 2. * END * cot(angles[0] / 2.)
        + torus(turn);
    pentagon.check(&points, &[1., 4.9], (2, 6), |r| {
        20. * area
            + (concave - 2. * perimeter * K - 20. * convex) * r * r
            + 2. * corners * r.powi(3)
    });
    // At 5 the rounds at both ends of the wall beside the sharpest corner overlap.
    let shell = &pentagon.prism.boundaries()[0];
    let ids: Vec<_> = points
        .iter()
        .map(|&p| common::blend::edge_through(shell, p).id())
        .collect();
    let before = serde_json::to_string(&pentagon.prism.compress()).unwrap();
    assert_eq!(
        try_fillet_solid_edges(&pentagon.prism, &ids, 5., TOL)
            .unwrap_err()
            .code,
        truck_base::diagnostics::Code::OutsideNeighbour
    );
    assert_eq!(
        before,
        serde_json::to_string(&pentagon.prism.compress()).unwrap()
    );
}

#[test]
fn oblique_toroidal_corner_ends_on_the_wall_of_a_sharp_rim() {
    let pentagon = Pentagon::new();
    let Pentagon { area, angles, .. } = pentagon;
    let turn = angles[4] - PI;
    // The rim round sweeps around the concave round until the plane of the other wall,
    // `r` from the torus axis. At inward distance `t` it stops `acos(r / (r + t))` short
    // of the full turn, and its straight part loses one tangent length `r · tan(turn / 2)`.
    let removed = |r: f64| {
        let section = |t: f64| {
            let profile = r - (r * r - (r - t) * (r - t)).sqrt();
            profile * ((turn - (r / (r + t)).acos()) * (r + t) - r * (turn / 2.).tan())
        };
        let n = 200_000;
        let h = r / n as f64;
        (0..=n)
            .map(|i| {
                let weight = if i == 0 || i == n {
                    1.
                } else if i % 2 == 1 {
                    4.
                } else {
                    2.
                };
                weight * section(i as f64 * h)
            })
            .sum::<f64>()
            * h
            / 3.
    };
    let points = [pentagon.vertical(4), pentagon.rim(3, 20.)];
    pentagon.check(&points, &[2., 4.], (1, 0), |r| {
        20. * area
            + (20. * ((turn / 2.).tan() - turn / 2.) - K * pentagon.length(3)) * r * r
            + END * cot(angles[3]) * r.powi(3)
            - removed(r)
    });
}
