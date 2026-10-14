mod common;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_edges, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/spherical-pcurves/F1.step")).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(1e-5);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .expect("spherical splitting pcurve imports"),
    )
    .unwrap()
}

#[test]
fn hemisphere_box_union_import_preserves_closed_material_and_step() {
    let solid = input();
    assert!(solid.is_geometric_consistent());
    // Dimension-only hemisphere/box overlap integral, derived beside the raw fixture.
    common::assert_solid(&solid, 21467.43404299306, &[0], 0.001);
    common::blend::assert_step(&solid, 21467.43404299306, 0.001);
}

#[test]
fn spherical_end_face_fillet_preserves_material_radius_normals_and_history() {
    for (scale, moved) in [(1., false), (1., true), (0.1, true)] {
        let transform = if moved {
            Matrix4::from_translation(Vector3::new(2., -3., 5.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = transform.invert().unwrap();
        let solid = builder::transformed(&input(), transform);
        let edge = solid
            .edge_iter()
            .find(|e| {
                [e.front(), e.back()].into_iter().all(|v| {
                    let p = inverse.transform_point(v.point());
                    (p.x - 10.).abs() < 1e-6 && (p.y + 10.).abs() < 1e-6
                })
            })
            .unwrap();
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let result = try_fillet_solid_edges(&solid, &[edge.id()], 2. * scale, 0.001).unwrap();
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), 1);
        let old: Vec<_> = solid.face_iter().collect();
        let new: Vec<_> = result.solid.face_iter().collect();
        let expected: Vec<_> = old
            .iter()
            .zip(&new)
            .filter(|(a, b)| a.id() != b.id())
            .map(|(a, b)| (a.id(), b.id()))
            .collect();
        assert_eq!(
            result
                .modified_faces
                .iter()
                .map(|(a, b)| (*a, b.id()))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(new.len(), old.len() + 1);
        assert_eq!(result.generated_faces[0].id(), new.last().unwrap().id());
        let surface = result.generated_faces[0].oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for i in 0..=8 {
            for j in 0..=8 {
                let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 8.);
                let p = inverse.transform_point(surface.subs(u, v));
                let radial = Vector3::new(p.x - 8., p.y + 8., 0.);
                assert!((radial.magnitude() - 2.).abs() * scale < 1e-6);
                let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                assert!(normal.dot(radial.normalize()) > 1. - 1e-6);
            }
        }
        let volume = 21454.331120055493 * scale.powi(3);
        common::assert_solid(&result.solid, volume, &[0], 0.001);
        common::blend::assert_step(&result.solid, volume, 0.001);
    }
}
