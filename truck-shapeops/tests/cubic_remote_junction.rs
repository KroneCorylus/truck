mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn body(reversed: bool) -> (Solid, [Point3; 3]) { body_variant(reversed, 2.) }

fn body_variant(reversed: bool, shear: f64) -> (Solid, [Point3; 3]) {
    let controls = vec![
        Point3::new(0., 0., 0.),
        Point3::new(2., 10., 0.),
        Point3::new(4., 6., 0.),
        Point3::new(10., 0., 0.),
    ];
    body_controls(reversed, shear, controls)
}

fn body_controls(reversed: bool, shear: f64, controls: Vec<Point3>) -> (Solid, [Point3; 3]) {
    let front = Vertex::new(controls[0]);
    let back = Vertex::new(controls[3]);
    let curve = BSplineCurve::new(KnotVec::bezier_knot(3), controls);
    let curved_midpoint = curve.subs(0.5) + Vector3::new(0., shear, 10.);
    let edge = Edge::new(&front, &back, Curve::BSplineCurve(curve));
    let mut face = builder::try_attach_plane(&[wire![edge, builder::line(&back, &front)]]).unwrap();
    face.invert();
    let input: Solid = builder::tsweep(&face, Vector3::new(0., shear, 10.));
    for face in input.face_iter() {
        if let Surface::BSplineSurface(surface) = face.surface() {
            let controls = surface.control_points();
            assert_eq!(controls.len(), 4);
            assert!(controls.iter().all(|row| row.len() == 2));
            let vector = controls[0][1] - controls[0][0];
            let curve = BSplineCurve::new(
                surface.uknot_vec().clone(),
                controls.iter().map(|row| row[0]).collect(),
            );
            face.set_surface(ExtrudedCurve::by_extrusion(curve.into(), vector).into());
        }
    }
    if reversed {
        for face in input.face_iter() {
            let Surface::Extruded(extrusion) = face.oriented_surface() else {
                continue;
            };
            let a = extrusion.extruding_vector();
            let mut curve = extrusion.entity_curve().clone();
            curve.transform_by(Matrix4::from_translation(a));
            curve.invert();
            let mut surface: Surface = ExtrudedCurve::by_extrusion(curve, -a).into();
            if !face.orientation() {
                surface.invert();
            }
            face.set_surface(surface);
        }
    }
    (
        input,
        [
            curved_midpoint,
            Point3::new(10., shear / 2., 5.),
            Point3::new(5., shear, 10.),
        ],
    )
}

#[test]
fn native_h1_cubic_remote_junction() {
    let (input, selection) = body(false);
    common::assert_solid(&input, 372., &[0], 0.001);
    let selected: Vec<_> = selection
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_fillet_solid_edges(&input, &selected, 0.5, 1e-5);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    let result = result.expect("H1 bounded wall-first cubic construction");
    assert!(result.solid.is_geometric_consistent());
    let supports: Vec<_> = result
        .solid
        .face_iter()
        .map(|f| serde_json::to_string(&f.oriented_surface()).unwrap())
        .collect();
    for face in input.face_iter() {
        assert!(
            supports.contains(&serde_json::to_string(&face.oriented_surface()).unwrap()),
            "original support replaced"
        );
    }

    common::assert_solid(&result.solid, 367.45551955681, &[0], 0.001);
    common::blend::assert_step(&result.solid, 367.45551955681, 0.001);
}

fn frame(t: f64, r: f64) -> (Point3, Vector3, Vector3, f64) {
    let b = Point3::new(
        6. * t + 4. * t.powi(3),
        30. * t - 42. * t * t + 12. * t.powi(3),
        0.,
    );
    let derivative = Vector3::new(6. + 12. * t * t, 30. - 84. * t + 36. * t * t, 0.);
    let a = Vector3::new(0., 2., 10.);
    let normal = a.cross(derivative).normalize();
    let center = b + a * ((10. - r + r * normal.z) / 10.) - r * normal;
    (
        center,
        normal,
        Vector3::new(normal.x, normal.y, 0.).normalize(),
        normal.z.acos(),
    )
}

fn root(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    assert!(f(lo) * f(hi) <= 0.);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.;
        if f(mid) * f(lo) > 0. {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.
}

fn check_material(solid: &Solid, r: f64, matrix: Matrix4) {
    use truck_meshalgo::prelude::*;
    let scale = matrix.transform_vector(Vector3::unit_z()).magnitude();
    let inverse = matrix.invert().unwrap();
    let mesh = solid.triangulation(0.0002 * scale).to_polygon();
    let check = |center: Point3, normal: Vector3| {
        for offset in [-0.003, 0.003] {
            let p = matrix.transform_point(center + (r + offset) * normal);
            assert_eq!(mesh.inside(p), offset < 0., "material witness {p:?}");
        }
    };
    let m = Vector3::new(0., -10., 2.).normalize();
    let ts = root(|t| m.dot(frame(t, r).0.to_vec()) + r, 0.8, 1.);
    let (center, normal, _, _) = frame(ts, r);
    for i in 1..5 {
        for j in 1..5 {
            check(
                center,
                (Vector3::unit_z() * i as f64 + m * j as f64 + normal * 3.).normalize(),
            );
        }
    }
    for t in [0.1, 0.3, 0.5, 0.7, 0.85] {
        let (center, _, lateral, alpha) = frame(t, r);
        for q in [0.2, 0.5, 0.8] {
            check(
                center,
                Vector3::unit_z() * (q * alpha).cos() + lateral * (q * alpha).sin(),
            );
        }
    }
    let gamma = normal.dot(m).acos();
    let tangent = (m - normal * gamma.cos()) / gamma.sin();
    for q in [0.2, 0.5, 0.8] {
        let normal = normal * (q * gamma).cos() + tangent * (q * gamma).sin();
        for height in [1., 5., 8.] {
            let c = center + Vector3::new(0., 2., 10.) * ((height - center.z - r * normal.z) / 10.);
            check(c, normal);
        }
        let p = center + r * normal;
        let bottom = p - Vector3::new(0., 2., 10.) * (p.z / 10.);
        let inside = bottom - 0.003 * normal;
        assert!(mesh.inside(matrix.transform_point(inside + Vector3::unit_z() * 0.003)));
        assert!(!mesh.inside(matrix.transform_point(inside - Vector3::unit_z() * 0.003)));
    }
    let beta = m.z.acos();
    let w = Vector3::new(0., -1., 0.);
    let line = (-r - (10. - r) * m.z) / m.dot(w);
    let edges: Vec<_> = solid.edge_iter().collect();
    let assert_shared = |p: Point3, label: &str| {
        let edge = edges
            .iter()
            .find(|edge| {
                let curve = edge.curve();
                let (lo, hi) = curve.range_tuple();
                curve
                    .search_nearest_parameter(p, None, 100)
                    .is_some_and(|t| curve.subs(t.clamp(lo, hi)).distance(p) < 1e-6 * scale)
            })
            .unwrap_or_else(|| panic!("missing {label} seam {p:?}"));
        let uses: Vec<_> = solid
            .face_iter()
            .flat_map(|face| {
                face.boundaries()
                    .iter()
                    .flat_map(|wire| wire.iter())
                    .filter(|e| e.id() == edge.id())
                    .map(|e| (face.id(), e.orientation()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(uses.len(), 2, "{label}: two incident faces");
        assert_ne!(uses[0].0, uses[1].0, "{label}: distinct incident faces");
        assert_ne!(uses[0].1, uses[1].1, "{label}: opposite edge uses");
    };
    for i in 0..=32 {
        let phi = beta * i as f64 / 32.;
        let t = root(
            |t| {
                let (c, _, v, _) = frame(t, r);
                w.dot(c.to_vec()) - line + r * phi.sin() * (w.dot(v) - 1.)
            },
            0.,
            0.0625,
        );
        let (c, _, v, _) = frame(t, r);
        let p = matrix.transform_point(c + r * (Vector3::unit_z() * phi.cos() + v * phi.sin()));
        assert_shared(p, "P-Q");
    }
    for i in 0..=32 {
        let phi = beta + (frame(0., r).3 - beta) * i as f64 / 32.;
        let t = if i == 32 {
            0.
        } else {
            root(
                |t| {
                    let (c, _, v, _) = frame(t, r);
                    m.dot((c + r * (Vector3::unit_z() * phi.cos() + v * phi.sin())).to_vec())
                },
                0.,
                0.0625,
            )
        };
        let (c, _, v, _) = frame(t, r);
        let p = matrix.transform_point(c + r * (Vector3::unit_z() * phi.cos() + v * phi.sin()));
        assert_shared(p, "Q-R");
    }
    // Evaluate fitted canal interiors, including either side of the inflection
    // and stations approaching the right sphere. These are sampled witnesses.
    for t in [0.85, 0.865, 0.88, ts - 0.01, ts - 0.001] {
        let (c, _, v, alpha) = frame(t, r);
        for q in [0.15, 0.5, 0.85] {
            let expected = matrix.transform_point(
                c + r * (Vector3::unit_z() * (q * alpha).cos() + v * (q * alpha).sin()),
            );
            let fitted = solid
                .face_iter()
                .filter_map(|face| {
                    let surface = face.surface();
                    if !matches!(
                        surface,
                        Surface::Fillet(_) | Surface::NurbsSurface(_) | Surface::BSplineSurface(_)
                    ) {
                        return None;
                    }
                    let uv = surface.search_nearest_parameter(expected, None, 100)?;
                    let (ur, vr) = surface.try_range_tuple();
                    if !ur.is_some_and(|(lo, hi)| lo < uv.0 && uv.0 < hi)
                        || !vr.is_some_and(|(lo, hi)| lo < uv.1 && uv.1 < hi)
                    {
                        return None;
                    }
                    let p = surface.subs(uv.0, uv.1);
                    (p.distance(expected) < 1e-6 * scale).then_some(p)
                })
                .next()
                .expect("actual fitted canal interior");
            let p = inverse.transform_point(fitted);
            assert!(p.x > 0. && p.x < 10. && p.z > 0. && p.z < 10.);
            let u = root(|u| 6. * u + 4. * u.powi(3) - p.x, 0., 1.);
            let y = p.y - 0.2 * p.z;
            assert!(
                y >= -1e-6 && y <= 30. * u - 42. * u * u + 12. * u.powi(3) + 1e-6,
                "fitted interior source containment {p:?}"
            );
        }
    }
    // The inflection makes tangent halfspaces invalid. Invert monotone x(t).
    for edge in solid.edge_iter() {
        let curve = edge.curve();
        let (lo, hi) = curve.range_tuple();
        for i in 0..=64 {
            let p = inverse.transform_point(curve.subs(lo + (hi - lo) * i as f64 / 64.));
            assert!(p.x >= -1e-6 && p.x <= 10. + 1e-6 && p.z >= -1e-6 && p.z <= 10. + 1e-6);
            let t = root(|t| 6. * t + 4. * t * t * t - p.x.clamp(0., 10.), 0., 1.);
            let y = p.y - 0.2 * p.z;
            assert!(
                y >= -1e-6 && y <= 30. * t - 42. * t * t + 12. * t * t * t + 1e-6,
                "source containment {p:?}"
            );
        }
    }
}

#[test]
fn cubic_radius_similarity_reversal_and_material() {
    for (radius, volume) in [
        (0.3, 370.337679523565),
        (0.5, 367.45551955681),
        (0.6, 365.50665287438),
    ] {
        for reversed in [false, true] {
            let (original, selection) = body(reversed);
            let scale = if reversed { 0.4 } else { 2.5 };
            let matrix = Matrix4::from_translation(Vector3::new(7., -11., 13.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale);
            let input = builder::transformed(&original, matrix);
            let mut selected: Vec<_> = selection
                .map(|p| {
                    common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p))
                        .id()
                })
                .into();
            let before = serde_json::to_string(&input.compress()).unwrap();
            for _ in 0..2 {
                let result =
                    try_fillet_solid_edges(&input, &selected, radius * scale, 1e-5 * scale);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                let result = result.expect("bounded cubic family");
                common::assert_solid(&result.solid, volume * scale.powi(3), &[0], 0.001 * scale);
                common::blend::assert_step(&result.solid, volume * scale.powi(3), 0.001 * scale);
                check_material(&result.solid, radius, matrix);
                selected.reverse();
            }
        }
    }
}

#[test]
fn cubic_admission_boundary() {
    // The global divided-difference clearance bound is 1.010217370438509;
    // the admitted side must still pass every construction/material check.
    for reversed in [false, true] {
        let (input, selection) = body(reversed);
        let selected: Vec<_> = selection
            .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
            .into();
        let before = serde_json::to_string(&input.compress()).unwrap();
        for radius in [-1., 0., 1., 1.011, 2.] {
            let result = try_fillet_solid_edges(&input, &selected, radius, 1e-5);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            if radius == 1. {
                let result = result.expect("inside conservative cubic certificate");
                common::assert_solid(&result.solid, 354.49009959089, &[0], 0.001);
                common::blend::assert_step(&result.solid, 354.49009959089, 0.001);
                check_material(&result.solid, radius, Matrix4::identity());
            } else {
                assert!(result.is_err());
            }
        }
    }
}

#[test]
fn cubic_profile_guard_controls() {
    for (name, x1, x2, y1, accepted) in [
        ("perturbed accepted control", 2., 4., 9.9, true),
        ("nonmonotone control polygon", 2., 1., 10., false),
        ("offset fold", 0.02, 0.04, 10., false),
    ] {
        let controls = vec![
            Point3::origin(),
            Point3::new(x1, y1, 0.),
            Point3::new(x2, 6., 0.),
            Point3::new(10., 0., 0.),
        ];
        let chord = controls[3] - controls[0];
        let monotone = controls.windows(2).all(|p| (p[1] - p[0]).dot(chord) > 0.);
        assert_eq!(monotone, name != "nonmonotone control polygon");
        let curve = BSplineCurve::new(KnotVec::bezier_knot(3), controls.clone());
        let axis = Vector3::new(0., 2., 10.).normalize();
        let project = |v: Vector3| v - axis * v.dot(axis);
        let max_r_curvature = (0..=4096)
            .map(|i| {
                let t = i as f64 / 4096.;
                let d = project(curve.der(t));
                0.5 * d.cross(project(curve.der2(t))).magnitude() / d.magnitude().powi(3)
            })
            .fold(0_f64, f64::max);
        assert_eq!(max_r_curvature > 1., name == "offset fold");
        let (input, selection) = body_controls(false, 2., controls);
        let selected: Vec<_> = selection
            .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
            .into();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_edges(&input, &selected, 0.5, 1e-5);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        if let Ok(result) = result {
            assert!(result.solid.is_geometric_consistent());
        }
    }
    // A nonplanar basis is malformed for this planar-cap extrusion contract.
    let (input, selection) = body(false);
    let selected: Vec<_> = selection
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .into();
    for face in input.face_iter() {
        if let Surface::Extruded(extrusion) = face.surface() {
            let mut curve = extrusion.entity_curve().clone();
            if let Curve::BSplineCurve(basis) = &mut curve {
                basis.control_point_mut(1).z += 0.1;
            }
            face.set_surface(
                ExtrudedCurve::by_extrusion(curve, extrusion.extruding_vector()).into(),
            );
        }
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges(&input, &selected, 0.5, 1e-5).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
