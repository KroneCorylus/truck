mod common;
use truck_modeling::*;
use truck_shapeops::local::{try_shell, try_shell_outward};
const TOL: f64 = 0.001;

fn prism(u: bool) -> Solid {
    let points = if u {
        vec![
            (0., 0.),
            (20., 0.),
            (20., 20.),
            (15., 20.),
            (15., 5.),
            (5., 5.),
            (5., 20.),
            (0., 20.),
        ]
    } else {
        vec![
            (0., 0.),
            (20., 0.),
            (20., 10.),
            (10., 10.),
            (10., 20.),
            (0., 20.),
        ]
    };
    let vertices = points
        .into_iter()
        .map(|(x, y)| Vertex::new(Point3::new(x, y, 0.)))
        .collect::<Vec<_>>();
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(vec![wire]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}

#[test]
fn planar_concave_shells_preserve_offset_geometry_and_material() {
    for u in [false, true] {
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(7., -3., 4.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7)),
        ] {
            let input = builder::transformed(&prism(u), transform);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let top = common::blend::face_through(
                &input.boundaries()[0],
                transform.transform_point(Point3::new(2., 2., 10.)),
            );
            for closed in [false, true] {
                for outward in [false, true] {
                    let selected = if closed {
                        vec![]
                    } else {
                        vec![input.boundaries()[0][top].id()]
                    };
                    let t = 0.5;
                    let result = if outward {
                        try_shell_outward(&input, &selected, t)
                    } else {
                        try_shell(&input, &selected, t)
                    }
                    .unwrap();
                    let (area, perimeter) = if u { (250., 110.) } else { (300., 80.) };
                    let volume = if outward {
                        (area + perimeter * t + 4. * t * t)
                            * (10. + if closed { 2. * t } else { t })
                            - area * 10.
                    } else {
                        area * 10.
                            - (area - perimeter * t + 4. * t * t)
                                * (10. - if closed { 2. * t } else { t })
                    };
                    common::assert_solid(&result, volume, if closed { &[0, 0] } else { &[0] }, TOL);
                    assert!(result.is_geometric_consistent());
                    common::blend::assert_step(&result, volume, TOL);
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}

#[test]
fn concave_shell_collapse_fails_without_modifying_input() {
    for u in [false, true] {
        let input = prism(u);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let top = common::blend::face_through(&input.boundaries()[0], Point3::new(2., 2., 10.));
        for thickness in [5., 6., 12.] {
            assert!(try_shell(&input, &[input.boundaries()[0][top].id()], thickness).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
