mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
use truck_stepio::{out::*, r#in::Table};

#[test]
fn tangent_inner_boundary_splits_without_panicking_or_losing_material() {
    let text = include_str!("fixtures/closed-face-splitting/tangent-stacked-cylinders.step");
    for transformed in [false, true] {
        for robust in [false, true] {
            let table = Table::from_step(text).unwrap();
            let entity = table.manifold_solid_brep.values().next().unwrap();
            let (mut compressed, skipped) = table.to_compressed_solid(entity).unwrap();
            assert!(skipped.is_empty());
            if transformed {
                let matrix = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
                compressed = compressed
                    .try_mapped(
                        |p| Some(matrix.transform_point(*p)),
                        |c| Some(c.transformed(matrix)),
                        |s| Some(s.transformed(matrix)),
                    )
                    .unwrap();
            }
            if robust {
                compressed.robust_split_closed_edges_and_faces(0.00001);
            } else {
                compressed.split_closed_edges_and_faces(0.00001);
            }
            let model = compressed
                .try_mapped(
                    |p| Some(*p),
                    |c| Curve::try_from(c).ok(),
                    |s| Surface::try_from(s).ok(),
                )
                .expect("preprocessed curves must convert to modeling geometry");
            let solid = Solid::extract(model).unwrap();
            assert!(solid.is_geometric_consistent());
            common::assert_solid(&solid, 2500. * PI, &[0], 0.001);
            let compressed = solid.compress();
            let text = StepDisplay::new(
                Default::default(),
                StepDesign::from_model(StepModel::from(&compressed)),
            )
            .to_string();
            let table = Table::from_step(&text).unwrap();
            let source = table.manifold_solid_brep.values().next().unwrap();
            let (mut compressed, skipped) = table.to_compressed_solid(source).unwrap();
            assert!(skipped.is_empty());
            compressed.split_closed_edges_and_faces(0.00001);
            let model = compressed
                .try_mapped(
                    |p| Some(*p),
                    |c| Curve::try_from(c).ok(),
                    |s| Surface::try_from(s).ok(),
                )
                .unwrap();
            let roundtrip = Solid::extract(model).unwrap();
            assert!(roundtrip.is_geometric_consistent());
            common::assert_solid(&roundtrip, 2500. * PI, &[0], 0.001);
        }
    }
}
