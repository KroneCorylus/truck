mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::*;

fn fixture(matrix: Matrix4) -> (Solid, Edge) {
    let a = Vertex::new(Point3::new(0., 0., 1.1));
    let b = Vertex::new(Point3::new(0., 1., 1.1));
    let lower = [
        Vertex::new(Point3::origin()),
        Vertex::new(Point3::new(0., 1., 0.)),
    ];
    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![a.point(), Point3::new(0., 0.5, 0.9), b.point()],
    );
    let guide = Edge::new(&a, &b, curve.into());
    let boundary: Wire = vec![
        builder::line(&lower[0], &a),
        guide,
        builder::line(&b, &lower[1]),
        builder::line(&lower[1], &lower[0]),
    ]
    .into();
    let solid: Solid = builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap().inverse(),
        Vector3::unit_x(),
    );
    let solid = builder::transformed(&solid, matrix);
    let (a, b) = (
        matrix.transform_point(a.point()),
        matrix.transform_point(b.point()),
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
    let (solid, selected) = fixture(matrix);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    common::assert_solid(&solid, 1. + 0.4 / 12., &[0], 1e-5);
    let inverse = matrix.invert().unwrap();
    // Integrate the ruled section of z=1+0.4*(y-0.5)^2, clipped by y=0 and y=1.
    for (cap, wall, expected) in [
        (0.1, 0.1, 1.0282092032510828),
        (0.1, 0.06, 1.030257419115786),
        (0.06, 0.1, 1.030258855283983),
    ] {
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
            let result =
                try_chamfer_solid_along_wire(&solid, &wire, distances[0], distances[1], 0.001)
                    .expect("parabolic contacts meet the original terminal planes");
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), 1);
            assert_eq!(
                result.solid.face_iter().count(),
                solid.face_iter().count() + 1
            );
            let mut count = 0;
            for (id, wall_side) in &supports {
                let support = &result
                    .modified_faces
                    .iter()
                    .find(|(old, _)| old == id)
                    .unwrap()
                    .1;
                for edge in result.generated_faces[0]
                    .edge_iter()
                    .filter(|edge| support.edge_iter().any(|q| q.id() == edge.id()))
                {
                    let curve = edge.oriented_curve();
                    let (a, b) = curve.range_tuple();
                    for i in 0..=128 {
                        let p = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 128.));
                        if *wall_side {
                            assert!(p.x.abs() < 1e-6);
                            let mut t = p.y;
                            for _ in 0..20 {
                                let f = 1. + 0.4 * (t - 0.5).powi(2);
                                let d = 0.8 * (t - 0.5);
                                t -= ((t - p.y) + (f - p.z) * d) / (1. + d * d + (f - p.z) * 0.8);
                            }
                            let q = Point3::new(0., t, 1. + 0.4 * (t - 0.5).powi(2));
                            assert!((p.distance(q) - wall).abs() < 1e-6);
                        } else {
                            assert!((p.x - cap).abs() < 1e-6);
                            assert!((p.z - (1. + 0.4 * (p.y - 0.5).powi(2))).abs() < 1e-6);
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

#[test]
fn parabolic_cap_chamfers_preserve_clipped_contacts_and_material() { check(Matrix4::identity()); }
#[test]
fn transformed_parabolic_cap_chamfers_preserve_clipped_contacts_and_material() {
    check(
        Matrix4::from_translation(Vector3::new(0.3, -0.2, 0.1))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7)),
    );
}
