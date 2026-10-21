mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::*;

fn fixture(matrix: Matrix4, curvature: f64) -> (Solid, Edge) {
    let a = Vertex::new(Point3::new(0., 0., 1.));
    let b = Vertex::new(Point3::new(1., 0., 1. + curvature));
    let lo = [
        Vertex::new(Point3::origin()),
        Vertex::new(Point3::new(1., 0., 0.)),
    ];
    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![a.point(), Point3::new(0.5, 0., 1.), b.point()],
    );
    let boundary: Wire = vec![
        builder::line(&lo[0], &a),
        Edge::new(&a, &b, curve.into()),
        builder::line(&b, &lo[1]),
        builder::line(&lo[1], &lo[0]),
    ]
    .into();
    let solid: Solid = builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::unit_y(),
    );
    let solid = builder::transformed(&solid, matrix);
    let (a, b) = (
        matrix.transform_point(a.point()),
        matrix.transform_point(a.point() + Vector3::unit_y()),
    );
    let mut seen = HashSet::new();
    let selected: Vec<_> = solid
        .edge_iter()
        .filter(|e| seen.insert(e.id()))
        .filter_map(|e| {
            if e.front().point().near(&a) && e.back().point().near(&b) {
                Some(e.clone())
            } else if e.front().point().near(&b) && e.back().point().near(&a) {
                Some(e.inverse())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(selected.len(), 1);
    (solid, selected[0].clone())
}

fn check(matrix: Matrix4) {
    let inverse = matrix.invert().unwrap();
    for curvature in [0.8, -0.4] {
        let (solid, selected) = fixture(matrix, curvature);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        common::assert_solid(&solid, 1. + curvature / 3., &[0], 1e-5);
        for (cap, wall) in [(0.3_f64, 0.2), (0.2, 0.3), (0.2, 0.2)] {
            // x^2 + (k*x^2)^2 = cap^2 on z=1+k*x^2; integrate the removed section exactly.
            let x = cap * (2. / (1. + (1. + 4. * curvature * curvature * cap * cap).sqrt())).sqrt();
            let expected = 1. + curvature / 3. - wall * x / 2. + curvature * x.powi(3) / 6.;
            for edge in [selected.clone(), selected.inverse()] {
                let mut distances = [0.; 2];
                let supports: Vec<_> = solid
                    .face_iter()
                    .filter_map(|face| {
                        let found = face.edge_iter().find(|e| e.id() == edge.id())?;
                        let wall_side = matches!(face.surface(), Surface::Plane(_));
                        distances[usize::from(found.front() != edge.front())] =
                            if wall_side { wall } else { cap };
                        Some((face.id(), wall_side))
                    })
                    .collect();
                assert_eq!(supports.len(), 2);
                let wire: Wire = vec![edge].into();
                let result = try_chamfer_solid_along_wire_chord(
                    &solid,
                    &wire,
                    distances[0],
                    distances[1],
                    0.001,
                )
                .unwrap();
                assert!(result.solid.is_geometric_consistent());
                assert_eq!(result.generated_faces.len(), 1);
                assert_eq!(
                    result.solid.face_iter().count(),
                    solid.face_iter().count() + 1
                );
                let mut count = 0;
                for (id, wall_side) in supports {
                    let support = &result
                        .modified_faces
                        .iter()
                        .find(|(old, _)| *old == id)
                        .unwrap()
                        .1;
                    for edge in result.generated_faces[0]
                        .edge_iter()
                        .filter(|edge| support.edge_iter().any(|q| q.id() == edge.id()))
                    {
                        let c = edge.oriented_curve();
                        let (a, b) = c.range_tuple();
                        for i in 0..=128 {
                            let p = inverse.transform_point(c.subs(a + (b - a) * i as f64 / 128.));
                            let d = if wall_side { wall } else { cap };
                            let guide = Point3::new(0., p.y, 1.);
                            assert!(
                                (p.distance(guide) - d).abs() < 1e-6,
                                "normal-section chord is {}, expected {d}",
                                p.distance(guide)
                            );
                            if wall_side {
                                assert!(p.x.abs() < 1e-6);
                                assert!((p.z - (1. - wall)).abs() < 1e-6);
                            } else {
                                assert!((p.x - x).abs() < 1e-6);
                                assert!((p.z - (1. + curvature * p.x * p.x)).abs() < 1e-6);
                            }
                            count += 1;
                        }
                    }
                }
                assert_eq!(count, 258);
                common::assert_solid(&result.solid, expected, &[0], 1e-6);
                common::blend::assert_step(&result.solid, expected, 2e-6);
                assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
            }
        }
    }
}
#[test]
fn normal_section_chamfers_preserve_chords_and_material() { check(Matrix4::identity()); }
#[test]
fn transformed_normal_section_chamfers_preserve_chords_and_material() {
    check(
        Matrix4::from_translation(Vector3::new(0.3, -0.2, 0.1))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7)),
    );
}
