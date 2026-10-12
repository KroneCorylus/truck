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

fn rounded_offset_cylinder_volume() -> f64 {
    let lens = |r: f64| {
        let (s, d): (f64, f64) = (5., 4.9);
        if r >= d + s {
            return PI * s * s;
        }
        let a = ((d * d + r * r - s * s) / (2. * d * r)).clamp(-1., 1.);
        let b = ((d * d + s * s - r * r) / (2. * d * s)).clamp(-1., 1.);
        r * r * a.acos() + s * s * b.acos()
            - ((-d + r + s) * (d + r - s) * (d - r + s) * (d + r + s))
                .max(0.)
                .sqrt()
                / 2.
    };
    let start = (2.9_f64 / 3.).acos();
    let step = (PI / 2. - start) / 4096.;
    let extra: f64 = (0..=4096)
        .map(|i| {
            let t = start + i as f64 * step;
            let weight = if i == 0 || i == 4096 {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * (25. * PI - lens(7. + 3. * t.cos())) * 3. * t.cos()
        })
        .sum::<f64>()
        * step
        / 3.;
    1865. * PI + 31.5 * PI * PI + 250. * PI + extra
}

#[test]
fn rounded_offset_cylinders_preprocess_without_reusing_the_seam() {
    for transformed in [false, true] {
        for robust in [false, true] {
            let table = Table::from_step(include_str!(
                "fixtures/closed-face-splitting/rounded-offset-cylinders.step"
            ))
            .unwrap();
            let entity = table.manifold_solid_brep.values().next().unwrap();
            let (mut raw, skipped) = table.to_compressed_solid(entity).unwrap();
            assert!(skipped.is_empty());
            if transformed {
                let matrix = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
                raw = raw
                    .try_mapped(
                        |p| Some(matrix.transform_point(*p)),
                        |c| Some(c.transformed(matrix)),
                        |s| Some(s.transformed(matrix)),
                    )
                    .unwrap();
            }
            if robust {
                raw.robust_split_closed_edges_and_faces(0.00001);
            } else {
                raw.split_closed_edges_and_faces(0.00001);
            }
            let solid = truck_topology::Solid::extract(raw).unwrap();
            common::assert_solid(&solid, rounded_offset_cylinder_volume(), &[0], 0.001);
        }
    }
}
