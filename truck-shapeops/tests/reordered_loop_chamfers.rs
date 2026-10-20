mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_along_wire;
const TOL: f64 = 0.001;
fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire {
    let v = builder::vertices([
        Point3::new(x0, y0, 0.),
        Point3::new(x1, y0, 0.),
        Point3::new(x1, y1, 0.),
        Point3::new(x0, y1, 0.),
    ]);
    (0..4)
        .map(|i| builder::line(&v[i], &v[(i + 1) % 4]))
        .collect()
}
fn check(inner: bool) {
    let face = builder::try_attach_plane(&[
        rectangle(0., 0., 10., 10.),
        rectangle(2., 2., 4., 4.).inverse(),
        rectangle(6., 6., 8., 8.).inverse(),
    ])
    .unwrap();
    let base = builder::tsweep(&face, Vector3::new(0., 0., 5.));
    for order in [[0, 1, 2], [1, 0, 2], [1, 2, 0], [2, 1, 0]] {
        for transformed in [false, true] {
            let scale = if transformed { 0.2 } else { 1. };
            let matrix = if transformed {
                Matrix4::from_translation(Vector3::new(4., -3., 7.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
                    * Matrix4::from_scale(scale)
            } else {
                Matrix4::identity()
            };
            let faces = base.boundaries()[0]
                .iter()
                .map(|f| {
                    let b = f.boundaries();
                    if b.len() == 3 {
                        Face::new(order.map(|i| b[i].clone()).to_vec(), f.oriented_surface())
                    } else {
                        f.clone()
                    }
                })
                .collect::<Shell>();
            let input = builder::transformed(&Solid::new(vec![faces]), matrix);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let p = if inner {
                Point3::new(3., 2., 5.)
            } else {
                Point3::new(5., 0., 5.)
            };
            let e = common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p));
            let result = try_chamfer_solid_along_wire(
                &input,
                &Wire::from(vec![e]),
                0.2 * scale,
                0.2 * scale,
                TOL,
            )
            .unwrap();
            let expected = (460. - if inner { 0.04 } else { 0.2 }) * scale.powi(3);
            common::assert_solid(&result.solid, expected, &[2], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            for old in input.face_iter() {
                let id = result
                    .modified_faces
                    .iter()
                    .find(|(id, _)| *id == old.id())
                    .map_or(old.id(), |(_, f)| f.id());
                assert!(result.solid.face_iter().any(|f| f.id() == id));
            }
        }
    }
}
#[test]
fn outer_rim_chamfer_preserves_reordered_holes() { check(false); }
#[test]
fn inner_rim_chamfer_preserves_reordered_holes() { check(true); }
