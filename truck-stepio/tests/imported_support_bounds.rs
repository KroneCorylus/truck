use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};
use truck_stepio::{
    out::*,
    r#in::{step_geometry::*, Table},
};
use truck_topology::compress::*;

#[test]
fn spline_sweep_bounds_include_complete_polynomial_and_rational_trim_hulls() {
    for rational in [false, true] {
        for reversed in [false, true] {
            for transformed in [false, true] {
                let mut affine = Matrix4::from_nonuniform_scale(1.2, 0.8, 1.4);
                affine.x.y = 0.2;
                affine.z.x = 0.3;
                let matrix = if transformed {
                    Matrix4::from_translation(Vector3::new(13., -7., 23.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                        * affine
                } else {
                    Matrix4::identity()
                };
                let points = [
                    Point3::new(-2., 0., -4.),
                    Point3::new(-1., 1., 7.),
                    Point3::new(2., 1., -2.),
                    Point3::new(3., 0., 3.),
                ];
                let weights = [1., 0.7, 1.3, 1.];
                let curve = |offsets: [f64; 4]| {
                    let knots = KnotVec::from(vec![2., 2., 2., 2., 5., 5., 5., 5.]);
                    let controls: Vec<_> = points
                        .iter()
                        .zip(offsets)
                        .map(|(p, v)| matrix.transform_point(*p + Vector3::unit_z() * v))
                        .collect();
                    let mut curve = if rational {
                        Curve3D::NurbsCurve(NurbsCurve::new(BSplineCurve::new(
                            knots,
                            controls
                                .into_iter()
                                .zip(weights)
                                .map(|(p, w)| p.to_homogeneous() * w)
                                .collect(),
                        )))
                    } else {
                        Curve3D::BSplineCurve(BSplineCurve::new(knots, controls))
                    };
                    if reversed {
                        curve.invert();
                    }
                    curve
                };
                let profile = curve([0.; 4]);
                let lower = curve([-5.; 4]);
                let upper = curve([2., 12., 10., 3.]);
                let surface =
                    Surface::SweptCurve(SweptCurve::ExtrudedCurve(ExtrudedCurve::by_extrusion(
                        profile,
                        matrix.transform_vector(Vector3::unit_z()),
                    )));
                let vertices = vec![lower.front(), lower.back(), upper.back(), upper.front()];
                let edges = vec![
                    CompressedEdge {
                        vertices: (0, 1),
                        curve: lower,
                    },
                    CompressedEdge {
                        vertices: (1, 2),
                        curve: Curve3D::Line(Line(vertices[1], vertices[2])),
                    },
                    CompressedEdge {
                        vertices: (3, 2),
                        curve: upper,
                    },
                    CompressedEdge {
                        vertices: (3, 0),
                        curve: Curve3D::Line(Line(vertices[3], vertices[0])),
                    },
                ];
                let shell = CompressedShell {
                    vertices,
                    edges,
                    faces: vec![CompressedFace {
                        surface: surface.clone(),
                        orientation: true,
                        boundaries: vec![vec![
                            CompressedEdgeIndex {
                                index: 0,
                                orientation: true,
                            },
                            CompressedEdgeIndex {
                                index: 1,
                                orientation: true,
                            },
                            CompressedEdgeIndex {
                                index: 2,
                                orientation: false,
                            },
                            CompressedEdgeIndex {
                                index: 3,
                                orientation: true,
                            },
                        ]],
                    }],
                };
                let step = StepDisplay::new(
                    Default::default(),
                    StepDesign::from_model(StepModel::from(&shell)),
                )
                .to_string();
                let table = Table::from_step(&step).unwrap();
                let (read, skipped) = table
                    .to_compressed_shell(table.shell.values().next().unwrap())
                    .unwrap();
                assert!(skipped.is_empty());
                let model = read
                    .try_mapped(
                        |p| Some(*p),
                        |c| ModelingCurve::try_from(c).ok(),
                        |s| ModelingSurface::try_from(s).ok(),
                    )
                    .unwrap();
                let face = &model.faces[0];
                let fitted = if face.orientation {
                    face.surface.clone()
                } else {
                    face.surface.inverse()
                };
                let (u, v) = fitted.try_range_tuple();
                let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                for (index, edge) in model.edges.iter().enumerate() {
                    assert!(fitted.include(&edge.curve),"rational={rational}, reversed={reversed}, transformed={transformed}, edge={index}");
                    let (t0, t1) = edge.curve.range_tuple();
                    for i in 0..=128 {
                        let t = (t0 + (t1 - t0) * i as f64 / 128.).clamp(t0, t1);
                        let p = edge.curve.subs(t);
                        let (u, v) = fitted.search_nearest_parameter(p, None, 100).unwrap();
                        assert!((a - 1e-9..=b + 1e-9).contains(&u), "u={u}");
                        assert!((c - 1e-9..=d + 1e-9).contains(&v), "v={v}");
                        assert!(fitted.subs(u, v).distance(p) < 1e-7);
                        let original_u = match index {
                            0 | 2 => t,
                            1 => 5.,
                            _ => 2.,
                        };
                        assert!(
                            fitted.normal(u, v).dot(surface.normal(original_u, 0.)) > 1. - 1e-8
                        );
                    }
                }
            }
        }
    }
}

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
