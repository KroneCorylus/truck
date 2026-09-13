use truck_modeling::*;
use truck_stepio::{out::*, r#in::Table};

#[test]
fn affine_conic_arcs_keep_their_locus_and_direction_in_step() {
    for reflected in [false, true] {
        let transform = Matrix4::from_cols(
            Vector4::new(5., 0., 2., 0.),
            Vector4::new(1., if reflected { -3. } else { 3. }, -1., 0.),
            Vector4::new(0., 0., 1., 0.),
            Vector4::new(17., -21., 32., 1.),
        );
        let curve = Curve::Conic(Processor::with_transform(
            TrimmedCurve::new(UnitCircle::new(), (0.2, 2.8)),
            transform,
        ));
        let edge = Edge::new(
            &builder::vertex(curve.front()),
            &builder::vertex(curve.back()),
            curve.clone(),
        );
        let chord = builder::line(edge.back(), edge.front());
        let face = builder::try_attach_plane(&[Wire::from(vec![edge, chord])]).unwrap();
        let shell = Shell::from(vec![face]);
        let compressed = shell.compress();
        let design = StepDesign::from_model(StepModel::from(&compressed));
        let text = StepDisplay::new(Default::default(), design).to_string();
        assert!(text.contains("ELLIPSE("));
        let table = Table::from_step(&text).unwrap();
        let source = table.shell.values().next().unwrap();
        let (imported, skipped) = table.to_compressed_shell(source).unwrap();
        assert!(skipped.is_empty());
        let converted = imported
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap();
        let read = Shell::extract(converted).unwrap();
        assert!(read.is_geometric_consistent());
        let edge = read
            .edge_iter()
            .find(|e| {
                e.front().point().near(&curve.front()) && e.back().point().near(&curve.back())
            })
            .unwrap();
        let imported = edge.oriented_curve();
        for i in 0..=32 {
            let t = 0.2 + 2.6 * i as f64 / 32.;
            let p = curve.subs(t);
            let u = imported
                .search_parameter(p, None, 100)
                .expect("exported ellipse changed the original locus");
            assert!(imported.subs(u).distance(p) < 1e-8);
            assert!(imported.der(u).dot(curve.der(t)) > 0.);
        }
    }
}
