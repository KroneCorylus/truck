mod common;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use truck_modeling::*;
use truck_shapeops::fillet::{
    try_chamfer_solid_along_wire, try_chamfer_solid_along_wire_angle,
    try_chamfer_solid_along_wire_chord, try_chamfer_solid_edges,
};

fn rounded_extrusion(concave: bool) -> Solid {
    if concave {
        let vertices = builder::vertices([
            Point3::new(0., 0., 0.),
            Point3::new(8., 0., 0.),
            Point3::new(10., -2., 0.),
            Point3::new(10., -10., 0.),
            Point3::new(15., -10., 0.),
            Point3::new(15., 10., 0.),
            Point3::new(0., 10., 0.),
        ]);
        let mut edges = vec![
            builder::line(&vertices[0], &vertices[1]),
            builder::circle_arc(
                &vertices[1],
                &vertices[2],
                Point3::new(8. + 2_f64.sqrt(), -2. + 2_f64.sqrt(), 0.),
            ),
        ];
        edges.extend(
            (2..vertices.len())
                .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()])),
        );
        return builder::tsweep(
            &builder::try_attach_plane(&[edges.into()]).unwrap(),
            Vector3::unit_z() * 10.,
        );
    }
    let vertices = builder::vertices([
        Point3::new(0., 0., 0.),
        Point3::new(8., 0., 0.),
        Point3::new(10., 2., 0.),
        Point3::new(10., 10., 0.),
        Point3::new(0., 10., 0.),
    ]);
    let edges = vec![
        builder::line(&vertices[0], &vertices[1]),
        builder::circle_arc(
            &vertices[1],
            &vertices[2],
            Point3::new(8. + 2_f64.sqrt(), 2. - 2_f64.sqrt(), 0.),
        ),
        builder::line(&vertices[2], &vertices[3]),
        builder::line(&vertices[3], &vertices[4]),
        builder::line(&vertices[4], &vertices[0]),
    ];
    let face = builder::try_attach_plane(&[edges.into()]).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * 10.)
}

fn overlap(a: f64, d: f64, inset: f64) -> f64 {
    let m = a.min(d);
    inset / d * (a * d * m - (a + d) * m * m / 2. + m * m * m / 3.)
}

fn check(d: f64, mode: usize, nurbs: bool, concave: bool) {
    let tolerance = 0.001;
    let inset = match mode {
        3 => d * 3_f64.sqrt(),
        4 => d / 5.,
        _ => d,
    };
    let a: f64 = 0.2;
    let base = rounded_extrusion(concave);
    let sign = if concave { -1. } else { 1. };
    let base_volume = 10. * if concave { 204. - PI } else { 96. + PI };
    common::assert_solid(&base, base_volume, &[0], tolerance);
    let base_before = serde_json::to_string(&base.compress()).unwrap();
    let first = common::blend::edge_through(&base.boundaries()[0], Point3::new(0., 5., 10.)).id();
    let input = try_chamfer_solid_edges(&base, &[first], a, tolerance)
        .unwrap()
        .solid;
    assert_eq!(
        base_before,
        serde_json::to_string(&base.compress()).unwrap()
    );
    let input = if nurbs {
        input.mapped(
            |p| *p,
            |curve| {
                NurbsCurve::try_from(curve)
                    .map(Curve::NurbsCurve)
                    .unwrap_or_else(|_| curve.clone())
            },
            Clone::clone,
        )
    } else {
        input
    };
    let input_volume = base_volume - 10. * a * a / 2.;
    common::assert_solid(&input, input_volume, &[0], tolerance);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let witnesses = [
        Point3::new(4., 0., 10.),
        Point3::new(8. + 2_f64.sqrt(), sign * (2. - 2_f64.sqrt()), 10.),
        Point3::new(10., sign * 6., 10.),
    ];
    let ids: Vec<_> = witnesses
        .into_iter()
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .collect();
    let cap = input
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| (v.point().z - 10.).abs() < 1e-9))
        .unwrap();
    let ordered: Vec<_> = ids
        .iter()
        .map(|id| cap.edge_iter().find(|e| e.id() == *id).unwrap().clone())
        .collect();
    let wire: Wire = ordered.into();
    assert_eq!(wire.len(), 3);
    assert!(wire.is_continuous());
    let consumed = common::blend::edge_through(
        &input.boundaries()[0],
        Point3::new(a / 2., 0., 10. - a / 2.),
    )
    .id();
    for reversed in [false, true] {
        let selected = if reversed {
            wire.inverse()
        } else {
            wire.clone()
        };
        let operation = match mode {
            0 | 4 => {
                let (d0, d1) = if reversed { (d, inset) } else { (inset, d) };
                try_chamfer_solid_along_wire(&input, &selected, d0, d1, tolerance)
            }
            1 => try_chamfer_solid_along_wire_chord(&input, &selected, d, d, tolerance),
            _ => try_chamfer_solid_along_wire_angle(
                &input,
                &selected,
                cap.id(),
                inset,
                Rad(if mode == 3 { PI / 6. } else { FRAC_PI_4 }),
                tolerance,
            ),
        };
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        let result = operation
            .expect("terminal contact may retain, exactly consume, or pass the old diagonal");
        // Integrate the two straight strips and quarter-annulus over the top depth.
        // Add back the overlap of the two removed wedges to avoid subtracting it twice.
        let second_wedge = (16. + PI) * inset * d / 2. - sign * PI * inset * inset * d / 12.;
        let expected = input_volume - second_wedge + overlap(a, d, inset);
        common::assert_solid(&result.solid, expected, &[0], tolerance);
        common::assert_volume(&result.solid, expected, 1e-6);
        common::blend::assert_step(&result.solid, expected, tolerance);
        if d > a {
            for point in [
                Point3::new(a / 2., inset * (1. - a / (2. * d)), 10. - a / 2.),
                Point3::new(0., inset * (d - a) / (2. * d), 10. - (a + d) / 2.),
            ] {
                let _ = common::blend::edge_through(&result.solid.boundaries()[0], point);
            }
        }
        assert!(result.solid.is_geometric_consistent());
        if d >= a {
            assert!(!result.solid.edge_iter().any(|e| e.id() == consumed));
        }
        for fraction in [0.25, 0.5, 0.75] {
            let angle = -FRAC_PI_2 * fraction;
            for (inset, z) in [(inset, 10.), (0., 10. - d)] {
                for point in [
                    Point3::new(4., inset, z),
                    Point3::new(10. - sign * inset, sign * 6., z),
                    Point3::new(
                        8. + (2. - sign * inset) * angle.cos(),
                        sign * (2. + (2. - sign * inset) * angle.sin()),
                        z,
                    ),
                ] {
                    let _ = common::blend::edge_through(&result.solid.boundaries()[0], point);
                }
            }
        }
    }
}

macro_rules! regression {
    ($name:ident, $distance:expr, $mode:expr) => {
        #[test]
        fn $name() { check($distance, $mode, false, false); }
    };
}
regression!(projected_preserves_neighbor, 0.1, 0);
regression!(projected_exactly_consumes_neighbor, 0.2, 0);
regression!(projected_passes_consumed_neighbor, 0.5, 0);
regression!(chord_preserves_neighbor, 0.1, 1);
regression!(chord_exactly_consumes_neighbor, 0.2, 1);
regression!(chord_passes_consumed_neighbor, 0.5, 1);
regression!(angle_preserves_neighbor, 0.1, 2);
regression!(angle_exactly_consumes_neighbor, 0.2, 2);
regression!(angle_passes_consumed_neighbor, 0.5, 2);

#[test]
fn nurbs_preserves_neighbor() { check(0.1, 0, true, false); }
#[test]
fn nurbs_passes_consumed_neighbor() { check(0.5, 0, true, false); }

#[test]
fn concave_preserves_neighbor() { check(0.1, 0, false, true); }
#[test]
fn concave_passes_consumed_neighbor() { check(0.5, 0, false, true); }

regression!(unequal_passes_consumed_neighbor, 0.5, 4);
regression!(angle_thirty_passes_consumed_neighbor, 0.5 / 3_f64.sqrt(), 3);

#[test]
fn oversized_consumed_contour_rejects_without_mutating_input() {
    let base = rounded_extrusion(false);
    let first = common::blend::edge_through(&base.boundaries()[0], Point3::new(0., 5., 10.)).id();
    let input = try_chamfer_solid_edges(&base, &[first], 0.2, 0.001)
        .unwrap()
        .solid;
    let cap = input
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| (v.point().z - 10.).abs() < 1e-9))
        .unwrap();
    let wire: Wire = [
        Point3::new(4., 0., 10.),
        Point3::new(8. + 2_f64.sqrt(), 2. - 2_f64.sqrt(), 10.),
        Point3::new(10., 6., 10.),
    ]
    .into_iter()
    .map(|p| {
        let id = common::blend::edge_through(&input.boundaries()[0], p).id();
        cap.edge_iter().find(|e| e.id() == id).unwrap().clone()
    })
    .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for selected in [wire.clone(), wire.inverse()] {
        for result in [
            try_chamfer_solid_along_wire(&input, &selected, 12., 12., 0.001),
            try_chamfer_solid_along_wire_chord(&input, &selected, 12., 12., 0.001),
            try_chamfer_solid_along_wire_angle(
                &input,
                &selected,
                cap.id(),
                12.,
                Rad(FRAC_PI_4),
                0.001,
            ),
        ] {
            let error = result.expect_err("the cut exceeds both extrusion height and arc radius");
            assert_eq!(
                error.code,
                truck_base::diagnostics::Code::BlendConstructionFailed
            );
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
