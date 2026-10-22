use truck_modeling::{Curve as ModelingCurve, Surface as ModelingSurface};
use truck_stepio::{
    out::*,
    r#in::{step_geometry::*, Table},
};
use truck_topology::compress::*;

#[test]
fn spline_pcurve_bounds_fit_complete_extrusion_trim_hulls() {
    for spline_trace in [false, true] {
        for rational in [false, true] {
            for reversed in [false, true] {
                for transformed in [false, true] {
                    let mut affine = Matrix4::from_nonuniform_scale(1.2, 0.8, 1.4);
                    affine.x.y = 0.2;
                    affine.z.x = 0.3;
                    let matrix = if transformed {
                        Matrix4::from_translation(Vector3::new(13., -7., 23.))
                            * Matrix4::from_axis_angle(
                                Vector3::new(1., 2., 3.).normalize(),
                                Rad(0.73),
                            )
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
                    let surface = Surface::SweptCurve(SweptCurve::ExtrudedCurve(
                        ExtrudedCurve::by_extrusion(
                            profile,
                            matrix.transform_vector(Vector3::unit_z()),
                        ),
                    ));
                    let trace = |offsets: [f64; 4]| {
                        let controls = [2., 3., 4., 5.]
                            .into_iter()
                            .zip(offsets)
                            .map(|(u, v)| Point2::new(u, v))
                            .collect();
                        let trace = if spline_trace {
                            Curve2D::BSplineCurve(BSplineCurve::new(
                                KnotVec::from(vec![2., 2., 2., 2., 5., 5., 5., 5.]),
                                controls,
                            ))
                        } else {
                            Curve2D::Line(Line(
                                Point2::new(2., offsets[0]),
                                Point2::new(5., offsets[3]),
                            ))
                        };
                        Curve3D::PCurve(PCurve::new(Box::new(trace), Box::new(surface.clone())))
                    };
                    let lower = trace([-5.; 4]);
                    let upper = trace([2., 12., 10., 3.]);
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
                                0 | 2 => {
                                    if spline_trace {
                                        t
                                    } else {
                                        2. + 3. * t
                                    }
                                }
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
}

#[test]
fn imported_c1_parameter_curve_preserves_its_exact_trace_and_face_domain() {
    let text = include_str!("fixtures/spline-extrusion-pcurves/C1-equal-006.step");
    let table = Table::from_step(text).unwrap();
    let (solid, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    let mut traces = 0;
    let mut faces = 0;
    for shell in &solid.boundaries {
        for edge in &shell.edges {
            let curve = ModelingCurve::try_from(&edge.curve)
                .expect("all C1 edge representations convert exactly");
            let Curve3D::PCurve(pcurve) = &edge.curve else {
                continue;
            };
            if !matches!(pcurve.curve().as_ref(), Curve2D::BSplineCurve(_)) {
                continue;
            }
            traces += 1;
            let (a, b) = edge.curve.range_tuple();
            for i in 0..=128 {
                let t = a + (b - a) * i as f64 / 128.;
                assert!(curve.subs(t).distance(edge.curve.subs(t)) < 1e-10);
                assert!(curve.der(t).distance(edge.curve.der(t)) < 1e-9);
                assert!(curve.der2(t).distance(edge.curve.der2(t)) < 1e-8);
            }
        }
        for face in &shell.faces {
            if !matches!(
                &face.surface,
                Surface::SweptCurve(SweptCurve::ExtrudedCurve(_))
            ) {
                continue;
            }
            let has_trace =
                face.boundaries
                    .iter()
                    .flatten()
                    .any(|e| match &shell.edges[e.index].curve {
                        Curve3D::PCurve(p) => {
                            matches!(p.curve().as_ref(), Curve2D::BSplineCurve(_))
                        }
                        _ => false,
                    });
            if !has_trace {
                continue;
            }
            faces += 1;
            let fitted = ModelingSurface::try_from(&face.surface).unwrap();
            let (u, v) = fitted.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            for index in face.boundaries.iter().flatten() {
                let curve = ModelingCurve::try_from(&shell.edges[index.index].curve).unwrap();
                let (t0, t1) = curve.range_tuple();
                for i in 0..=128 {
                    let point = curve.subs(t0 + (t1 - t0) * i as f64 / 128.);
                    let (u, v) = fitted.search_nearest_parameter(point, None, 100).unwrap();
                    assert!((a - 1e-9..=b + 1e-9).contains(&u));
                    assert!((c - 1e-9..=d + 1e-9).contains(&v));
                    assert!(fitted.subs(u, v).distance(point) < 1e-6);
                }
            }
        }
    }
    assert!(traces > 0 && faces > 0);
}
