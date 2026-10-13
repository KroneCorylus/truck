mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{fillet::try_fillet_solid_along_wire, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/conical-sectors/H6.step")).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
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

fn input_volume() -> f64 { 3. * PI / 4. * (2. * 200_f64.powi(3) / 3. + 50_f64.powi(2) * 100.) }

fn fillet_volume() -> f64 {
    let radius = 10.;
    let area = radius * radius * (1. - PI / 4.);
    let centroid = radius * (10. - 3. * PI) / (12. - 3. * PI);
    input_volume() + 1.5 * PI * (50. + centroid) * area
}

fn transform(scale: f64, moved: bool) -> Matrix4 {
    (if moved {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    }) * Matrix4::from_scale(scale)
}

#[test]
fn conical_sector_import_preserves_apices_material_and_step() {
    for (scale, moved) in [(0.01, false), (0.01, true), (1., false)] {
        let matrix = transform(scale, moved);
        let inverse = matrix.invert().unwrap();
        let solid = builder::transformed(&input(), matrix);
        assert!(solid.is_geometric_consistent());
        let mut cones = 0;
        for face in solid.face_iter() {
            let surface = face.oriented_surface();
            if !matches!(surface, Surface::RevolutedCurve(_)) {
                continue;
            }
            cones += 1;
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            for edge in face.edge_iter() {
                let curve = edge.curve();
                let (t0, t1) = curve.range_tuple();
                for i in 0..=64 {
                    let point = curve.subs((t0 + (t1 - t0) * i as f64 / 64.).clamp(t0, t1));
                    let (u, v) = surface.search_parameter(point, None, 100).unwrap();
                    assert!((a - 1e-9..=b + 1e-9).contains(&u));
                    assert!((c - 1e-9..=d + 1e-9).contains(&v));
                    assert!(surface.subs(u, v).distance(point) < 1e-7);
                    let p = inverse.transform_point(point);
                    let rho = p.x.hypot(p.y);
                    if rho > 1e-6 {
                        let expected = Vector3::new(p.x / rho, p.y / rho, p.z.signum()).normalize();
                        let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                        assert!(normal.dot(expected) > 1. - 1e-8);
                    }
                }
            }
        }
        assert_eq!(cones, 2);
        common::assert_solid(&solid, input_volume() * scale.powi(3), &[0], 0.001);
        common::blend::assert_step(&solid, input_volume() * scale.powi(3), 0.001);
    }
}

#[test]
fn conical_sector_neck_fillet_preserves_radius_normals_material_and_step() {
    for (scale, reversed) in [(0.01, false), (0.01, true), (1., false)] {
        let matrix = transform(scale, reversed);
        let inverse = matrix.invert().unwrap();
        let solid = builder::transformed(&input(), matrix);
        assert!(solid.is_geometric_consistent());
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let edge = common::blend::edge_through(
            &solid.boundaries()[0],
            matrix.transform_point(Point3::new(-50., 0., -50.)),
        );
        let edge = if reversed { edge.inverse() } else { edge };
        let result = try_fillet_solid_along_wire(&solid, &wire![edge], 10. * scale, 1e-6).unwrap();
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(result.generated_faces.len(), 1);
        let surface = result.generated_faces[0].oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        let (mut min_r, mut max_r, mut min_z, mut max_z) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        for i in 0..=16 {
            for j in 0..=16 {
                let (u, v) = (a + (b - a) * i as f64 / 16., c + (d - c) * j as f64 / 16.);
                let p = inverse.transform_point(surface.subs(u, v));
                let rho = p.x.hypot(p.y);
                assert!(
                    ((rho - 60.).hypot(p.z + 40.) - 10.).abs() * scale < 1e-6,
                    "scale={scale} reversed={reversed} uv=({u},{v}) point={p:?} radius={}",
                    (rho - 60.).hypot(p.z + 40.)
                );
                let expected =
                    Vector3::new((60. - rho) * p.x / rho, (60. - rho) * p.y / rho, -40. - p.z)
                        .normalize();
                let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                assert!(normal.dot(expected) > 1. - 1e-5);
                min_r = min_r.min(rho);
                max_r = max_r.max(rho);
                min_z = min_z.min(p.z);
                max_z = max_z.max(p.z);
            }
        }
        for (got, want) in [(min_r, 50.), (max_r, 60.), (min_z, -50.), (max_z, -40.)] {
            assert!((got - want).abs() * scale < 1e-6, "{got} != {want}");
        }
        common::assert_solid(&result.solid, fillet_volume() * scale.powi(3), &[0], 0.001);
        common::blend::assert_step(&result.solid, fillet_volume() * scale.powi(3), 0.001);
    }
}
