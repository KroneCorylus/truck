use truck_meshalgo::prelude::*;
use truck_modeling::*;
#[test]
fn circular_boolean_boundaries_remain_exact_circles_in_step() {
    let solid: CompressedSolid = serde_json::from_str(include_str!(
        "../../truck-shapeops/tests/contact-bored-block.json"
    ))
    .unwrap();
    let prepared = truck_stepio::out::prepare_for_step(&solid, 0.00005).unwrap();
    let mut circles = 0;
    for (original, output) in solid
        .boundaries
        .iter()
        .flat_map(|s| &s.edges)
        .zip(prepared.boundaries.iter().flat_map(|s| &s.edges))
    {
        let Curve::IntersectionCurve(curve) = &original.curve else {
            continue;
        };
        let surfaces = [
            curve.surface0().elementary().unwrap().0,
            curve.surface1().elementary().unwrap().0,
        ];
        if surfaces.iter().any(|s| matches!(s, Elementary::Plane(_)))
            && surfaces
                .iter()
                .any(|s| matches!(s, Elementary::Cylinder { .. }))
        {
            assert!(
                matches!(output.curve, Curve::Conic(_)),
                "{:?}",
                output.curve
            );
            assert!(original.curve.front().distance(output.curve.front()) < 1e-10);
            assert!(original.curve.back().distance(output.curve.back()) < 1e-10);
            circles += 1;
        }
    }
    assert!(circles >= 4);
    let solid = Solid::extract(prepared).unwrap();
    let mut mesh = solid.triangulation(0.001).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces();
    assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
    assert!((mesh.volume() - (64.0 - 2.5 * std::f64::consts::PI)).abs() < 0.03);
}
