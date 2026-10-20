mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edge;
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
    let solid = builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::new(0., 0., 10.),
    );
    for face in solid.face_iter() {
        if matches!(face.surface(), Surface::Extruded(_)) {
            let generator = Curve::Conic(Processor::with_transform(
                TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., std::f64::consts::TAU)),
                Matrix4::from_translation(Vector3::new(0., 0., -1e-6))
                    * Matrix4::from_angle_z(Rad(0.02))
                    * Matrix4::from_scale(5.),
            ));
            let mut replacement: Surface =
                ExtrudedCurve::by_extrusion(generator, Vector3::unit_z() * (10. + 2e-6)).into();
            if replacement.elementary().unwrap().1 != face.surface().elementary().unwrap().1 {
                replacement.invert();
            }
            face.set_surface(replacement);
        }
    }
    solid
}

#[test]
fn chamfers_cross_the_conic_parameter_seam() {
    for transformed in [false, true] {
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(4., -3., 7.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
        } else {
            Matrix4::identity()
        };
        let input = builder::transformed(&body(1), matrix);
        assert!(input.is_geometric_consistent());
        let before = serde_json::to_string(&input.compress()).unwrap();
        let edge = common::blend::edge_through(
            &input.boundaries()[0],
            matrix.transform_point(Point3::new(2.5, 0., 0.)),
        );
        let result = try_chamfer_solid_edge(&input, edge.id(), 0.2, 0.2, TOL).unwrap();
        let r: f64 = 5.;
        let a: f64 = 0.2;
        let bottom_removed = a / 2. * (a * (r * r - a * a).sqrt() + r * r * (a / r).asin())
            - (r.powi(3) - (r * r - a * a).powf(1.5)) / 3.;
        let volume = std::f64::consts::PI / 4. * r * r * 10. - bottom_removed;
        common::assert_solid(&result.solid, volume, &[0], TOL);
        common::blend::assert_step(&result.solid, volume, TOL);
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), 1);
        let inverse = matrix.invert().unwrap();
        for vertex in result.generated_faces[0].vertex_iter() {
            let p = inverse.transform_point(vertex.point());
            assert!(
                (p.y.abs() < 1e-6 && (p.z - a).abs() < 1e-6)
                    || (p.z.abs() < 1e-6 && (p.y - a).abs() < 1e-6)
            );
        }
        for old in input.face_iter() {
            let id = result
                .modified_faces
                .iter()
                .find(|(id, _)| *id == old.id())
                .map_or(old.id(), |(_, face)| face.id());
            assert!(result.solid.face_iter().any(|face| face.id() == id));
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
