mod common;
use truck_modeling::*;
use truck_stepio::r#in::step_geometry::{Conic3D, Curve3D};

#[test]
fn imported_parabolic_and_hyperbolic_profiles_make_exact_prisms() {
    let parabolic = Curve3D::Conic(Conic3D::Parabola(Processor::new(TrimmedCurve::new(
        UnitParabola::new(),
        (-1., 1.),
    ))));
    let hyperbolic = Curve3D::Conic(Conic3D::Hyperbola(Processor::new(TrimmedCurve::new(
        UnitHyperbola::new(),
        (-1., 1.),
    ))));
    for (source, area) in [
        (parabolic, 8. / 3.),
        (hyperbolic, 1f64.sinh() * 1f64.cosh() - 1.),
    ] {
        for transformed in [false, true] {
            let curve = Curve::try_from(&source).unwrap().inverse();
            let a = builder::vertex(curve.front());
            let b = builder::vertex(curve.back());
            let boundary = Wire::from(vec![Edge::new(&a, &b, curve), builder::line(&b, &a)]);
            let face = builder::try_attach_plane(&[boundary]).unwrap();
            let mut solid: Solid = builder::tsweep(&face, Vector3::new(0.2, -0.3, 3.));
            if transformed {
                let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
                solid = builder::transformed(&solid, transform);
            }
            assert!(solid.is_geometric_consistent());
            common::assert_solid(&solid, area * 3., &[0], 0.001);
            let prepared = truck_stepio::out::prepare_for_step(&solid.compress(), 0.0001).unwrap();
            let design = truck_stepio::out::StepDesign::from_model(
                truck_stepio::out::StepModel::from(&prepared),
            );
            let text = truck_stepio::out::StepDisplay::new(Default::default(), design).to_string();
            let table = truck_stepio::r#in::Table::from_step(&text).unwrap();
            let entity = table.manifold_solid_brep.values().next().unwrap();
            let (compressed, skipped) = table.to_compressed_solid(entity).unwrap();
            assert!(skipped.is_empty());
            let read = truck_topology::Solid::extract(compressed).unwrap();
            common::assert_solid(&read, area * 3., &[0], 0.001);
        }
    }
}
