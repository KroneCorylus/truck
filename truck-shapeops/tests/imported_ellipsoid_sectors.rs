mod common;
use std::f64::consts::{FRAC_PI_2, PI};
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::{step_geometry, Table};

fn table(negative_sheet: bool, start: usize) -> Table {
    let mut table = Table::from_step(include_str!("fixtures/ellipsoid-sectors/H2.step")).unwrap();
    if negative_sheet {
        // Reparameterize the same physical ellipse by t -> pi - t, including its STEP pcurves.
        for id in [37, 38] {
            for coordinate in &mut table.direction.get_mut(&id).unwrap().direction_ratios {
                *coordinate = -*coordinate;
            }
        }
        for id in [44, 45, 71, 72, 97, 98] {
            let coordinate = &mut table.cartesian_point.get_mut(&id).unwrap().coordinates[1];
            *coordinate = PI - *coordinate;
        }
        table.face_surface.get_mut(&17).unwrap().same_sense = false;
    }
    table
        .edge_loop
        .get_mut(&19)
        .unwrap()
        .edge_list
        .rotate_left(start);
    table
}

fn input(negative_sheet: bool, start: usize) -> Solid {
    let table = table(negative_sheet, start);
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    let meridians: Vec<_> = raw
        .boundaries
        .iter()
        .flat_map(|shell| &shell.faces)
        .filter_map(|face| {
            if let step_geometry::Surface::SweptCurve(step_geometry::SweptCurve::RevolutedCurve(
                surface,
            )) = &face.surface
            {
                Some(surface.entity().entity_curve().range_tuple())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(meridians.len(), 1);
    let expected = if negative_sheet {
        (FRAC_PI_2, PI)
    } else {
        (0., FRAC_PI_2)
    };
    assert!((meridians[0].0 - expected.0).abs() < 1e-12, "{meridians:?}");
    assert!((meridians[0].1 - expected.1).abs() < 1e-12, "{meridians:?}");
    raw.split_closed_edges_and_faces(0.00001);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn volume() -> f64 {
    3. * PI / 4. * ((250_f64.powi(3) - 150_f64.powi(3)) / 3. + 2. * 150_f64.powi(2) * 100. / 3.)
}

fn check(negative_sheet: bool, start: usize, scale: f64, transformed: bool) {
    let transform = if transformed {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    let model = builder::transformed(&input(negative_sheet, start), transform);
    assert!(model.is_geometric_consistent());
    common::assert_solid(&model, volume() * scale.powi(3), &[0], 0.001);
    common::blend::assert_step(&model, volume() * scale.powi(3), 0.001);
}

#[test]
fn elliptical_revolutions_preserve_the_native_meridian_sheet() {
    for negative_sheet in [false, true] {
        for start in 0..3 {
            for transformed in [false, true] {
                check(negative_sheet, start, 0.01, transformed);
            }
        }
    }
}

#[test]
fn original_scale_ellipsoid_sector_preserves_material_and_step() { check(false, 0, 1., false); }

#[test]
fn hole_only_or_invalid_pcurves_do_not_shrink_revolution_supports() {
    for invalid in 0..3 {
        let mut table = table(false, 0);
        match invalid {
            0 => table.face_bound.get_mut(&18).unwrap().orientation = false,
            1 => table.cartesian_point.get_mut(&44).unwrap().coordinates[1] = f64::NAN,
            _ => table.cartesian_point.get_mut(&44).unwrap().coordinates[1] = 0.1,
        }
        let (raw, skipped) = table
            .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
            .unwrap();
        assert!(skipped.is_empty(), "{skipped:?}");
        let mut found = false;
        for face in raw.boundaries.iter().flat_map(|shell| &shell.faces) {
            if let step_geometry::Surface::SweptCurve(step_geometry::SweptCurve::RevolutedCurve(
                surface,
            )) = &face.surface
            {
                found = true;
                assert_eq!(surface.entity().entity_curve().range_tuple(), (0., 2. * PI));
            }
        }
        assert!(found);
    }
}

fn fillet_section() -> (Point2, f64, f64, f64) {
    let center = |phi: f64| {
        let normal = Vector2::new(100. * phi.cos(), 150. * phi.sin()).normalize();
        Point2::new(150. * phi.cos(), 100. * phi.sin()) + 10. * normal
    };
    let (mut lo, mut hi) = (0., (100_f64 / 150.).atan());
    for _ in 0..60 {
        let mid = (lo + hi) / 2.;
        let c = center(mid);
        if c.x + c.y < 150. + 10. * 2_f64.sqrt() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let phi = (lo + hi) / 2.;
    let c = center(phi);
    let (z0, z1) = (c.y - 10. / 2_f64.sqrt(), 100. * phi.sin());
    // Integrate squared outer radius across the circular replacement of cone and ellipse.
    let fillet = |z: f64| {
        let d = z - c.y;
        (c.x * c.x + 100.) * d
            - d.powi(3) / 3.
            - c.x * (d * (100. - d * d).sqrt() + 100. * (d / 10.).asin())
    };
    let cone = |z: f64| -(150. - z).powi(3) / 3.;
    let ellipse = |z: f64| 150_f64.powi(2) * (z - z.powi(3) / 30000.);
    let added =
        3. * PI / 4. * (fillet(z1) - fillet(z0) - cone(0.) + cone(z0) - ellipse(z1) + ellipse(0.));
    (c, z0, z1, volume() + added)
}

#[test]
fn ellipse_cone_junction_fillet_preserves_radius_material_and_step() {
    use truck_shapeops::fillet::try_fillet_solid_along_wire;
    let (center, z0, z1, expected) = fillet_section();
    let scale = 0.01;
    for negative_sheet in [false, true] {
        for reversed in [false, true] {
            let matrix = if reversed {
                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            } else {
                Matrix4::identity()
            } * Matrix4::from_scale(scale);
            let inverse = matrix.invert().unwrap();
            let solid = builder::transformed(&input(negative_sheet, 0), matrix);
            let before = serde_json::to_string(&solid.compress()).unwrap();
            let point = matrix.transform_point(Point3::new(-150., 0., 0.));
            let edge = common::blend::edge_through(&solid.boundaries()[0], point);
            let edge = if reversed { edge.inverse() } else { edge };
            let result =
                try_fillet_solid_along_wire(&solid, &wire![edge], 10. * scale, 0.001).unwrap();
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
            assert!(result.solid.is_geometric_consistent());
            assert_eq!(result.generated_faces.len(), 1);
            let surface = result.generated_faces[0].oriented_surface();
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            let (mut min_z, mut max_z) = (f64::INFINITY, f64::NEG_INFINITY);
            for i in 0..=8 {
                for j in 0..=16 {
                    let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                    let p = inverse.transform_point(surface.subs(u, v));
                    let rho = p.x.hypot(p.y);
                    assert!(((rho - center.x).hypot(p.z - center.y) - 10.).abs() * scale < 1e-6);
                    let expected = Vector3::new(
                        (center.x - rho) * p.x / rho,
                        (center.x - rho) * p.y / rho,
                        center.y - p.z,
                    )
                    .normalize();
                    let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(normal.dot(expected) > 1. - 1e-5);
                    min_z = min_z.min(p.z);
                    max_z = max_z.max(p.z);
                }
            }
            assert!((min_z - z0).abs() * scale < 1e-6);
            assert!((max_z - z1).abs() * scale < 1e-6);
            common::assert_solid(&result.solid, expected * scale.powi(3), &[0], 0.001);
            common::blend::assert_step(&result.solid, expected * scale.powi(3), 0.001);
        }
    }
}
