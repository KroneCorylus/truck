mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn pipe_volume(n: usize) -> f64 {
    // Radius-50 disk swept along the cubic with controls (0,0), (100,0), (100,100), (200,100).
    let length = (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let speed = (300. * (1. - 2. * t + 2. * t * t)).hypot(600. * t * (1. - t));
            let weight = if i == 0 || i == n {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            weight * speed
        })
        .sum::<f64>()
        / (3. * n as f64);
    PI * 50_f64.powi(2) * length
}

#[test]
fn imported_closed_spline_pipe_preserves_its_seams_and_material() {
    let expected = pipe_volume(512);
    assert!((expected - pipe_volume(1024)).abs() < 1e-7);
    let table = Table::from_step(include_str!("fixtures/spline-pipe/input.step")).unwrap();
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(TOL / 100.);
    let mapped = compressed
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .expect("the split pipe seams must convert to modeling curves");
    let input = Solid::extract(mapped).unwrap();
    assert!(input.is_geometric_consistent());
    for (scale, transformed) in [(0.01_f64, false), (0.025, true)] {
        let matrix = (if transformed {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        }) * Matrix4::from_scale(scale);
        let solid = builder::transformed(&input, matrix);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        common::assert_solid(&solid, expected * scale.powi(3), &[0], TOL);
        common::blend::assert_step(&solid, expected * scale.powi(3), TOL);
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}
