mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
const TOL: f64 = 0.001;
fn bracket() -> Solid {
    let v = builder::vertices(
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
    let wire: Wire = (0..v.len())
        .map(|i| builder::line(&v[i], &v[(i + 1) % v.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}
fn expected(radius: f64) -> f64 {
    // Exact horizontal section: one inset straight wall, joined to the unchanged wall
    // by a circle of radius 2r-sqrt(r²-z²). Integrate its circular-segment area.
    let n = 4096;
    let h = PI / 2. / n as f64;
    let area = |theta: f64| {
        let q = 2. - theta.cos();
        (q - 0.5 * ((q * q - 1.).max(0.).sqrt() + q * q * (1. / q).asin())) * theta.cos()
    };
    let integral = (0..=n)
        .map(|i| {
            area(i as f64 * h)
                * if i == 0 || i == n {
                    1.
                } else if i % 2 == 0 {
                    2.
                } else {
                    4.
                }
        })
        .sum::<f64>()
        * h
        / 3.;
    3000. + radius.powi(3) * (integral - (1. - PI / 4.))
}
#[test]
fn concave_and_convex_pair_gets_a_trimmed_toroidal_transition() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&bracket(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for radius in [0.2, 1., 3.] {
            for z in [0., 10.] {
                for horizontal in [Point3::new(15., 10., z), Point3::new(10., 15., z)] {
                    let mut ids = [Point3::new(10., 10., 5.), horizontal].map(|p| {
                        common::blend::edge_through(
                            &input.boundaries()[0],
                            transform.transform_point(p),
                        )
                        .id()
                    });
                    let result = try_fillet_solid_edges(&input, &ids, radius, TOL).unwrap();
                    assert_eq!(result.generated_faces.len(), 3);
                    assert!(result.solid.is_geometric_consistent());
                    let torus = result
                        .generated_faces
                        .iter()
                        .find(|face| {
                            matches!(
                                face.surface().elementary(),
                                Some((geometry::Elementary::Torus { .. }, _))
                            )
                        })
                        .unwrap();
                    let mut trimmed_wall = 0;
                    for edge in torus.edge_iter() {
                        let neighbor = result
                            .solid
                            .face_iter()
                            .find(|face| {
                                face.id() != torus.id()
                                    && face.edge_iter().any(|e| e.id() == edge.id())
                            })
                            .unwrap();
                        let curve = edge.curve();
                        let (a, b) = curve.range_tuple();
                        let cut = matches!(curve, Curve::IntersectionCurve(_));
                        trimmed_wall += usize::from(cut);
                        if cut {
                            assert!(matches!(neighbor.surface(), Surface::Plane(_)));
                        }
                        for t in [0.2, 0.5, 0.8] {
                            let point = curve.subs(a + t * (b - a));
                            let normals = [torus, neighbor].map(|face| {
                                let surface = face.oriented_surface();
                                let (u, v) = surface.search_parameter(point, None, 100).unwrap();
                                surface.normal(u, v).normalize()
                            });
                            let dot = normals[0].dot(normals[1]);
                            if cut {
                                assert!(dot < 1. - 1e-6);
                            } else {
                                assert!(dot > 1. - 1e-8);
                            }
                        }
                    }
                    assert_eq!(trimmed_wall, 1);
                    common::assert_solid(&result.solid, expected(radius), &[0], TOL);
                    if radius == 1. {
                        common::blend::assert_step(&result.solid, expected(radius), TOL);
                    }
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
}
