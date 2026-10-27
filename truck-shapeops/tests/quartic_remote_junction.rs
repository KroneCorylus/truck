mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn body(reversed: bool) -> (Solid, [Point3; 3]) { body_variant(reversed, 25., 2.) }

fn body_variant(reversed: bool, control_x: f64, shear: f64) -> (Solid, [Point3; 3]) {
    let controls = vec![
        Point3::new(0., 0., 0.),
        Point3::new(20., 0., 0.),
        Point3::new(20., 5., 0.),
        Point3::new(control_x, 10., 0.),
        Point3::new(10., 20., 0.),
    ];
    let front = Vertex::new(controls[0]);
    let back = Vertex::new(controls[4]);
    let curve = BSplineCurve::new(KnotVec::bezier_knot(4), controls);
    let curved_midpoint = curve.subs(0.5) + Vector3::new(0., shear, 10.);
    let edge = Edge::new(&front, &back, Curve::BSplineCurve(curve));
    let face = builder::try_attach_plane(&[wire![edge, builder::line(&back, &front)]]).unwrap();
    let input: Solid = builder::tsweep(&face, Vector3::new(0., shear, 10.));
    for face in input.face_iter() {
        if let Surface::BSplineSurface(surface) = face.surface() {
            let controls = surface.control_points();
            assert_eq!(controls.len(), 5);
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
            Point3::new(0., shear / 2., 5.),
            Point3::new(5., 10. + shear, 10.),
        ],
    )
}

#[test]
fn native_g6_quartic_remote_junction() { check(1., 2280.3799002674264, false, false); }

#[test]
fn quartic_remote_radius_transform_and_reversal() {
    check(0.5, 2295.0350363292923, true, false);
    check(1.2, 2271.8836341628294, true, true);
}

fn check(radius: f64, volume: f64, moved: bool, reversed: bool) {
    let (wall_station, peak_station, witnesses) = remote_witness(radius);
    assert!(wall_station > 1. && peak_station > wall_station);
    let (original, selection) = body(reversed);
    // Green's theorem on the exact quartic gives area 230; prism height is 10.
    common::assert_solid(&original, 2300., &[0], 0.001);
    let factor = if moved { 2.5 } else { 1. };
    let matrix = if moved {
        Matrix4::from_translation(Vector3::new(7., -11., 13.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(factor)
    } else {
        Matrix4::identity()
    };
    let inverse = matrix.invert().unwrap();
    let input = builder::transformed(&original, matrix);
    let mut selected: Vec<_> = selection
        .map(|p| {
            common::blend::edge_through(&input.boundaries()[0], matrix.transform_point(p)).id()
        })
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for _ in 0..2 {
        let result = try_fillet_solid_edges(&input, &selected, radius * factor, 1e-5 * factor);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        let result = result.expect("G6 wall-first construction");
        assert!(result.solid.is_geometric_consistent());
        // Independent flux integration: audits/fillet-chamfer/oracles/g6_material.py.
        common::assert_solid(&result.solid, volume * factor.powi(3), &[0], 0.001 * factor);
        common::blend::assert_step(&result.solid, volume * factor.powi(3), 0.001 * factor);
        let edges: Vec<_> = result.solid.edge_iter().map(|e| e.curve()).collect();
        for p in &witnesses {
            let p = matrix.transform_point(*p);
            assert!(
                edges.iter().any(|curve| {
                    let (lo, hi) = curve.range_tuple();
                    curve
                        .search_nearest_parameter(p, None, 100)
                        .is_some_and(|t| curve.subs(t.clamp(lo, hi)).distance(p) < 1e-6 * factor)
                }),
                "missing independent seam/continuation witness {p:?}"
            );
        }
        let wall = Vector3::new(-10., 5., -1.).normalize();
        for curve in &edges {
            let (lo, hi) = curve.range_tuple();
            for i in 0..=64 {
                let p = inverse.transform_point(curve.subs(lo + (hi - lo) * i as f64 / 64.));
                assert!(wall.dot(p.to_vec()) < 1e-6, "outside wall: {p:?}");
                assert!(p.z >= -1e-6 && p.z <= 10. + 1e-6);
                for j in 0..=128 {
                    let (c, lateral, alpha) = frame(j as f64 / 128., radius);
                    let normal = Vector3::unit_z() * alpha.cos() + lateral * alpha.sin();
                    assert!(
                        normal.dot(p - c - radius * normal) < 1e-6,
                        "outside profile: {p:?}"
                    );
                }
            }
        }
        assert_material_sections(&result.solid, radius, 25., 2., matrix);
        for face in result.generated_faces {
            let Surface::Fillet(fit) = face.surface() else {
                continue;
            };
            let af = fit.entity();
            let (side, plane) = [af.surface0(), af.surface1()]
                .into_iter()
                .enumerate()
                .find_map(|(i, s)| match s.as_ref() {
                    Surface::Plane(p) => Some((i, p)),
                    _ => None,
                })
                .unwrap();
            let normal = fit.transform().transform_vector(plane.normal()).normalize();
            let (lo, hi) = af.range_tuple().1;
            for j in 0..=16 {
                let v = lo + (hi - lo) * j as f64 / 16.;
                let center = fit.transform().transform_point(af.subs(side as f64, v))
                    - radius * factor * normal;
                for i in 0..=16 {
                    let p = fit.transform().transform_point(af.subs(i as f64 / 16., v));
                    assert!((p.distance(center) / factor - radius).abs() < 1e-5);
                }
            }
        }
        selected.reverse();
    }
}

#[test]
fn quartic_remote_rejects_invalid_and_folded_offsets_without_mutation() {
    let (input, selection) = body(false);
    let selected: Vec<_> = selection
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [-1., 0., 5., 10.] {
        assert!(try_fillet_solid_edges(&input, &selected, radius, 1e-5).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

fn frame(t: f64, radius: f64) -> (Point3, Vector3, f64) {
    let x = 80. * t - 120. * t.powi(2) + 100. * t.powi(3) - 50. * t.powi(4);
    let y = 30. * t.powi(2) - 20. * t.powi(3) + 10. * t.powi(4);
    let tangent = Vector3::new(
        80. - 240. * t + 300. * t.powi(2) - 200. * t.powi(3),
        60. * t - 60. * t.powi(2) + 40. * t.powi(3),
        0.,
    );
    let a = Vector3::new(0., 2., 10.);
    let normal = tangent.cross(a).normalize();
    let v = (10. - radius + radius * normal.z) / 10.;
    (
        Point3::new(x, y, 0.) + v * a - radius * normal,
        Vector3::new(normal.x, normal.y, 0.).normalize(),
        normal.z.acos(),
    )
}

fn root(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    assert!(f(lo) * f(hi) < 0.);
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

// Polynomial source and rolling-ball equations; no native-kernel output values.
fn remote_witness(radius: f64) -> (f64, f64, Vec<Point3>) {
    let m = Vector3::new(-10., 5., -1.).normalize();
    let w = Vector3::new(-2., 1., 0.).normalize();
    let beta = m.z.acos();
    let line = (-radius - (10. - radius) * m.z) / m.dot(w);
    let station = |phi: f64| {
        root(
            |t| {
                let (c, lateral, _) = frame(t, radius);
                w.dot(c.to_vec()) - line + radius * phi.sin() * (w.dot(lateral) - 1.)
            },
            0.5,
            1.1,
        )
    };
    let mut points = Vec::new();
    for i in 0..=32 {
        let phi = beta * i as f64 / 32.;
        let (c, lateral, alpha) = frame(station(phi), radius);
        assert!(alpha > beta);
        let p = c + radius * (Vector3::unit_z() * phi.cos() + lateral * phi.sin());
        assert!(m.dot(p.to_vec()) < 1e-12);
        assert!(((p.z - (10. - radius)).hypot(w.dot(p.to_vec()) - line) - radius).abs() < 1e-12);
        points.push(p);
    }
    let alpha = frame(1., radius).2;
    for i in 0..32 {
        let phi = beta + (alpha - beta) * i as f64 / 32.;
        let t = root(
            |t| {
                let (c, lateral, _) = frame(t, radius);
                m.dot((c + radius * (Vector3::unit_z() * phi.cos() + lateral * phi.sin())).to_vec())
            },
            0.999,
            1.001,
        );
        let (c, lateral, _) = frame(t, radius);
        points.push(c + radius * (Vector3::unit_z() * phi.cos() + lateral * phi.sin()));
    }
    (station(beta), station(std::f64::consts::FRAC_PI_2), points)
}

#[test]
fn imported_quartic_remote_wall_chart_and_step() {
    use truck_shapeops::SplitClosedEdgesAndFaces;
    use truck_stepio::r#in::Table;
    let table = Table::from_step(include_str!("fixtures/conic-import/quartic-prism.step")).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(1e-5);
    let input = Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap();
    let (_, witnesses) = body(false);
    let selected: Vec<_> = witnesses
        .map(|p| {
            common::blend::edge_through(&input.boundaries()[0], Point3::from_vec(p.to_vec() * 10.))
                .id()
        })
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_fillet_solid_edges(&input, &selected, 10., 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert!(result.solid.is_geometric_consistent());
    common::assert_solid(&result.solid, 2280379.9002674264, &[0], 0.001);
    common::blend::assert_step(&result.solid, 2280379.9002674264, 0.001);
}

fn independent_frame(t: f64, radius: f64, control_x: f64, shear: f64) -> (Point3, Vector3) {
    let delta = control_x - 25.;
    let b = Point3::new(
        80. * t - 120. * t * t + 100. * t.powi(3) - 50. * t.powi(4)
            + 4. * delta * t.powi(3) * (1. - t),
        30. * t * t - 20. * t.powi(3) + 10. * t.powi(4),
        0.,
    );
    let derivative = Vector3::new(
        80. - 240. * t + 300. * t * t - 200. * t.powi(3)
            + 4. * delta * (3. * t * t - 4. * t.powi(3)),
        60. * t - 60. * t * t + 40. * t.powi(3),
        0.,
    );
    let a = Vector3::new(0., shear, 10.);
    let normal = derivative.cross(a).normalize();
    (
        b + a * ((10. - radius + radius * normal.z) / 10.) - radius * normal,
        normal,
    )
}

// These finite sections check material on both sides of independently derived
// sphere/canal interiors; they are not a global nonintersection certificate.
fn assert_material_sections(solid: &Solid, radius: f64, x: f64, shear: f64, matrix: Matrix4) {
    use truck_meshalgo::prelude::*;
    let scale = matrix.transform_vector(Vector3::unit_z()).magnitude();
    let mesh = solid.triangulation(0.0002 * scale).to_polygon();
    let surfaces: Vec<_> = solid.face_iter().map(|f| f.surface()).collect();
    let check = |center: Point3, normal: Vector3| {
        let p = matrix.transform_point(center + radius * normal);
        assert!(
            surfaces.iter().any(|s| s
                .search_parameter(p, None, 100)
                .is_some_and(|(u, v)| s.subs(u, v).distance(p) < 1e-5 * scale)),
            "missing analytic surface point {p:?}"
        );
        for offset in [-0.005, 0.005] {
            let p = matrix.transform_point(center + (radius + offset) * normal);
            assert_eq!(mesh.inside(p), offset < 0., "section {p:?} radius {radius}");
        }
    };
    let m = Vector3::new(-10., 5., -shear / 2.).normalize();
    let t = root(
        |t| m.dot(independent_frame(t, radius, x, shear).0.to_vec()) + radius,
        0.,
        0.5,
    );
    let (c, ns) = independent_frame(t, radius, x, shear);
    for i in 1..5 {
        for j in 1..5 {
            let normal = (Vector3::unit_z() * (i as f64) + m * (j as f64) + ns * 3.).normalize();
            let p = c + radius * normal;
            assert!(p.z > 0. && p.z < 10. && m.dot(p.to_vec()) < 0.);
            for k in 0..=256 {
                let (q, n) = independent_frame(k as f64 / 256., radius, x, shear);
                assert!(n.dot(p - q - radius * n) < 1e-8);
            }
            check(c, normal);
        }
    }
    for t in [0.3, 0.5, 0.7, 0.85] {
        let (c, n) = independent_frame(t, radius, x, shear);
        let lateral = Vector3::new(n.x, n.y, 0.).normalize();
        for fraction in [0.2, 0.5, 0.8] {
            let angle = n.z.acos() * fraction;
            check(c, Vector3::unit_z() * angle.cos() + lateral * angle.sin());
        }
    }
}

#[test]
fn quartic_conservative_certificate_boundary_and_perturbed_profile() {
    // The Bernstein guard changes at r=4.88412578, below the analytic
    // curvature limit 4.941374. These radii straddle that conservative guard.
    for (x, radius, expected) in [
        (25., 4.884, Some(1870.9557057238753)),
        (25., 4.8842, None),
        (24.9, 1., Some(2275.250012570808)),
    ] {
        let (input, selection) = body_variant(false, x, 2.);
        let selected: Vec<_> = selection
            .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
            .into();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_edges(&input, &selected, radius, 1e-5);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if let Some(volume) = expected {
            let output = result.unwrap();
            assert!(output.solid.is_geometric_consistent());
            assert_material_sections(&output.solid, radius, x, 2., Matrix4::identity());
            common::blend::assert_step(&output.solid, volume, 0.001);
            common::assert_solid(&output.solid, volume, &[0], 0.0002);
        } else {
            assert!(result.is_err());
        }
    }
}

#[test]
fn quartic_nearly_collapsed_continuation_rejects_cleanly() {
    let wall_angle = Vector3::new(-10., 5., -0.00005).normalize().z.acos();
    let contact_angle = independent_frame(1., 1., 25., 0.0001).1.z.acos();
    assert!(contact_angle > wall_angle && contact_angle - wall_angle < 1e-5);
    let (input, selection) = body_variant(false, 25., 0.0001);
    let selected: Vec<_> = selection
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for _ in 0..2 {
        assert!(try_fillet_solid_edges(&input, &selected, 1., 1e-5).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn quartic_private_extension_budget_exhaustion_rejects_cleanly() {
    let radius = 0.01;
    let shear = 100.;
    let m = Vector3::new(-10., 5., -shear / 2.).normalize();
    let w = Vector3::new(-2., 1., 0.).normalize();
    let line = (-radius - (10. - radius) * m.z) / m.dot(w);
    let required = root(
        |t| {
            let (c, n) = independent_frame(t, radius, 25., shear);
            let lateral = Vector3::new(n.x, n.y, 0.).normalize();
            w.dot(c.to_vec()) - line + radius * (w.dot(lateral) - 1.)
        },
        1.,
        1.1,
    );
    let budget = 1. + radius / 80.;
    assert!(required > budget && required < 1.00014);
    let (input, selection) = body_variant(false, 25., shear);
    let selected: Vec<_> = selection
        .map(|p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .into();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges(&input, &selected, radius, 1e-7).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
