mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;
const TOL: f64 = 0.001;
fn body(segments: usize, height: f64) -> Solid {
    let start = builder::vertex(Point3::new(5., 0., 0.));
    let arc: Wire = builder::rsweep(
        &start,
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::FRAC_PI_2),
        segments,
    );
    let center = builder::vertex(Point3::origin());
    let mut boundary = arc.clone();
    boundary.extend([
        builder::line(arc.back_vertex().unwrap(), &center),
        builder::line(&center, arc.front_vertex().unwrap()),
    ]);
    builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::new(0., 0., height),
    )
}

fn check(full: bool, reject: bool) {
    for segments in if full { vec![2] } else { vec![1, 2, 4] } {
        for transformed in [false, true] {
            let height = if reject { 0.8 } else { 10. };
            let radius: f64 = 0.5;
            let input = if full {
                common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 5., height)
            } else {
                body(segments, height)
            };
            let scale = if transformed { 0.2 } else { 1. };
            let matrix = if transformed {
                Matrix4::from_translation(Vector3::new(4., -3., 7.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
                    * Matrix4::from_scale(scale)
            } else {
                Matrix4::identity()
            };
            let inverse = matrix.invert().unwrap();
            let mut input = builder::transformed(&input, matrix);
            if transformed {
                let mut faces: Vec<_> = input.boundaries()[0].iter().cloned().collect();
                faces.reverse();
                input = Solid::new(vec![faces.into()]);
            }
            let mut seen = rustc_hash::FxHashSet::default();
            let mut selected: Vec<_> = input
                .edge_iter()
                .filter(|e| {
                    let pts = [
                        inverse.transform_point(e.front().point()),
                        inverse.transform_point(e.back().point()),
                    ];
                    pts.iter()
                        .all(|p| ((p.x * p.x + p.y * p.y).sqrt() - 5.).abs() < 1e-6)
                        && ((pts[0].z - pts[1].z).abs() < 1e-6)
                        && seen.insert(e.id())
                })
                .map(|e| e.id())
                .collect();
            if transformed {
                selected.reverse();
            }
            let before = serde_json::to_string(&input.compress()).unwrap();
            let result = try_fillet_solid_edges(&input, &selected, radius * scale, TOL);
            if reject {
                assert!(result.is_err());
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                continue;
            }
            let result = result.unwrap();
            let theta = if full {
                std::f64::consts::TAU
            } else {
                std::f64::consts::FRAC_PI_2
            };
            let removed = theta
                * (5. * radius.powi(2) * (1. - std::f64::consts::PI / 4.)
                    - radius.powi(3) * (5. / 6. - std::f64::consts::PI / 4.));
            let expected = (theta / 2. * 25. * height - 2. * removed) * scale.powi(3);
            common::assert_solid(&result.solid, expected, &[0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), selected.len());
            for face in &result.generated_faces {
                for vertex in face.vertex_iter() {
                    let p = inverse.transform_point(vertex.point());
                    let radial = (p.x * p.x + p.y * p.y).sqrt();
                    assert!(
                        ((p.z.abs() < 1e-6 || (p.z - height).abs() < 1e-6)
                            && (radial - (5. - radius)).abs() < 1e-6)
                            || (((p.z - radius).abs() < 1e-6
                                || (p.z - (height - radius)).abs() < 1e-6)
                                && (radial - 5.).abs() < 1e-6)
                    );
                }
            }
            for old in input.face_iter() {
                let id = result
                    .modified_faces
                    .iter()
                    .find(|(id, _)| *id == old.id())
                    .map_or(old.id(), |(_, face)| face.id());
                assert!(result.solid.face_iter().any(|f| f.id() == id));
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
#[test]
fn closed_fillet_rims_share_the_original_wall() { check(true, false); }
#[test]
fn open_fillet_rims_share_the_original_walls() { check(false, false); }
#[test]
fn overlapping_fillet_rims_preserve_input() {
    check(true, true);
    check(false, true);
}
