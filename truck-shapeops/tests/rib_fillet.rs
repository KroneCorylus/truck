mod common;

use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

const TOL: f64 = 0.001;

fn ribs() -> Solid {
    let mut solid = common::modeling::cuboid(Point3::origin(), Point3::new(2., 28., 10.));
    for i in 0..7 {
        let y = 1. + 4. * i as f64;
        let rib = common::modeling::cuboid(Point3::new(-4., y, 0.), Point3::new(0., y + 2., 25.));
        solid = truck_shapeops::try_or(&solid, &rib, TOL).unwrap();
    }
    solid
}

#[test]
fn reinforces_stepped_ribs_including_touching_rounds() {
    let input = ribs();
    let before = serde_json::to_string(&input.compress()).unwrap();
    common::assert_solid(&input, 1960., &[0], TOL);
    let edges: Vec<_> = (0..7)
        .flat_map(|i| [1. + 4. * i as f64, 3. + 4. * i as f64])
        .map(|y| common::blend::edge_through(&input.boundaries()[0], Point3::new(0., y, 5.)).id())
        .collect();
    for count in [1, 14] {
        for radius in [0.1, 0.5, 1.] {
            let result = try_fillet_solid_edges(&input, &edges[..count], radius, TOL).unwrap();
            let expected = 1960. + count as f64 * common::fillet_removed_volume(radius, 10.);
            assert!(result.solid.is_geometric_consistent());
            common::assert_solid(&result.solid, expected, &[0], TOL);
            assert_eq!(result.generated_faces.len(), count);
            for face in result.solid.face_iter() {
                let modified = result
                    .modified_faces
                    .iter()
                    .filter(|(_, f)| f.id() == face.id())
                    .count();
                let generated = result
                    .generated_faces
                    .iter()
                    .filter(|f| f.id() == face.id())
                    .count();
                assert_eq!(modified + generated, 1);
            }
            if radius == 1. {
                common::blend::assert_step(&result.solid, expected, TOL);
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

#[test]
fn rib_fillets_preserve_placement_selection_order_and_radius_limits() {
    let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    let input = builder::transformed(&ribs(), transform);
    let mut edges: Vec<_> = (0..7)
        .flat_map(|i| [1. + 4. * i as f64, 3. + 4. * i as f64])
        .map(|y| {
            common::blend::edge_through(
                &input.boundaries()[0],
                transform.transform_point(Point3::new(0., y, 5.)),
            )
            .id()
        })
        .collect();
    let result = try_fillet_solid_edges(&input, &edges, 1., TOL).unwrap();
    common::assert_solid(
        &result.solid,
        1960. + 14. * common::fillet_removed_volume(1., 10.),
        &[0],
        TOL,
    );
    edges.reverse();
    let reversed = try_fillet_solid_edges(&input, &edges, 1., TOL).unwrap();
    assert_eq!(
        serde_json::to_string(&result.solid.compress()).unwrap(),
        serde_json::to_string(&reversed.solid.compress()).unwrap()
    );
    assert!(try_fillet_solid_edges(&input, &edges, 1.1, TOL).is_err());
    assert!(try_fillet_solid_edges(&input, &edges[..1], 3., TOL).is_err());
}
