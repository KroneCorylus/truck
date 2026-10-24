use truck_stepio::r#in::{ruststep::tables::IntoOwned, step_geometry::*, Table};

#[test]
fn declared_spline_master_preserves_position_derivatives_and_domain() {
    let original = include_str!("fixtures/spline-extrusion-pcurves/C1-equal-006.step");
    for second_master in [false, true] {
        let text = if second_master {
            original.replace("(#2224,#2337),.PCURVE_S1.", "(#2337,#2224),.PCURVE_S2.")
        } else {
            original.to_owned()
        };
        assert!(!second_master || text != original);
        let table = Table::from_step(&text).unwrap();
        let source = table.surface_curve[&2036]
            .clone()
            .into_owned(&table)
            .unwrap();
        let master = table.pcurve[&2224].clone().into_owned(&table).unwrap();
        let expected = PCurve::try_from(&master).unwrap();
        let converted = Curve3D::try_from(&source).unwrap();
        assert!(matches!(converted, Curve3D::PCurve(_)));
        assert_eq!(converted.range_tuple(), expected.range_tuple());
        let (a, b) = expected.range_tuple();
        for i in 0..=128 {
            let t = a + (b - a) * i as f64 / 128.;
            assert!(converted.subs(t).distance(expected.subs(t)) < 1e-10);
            assert!(converted.der(t).distance(expected.der(t)) < 1e-9);
            assert!(converted.der2(t).distance(expected.der2(t)) < 1e-8);
        }
    }
}
