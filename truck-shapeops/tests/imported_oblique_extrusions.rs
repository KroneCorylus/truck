mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_along_wire, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

struct Case {
    step: &'static str,
    a: f64,
    shift: f64,
    volume: f64,
    frame: Matrix4,
}

fn cases() -> [Case; 2] {
    [
        Case {
            step: include_str!("fixtures/oblique-extrusions/J6.step"),
            a: 100.,
            shift: 20.,
            volume: 1560464.3531795258,
            frame: Matrix4::identity(),
        },
        Case {
            step: include_str!("fixtures/oblique-extrusions/J8.step"),
            a: 50.,
            shift: 100.,
            volume: 772406.7218958114,
            frame: Matrix4::from_cols(
                Vector4::new(0., 0., -1., 0.),
                Vector4::new(-1., 0., 0., 0.),
                Vector4::new(0., 1., 0., 0.),
                Vector4::new(100., 0., 100., 1.),
            ),
        },
    ]
}

fn input(case: &Case) -> Solid {
    let table = Table::from_step(case.step).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(0.00001);
    let mapped = raw
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .expect("preprocessed extrusion curves preserve their parameters");
    builder::transformed(&Solid::extract(mapped).unwrap(), case.frame)
}

fn transform(scale: f64, moved: bool) -> Matrix4 {
    (if moved {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    }) * Matrix4::from_scale(scale)
}

#[test]
fn imported_oblique_extrusions_preserve_material_and_step() {
    for case in cases() {
        for (scale, moved) in [(0.01, false), (0.01, true), (1., false)] {
            let solid = builder::transformed(&input(&case), transform(scale, moved));
            assert!(solid.is_geometric_consistent());
            let volume = PI * case.a * 50. * 100. * scale.powi(3);
            common::assert_solid(&solid, volume, &[0], 0.001);
            common::blend::assert_step(&solid, volume, 0.001);
        }
    }
}

#[test]
fn oblique_rim_fillets_preserve_radius_contacts_material_and_step() {
    for case in cases() {
        for (scale, reversed) in [(0.01, false), (0.01, true), (1., false)] {
            let matrix = transform(scale, reversed);
            let inverse = matrix.invert().unwrap();
            let solid = builder::transformed(&input(&case), matrix);
            let before = serde_json::to_string(&solid.compress()).unwrap();
            let mut wire = solid
                .face_iter()
                .find(|face| {
                    face.edge_iter().all(|edge| {
                        let curve = edge.curve();
                        let (a, b) = curve.range_tuple();
                        [a, (a + b) / 2., b]
                            .into_iter()
                            .all(|t| (inverse.transform_point(curve.subs(t)).z - 100.).abs() < 1e-7)
                    })
                })
                .unwrap()
                .boundaries()[0]
                .clone();
            assert_eq!(wire.len(), 2);
            if reversed {
                wire.invert();
            }
            let result = try_fillet_solid_along_wire(&solid, &wire, 10. * scale, 1e-7).unwrap();
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), 2);
            let (mut cap_contacts, mut side_contacts, mut radius_checks) = (0, 0, 0);
            for face in &result.generated_faces {
                let surface = face.oriented_surface();
                let (u, v) = surface.try_range_tuple();
                let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                for i in 0..=16 {
                    for j in 0..=16 {
                        let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 16.);
                        let p = inverse.transform_point(surface.subs(u, v));
                        let n = inverse.transform_vector(surface.normal(u, v)).normalize();
                        if n.x.hypot(n.y) > 1e-6 {
                            let phi = (50. * n.y).atan2(case.a * n.x);
                            let e = Point3::new(case.a * phi.cos(), 50. * phi.sin(), 0.);
                            let lateral = Vector3::new(
                                phi.cos() / case.a,
                                phi.sin() / 50.,
                                -case.shift / 100. * phi.cos() / case.a,
                            )
                            .normalize();
                            let v = (90. + 10. * lateral.z) / 100.;
                            let center = e + v * Vector3::new(case.shift, 0., 100.) - 10. * lateral;
                            assert!(
                                (p.distance(center) - 10.).abs() * scale < 1e-6,
                                "a={} scale={scale} p={p:?} radius={}",
                                case.a,
                                p.distance(center)
                            );
                            assert!(n.dot((p - center).normalize()) > 1. - 1e-8);
                            radius_checks += 1;
                        }
                        if (p.z - 100.).abs() * scale < 1e-7 {
                            assert!(n.z > 1. - 1e-8);
                            cap_contacts += 1;
                        }
                        let e = Vector3::new(p.x - case.shift * p.z / 100., p.y, 0.);
                        if ((e.x / case.a).powi(2) + (e.y / 50.).powi(2) - 1.).abs() < 1e-8 {
                            let lateral = Vector3::new(
                                e.x / case.a.powi(2),
                                e.y / 2500.,
                                -case.shift / 100. * e.x / case.a.powi(2),
                            )
                            .normalize();
                            assert!(n.dot(lateral) > 1. - 1e-8);
                            side_contacts += 1;
                        }
                    }
                }
            }
            assert!(cap_contacts >= 34);
            assert!(side_contacts >= 34);
            assert!(radius_checks >= 500);
            common::assert_solid(&result.solid, case.volume * scale.powi(3), &[0], 0.001);
            common::blend::assert_step(&result.solid, case.volume * scale.powi(3), 0.001);
        }
    }
}
