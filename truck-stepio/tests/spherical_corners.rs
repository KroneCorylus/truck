use truck_modeling::*;

#[test]
fn spherical_corner_step_trims_are_exact_meridians_and_equators() {
    for transformed in [false, true] {
        let cube = primitive::cuboid(BoundingBox::from_iter([
            Point3::origin(),
            Point3::new(10., 10., 10.),
        ]));
        let mut seen = std::collections::HashSet::new();
        let edges: Vec<_> = cube
            .edge_iter()
            .map(|e| e.id())
            .filter(|id| seen.insert(*id))
            .collect();
        let blend = truck_shapeops::fillet::try_fillet_solid_edges(&cube, &edges, 4.999999, 0.001)
            .unwrap()
            .solid;
        let blend = if transformed {
            builder::transformed(
                &blend,
                Matrix4::from_translation(Vector3::new(13., -7., 2.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.43)),
            )
        } else {
            blend
        };
        let input = blend.compress();
        let before = serde_json::to_string(&input).unwrap();
        let prepared = truck_stepio::out::prepare_for_step(&input, 0.00005).unwrap();
        assert_eq!(serde_json::to_string(&input).unwrap(), before);
        let shell = &prepared.boundaries[0];
        assert_eq!(
            shell
                .edges
                .iter()
                .filter(|e| matches!(e.curve, Curve::Conic(_)))
                .count(),
            24
        );
        let mut corners = 0;
        for (old, face) in input.boundaries[0].faces.iter().zip(&shell.faces) {
            assert_eq!(old.boundaries, face.boundaries);
            assert_eq!(old.orientation, face.orientation);
            let Surface::Sphere(sphere) = &face.surface else {
                continue;
            };
            corners += 1;
            assert_eq!(
                old.surface.elementary().unwrap().1,
                face.surface.elementary().unwrap().1
            );
            let z = sphere.transform()[2].truncate().normalize();
            let mut equators = 0;
            for edge in face.boundaries.iter().flatten() {
                let Curve::Conic(circle) = &shell.edges[edge.index].curve else {
                    panic!("spline arc")
                };
                let frame = circle.transform();
                let normal = frame[0].truncate().cross(frame[1].truncate()).normalize();
                let dot = normal.dot(z).abs();
                assert!(dot < 1e-12 || (dot - 1.).abs() < 1e-12);
                equators += usize::from(dot > 0.5);
                let original = &input.boundaries[0].edges[edge.index].curve;
                assert!(original.front().distance(circle.front()) < 1e-10);
                assert!(original.back().distance(circle.back()) < 1e-10);
                for i in 0..=16 {
                    let (a, b) = original.range_tuple();
                    let point = original.subs(a + (b - a) * i as f64 / 16.);
                    let parameter = circle.search_parameter(point, None, 100).unwrap();
                    assert!(circle.subs(parameter).distance(point) < 1e-10);
                }
            }
            assert_eq!(equators, 1);
        }
        assert_eq!(corners, 8);
    }
}
