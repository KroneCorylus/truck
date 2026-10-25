// Copy to truck-shapeops/tests only after coordinator grants the build slot.
mod common;
use std::f64::consts::{FRAC_PI_2, PI};
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_along_wire, try_chamfer_solid_edges};

fn rounded_extrusion() -> Solid {
    let vertices = builder::vertices([
        Point3::new(0., 0., 0.),
        Point3::new(8., 0., 0.),
        Point3::new(10., 2., 0.),
        Point3::new(10., 10., 0.),
        Point3::new(0., 10., 0.),
    ]);
    let edges = vec![
        builder::line(&vertices[0], &vertices[1]),
        builder::circle_arc(&vertices[1], &vertices[2], Point3::new(8. + 2_f64.sqrt(), 2. - 2_f64.sqrt(), 0.)),
        builder::line(&vertices[2], &vertices[3]),
        builder::line(&vertices[3], &vertices[4]),
        builder::line(&vertices[4], &vertices[0]),
    ];
    let face = builder::try_attach_plane(&[edges.into()]).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * 10.)
}

fn overlap(a: f64, d: f64) -> f64 {
    let m = a.min(d);
    a * d * m - (a + d) * m * m / 2. + m * m * m / 3.
}

fn check(d: f64) {
    let tolerance = 0.001;
    let a: f64 = 0.2;
    let base = rounded_extrusion();
    let base_volume = 10. * (96. + PI);
    common::assert_solid(&base, base_volume, &[0], tolerance);
    let base_before = serde_json::to_string(&base.compress()).unwrap();
    let first = common::blend::edge_through(&base.boundaries()[0], Point3::new(0., 5., 10.)).id();
    let input = try_chamfer_solid_edges(&base, &[first], a, tolerance).unwrap().solid;
    assert_eq!(base_before, serde_json::to_string(&base.compress()).unwrap());
    let input_volume = base_volume - 10. * a * a / 2.;
    common::assert_solid(&input, input_volume, &[0], tolerance);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let witnesses = [Point3::new(4., 0., 10.), Point3::new(8. + 2_f64.sqrt(), 2. - 2_f64.sqrt(), 10.), Point3::new(10., 6., 10.)];
    let ids: Vec<_> = witnesses.into_iter().map(|p| common::blend::edge_through(&input.boundaries()[0], p).id()).collect();
    let cap = input.face_iter().find(|f| f.vertex_iter().all(|v| (v.point().z - 10.).abs() < 1e-9)).unwrap();
    let ordered: Vec<_> = ids.iter().map(|id| cap.edge_iter().find(|e| e.id() == *id).unwrap().clone()).collect();
    let wire: Wire = ordered.into();
    assert_eq!(wire.len(), 3);
    assert!(wire.is_continuous());
    let consumed = common::blend::edge_through(&input.boundaries()[0], Point3::new(a / 2., 0., 10. - a / 2.)).id();
        for reversed in [false, true] {
            let selected = if reversed { wire.inverse() } else { wire.clone() };
            let operation = try_chamfer_solid_along_wire(&input, &selected, d, d, tolerance);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            let result = operation.expect("terminal contact may retain, exactly consume, or pass the old diagonal");
            let second_wedge = (16. + PI) * d * d / 2. - PI * d.powi(3) / 12.;
            let expected = input_volume - second_wedge + overlap(a, d);
            common::assert_solid(&result.solid, expected, &[0], tolerance);
            common::blend::assert_step(&result.solid, expected, tolerance);
            assert!(result.solid.is_geometric_consistent());
            if d >= a { assert!(!result.solid.edge_iter().any(|e| e.id() == consumed)); }
            for fraction in [0.25, 0.5, 0.75] {
                let angle = -FRAC_PI_2 * fraction;
                for (inset, z) in [(d, 10.), (0., 10. - d)] {
                    for point in [Point3::new(4., inset, z), Point3::new(10. - inset, 6., z), Point3::new(8. + (2. - inset) * angle.cos(), 2. + (2. - inset) * angle.sin(), z)] {
                        let _ = common::blend::edge_through(&result.solid.boundaries()[0], point);
                    }
                }
            }
        }
}

#[test]
fn tangent_contour_preserves_a_short_terminal_neighbor() { check(0.1); }
#[test]
fn tangent_contour_exactly_consumes_a_terminal_neighbor() { check(0.2); }
#[test]
fn tangent_contour_passes_a_consumed_terminal_neighbor() { check(0.5); }
