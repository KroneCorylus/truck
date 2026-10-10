mod common;
use truck_modeling::*;
use truck_shapeops::local::{try_shell, try_shell_outward};
const TOL: f64 = 0.001;

fn circle(center: Point3, radius: f64) -> Wire {
    builder::rsweep(
        &builder::vertex(center + Vector3::unit_x() * radius),
        center,
        Vector3::unit_z(),
        Rad(std::f64::consts::TAU),
        2,
    )
}

#[test]
fn annular_shell_preserves_hole_clearance_and_volume() {
    let face = builder::try_attach_plane(&[
        circle(Point3::origin(), 10.0),
        circle(Point3::origin(), 3.0).inverse(),
    ])
    .unwrap();
    check_openings(&face, 1, [169.0, 182.0].map(|v| v * std::f64::consts::PI));
}

#[test]
fn perforated_shell_keeps_separate_opening_rims() {
    let vertices = [(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)));
    let exterior: Wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let face = builder::try_attach_plane(&[
        exterior,
        circle(Point3::new(6.0, 6.0, 0.0), 2.0).inverse(),
        circle(Point3::new(14.0, 14.0, 0.0), 2.0).inverse(),
    ])
    .unwrap();
    let pi = std::f64::consts::PI;
    check_openings(&face, 2, [570.5 + 38.75 * pi, 630.5 + 32.75 * pi]);
}

fn check_openings(face: &Face, genus: usize, volumes: [f64; 2]) {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(7.0, -3.0, 4.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7)),
    ] {
        let source = builder::tsweep(face, Vector3::unit_z() * 10.0);
        let input = builder::transformed(&source, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let inverse = transform.invert().unwrap();
        let top = input
            .face_iter()
            .find(|face| {
                face.vertex_iter()
                    .all(|vertex| inverse.transform_point(vertex.point()).z.near(&10.0))
            })
            .unwrap()
            .id();
        for (outward, volume) in [false, true].into_iter().zip(volumes) {
            let history =
                truck_shapeops::local::try_shell_with_history(&input, &[top], 0.5, outward)
                    .unwrap();
            let result = history.solid;
            let output_ids: std::collections::HashSet<_> =
                result.face_iter().map(Face::id).collect();
            let mapped: std::collections::HashSet<_> = history
                .outer_faces
                .iter()
                .chain(&history.inner_faces)
                .map(|&(output, _)| output)
                .collect();
            assert_eq!(output_ids, mapped);
            assert_eq!(
                mapped.len(),
                history.outer_faces.len() + history.inner_faces.len()
            );
            for face in input.face_iter() {
                let count = history
                    .outer_faces
                    .iter()
                    .filter(|&&(_, source)| source == face.id())
                    .count();
                assert_eq!(
                    count,
                    if face.id() == top {
                        face.boundaries().len()
                    } else {
                        1
                    }
                );
                let count = history
                    .inner_faces
                    .iter()
                    .filter(|&&(_, source)| source == face.id())
                    .count();
                assert_eq!(count, usize::from(face.id() != top));
            }
            common::assert_solid(&result, volume, &[genus], TOL);
            assert!(result.is_geometric_consistent());
            common::blend::assert_step(&result, volume, TOL);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

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
