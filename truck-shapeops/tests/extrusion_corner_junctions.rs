mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges;

fn body(height: f64) -> (Solid, Vec<EdgeID>) {
    let right = Vertex::new(Point3::new(1., 0., 0.));
    let left = Vertex::new(Point3::new(-1., 0., 0.));
    let bottom = Vertex::new(Point3::new(0., -0.5, 0.));
    let arc = builder::circle_arc(&right, &left, Point3::new(0., 1., 0.));
    let wire = wire![
        arc,
        builder::line(&left, &bottom),
        builder::line(&bottom, &right)
    ];
    let wire = builder::transformed(&wire, Matrix4::from_nonuniform_scale(1.5, 1., 1.));
    let face = builder::try_attach_plane(&[wire]).unwrap();
    let solid: Solid = builder::tsweep(&face, Vector3::new(0.2 * height, 0., height));
    let mut seen = HashSet::new();
    let ids = solid
        .edge_iter()
        .filter(|e| {
            seen.insert(e.id())
                && [e.front(), e.back()]
                    .iter()
                    .any(|v| v.point().near(&Point3::new(1.5 + 0.2 * height, 0., height)))
        })
        .map(|e| e.id())
        .collect();
    (solid, ids)
}

fn center(radius: f64) -> Point3 {
    let normal =
        |theta: f64| Vector3::new(theta.cos(), 1.5 * theta.sin(), -0.2 * theta.cos()).normalize();
    let wall = Vector3::new(1. / 3., -1., -0.2 / 3.).normalize();
    let length = Vector3::new(1. / 3., -1., -0.2 / 3.).magnitude();
    let mut lo = 0.;
    let mut hi = std::f64::consts::FRAC_PI_2;
    for _ in 0..80 {
        let theta = (lo + hi) / 2.;
        let q = Point3::new(1.5 * theta.cos(), theta.sin(), 0.) - radius * normal(theta);
        if wall.dot(q.to_vec()) - 0.5 / length + radius > 0. {
            lo = theta;
        } else {
            hi = theta;
        }
    }
    let theta = (lo + hi) / 2.;
    let n = normal(theta);
    let z = 1. - radius + radius * n.z;
    Point3::new(1.5 * theta.cos() + 0.2 * z, theta.sin(), z) - radius * n
}

#[test]
fn elliptical_extrusion_corner_preserves_radius_tangency_material_and_step() {
    for (radius, volume) in [
        (0.03, 3.104963665804056),
        (0.06, 3.1012708508906512),
        (0.1, 3.092505604759356),
    ] {
        for moved in [false, true] {
            let (original, selected) = body(1.);
            assert_eq!(selected.len(), 3);
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
            let before = serde_json::to_string(&input.compress()).unwrap();
            let mut ids: Vec<_> = selected
                .iter()
                .map(|id| {
                    let e = original.edge_iter().find(|e| e.id() == *id).unwrap();
                    let c = e.curve();
                    let (a, b) = c.range_tuple();
                    common::blend::edge_through(
                        &input.boundaries()[0],
                        matrix.transform_point(c.subs((a + b) / 2.)),
                    )
                    .id()
                })
                .collect();
            let supports: HashSet<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| ids.contains(&e.id())))
                .map(|f| f.id())
                .collect();
            for _ in 0..2 {
                let result =
                    try_fillet_solid_edges(&input, &ids, radius * factor, 1e-5 * factor).unwrap();
                let solid = &result.solid;
                assert_eq!(result.generated_faces.len(), 4);
                assert!(solid.is_geometric_consistent());
                let mut fitted = 0;
                let mut spheres = 0;
                let mut cylinders = 0;
                let mut worst_radius = 0_f64;
                for face in &result.generated_faces {
                    if let Surface::Fillet(fit) = face.surface() {
                        fitted += 1;
                        let af = fit.entity();
                        let (side, plane) = [af.surface0(), af.surface1()]
                            .into_iter()
                            .enumerate()
                            .find_map(|(i, s)| {
                                if let Surface::Plane(p) = s.as_ref() {
                                    Some((i, p))
                                } else {
                                    None
                                }
                            })
                            .unwrap();
                        let n = fit.transform().transform_vector(plane.normal()).normalize();
                        let (a, b) = af.range_tuple().1;
                        for j in 0..=32 {
                            let v = a + (b - a) * j as f64 / 32.;
                            let contact = fit.transform().transform_point(af.subs(side as f64, v));
                            let c = contact - radius * factor * n;
                            for i in 0..=16 {
                                let p = fit.transform().transform_point(af.subs(i as f64 / 16., v));
                                worst_radius =
                                    worst_radius.max((p.distance(c) / factor - radius).abs());
                            }
                        }
                    } else {
                        match face.oriented_surface().elementary() {
                            Some((
                                Elementary::Sphere {
                                    center: c,
                                    radius: r,
                                },
                                _,
                            )) => {
                                spheres += 1;
                                assert!(inverse.transform_point(c).distance(center(radius)) < 1e-8);
                                assert!((r / factor - radius).abs() < 1e-8);
                            }
                            _ => {
                                cylinders += 1;
                                // Oblique terminal sections can retain a NURBS cylinder representation.
                                let axis = Vector3::new(1., 1. / 3., 0.).normalize();
                                let surface = face.surface();
                                let (ur, vr) = surface.try_range_tuple();
                                let ((u0, u1), (v0, v1)) = (ur.unwrap(), vr.unwrap());
                                for i in 0..=16 {
                                    for j in 0..=16 {
                                        let p = inverse.transform_point(surface.subs(
                                            u0 + (u1 - u0) * i as f64 / 16.,
                                            v0 + (v1 - v0) * j as f64 / 16.,
                                        ));
                                        let delta = p - center(radius);
                                        assert!(
                                            ((delta - axis * delta.dot(axis)).magnitude() - radius)
                                                .abs()
                                                < 1e-8
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                assert_eq!((fitted, spheres, cylinders), (2, 1, 1));
                assert!(worst_radius < 1e-5, "radius residual {worst_radius}");
                let mut tangent_faces: HashSet<_> =
                    result.generated_faces.iter().map(|f| f.id()).collect();
                tangent_faces.extend(
                    result
                        .modified_faces
                        .iter()
                        .filter(|(id, _)| supports.contains(id))
                        .map(|(_, f)| f.id()),
                );
                let mut checked = HashSet::new();
                let mut worst_dot = 1_f64;
                for face in &result.generated_faces {
                    for edge in face.edge_iter() {
                        if checked.contains(&edge.id()) {
                            continue;
                        }
                        let other = solid
                            .face_iter()
                            .find(|f| {
                                f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id())
                            })
                            .unwrap();
                        if !tangent_faces.contains(&other.id()) {
                            continue;
                        }
                        checked.insert(edge.id());
                        let c = edge.oriented_curve();
                        let (a, b) = c.range_tuple();
                        let surfaces = [face.oriented_surface(), other.oriented_surface()];
                        for i in 0..=16 {
                            let p = c.subs(a + (b - a) * i as f64 / 16.);
                            let normals = surfaces.each_ref().map(|s| {
                                let (u, v) = s.search_parameter(p, None, 100).unwrap();
                                s.normal(u, v).normalize()
                            });
                            worst_dot = worst_dot.min(normals[0].dot(normals[1]));
                        }
                    }
                }
                assert_eq!(checked.len(), 9);
                assert!(worst_dot > 1. - 1e-7, "normal dot {worst_dot}");
                common::assert_solid(solid, volume * factor.powi(3), &[0], 0.0003 * factor);
                common::blend::assert_step(solid, volume * factor.powi(3), 0.0003 * factor);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                println!("radius {radius} moved {moved}: radius error {worst_radius}, minimum normal dot {worst_dot}");
                ids.reverse();
            }
        }
    }
}

#[test]
fn oversized_elliptical_extrusion_corner_preserves_input() {
    let (input, mut selected) = body(1.);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radius in [1., 1.6] {
        for _ in 0..2 {
            assert!(try_fillet_solid_edges(&input, &selected, radius, 1e-5).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            selected.reverse();
        }
    }
}

// Short bodies give the fitted blend a parameter domain shorter than the unit square.
#[test]
fn short_elliptical_extrusion_corner_preserves_material_and_step() {
    for (height, volume) in [(0.13, 0.391151182497675), (0.15, 0.45325128415886307)] {
        let (input, mut selected) = body(height);
        assert_eq!(selected.len(), 3);
        let before = serde_json::to_string(&input.compress()).unwrap();
        for _ in 0..2 {
            let result = try_fillet_solid_edges(&input, &selected, 0.1, 1e-5).unwrap();
            assert_eq!(result.generated_faces.len(), 4);
            assert!(result.solid.is_geometric_consistent());
            common::assert_solid(&result.solid, volume, &[0], 0.0003);
            common::blend::assert_step(&result.solid, volume, 0.0003);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            selected.reverse();
        }
    }
}
