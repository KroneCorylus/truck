mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edge, try_fillet_solid_edges};
const TOL: f64 = 0.001;
fn channel() -> Solid { channel_height(10.) }
fn channel_height(height: f64) -> Solid {
    let block =
        common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., height));
    let mut tool = common::modeling::cylinder(Point3::new(0., 0., 10.), Vector3::unit_y(), 6., 11.);
    tool.not();
    truck_shapeops::and(&block, &tool, TOL).unwrap()
}
fn removed(fillet: bool, a: f64, b: f64) -> f64 { removed_at_height(fillet, a, b, 10.) }
fn removed_at_height(fillet: bool, a: f64, b: f64, height: f64) -> f64 {
    // Integrate the removed section against the channel's circular chord width.
    let n = 1024;
    let integral = |theta: f64| {
        let (s, width, ds) = if fillet {
            (
                a * (1. - theta.cos()),
                a * (1. - theta.sin()),
                a * theta.sin(),
            )
        } else {
            (b * theta.sin(), a * (1. - theta.sin()), b * theta.cos())
        };
        2. * (36. - (height - 10. - s).powi(2)).sqrt() * width * ds
    };
    let h = PI / (2. * n as f64);
    (0..=n)
        .map(|i| {
            integral(i as f64 * h)
                * if i == 0 || i == n {
                    1.
                } else if i % 2 == 0 {
                    2.
                } else {
                    4.
                }
        })
        .sum::<f64>()
        * h
        / 3.
}

#[test]
fn oversized_curved_end_blends_fail_without_changing_the_input() {
    let input = channel();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let id = common::blend::edge_through(&input.boundaries()[0], Point3::new(0., 0., 10.)).id();
    for size in [6., 8., 20.] {
        assert!(try_chamfer_solid_edge(&input, id, size, size, TOL).is_err());
        assert!(try_fillet_solid_edges(&input, &[id], size, TOL).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
#[test]
fn straight_blends_extend_and_trim_cylindrical_end_faces() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&channel(), transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            transform.transform_point(Point3::new(0., 0., 10.)),
        )
        .id();
        for (fillet, a, b) in [
            (false, 0.2, 0.2),
            (false, 1., 2.),
            (true, 0.2, 0.2),
            (true, 2., 2.),
        ] {
            let result = if fillet {
                try_fillet_solid_edges(&input, &[edge], a, TOL)
            } else {
                try_chamfer_solid_edge(&input, edge, a, b, TOL)
            }
            .unwrap();
            let expected = 4000. - 180. * PI - removed(fillet, a, b);
            common::assert_solid(&result.solid, expected, &[0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), 1);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn revolved_cylinder_terminations_preserve_exact_geometry() {
    let input = channel();
    for face in input.face_iter() {
        let surface = face.surface();
        let Some((Elementary::Cylinder { origin, axis, .. }, outward)) = surface.elementary()
        else {
            continue;
        };
        let Surface::Extruded(extrusion) = surface else {
            continue;
        };
        let start = extrusion.entity_curve().front();
        let line = Curve::Line(Line(start, start + extrusion.extruding_vector()));
        let mut replacement = Surface::RevolutedCurve(Processor::new(
            RevolutedCurve::by_revolution(line, origin, axis),
        ));
        if replacement.elementary().unwrap().1 != outward {
            replacement.invert();
        }
        face.set_surface(replacement);
    }
    assert!(input.is_geometric_consistent());
    let id = common::blend::edge_through(&input.boundaries()[0], Point3::new(0., 0., 10.)).id();
    for fillet in [false, true] {
        let size = if fillet { 2. } else { 0.2 };
        let result = if fillet {
            try_fillet_solid_edges(&input, &[id], size, TOL)
        } else {
            try_chamfer_solid_edge(&input, id, size, size, TOL)
        }
        .unwrap();
        let volume = 4000. - 180. * PI - removed(fillet, size, size);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
        assert!(result.solid.is_geometric_consistent());
    }
}

#[test]
fn curved_terminations_can_turn_beyond_the_contact_line_endpoints() {
    let input = channel_height(12.);
    let id = common::blend::edge_through(&input.boundaries()[0], Point3::new(0., 0., 12.)).id();
    let section = 36. * (PI / 2. + (2_f64 / 6.).asin()) + 2. * 32_f64.sqrt();
    for fillet in [false, true] {
        let result = if fillet {
            try_fillet_solid_edges(&input, &[id], 3., TOL)
        } else {
            try_chamfer_solid_edge(&input, id, 3., 3., TOL)
        }
        .unwrap();
        assert!(result.solid.is_geometric_consistent());
        let volume = 4800. - 10. * section - removed_at_height(fillet, 3., 3., 12.);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
    }
}
