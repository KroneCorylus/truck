//! Exact CAD operands from the R7 performance and R8 conic-volume reports.
mod common;
use std::time::Instant;
use truck_meshalgo::prelude::*;
use truck_modeling::*;

fn operands(text: &str) -> [Solid; 2] {
    let compressed = serde_json::from_str::<[_; 2]>(text).unwrap();
    compressed.map(|solid| Solid::extract(solid).unwrap())
}

fn cut_and_measure(input: [Solid; 2]) -> f64 {
    let [target, tool] = input;
    let before = serde_json::to_string(&[target.compress(), tool.compress()]).unwrap();
    let mut last = None;
    for iteration in 0..3 {
        truck_shapeops::profile::take();
        let start = Instant::now();
        let result = truck_shapeops::subtract_with_effect(&target, &tool, 0.01)
            .expect("subtraction succeeds");
        eprintln!(
            "subtract_with_effect [{iteration}]: {:?}; {:?}",
            start.elapsed(),
            truck_shapeops::profile::take()
        );
        assert!(result.removed_material);
        assert_eq!(result.solid.face_iter().count(), 22);
        assert!((exact_cap_volume(&result.solid) - 46009.72603).abs() < 0.001);
        last = Some(result.solid);
    }
    let cut = last.unwrap();
    assert_eq!(cut.face_iter().count(), 22);
    assert_eq!(
        truck_shapeops::solid_components(&cut, 0.01).unwrap().len(),
        1
    );
    assert_eq!(
        before,
        serde_json::to_string(&[target.compress(), tool.compress()]).unwrap()
    );
    common::assert_topology(&cut, &[0]);
    common::assert_mesh_closed(&cut, 0.0005);
    let volume = cut.triangulation(0.0005).to_polygon().volume();
    eprintln!("fine mesh volume: {volume:.9} mm^3");
    volume
}

#[test]
fn original_nurbs_remove_preserves_the_reference_volume() {
    let volume = cut_and_measure(operands(include_str!("R7-extrude-remove-inputs.json")));
    assert!((volume - 46009.72603).abs() < 1.0, "volume {volume}");
}

#[test]
fn equivalent_conics_must_preserve_the_reference_volume() {
    let volume = cut_and_measure(operands(include_str!("R8-conic-remove-inputs.json")));
    assert!((volume - 46009.72603).abs() < 1.0, "volume {volume}");
}

// For these vertical extrusions, the divergence theorem reduces volume to z times
// the signed area of horizontal caps. Integrate exact curves, independently of the mesher.
fn exact_cap_volume(solid: &Solid) -> f64 {
    solid
        .face_iter()
        .filter(|f| matches!(f.surface(), Surface::Plane(p) if p.normal().z.abs() > 0.9))
        .flat_map(|f| f.boundaries())
        .flat_map(|w| w.into_iter())
        .map(|e| {
            let c = e.oriented_curve();
            let (a, b) = c.range_tuple();
            let n = 1000;
            let dt = (b - a) / n as f64;
            (0..=n)
                .map(|i| {
                    let t = a + i as f64 * dt;
                    let p = c.subs(t);
                    let weight = if i == 0 || i == n {
                        1.0
                    } else if i % 2 == 0 {
                        2.0
                    } else {
                        4.0
                    };
                    p.z * p.x * c.der(t).y * dt * weight / 3.0
                })
                .sum::<f64>()
        })
        .sum()
}

#[test]
fn reflected_conics_cut_in_the_positive_z_direction() {
    let input = operands(include_str!("R8-conic-remove-inputs.json")).map(|solid| {
        let mut reflected = builder::scaled(&solid, Point3::origin(), Vector3::new(1.0, 1.0, -1.0));
        reflected.not();
        reflected
    });
    let volume = cut_and_measure(input);
    assert!((volume - 46009.72603).abs() < 1.0, "volume {volume}");
}

fn conic_boss(center: Point3, a: Vector3, b: Vector3) -> Solid {
    use std::f64::consts::FRAC_PI_2;
    let transform = Matrix4::from_cols(
        a.extend(0.0),
        b.extend(0.0),
        Vector4::unit_z(),
        center.to_homogeneous(),
    );
    let curves: Vec<_> = (0..4)
        .map(|i| {
            Processor::with_transform(
                TrimmedCurve::new(
                    UnitCircle::<Point3>::new(),
                    (i as f64 * FRAC_PI_2, (i + 1) as f64 * FRAC_PI_2),
                ),
                transform,
            )
        })
        .collect();
    let vertices: Vec<_> = curves.iter().map(|c| builder::vertex(c.front())).collect();
    let wire: Wire = (0..4)
        .map(|i| Edge::new(&vertices[i], &vertices[(i + 1) % 4], curves[i].into()))
        .collect();
    let cap = builder::try_attach_plane(&[wire]).unwrap();
    builder::tsweep(&cap, Vector3::new(0.0, 0.0, 10.0))
}

#[test]
fn sequential_conic_union_then_subtract() {
    let plate = common::modeling::cuboid(
        Point3::new(-31.842650625184664, -26.58168412458828, 0.0),
        Point3::new(18.157349374816608, 23.418315875423563, 10.0),
    );
    let circle = conic_boss(
        Point3::new(17.781270943734786, 23.11119839443968, 10.0),
        Vector3::new(25.0, 0.0, 0.0),
        Vector3::new(0.0, 25.0, 0.0),
    );
    let ellipse = conic_boss(
        Point3::new(-38.53496025925936, -8.926093321483627, 10.0),
        Vector3::new(20.315370840784283, -6.5925394208912955, 0.0),
        Vector3::new(10.687311491003078, 32.933697073190814, 0.0),
    );
    let target = truck_shapeops::or(&plate, &circle, 0.01).unwrap();
    let target = truck_shapeops::or(&target, &ellipse, 0.01).unwrap();
    assert!((exact_cap_volume(&target) - 67867.559279005).abs() < 0.001);
    let [_, tool] = operands(include_str!("R8-conic-remove-inputs.json"));
    let volume = cut_and_measure([target, tool]);
    assert!((volume - 46009.72603).abs() < 1.0, "volume {volume}");
}
