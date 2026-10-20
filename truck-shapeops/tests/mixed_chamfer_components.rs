mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;
const TOL: f64 = 0.001;
fn body(segments: usize) -> Solid {
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
        Vector3::new(0., 0., 10.),
    )
}
fn check(top_axial: f64, bottom_axial: f64, reject: bool) {
    let top_radial = 0.5;
    let bottom_planar = 0.2;
    for segments in [1, 2, 4] {
        for transformed in [false, true] {
            let scale = if transformed { 0.2 } else { 1. };
            let matrix = if transformed {
                Matrix4::from_translation(Vector3::new(4., -3., 7.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
                    * Matrix4::from_scale(scale)
            } else {
                Matrix4::identity()
            };
            let inverse = matrix.invert().unwrap();
            let mut input = builder::transformed(&body(segments), matrix);
            if transformed {
                let mut faces: Vec<_> = input.boundaries()[0].iter().cloned().collect();
                faces.reverse();
                input = Solid::new(vec![faces.into()]);
            }
            let mut assignments = Vec::new();
            for i in 0..=segments {
                let (point, planar, axial) = if i == segments {
                    (Point3::new(2.5, 0., 0.), bottom_planar, bottom_axial)
                } else {
                    let angle = std::f64::consts::FRAC_PI_2 * (i as f64 + 0.5) / segments as f64;
                    (
                        Point3::new(5. * angle.cos(), 5. * angle.sin(), 10.),
                        top_radial,
                        top_axial,
                    )
                };
                let edge = common::blend::edge_through(
                    &input.boundaries()[0],
                    matrix.transform_point(point),
                );
                let first = input
                    .face_iter()
                    .find(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                    .unwrap();
                let cap_first = first
                    .oriented_surface()
                    .normal(0., 0.)
                    .normalize()
                    .dot(matrix.transform_vector(Vector3::unit_z()).normalize())
                    .abs()
                    > 0.9;
                assignments.push((
                    edge.id(),
                    if cap_first {
                        [planar * scale, axial * scale]
                    } else {
                        [axial * scale, planar * scale]
                    },
                ));
            }
            if transformed {
                assignments.reverse();
            }
            let before = serde_json::to_string(&input.compress()).unwrap();
            let result = try_chamfer_solid_edges_with_distances(&input, &assignments, TOL);
            if reject {
                assert!(result.is_err(), "overlapping components must fail");
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                continue;
            }
            let result = result.unwrap();
            let r: f64 = 5.;
            let a = bottom_planar;
            let straight = bottom_axial / a
                * (a / 2. * (a * (r * r - a * a).sqrt() + r * r * (a / r).asin())
                    - (r.powi(3) - (r * r - a * a).powf(1.5)) / 3.);
            let top = std::f64::consts::PI / 4.
                * top_axial
                * (r * top_radial - top_radial * top_radial / 3.);
            let volume = (std::f64::consts::PI / 4. * r * r * 10. - top - straight) * scale.powi(3);
            common::assert_solid(&result.solid, volume, &[0], TOL);
            common::blend::assert_step(&result.solid, volume, TOL);
            assert!(result.solid.is_geometric_consistent());
            let mut kinds = [0, 0];
            for face in &result.generated_faces {
                let points: Vec<_> = face
                    .vertex_iter()
                    .map(|v| inverse.transform_point(v.point()))
                    .collect();
                let is_top = points.iter().all(|p| {
                    let radius = (p.x * p.x + p.y * p.y).sqrt();
                    ((p.z - 10.).abs() < 1e-6 && (radius - (r - top_radial)).abs() < 1e-6)
                        || ((p.z - (10. - top_axial)).abs() < 1e-6 && (radius - r).abs() < 1e-6)
                });
                let is_bottom = points.iter().all(|p| {
                    (p.y.abs() < 1e-6 && (p.z - bottom_axial).abs() < 1e-6)
                        || (p.z.abs() < 1e-6 && (p.y - bottom_planar).abs() < 1e-6)
                });
                assert!(is_top || is_bottom, "unexpected bevel contacts {points:?}");
                kinds[0] += usize::from(is_top);
                kinds[1] += usize::from(is_bottom);
            }
            assert_eq!(kinds, [segments, 1]);
            for old in input.face_iter() {
                let id = result
                    .modified_faces
                    .iter()
                    .find(|(id, _)| *id == old.id())
                    .map_or(old.id(), |(_, f)| f.id());
                assert!(result.solid.face_iter().any(|f| f.id() == id));
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
#[test]
fn equal_mixed_chamfer_components() { check(0.5, 0.2, false); }
#[test]
fn unequal_mixed_chamfer_components() { check(0.3, 0.1, false); }
#[test]
fn angle_mixed_chamfer_components() { check(0.5 / 3_f64.sqrt(), 0.2 / 3_f64.sqrt(), false); }
#[test]
fn overlapping_mixed_components_preserve_input() { check(6., 6., true); }
