mod common;
use std::f64::consts::{FRAC_PI_2, PI};
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn sector(radius: f64) -> (Solid, Vec<EdgeID>) {
    let z = radius * 3_f64.sqrt() / 2.;
    let pole = builder::vertex(Point3::new(0., 0., -radius));
    let axis = builder::vertex(Point3::new(0., 0., z));
    let x = builder::vertex(Point3::new(radius / 2., 0., z));
    let y = builder::vertex(Point3::new(0., radius / 2., z));
    let axial = builder::line(&pole, &axis);
    let radial_x = builder::line(&axis, &x);
    let radial_y = builder::line(&axis, &y);
    let top_arc = builder::circle_arc(
        &x,
        &y,
        Point3::new(radius / 8_f64.sqrt(), radius / 8_f64.sqrt(), z),
    );
    let meridian_x = builder::circle_arc(&pole, &x, Point3::new(radius, 0., 0.));
    let meridian_y = builder::circle_arc(&pole, &y, Point3::new(0., radius, 0.));
    let cap =
        builder::try_attach_plane(&[wire![radial_x.clone(), top_arc.clone(), radial_y.inverse()]])
            .unwrap();
    let side_x = builder::try_attach_plane(&[wire![
        meridian_x.clone(),
        radial_x.inverse(),
        axial.inverse()
    ]])
    .unwrap();
    let side_y =
        builder::try_attach_plane(&[wire![axial.clone(), radial_y.clone(), meridian_y.inverse()]])
            .unwrap();
    let sphere = Face::new(
        vec![wire![meridian_x.inverse(), meridian_y, top_arc.inverse()]],
        Surface::Sphere(Processor::new(Sphere::new(Point3::origin(), radius))),
    );
    (
        Solid::new(vec![vec![cap, side_x, side_y, sphere].into()]),
        vec![axial.id(), radial_x.id(), radial_y.id()],
    )
}

// Integrate the removed vertical corner and the additional rounded upper strip.
fn volume(radius: f64, round: f64) -> f64 {
    let top = radius * 3_f64.sqrt() / 2.;
    let k = 1. - PI / 4.;
    let base = PI / 4. * (radius * radius * (top + radius) - (top.powi(3) + radius.powi(3)) / 3.);
    let vertical = |t: f64| {
        let x = round * (1. - t.cos());
        let y = round * (1. - t.sin());
        let a = radius * radius - x * x;
        (top * y + 0.5 * (y * (a - y * y).sqrt() + a * (y / a.sqrt()).asin())) * round * t.sin()
    };
    let top_cut = |t: f64| {
        let rho = round * t.cos();
        let a = round - rho;
        let z = top - round + round * t.sin();
        let outer = (radius * radius - z * z).sqrt();
        (outer * outer * (a / outer).asin() + a * (outer * outer - a * a).sqrt()
            - a * a
            - k * round * round
            + k * rho * rho)
            * round
            * t.cos()
    };
    let n = 4096;
    let d = FRAC_PI_2 / n as f64;
    let f = |t| vertical(t) + top_cut(t);
    base - d / 3.
        * (f(0.)
            + f(FRAC_PI_2)
            + (1..n)
                .map(|i| (if i % 2 == 0 { 2. } else { 4. }) * f(i as f64 * d))
                .sum::<f64>())
}

#[test]
fn planar_corner_fillets_trim_their_polar_terminal() {
    let (original, ids) = sector(1.5);
    common::assert_solid(&original, volume(1.5, 0.), &[0], 0.001);
    for round in [0.1, 0.2] {
        for moved in [false, true] {
            let scale = if moved { 2.5 } else { 1. };
            let matrix = if moved {
                Matrix4::from_translation(Vector3::new(7., -11., 13.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(scale)
            } else {
                Matrix4::identity()
            };
            let inverse = matrix.invert().unwrap();
            let input = builder::transformed(&original, matrix);
            let mut selected: Vec<_> = ids
                .iter()
                .map(|id| {
                    let edge = original.edge_iter().find(|e| e.id() == *id).unwrap();
                    common::blend::edge_through(
                        &input.boundaries()[0],
                        matrix.transform_point(edge.front().point().midpoint(edge.back().point())),
                    )
                    .id()
                })
                .collect();
            let before = serde_json::to_string(&input.compress()).unwrap();
            for _ in 0..2 {
                let result =
                    try_fillet_solid_edges(&input, &selected, round * scale, 0.001 * scale)
                        .unwrap();
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                assert!(result.solid.is_geometric_consistent());
                assert_eq!(result.generated_faces.len(), 4);
                let mut counts = [0, 0];
                for face in &result.generated_faces {
                    match face.surface().elementary().unwrap().0 {
                        Elementary::Cylinder { radius, .. } => {
                            assert!((radius - round * scale).abs() < 1e-8);
                            counts[0] += 1;
                        }
                        Elementary::Sphere { radius, .. } => {
                            assert!((radius - round * scale).abs() < 1e-8);
                            counts[1] += 1;
                        }
                        _ => panic!("constant-radius fillet surfaces"),
                    }
                }
                assert_eq!(counts, [3, 1]);
                let mut seen = std::collections::HashSet::new();
                let mut terminal_edges = 0;
                for edge in result
                    .generated_faces
                    .iter()
                    .flat_map(Face::edge_iter)
                    .filter(|e| seen.insert(e.id()))
                {
                    let curve = edge.oriented_curve();
                    let (a, b) = curve.range_tuple();
                    if inverse.transform_point(curve.subs((a + b) / 2.)).z > -1. {
                        continue;
                    }
                    terminal_edges += 1;
                    for i in 0..=32 {
                        let p = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 32.));
                        assert!((p.to_vec().magnitude() - 1.5).abs() * scale < 1e-7);
                        assert!(((p.x - round).hypot(p.y - round) - round).abs() * scale < 1e-7);
                        assert!(p.z > -1.5);
                    }
                }
                assert_eq!(terminal_edges, 1);
                let expected = volume(1.5, round) * scale.powi(3);
                common::assert_solid(&result.solid, expected, &[0], 0.0003 * scale);
                common::blend::assert_step(&result.solid, expected, 0.0003 * scale);
                selected.reverse();
            }
        }
    }
}

#[test]
fn polar_terminal_collisions_leave_the_input_unchanged() {
    let (input, selected) = sector(1.5);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [0.75, 3.] {
        assert!(try_fillet_solid_edges(&input, &selected, radius, 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
