use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};
use truck_stepio::{
    out::*,
    r#in::{step_geometry::*, Table},
};
use truck_topology::compress::*;

#[test]
fn curved_trim_extrema_and_extended_parabola_parameters_are_inside_imported_support() {
    let parabola = Curve3D::Conic(Conic3D::Parabola(Processor::new(TrimmedCurve::new(
        UnitParabola::new(),
        (-3., 2.),
    ))));
    let top = Curve3D::BSplineCurve(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(9., -6., 0.),
            Point3::new(-6., -1., 10.),
            Point3::new(4., 4., 0.),
        ],
    ));
    let surface = Surface::SweptCurve(SweptCurve::ExtrudedCurve(ExtrudedCurve::by_extrusion(
        parabola.clone(),
        Vector3::unit_z(),
    )));
    let shell = CompressedShell {
        vertices: vec![parabola.front(), parabola.back()],
        edges: vec![
            CompressedEdge {
                vertices: (0, 1),
                curve: parabola,
            },
            CompressedEdge {
                vertices: (0, 1),
                curve: top,
            },
        ],
        faces: vec![CompressedFace {
            surface,
            orientation: true,
            boundaries: vec![vec![
                CompressedEdgeIndex {
                    index: 0,
                    orientation: true,
                },
                CompressedEdgeIndex {
                    index: 1,
                    orientation: false,
                },
            ]],
        }],
    };
    let text = StepDisplay::new(
        Default::default(),
        StepDesign::from_model(StepModel::from(&shell)),
    )
    .to_string();
    let table = Table::from_step(&text).unwrap();
    let entity = table.shell.values().next().unwrap();
    let (shell, skipped) = table.to_compressed_shell(entity).unwrap();
    assert!(skipped.is_empty());
    let model = shell
        .try_mapped(
            |p| Some(*p),
            |c| ModelingCurve::try_from(c).ok(),
            |s| ModelingSurface::try_from(s).ok(),
        )
        .unwrap();
    let surface = &model.faces[0].surface;
    for edge in &model.edges {
        assert!(surface.include(&edge.curve));
        let (a, b) = edge.curve.range_tuple();
        for i in 0..=127 {
            let p = edge.curve.subs(a + (b - a) * i as f64 / 127.);
            let (u, v) = surface.search_nearest_parameter(p, None, 100).unwrap();
            assert!((-1e-9..=1. + 1e-9).contains(&u));
            assert!((-1e-9..=1. + 1e-9).contains(&v));
            assert!(surface.subs(u, v).distance(p) < 1e-8);
        }
    }
}

#[test]
fn cylinder_step_export_uses_radial_distance_for_axially_shifted_generators() {
    use ruststep::{ast::DataSection, tables::EntityTable};
    use std::str::FromStr;
    use truck_stepio::r#in::CylindricalSurfaceHolder;
    for height in [-37., 0., 17.] {
        let cylinder = Processor::new(RevolutedCurve::by_revolution(
            Line(
                Point3::new(2., 0., height),
                Point3::new(2., 0., height + 16.),
            ),
            Point3::origin(),
            Vector3::unit_z(),
        ));
        let surface = ElementarySurface::CylindricalSurface(cylinder);
        let data = format!("DATA;{}ENDSEC;", StepDataDisplay::new(surface, 1));
        let table = Table::from_data_section(&DataSection::from_str(&data).unwrap());
        let entity = EntityTable::<CylindricalSurfaceHolder>::get_owned(&table, 1).unwrap();
        let parsed: CylindricalSurface = (&entity).into();
        let p = parsed.entity().entity_curve().0;
        assert!(
            (p.x.hypot(p.y) - 2.).abs() < 1e-12,
            "height={height}, point={p:?}"
        );
        for i in 0..=31 {
            let u = i as f64 / 31.;
            let angle = 5. * u;
            assert!(
                parsed
                    .subs(angle, height + 16. * u)
                    .distance(cylinder.subs(u, angle))
                    < 1e-10
            );
        }
    }
}
