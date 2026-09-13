use truck_geometry::prelude::*;

#[test]
fn reversed_curve_derivative_tables_follow_the_chain_rule() {
    let range = (0.2, 2.7);
    let transform = Matrix4::from_translation(Vector3::new(4., -7., 2.))
        * Matrix4::from_nonuniform_scale(3., 2., 1.);
    let original = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), range),
        transform,
    );
    let reversed = original.inverse();
    for t in [0.2, 0.7, 1.4, 2.7] {
        assert_near!(reversed.der_n(0, t), reversed.subs(t).to_vec());
        assert_near!(reversed.der_n(1, t), reversed.der(t));
        assert_near!(reversed.der_n(2, t), reversed.der2(t));
        let ders = reversed.ders(5, t);
        for n in 1..=5 {
            let expected = original.der_n(n, range.0 + range.1 - t)
                * if n.is_multiple_of(2) { 1. } else { -1. };
            assert_near!(reversed.der_n(n, t), expected);
            assert_near!(ders[n], expected);
        }
    }
    let arc2 = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point2>::new(), range),
        Matrix3::from_nonuniform_scale(2., 0.7),
    );
    for n in 1..=5 {
        assert_near!(
            arc2.inverse().der_n(n, 0.7),
            arc2.der_n(n, 2.2) * if n.is_multiple_of(2) { 1. } else { -1. }
        );
    }
}
