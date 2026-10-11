mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::{out::*, r#in::Table};

fn read(step: &str) -> Solid {
    let table = Table::from_step(step).unwrap();
    let source = table.manifold_solid_brep.values().next().unwrap();
    let (mut solid, skipped) = table.to_compressed_solid(source).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    for shell in &mut solid.boundaries {
        shell.split_closed_edges_and_faces(0.00001);
    }
    Solid::extract(
        solid
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap()
}

fn check_once(solid: &Solid, volume: f64, tol: f64) {
    assert!(solid.is_geometric_consistent());
    common::assert_solid(solid, volume, &[0], tol);
    for face in solid.face_iter() {
        let surface = face.surface();
        if let Surface::Extruded(_) = &surface {
            for edge in face.boundary_iters().into_iter().flatten() {
                let curve = edge.curve();
                let (a, b) = curve.range_tuple();
                for i in 0..=23 {
                    let p = curve.subs(a + (b - a) * i as f64 / 23.);
                    let (u, v) = surface.search_nearest_parameter(p, None, 100).unwrap();
                    assert!((-1e-8..=1. + 1e-8).contains(&v), "extrusion parameter {v}");
                    assert!(surface.subs(u, v).distance(p) < 1e-6);
                }
            }
        }
    }
}

fn check(solid: &Solid, volume: f64, tol: f64) {
    check_once(solid, volume, tol);
    let compressed = solid.compress();
    let text = StepDisplay::new(
        Default::default(),
        StepDesign::from_model(StepModel::from(&compressed)),
    )
    .to_string();
    check_once(&read(&text), volume, tol);
}

#[test]
fn cylindrical_and_oblique_extrusion_imports_cover_the_face_boundaries() {
    for vector in [
        Vector3::new(0., 0., 0.2),
        Vector3::new(0., 0., 100.),
        Vector3::new(0., 0., -10.),
        Vector3::new(3., 2., 40.),
    ] {
        for transformed in [false, true] {
            let p = builder::vertex(Point3::new(2., 0., 0.));
            let wire: Wire = builder::rsweep(&p, Point3::origin(), Vector3::unit_z(), Rad(7.), 3);
            let mut face = builder::try_attach_plane(&[wire]).unwrap();
            if vector.z < 0. {
                face.invert();
            }
            let mut solid: Solid = builder::tsweep(&face, vector);
            if transformed {
                solid = builder::transformed(
                    &solid,
                    Matrix4::from_translation(Vector3::new(13., -7., 23.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
                );
            }
            let compressed = solid.compress();
            let text = StepDisplay::new(
                Default::default(),
                StepDesign::from_model(StepModel::from(&compressed)),
            )
            .to_string();
            let imported = read(&text);
            check(&imported, 4. * PI * vector.z.abs(), 0.001);
        }
    }
}

#[test]
fn native_parabolic_prism_import_has_bounded_consistent_supports() {
    let text = include_str!("fixtures/conic-import/parabolic-prism.step");
    let solid = read(text);
    check(&solid, 400_000_000. / 3., 0.01);
}
