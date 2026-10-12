mod common;
use std::f64::consts::{FRAC_PI_2, PI};
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/conical-pipe/input.step")).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(TOL / 100.);
    Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap()
}

fn rounded_volume(n: usize) -> f64 {
    let removed = (0..=n)
        .map(|i| {
            let t = FRAC_PI_2 * i as f64 / n as f64;
            let x = 10. * (1. - t.cos());
            let z = 10. * (1. - t.sin());
            let primitive = |s: f64| {
                let root = (s * s - x * x).sqrt();
                (s * root - x * x * (s + root).ln()) / 2.
            };
            let section =
                primitive(300.) - primitive(300. - z) - primitive(200. + z) + primitive(200.);
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * section * 10. * t.sin()
        })
        .sum::<f64>()
        * FRAC_PI_2
        / (3. * n as f64);
    312_500. * PI - removed
}

#[test]
fn conical_end_trims_mesh_with_physical_tolerance_despite_unequal_parameter_scales() {
    let expected = rounded_volume(512);
    assert!((expected - rounded_volume(1024)).abs() < 1e-7);
    for (scale, transformed) in [(1_f64, false), (0.01, true)] {
        let matrix = (if transformed {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        }) * Matrix4::from_scale(scale);
        let input = builder::transformed(&input(), matrix);
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            matrix.transform_point(Point3::new(-200., 250., 0.)),
        );
        let wire = wire![if transformed { edge.inverse() } else { edge }];
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_along_wire(&input, &wire, 10. * scale, TOL * scale).unwrap();
        assert!(result.solid.is_geometric_consistent());
        let mut coarse = result.solid.triangulation(10. * scale).to_polygon();
        coarse
            .put_together_same_attrs(TOLERANCE)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(coarse.shell_condition(), ShellCondition::Closed);
        assert!(
            coarse.positions().len() < 10_000,
            "excessive coarse refinement: {} vertices",
            coarse.positions().len()
        );
        common::assert_solid(&input, 312_500. * PI * scale.powi(3), &[0], TOL * scale);
        common::assert_solid(&result.solid, expected * scale.powi(3), &[0], TOL * scale);
        common::blend::assert_step(&result.solid, expected * scale.powi(3), TOL * scale);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
