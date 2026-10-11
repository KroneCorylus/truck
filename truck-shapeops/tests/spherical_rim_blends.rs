mod common;

use std::f64::consts::{PI, TAU};
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Representation {
    Sphere,
    Revolution,
    Nurbs,
}

fn band(top: f64, representation: Representation) -> Solid {
    let point = |z: f64| Point3::new((25. - z * z).sqrt(), 0., z);
    let middle = ((top / 5.).asin() + (-4.0_f64 / 5.).asin()) / 2.;
    let profile: Edge = builder::circle_arc(
        &builder::vertex(point(top)),
        &builder::vertex(point(-4.)),
        Point3::new(5. * middle.cos(), 0., 5. * middle.sin()),
    );
    let mut shell: Shell =
        builder::rsweep(&profile, Point3::origin(), Vector3::unit_z(), Rad(TAU), 2);
    if representation == Representation::Sphere {
        for face in &shell {
            face.set_surface(Surface::Sphere(Processor::new(Sphere::new(
                Point3::origin(),
                5.,
            ))));
        }
    } else if representation == Representation::Nurbs {
        let meridian = NurbsCurve::try_from(&profile.curve()).unwrap();
        for (i, face) in shell.iter().enumerate() {
            let azimuth: NurbsCurve<Vector4> = Processor::<_, Matrix4>::new(TrimmedCurve::new(
                UnitCircle::<Point3>::new(),
                (i as f64 * PI, (i + 1) as f64 * PI),
            ))
            .to_same_geometry();
            let surface = NurbsSurface::new(BSplineSurface::new(
                (meridian.knot_vec().clone(), azimuth.knot_vec().clone()),
                meridian
                    .control_points()
                    .iter()
                    .map(|p| {
                        azimuth
                            .control_points()
                            .iter()
                            .map(|q| Vector4::new(p.x * q.x, p.x * q.y, p.z * q.w, p.w * q.w))
                            .collect()
                    })
                    .collect(),
            ));
            face.set_surface(surface.into());
        }
    }
    for boundary in shell.extract_boundaries() {
        shell.push(builder::try_attach_plane(&[boundary.inverse()]).unwrap());
    }
    Solid::new(vec![shell])
}

fn volume(top: f64, radius: f64) -> f64 {
    let sphere = |z: f64| 25. * z - z.powi(3) / 3.;
    if radius == 0. {
        return PI * (sphere(top) - sphere(-4.));
    }
    let center_z = top - radius;
    let center_r = ((5. - radius).powi(2) - center_z.powi(2)).sqrt();
    let contact_z = 5. * center_z / (5. - radius);
    let arc = |u: f64| {
        (center_r.powi(2) + radius.powi(2)) * u - u.powi(3) / 3.
            + center_r
                * (u * (radius.powi(2) - u.powi(2)).max(0.).sqrt()
                    + radius.powi(2) * (u / radius).clamp(-1., 1.).asin())
    };
    PI * (sphere(contact_z) - sphere(-4.) + arc(radius) - arc(contact_z - center_z))
}

fn assert_dimensions(result: &BlendResult, top: f64, transform: Matrix4, cavity: bool) {
    let inverse = transform.invert().unwrap();
    let center_z = top - 0.5;
    let center_r = (4.5_f64.powi(2) - center_z.powi(2)).sqrt();
    assert_eq!(result.generated_faces.len(), 2);
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for i in 0..=8 {
            for j in 0..=16 {
                let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                let point = inverse.transform_point(surface.subs(u, v));
                let rho = point.x.hypot(point.y);
                assert!(((rho - center_r).hypot(point.z - center_z) - 0.5).abs() <= TOL);
                let mut normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                if cavity {
                    normal = -normal;
                }
                let expected = Vector3::new(
                    point.x / rho * (rho - center_r),
                    point.y / rho * (rho - center_r),
                    point.z - center_z,
                )
                .normalize();
                assert!(normal.dot(expected) > 1. - 1e-4);
            }
        }
        for edge in face.edge_iter() {
            let other = result
                .solid
                .face_iter()
                .find(|f| f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id()))
                .unwrap();
            let other = other.oriented_surface();
            let curve = edge.oriented_curve();
            let (a, b) = curve.range_tuple();
            for i in 1..8 {
                let p = curve.subs(a + (b - a) * i as f64 / 8.);
                let normals = [&surface, &other].map(|s| {
                    let (u, v) = s.search_parameter(p, None, 100).unwrap();
                    s.normal(u, v).normalize()
                });
                assert!(normals[0].dot(normals[1]) > 1. - 1e-4);
            }
        }
    }
}

fn assert_history(input: &Solid, result: &BlendResult) {
    let mut faces = std::collections::HashSet::new();
    for face in input.face_iter() {
        if result.solid.face_iter().any(|f| f.id() == face.id()) {
            assert!(faces.insert(face.id()));
        } else {
            let replacements: Vec<_> = result
                .modified_faces
                .iter()
                .filter(|(id, _)| *id == face.id())
                .collect();
            assert_eq!(replacements.len(), 1);
            assert!(faces.insert(replacements[0].1.id()));
        }
    }
    for face in &result.generated_faces {
        assert!(faces.insert(face.id()));
    }
    assert_eq!(faces.len(), result.solid.face_iter().count());
    assert!(result.solid.face_iter().all(|f| faces.contains(&f.id())));
}

#[test]
fn plane_sphere_rims_preserve_rolling_radius() {
    for top in [-3., 3.] {
        for representation in [
            Representation::Sphere,
            Representation::Revolution,
            Representation::Nurbs,
        ] {
            for transform in [
                Matrix4::identity(),
                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
            ] {
                eprintln!(
                    "sphere top={top}, representation={representation:?}, transform={transform:?}"
                );
                let input = builder::transformed(&band(top, representation), transform);
                assert!(input.is_geometric_consistent());
                common::assert_solid(&input, volume(top, 0.), &[0], TOL);
                let inverse = transform.invert().unwrap();
                let face = input
                    .face_iter()
                    .find(|face| {
                        matches!(face.surface(), Surface::Plane(_))
                            && face
                                .vertex_iter()
                                .all(|v| (inverse.transform_point(v.point()).z - top).abs() < 1e-6)
                    })
                    .unwrap();
                let wire = face.boundaries()[0].clone();
                let before = serde_json::to_string(&input.compress()).unwrap();
                for wire in [wire.clone(), wire.inverse()] {
                    let result = try_fillet_solid_along_wire(&input, &wire, 0.5, TOL).unwrap();
                    assert!(result.solid.is_geometric_consistent());
                    assert_dimensions(&result, top, transform, false);
                    assert_history(&input, &result);
                    common::assert_solid(&result.solid, volume(top, 0.5), &[0], TOL);
                    common::blend::assert_step(&result.solid, volume(top, 0.5), TOL);
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}

#[test]
fn spherical_cavity_rims_preserve_orientation() {
    let outer = common::modeling::cuboid(Point3::new(-8., -8., -8.), Point3::new(8., 8., 8.));
    for top in [-3., 3.] {
        for representation in [Representation::Sphere, Representation::Nurbs] {
            let mut void = band(top, representation);
            void.not();
            let wire = void
                .face_iter()
                .find(|f| {
                    matches!(f.surface(), Surface::Plane(_))
                        && f.vertex_iter().all(|v| (v.point().z - top).abs() < 1e-6)
                })
                .unwrap()
                .boundaries()[0]
                .clone();
            let input = Solid::new(vec![
                outer.boundaries()[0].clone(),
                void.boundaries()[0].clone(),
            ]);
            let before = serde_json::to_string(&input.compress()).unwrap();
            common::assert_solid(&input, 4096. - volume(top, 0.), &[0, 0], TOL);
            let result = try_fillet_solid_along_wire(&input, &wire, 0.5, TOL).unwrap();
            assert_dimensions(&result, top, Matrix4::identity(), true);
            assert_history(&input, &result);
            assert!(result.solid.is_geometric_consistent());
            common::assert_solid(&result.solid, 4096. - volume(top, 0.5), &[0, 0], TOL);
            common::blend::assert_step(&result.solid, 4096. - volume(top, 0.5), TOL);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
