mod common;
use truck_modeling::*;
use truck_shapeops::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input(quartic: bool, robust: bool, start: usize) -> Solid {
    let text = if quartic {
        include_str!("fixtures/closed-bezier-prisms/quartic.step")
    } else {
        include_str!("fixtures/closed-bezier-prisms/cubic.step")
    };
    let table = Table::from_step(text).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    for shell in &mut raw.boundaries {
        for face in &mut shell.faces {
            for wire in &mut face.boundaries {
                let rotation = start % wire.len();
                wire.rotate_left(rotation);
            }
        }
    }
    if robust {
        raw.robust_split_closed_edges_and_faces(1e-5);
    } else {
        raw.split_closed_edges_and_faces(1e-5);
    }
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .expect("closed Bézier extrusion must split into simple face boundaries")
}

#[test]
fn closed_bezier_import_preserves_material_mesh_and_step() {
    for quartic in [false, true] {
        for robust in [false, true] {
            for start in 0..3 {
                let scale: f64 = if start == 2 { 0.1 } else { 1. };
                let transform = if start == 0 {
                    Matrix4::identity()
                } else {
                    Matrix4::from_translation(Vector3::new(2., -3., 5.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                        * Matrix4::from_scale(scale)
                };
                let solid = builder::transformed(&input(quartic, robust, start), transform);
                let volume = (if quartic { 7200. / 7. } else { 150. }) * scale.powi(3);
                assert!(
                    solid.is_geometric_consistent(),
                    "quartic={quartic} robust={robust} start={start}"
                );
                common::assert_counts(&solid, 4, 6, 4);
                common::assert_solid(&solid, volume, &[0], 0.001);
                common::blend::assert_step(&solid, volume, 0.001);
            }
        }
    }
}

#[test]
fn unsupported_closed_cusp_rim_fillets_preserve_the_recovered_input() {
    for quartic in [false, true] {
        let solid = input(quartic, false, 0);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let wire = solid
            .face_iter()
            .find(|face| {
                matches!(face.surface(), Surface::Plane(_))
                    && face.vertex_iter().all(|v| (v.point().z - 10.).abs() < 1e-8)
            })
            .unwrap()
            .boundaries()[0]
            .clone();
        for wire in [wire.clone(), wire.inverse()] {
            let error = truck_shapeops::fillet::try_fillet_solid_along_wire(
                &solid,
                &wire,
                if quartic { 0.1 } else { 0.5 },
                0.001,
            )
            .unwrap_err();
            assert_eq!(
                error.code,
                truck_base::diagnostics::Code::UnsupportedGeometry
            );
            assert_eq!(error.stage, "validate_wire");
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}

fn cubic_cusp_section(radius: f64) -> (f64, f64) {
    let profile = |t: f64| {
        (
            30. * t * (1. - t).powi(2),
            30. * t * t * (1. - t),
            30. * (1. - 4. * t + 3. * t * t),
            30. * (2. * t - 3. * t * t),
        )
    };
    let (mut a, mut b) = (0., 0.1);
    for _ in 0..80 {
        let t = (a + b) / 2.;
        let (x, y, dx, dy) = profile(t);
        if x - y - radius * (dx + dy) / dx.hypot(dy) < 0. {
            a = t;
        } else {
            b = t;
        }
    }
    let t = (a + b) / 2.;
    let (x, y, dx, dy) = profile(t);
    let center = x - radius * dy / dx.hypot(dy);
    assert!((center - y - radius * dx / dx.hypot(dy)).abs() < 1e-12);
    let start = (x - center).atan2(y - center);
    let end = (y - center).atan2(x - center);
    let angle = (end - start).rem_euclid(std::f64::consts::TAU);
    // Green's theorem: replace the two cubic legs by their tangent circular arc.
    let arc = (radius * center * (end.sin() - start.sin() - end.cos() + start.cos())
        + radius * radius * angle)
        / 2.;
    let legs = 900. * (t.powi(3) / 3. - t.powi(4) / 2. + t.powi(5) / 5.);
    (center, 10. * (15. - legs + arc))
}

#[test]
fn cubic_cusp_fillet_preserves_radius_normals_material_and_step() {
    let (center, volume) = cubic_cusp_section(0.5);
    for (scale, moved, reversed) in [(1., false, false), (1., true, true), (0.1, true, false)] {
        let matrix = if moved {
            Matrix4::from_translation(Vector3::new(2., -3., 5.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let solid = builder::transformed(&input(false, false, 0), matrix);
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let edge = solid
            .edge_iter()
            .find(|edge| {
                [edge.front().point(), edge.back().point()]
                    .into_iter()
                    .all(|p| {
                        let p = inverse.transform_point(p);
                        p.x.hypot(p.y) < 1e-8
                    })
            })
            .unwrap();
        let edge = if reversed {
            edge.inverse()
        } else {
            edge.clone()
        };
        let result = truck_shapeops::fillet::try_fillet_solid_along_wire(
            &solid,
            &wire![edge],
            0.5 * scale,
            1e-6,
        )
        .unwrap();
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), 1);
        let surface = result.generated_faces[0].oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        let (mut min_z, mut max_z) = (f64::INFINITY, f64::NEG_INFINITY);
        for i in 0..=16 {
            for j in 0..=16 {
                let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 16.);
                let p = inverse.transform_point(surface.subs(u, v));
                let radial = Vector3::new(p.x - center, p.y - center, 0.);
                assert!((radial.magnitude() - 0.5).abs() * scale < 1e-6, "{p:?}");
                let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                assert!(normal.dot(radial.normalize()) > 1. - 1e-5);
                min_z = min_z.min(p.z);
                max_z = max_z.max(p.z);
            }
        }
        assert!(min_z.abs() * scale < 1e-6);
        assert!((max_z - 10.).abs() * scale < 1e-6);
        common::assert_solid(&result.solid, volume * scale.powi(3), &[0], 0.001);
        common::blend::assert_step(&result.solid, volume * scale.powi(3), 0.001);
    }
}

#[test]
fn rolling_support_hints_survive_serialization() {
    type Support = ExtrudedCurve<BSplineCurve<Point3>, Vector3>;
    type Blend = RbfSurface<Line<Point3>, Support, Support, f64>;
    type Contact = RbfContactCurve<Line<Point3>, Support, Support, f64>;
    let profile = BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Point3::origin(),
            Point3::new(10., 0., 0.),
            Point3::new(0., 10., 0.),
            Point3::origin(),
        ],
    );
    let support = ExtrudedCurve::by_extrusion(profile, Vector3::new(0., 0., 10.));
    let plain = RbfSurface::new(
        Line(Point3::new(0., 0., 10.), Point3::origin()),
        support.clone(),
        support,
        0.5,
    );
    let json = serde_json::to_value(&plain).unwrap();
    assert!(json.get("support_hints").is_none());
    assert_eq!(serde_json::from_value::<Blend>(json).unwrap(), plain);
    let seeded = plain
        .try_with_support_hints([(0., 0.5), (1., 0.5)])
        .unwrap();
    let restored: Blend = serde_json::from_str(&serde_json::to_string(&seeded).unwrap()).unwrap();
    assert_eq!(restored, seeded);
    for u in [0., 0.3, 0.7, 1.] {
        for v in [0., 0.5, 1.] {
            assert!(restored.subs(u, v).distance(seeded.subs(u, v)) < 1e-12);
        }
    }
    for contact in [seeded.contact_curve0(), seeded.contact_curve1()] {
        let restored: Contact =
            serde_json::from_str(&serde_json::to_string(&contact).unwrap()).unwrap();
        assert_eq!(restored, contact);
        assert!(restored.subs(0.3).distance(contact.subs(0.3)) < 1e-12);
    }
}
