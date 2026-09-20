mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_fillet_solid_edges};

fn cube(transformed: bool) -> Solid {
    let cube = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    if transformed {
        builder::transformed(
            &cube,
            Matrix4::from_translation(Vector3::new(13., -7., 2.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.43)),
        )
    } else {
        cube
    }
}

fn edges(solid: &Solid) -> Vec<EdgeID> {
    let mut seen = std::collections::HashSet::new();
    solid
        .edge_iter()
        .map(|edge| edge.id())
        .filter(|id| seen.insert(*id))
        .collect()
}

#[test]
fn near_limit_fillet_retains_valid_small_faces() {
    for transformed in [false, true] {
        let input = cube(transformed);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected = edges(&input);
        if transformed {
            selected.reverse();
        }
        let radius: f64 = 4.999999;
        let result = try_fillet_solid_edges(&input, &selected, radius, 0.001).unwrap();
        assert!(result.solid.is_geometric_consistent());
        let inset = 10.0 - 2.0 * radius;
        let expected = inset.powi(3)
            + 6.0 * radius * inset.powi(2)
            + 3.0 * PI * radius.powi(2) * inset
            + 4.0 * PI * radius.powi(3) / 3.0;
        common::assert_solid(&result.solid, expected, &[0], 0.001);
        assert_eq!(serde_json::to_string(&input.compress()).unwrap(), before);
    }
}

#[test]
fn near_limit_chamfer_preserves_miter_contract() {
    for transformed in [false, true] {
        let input = cube(transformed);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut selected = edges(&input);
        if transformed {
            selected.reverse();
        }
        let distance: f64 = 4.999999;
        let result = try_chamfer_solid_edges(&input, &selected, distance, 0.001).unwrap();
        assert!(result.solid.is_geometric_consistent());
        // Inclusion-exclusion of the twelve triangular edge cuts and eight miter junctions.
        let expected = 1000.0 - 60.0 * distance.powi(2) + 6.0 * distance.powi(3);
        common::assert_solid(&result.solid, expected, &[0], 0.001);
        assert_eq!(serde_json::to_string(&input.compress()).unwrap(), before);
    }
}
