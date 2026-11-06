use ruststep::{ast::DataSection, tables::EntityTable};
use std::f64::consts::{PI, TAU};
use truck_stepio::r#in::{step_geometry::*, EdgeCurveHolder, Table};

fn point(t: f64) -> Point3 { Point3::new(7. - 2. * t.sin(), -3. + 4. * t.cos(), 5.) }

fn table(a: f64, b: f64, sense: bool, edge_sense: bool, inner: bool) -> Table {
    let t = Table::from_data_section(
        &source(a, b, sense, edge_sense, inner)
            .parse::<DataSection>()
            .unwrap(),
    );
    assert!(t.errors.is_empty(), "{:?}", t.errors);
    t
}

fn source(a: f64, b: f64, sense: bool, edge_sense: bool, inner: bool) -> String {
    let (p, q) = if edge_sense {
        (point(a), point(b))
    } else {
        (point(b), point(a))
    };
    let sense = if sense { ".T." } else { ".F." };
    let edge_sense = if edge_sense { ".T." } else { ".F." };
    let geometry = if inner { 12 } else { 5 };
    let text = format!(
        "DATA;
#1=CARTESIAN_POINT('',(7.,-3.,5.));
#2=DIRECTION('',(0.,0.,1.)); #3=DIRECTION('',(0.,1.,0.));
#4=AXIS2_PLACEMENT_3D('',#1,#2,#3);
#6=ELLIPSE('',#4,4.,2.);
#5=TRIMMED_CURVE('',#6,(PARAMETER_VALUE({a})),(PARAMETER_VALUE({b})),{sense},.PARAMETER.);
#7=CARTESIAN_POINT('',({},{},{})); #8=CARTESIAN_POINT('',({},{},{}));
#9=VERTEX_POINT('',#7); #10=VERTEX_POINT('',#8);
#11=EDGE_CURVE('',#9,#10,#{geometry},{edge_sense});
#12=SURFACE_CURVE('',#5,(#13),.CURVE_3D.); #13=PLANE('',#4); ENDSEC;",
        p.x, p.y, p.z, q.x, q.y, q.z
    );
    text
}

#[test]
fn explicit_trim_preserves_branch_transform_and_both_senses() {
    for (a, b) in [
        (0.2, 2.4),
        (6.1, 0.3),
        (TAU - 1e-7, 1e-7),
        (-0.2, 0.3),
        (TAU + 0.2, TAU + 2.4),
        (0.2, 0.2 + TAU),
        (0.7, 0.7 + 1e-7),
    ] {
        for sense in [true, false] {
            for edge_sense in [true, false] {
                for inner in [true, false] {
                    let t = table(a, b, sense, edge_sense, inner);
                    let edge = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11).unwrap();
                    let curve = edge.parse_curve3d().unwrap();
                    let (lo, hi) = curve.range_tuple();
                    let (expected_lo, end) = if sense { (a, b) } else { (b, a) };
                    let expected_hi = if end - expected_lo == -TAU {
                        expected_lo + TAU
                    } else if end < expected_lo {
                        end + TAU
                    } else {
                        end
                    };
                    assert!((lo - expected_lo).abs() < 1e-12 && (hi - expected_hi).abs() < 1e-12);
                    for i in 0..=32 {
                        let u = lo + (hi - lo) * i as f64 / 32.;
                        let angle = if sense == edge_sense { u } else { lo + hi - u };
                        assert!(curve.subs(u).distance(point(angle)) < 1e-10);
                    }
                }
            }
        }
    }
}

#[test]
fn same_endpoints_select_complementary_arcs() {
    let curves = [true, false].map(|sense| {
        let t = table(0., PI / 2., sense, true, true);
        EntityTable::<EdgeCurveHolder>::get_owned(&t, 11)
            .unwrap()
            .parse_curve3d()
            .unwrap()
    });
    assert!(curves[0].front().near(&curves[1].front()));
    assert!(curves[0].back().near(&curves[1].back()));
    for (curve, span) in curves.iter().zip([PI / 2., 3. * PI / 2.]) {
        let (a, b) = curve.range_tuple();
        assert!((b - a - span).abs() < 1e-12);
    }
    let mid = |c: &Curve3D| {
        let (a, b) = c.range_tuple();
        c.subs((a + b) / 2.)
    };
    assert!(mid(&curves[0]).distance(mid(&curves[1])) > 4.);
}

#[test]
fn explicit_domain_rejects_other_branch_vertices_without_mutation() {
    let t = table(0.2, 1.4, true, true, true);
    let mut edge = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11).unwrap();
    edge.edge_end.vertex_geometry.coordinates = vec![point(3.).x, point(3.).y, 5.];
    let before = edge.clone();
    assert!(edge.parse_curve3d().is_err());
    assert_eq!(edge, before);
}

#[test]
fn malformed_unsupported_and_cyclic_trims_are_errors() {
    for entity in [
        "TRIMMED_CURVE('',#6,(),(PARAMETER_VALUE(1.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#6,(PARAMETER_VALUE(0.),PARAMETER_VALUE(2.)),(PARAMETER_VALUE(1.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#6,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(1.)),.T.,.CARTESIAN.)",
        "TRIMMED_CURVE('',#6,(#1),(#1),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#6,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(0.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#6,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(7.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#5,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(1.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#12,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(1.)),.T.,.PARAMETER.)",
        "TRIMMED_CURVE('',#99,(PARAMETER_VALUE(0.)),(PARAMETER_VALUE(1.)),.T.,.PARAMETER.)",
    ] {
        let mut text=source(0.,1.,true,true,true);
        let start=text.find("#5=").unwrap();
        let end=start+text[start..].find(';').unwrap();
        text.replace_range(start..end,&format!("#5={entity}"));
        let Ok(data)=text.parse::<DataSection>() else { assert!(entity.contains(",(),"), "unexpected malformed test record: {entity}"); continue; };
        let t=Table::from_data_section(&data);
        let before=t.clone();
        let result=EntityTable::<EdgeCurveHolder>::get_owned(&t,11).map_err(|e|e.to_string()).and_then(|e|e.parse_curve3d().map_err(|e|e.to_string()));
        assert!(result.is_err(),"{entity}"); assert_eq!(t,before);
    }
}

#[test]
fn two_dimensional_trims_use_radians_and_preserve_orientation() {
    for sense in [true, false] {
        for edge_sense in [true, false] {
            let text = source(6.1, 0.3, sense, edge_sense, false).replace(
                "AXIS2_PLACEMENT_3D('',#1,#2,#3)",
                "AXIS2_PLACEMENT_2D('',#1,#3)",
            );
            let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
            let curve = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11)
                .unwrap()
                .parse_curve2d()
                .unwrap();
            let (a, b) = curve.range_tuple();
            for i in 0..=32 {
                let u = a + (b - a) * i as f64 / 32.;
                let p = point(if sense == edge_sense { u } else { a + b - u });
                assert!(curve.subs(u).distance(Point2::new(p.x, p.y)) < 1e-10);
            }
        }
    }
}

#[test]
fn invalid_axes_and_nonfinite_parameters_are_rejected() {
    for (from, to) in [
        ("4.,2.", "0.,2."),
        ("4.,2.", "-4.,2."),
        ("PARAMETER_VALUE(1)", "PARAMETER_VALUE(1.E999)"),
    ] {
        let text = source(0., 1., true, true, true).replace(from, to);
        assert!(text.contains(to));
        let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
        let result = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11)
            .map_err(|e| e.to_string())
            .and_then(|e| e.parse_curve3d().map_err(|e| e.to_string()));
        assert!(result.is_err());
    }
}

#[test]
fn guarded_writer_graphs_keep_shared_edges_and_bounded_supports() {
    for source in [
        include_str!("fixtures/guarded-g4-trimmed.step"),
        include_str!("fixtures/guarded-g8-trimmed.step"),
    ] {
        let table = Table::from_step(source).unwrap();
        let before = table.clone();
        assert!(!table.pcurve.is_empty());
        for shell in table.shell.values() {
            let (shell, skipped) = table.to_compressed_shell(shell).unwrap();
            assert!(skipped.is_empty(), "{skipped:?}");
            assert_eq!(shell.edges.len(), table.edge_curve.len());
            assert_eq!(shell.vertices.len(), table.vertex_point.len());
            assert_eq!(shell.faces.len(), table.face_surface.len());
            let mut uses = vec![0; shell.edges.len()];
            for face in &shell.faces {
                let (u, v) = face.surface.try_range_tuple();
                for (a, b) in [u.unwrap(), v.unwrap()] {
                    assert!(a.is_finite() && b.is_finite() && a < b);
                }
                for boundary in &face.boundaries {
                    for edge in boundary {
                        uses[edge.index] += 1;
                    }
                }
            }
            assert!(uses.iter().all(|&n| n == 2));
            for edge in &shell.edges {
                let (a, b) = edge.curve.range_tuple();
                assert!(a.is_finite() && b.is_finite() && a < b);
                assert!(edge.curve.front().near(&shell.vertices[edge.vertices.0]));
                assert!(edge.curve.back().near(&shell.vertices[edge.vertices.1]));
            }
        }
        assert_eq!(table, before);
    }
}

#[test]
fn circle_trims_retain_radius_and_complementary_arcs() {
    for sense in [true, false] {
        let text =
            source(0., PI, sense, true, true).replace("ELLIPSE('',#4,4.,2.)", "CIRCLE('',#4,4.)");
        let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
        let curve = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11)
            .unwrap()
            .parse_curve3d()
            .unwrap();
        let (a, b) = curve.range_tuple();
        let middle = curve.subs((a + b) / 2.);
        assert!(middle.near(&Point3::new(if sense { 3. } else { 11. }, -3., 5.)));
    }
}

#[test]
fn nonperiodic_trim_bases_are_explicitly_unsupported() {
    let text =
        source(0., 1., true, true, true).replace("ELLIPSE('',#4,4.,2.)", "HYPERBOLA('',#4,4.,2.)");
    let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
    let result = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11)
        .map_err(|e| e.to_string())
        .and_then(|e| e.parse_curve3d().map_err(|e| e.to_string()));
    assert!(result.is_err());
}

#[test]
fn owned_trimmed_edge_serde_roundtrip_preserves_parameters() {
    let table = table(6.1, 0.3, false, false, true);
    let edge = EntityTable::<EdgeCurveHolder>::get_owned(&table, 11).unwrap();
    let json = serde_json::to_string(&edge).unwrap();
    let restored: truck_stepio::r#in::EdgeCurve = serde_json::from_str(&json).unwrap();
    assert_eq!(edge, restored);
}

#[test]
fn cyclic_trim_basis_reports_the_trim_entity_in_shell_diagnostics() {
    let text=source(0.,1.,true,true,true).replace("TRIMMED_CURVE('',#6", "TRIMMED_CURVE('',#5")
        .replace("ENDSEC;", "#20=EDGE_LOOP('',(#11)); #21=FACE_OUTER_BOUND('',#20,.T.); #22=ADVANCED_FACE('',(#21),#13,.T.); #23=OPEN_SHELL('',(#22)); ENDSEC;");
    let table = Table::from_data_section(&text.parse::<DataSection>().unwrap());
    let before = table.clone();
    let (_, skipped) = table
        .to_compressed_shell(table.shell.values().next().unwrap())
        .unwrap();
    let error = skipped
        .iter()
        .find(|s| s.id == 11)
        .unwrap()
        .reason
        .to_string();
    assert!(
        error.contains("TRIMMED_CURVE") && error.contains("#5"),
        "{error}"
    );
    assert_eq!(table, before);
}

#[test]
fn shifted_full_turns_preserve_both_orientations() {
    for a in [2.2, 100.] {
        for sense in [true, false] {
            for edge_sense in [true, false] {
                let t = table(a, a + TAU, sense, edge_sense, true);
                let edge = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11).unwrap();
                let curve = edge.parse_curve3d().unwrap();
                let (lo, hi) = curve.range_tuple();
                assert!((hi - lo - TAU).abs() < 1e-12);
                assert!(curve.front().distance(point(a)) < 1e-10);
                assert!(curve.back().distance(point(a)) < 1e-10);
                let quarter = curve.subs(lo + (hi - lo) / 4.);
                let direction = if sense == edge_sense { 1. } else { -1. };
                assert!(quarter.distance(point(a + direction * PI / 2.)) < 1e-10);
            }
        }
        let excessive = f64::from_bits((a + TAU).to_bits() + 1);
        let t = table(a, excessive, true, true, false);
        let trim = EntityTable::<truck_stepio::r#in::TrimmedConicHolder>::get_owned(&t, 5).unwrap();
        assert!(Curve3D::try_from(&trim).is_err());
    }
}

#[test]
fn large_phase_direct_and_edge_conversions_reject_immutably() {
    for b in [9999999999999998., 10000000000000002.] {
        for sense in [true, false] {
            let t = table(1e16, b, sense, true, false);
            let trim =
                EntityTable::<truck_stepio::r#in::TrimmedConicHolder>::get_owned(&t, 5).unwrap();
            let before = trim.clone();
            assert!(Curve3D::try_from(&trim).is_err());
            assert_eq!(trim, before);
            let edge = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11).unwrap();
            let before = edge.clone();
            assert!(edge.parse_curve3d().is_err());
            assert_eq!(edge, before);
        }
    }
}

#[test]
fn large_phase_direct_2d_conversion_has_a_valid_placement() {
    for sense in [true, false] {
        for (a, b, accepted) in [
            (0.2, 1.4, true),
            (1e16, 9999999999999998., false),
            (1e16, 10000000000000002., false),
        ] {
            let text = source(a, b, sense, true, false).replace(
                "AXIS2_PLACEMENT_3D('',#1,#2,#3)",
                "AXIS2_PLACEMENT_2D('',#1,#3)",
            );
            let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
            let trim =
                EntityTable::<truck_stepio::r#in::TrimmedConicHolder>::get_owned(&t, 5).unwrap();
            let before = trim.clone();
            let result = Curve2D::try_from(&trim);
            assert_eq!(result.is_ok(), accepted, "phase {a} to {b}, sense {sense}");
            if let Ok(curve) = result {
                let p = point(a);
                let q = point(b);
                assert!(curve.front().distance(Point2::new(p.x, p.y)) < 1e-10);
                assert!(curve.back().distance(Point2::new(q.x, q.y)) < 1e-10);
            }
            assert_eq!(trim, before);
        }
    }
}

#[test]
fn overflowing_placements_reject_before_direct_evaluation() {
    for dim in [2, 3] {
        for scale in ["1.E200", "1.E308"] {
            let mut text = source(0., 1., true, true, false)
                .replace("(7.,-3.,5.)", "(1.E308,0.,0.)")
                .replace("ELLIPSE('',#4,4.,2.)", &format!("CIRCLE('',#4,{scale})"));
            if dim == 2 {
                text = text.replace(
                    "AXIS2_PLACEMENT_3D('',#1,#2,#3)",
                    "AXIS2_PLACEMENT_2D('',#1,#3)",
                );
            }
            let t = Table::from_data_section(&text.parse::<DataSection>().unwrap());
            let trim =
                EntityTable::<truck_stepio::r#in::TrimmedConicHolder>::get_owned(&t, 5).unwrap();
            if dim == 2 {
                assert!(Curve2D::try_from(&trim).is_err());
            } else {
                assert!(Curve3D::try_from(&trim).is_err());
            }
        }
    }
}

#[test]
fn shifted_interior_vertex_trims_preserve_reversed_arcs() {
    for (sense, start, end) in [(true, 100.4, 101.8), (false, 105., 103.)] {
        for edge_sense in [true, false] {
            let t = table(100.2, 102.2, sense, edge_sense, true);
            let mut edge = EntityTable::<EdgeCurveHolder>::get_owned(&t, 11).unwrap();
            let (a, b) = if edge_sense {
                (start, end)
            } else {
                (end, start)
            };
            let p = point(a);
            let q = point(b);
            edge.edge_start.vertex_geometry.coordinates = vec![p.x, p.y, p.z];
            edge.edge_end.vertex_geometry.coordinates = vec![q.x, q.y, q.z];
            let curve = edge.parse_curve3d().unwrap();
            let (lo, hi) = curve.range_tuple();
            assert!(curve.front().distance(p) < 1e-10);
            assert!(curve.back().distance(q) < 1e-10);
            for i in 0..=16 {
                let f = i as f64 / 16.;
                assert!(
                    curve
                        .subs(lo + (hi - lo) * f)
                        .distance(point(a + (b - a) * f))
                        < 1e-9
                );
            }
        }
    }
}
