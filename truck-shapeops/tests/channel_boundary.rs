mod common;

use std::collections::HashSet;
use std::f64::consts::PI;
use truck_modeling::*;

#[test]
fn a_cut_along_cylinder_seams_keeps_each_straight_rim_whole() {
    let operands =
        serde_json::from_str::<[_; 2]>(include_str!("channel-boundary-operands.json")).unwrap();
    let [target, tool] = operands.map(|solid| Solid::extract(solid).unwrap());
    let before = serde_json::to_string(&[target.compress(), tool.compress()]).unwrap();
    for (tol, reversed) in [(0.01, false), (0.005, false), (0.01, true)] {
        let cutter = if reversed {
            Solid::new(
                tool.boundaries()
                    .iter()
                    .map(|shell| shell.iter().rev().cloned().collect())
                    .collect(),
            )
        } else {
            tool.clone()
        };
        let cut = truck_shapeops::try_subtract(&target, &cutter, tol).unwrap();
        for x in [5.243115014889777, 17.243115014889746] {
            let mut seen = HashSet::new();
            let rim: Vec<_> = cut
                .edge_iter()
                .filter(|edge| seen.insert(edge.id()))
                .filter(|edge| {
                    [edge.front(), edge.back()].iter().all(|vertex| {
                        let p = vertex.point();
                        (p.x - x).abs() < 1.0e-6 && (p.z - 10.0).abs() < 1.0e-6
                    })
                })
                .collect();
            assert_eq!(rim.len(), 1, "rim at x={x}, tolerance={tol}");
            let a = rim[0].front().point();
            let b = rim[0].back().point();
            assert!((a.distance(b) - 10.0).abs() < 1.0e-6);
        }
        common::assert_solid(&cut, 10937.5 - 180.0 * PI, &[0], 0.001);
    }
    assert_eq!(
        serde_json::to_string(&[target.compress(), tool.compress()]).unwrap(),
        before
    );
}
