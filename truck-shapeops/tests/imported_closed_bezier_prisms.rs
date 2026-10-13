mod common;
use truck_modeling::*;
use truck_shapeops::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input(quartic: bool, robust: bool, start: usize) -> Solid {
    let text = if quartic {
        include_str!("fixtures/closed-bezier-prisms/quartic.step")
    } else {
        include_str!("fixtures/closed-bezier-prisms/cubic.step")
    };
    let table = Table::from_step(text).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    for shell in &mut raw.boundaries {
        for face in &mut shell.faces {
            for wire in &mut face.boundaries {
                let rotation = start % wire.len();
                wire.rotate_left(rotation);
            }
        }
    }
    if robust {
        raw.robust_split_closed_edges_and_faces(1e-5);
    } else {
        raw.split_closed_edges_and_faces(1e-5);
    }
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .expect("closed Bézier extrusion must split into simple face boundaries")
}

#[test]
fn closed_bezier_import_preserves_material_mesh_and_step() {
    for quartic in [false, true] {
        for robust in [false, true] {
            for start in 0..3 {
                let scale: f64 = if start == 2 { 0.1 } else { 1. };
                let transform = if start == 0 {
                    Matrix4::identity()
                } else {
                    Matrix4::from_translation(Vector3::new(2., -3., 5.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                        * Matrix4::from_scale(scale)
                };
                let solid = builder::transformed(&input(quartic, robust, start), transform);
                let volume = (if quartic { 7200. / 7. } else { 150. }) * scale.powi(3);
                assert!(
                    solid.is_geometric_consistent(),
                    "quartic={quartic} robust={robust} start={start}"
                );
                common::assert_counts(&solid, 4, 6, 4);
                common::assert_solid(&solid, volume, &[0], 0.001);
                common::blend::assert_step(&solid, volume, 0.001);
            }
        }
    }
}

#[test]
fn unsupported_closed_cusp_rim_fillets_preserve_the_recovered_input() {
    for quartic in [false, true] {
        let solid = input(quartic, false, 0);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let wire = solid
            .face_iter()
            .find(|face| {
                matches!(face.surface(), Surface::Plane(_))
                    && face.vertex_iter().all(|v| (v.point().z - 10.).abs() < 1e-8)
            })
            .unwrap()
            .boundaries()[0]
            .clone();
        for wire in [wire.clone(), wire.inverse()] {
            let error = truck_shapeops::fillet::try_fillet_solid_along_wire(
                &solid,
                &wire,
                if quartic { 0.1 } else { 0.5 },
                0.001,
            )
            .unwrap_err();
            assert_eq!(
                error.code,
                truck_base::diagnostics::Code::UnsupportedGeometry
            );
            assert_eq!(error.stage, "validate_wire");
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
