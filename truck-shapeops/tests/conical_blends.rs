mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

fn frustum(slope: f64, analytic: bool) -> Solid {
    let bottom: Wire = builder::rsweep(
        &builder::vertex(Point3::new(5., 0., 0.)),
        Point3::origin(),
        Vector3::unit_z(),
        Rad(2. * PI),
        2,
    );
    let top = builder::transformed(
        &bottom,
        Matrix4::from_translation(Vector3::new(0., 0., 8.))
            * Matrix4::from_nonuniform_scale(1. + 8. * slope / 5., 1. + 8. * slope / 5., 1.),
    );
    let mut shell: Shell = builder::try_wire_homotopy(&bottom, &top).unwrap();
    if analytic {
        for face in &shell {
            let line = Curve::Line(Line(
                Point3::new(5., 0., 0.),
                Point3::new(5. + 8. * slope, 0., 8.),
            ));
            let mut surface = Surface::RevolutedCurve(Processor::new(
                RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_z()),
            ));
            let point = Point3::new(5. + 4. * slope, 0., 4.);
            let (u, v) = surface.search_parameter(point, None, 100).unwrap();
            if surface.normal(u, v).dot(Vector3::new(1., 0., -slope)) < 0. {
                surface.invert();
            }
            face.set_surface(surface);
        }
    }
    shell.push(builder::try_attach_plane(&[bottom]).unwrap().inverse());
    shell.push(builder::try_attach_plane(&[top]).unwrap());
    Solid::new(vec![shell])
}

fn input_volume(slope: f64) -> f64 {
    let top = 5. + 8. * slope;
    PI * 8. / 3. * (25. + 5. * top + top * top)
}

fn fillet_volume(slope: f64, radius: f64) -> f64 {
    let top = 5. + 8. * slope;
    let q = (1. + slope * slope).sqrt();
    let beta = (-slope / q).acos();
    let area = radius.powi(2) * (q + slope - beta / 2.);
    let center = top - radius * (slope + q);
    let polygon_moment = radius.powi(3) * (q + slope) * (2. * (q + slope) + 1. / q) / 6.;
    let sector_moment = radius.powi(3) * (1. + slope / q) / 3.;
    input_volume(slope) - 2. * PI * (center * area + polygon_moment - sector_moment)
}

fn assert_dimensions(
    result: &BlendResult,
    slope: f64,
    transform: Matrix4,
    fillet: bool,
    cavity: bool,
) {
    let inverse = transform.invert().unwrap();
    let q = (1. + slope * slope).sqrt();
    let top = 5. + 8. * slope;
    let center = top - 0.5 * (slope + q);
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for i in 0..=8 {
            for j in 0..=16 {
                let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 16.);
                let point = inverse.transform_point(surface.subs(u, v));
                let rho = point.x.hypot(point.y);
                let mut normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                if cavity {
                    normal = -normal;
                }
                if fillet {
                    assert!(((rho - center).hypot(point.z - 7.5) - 0.5).abs() <= TOL);
                    let expected = Vector3::new(
                        point.x / rho * (rho - center),
                        point.y / rho * (rho - center),
                        point.z - 7.5,
                    )
                    .normalize();
                    assert!(normal.dot(expected) > 1. - 1e-4);
                } else {
                    let rise = 0.5 * q / 0.3 - slope;
                    assert!(
                        (rho + rise * (point.z - 8.) - top + 0.5).abs() / rise.hypot(1.) <= TOL
                    );
                    let expected = Vector3::new(point.x / rho, point.y / rho, rise).normalize();
                    assert!(normal.dot(expected) > 1. - 1e-4);
                }
            }
        }
    }
}

fn assert_history(input: &Solid, result: &BlendResult) {
    let mut accounted = std::collections::HashSet::new();
    for face in input.face_iter() {
        if result
            .solid
            .face_iter()
            .any(|output| output.id() == face.id())
        {
            assert!(accounted.insert(face.id()));
        } else {
            let replacements: Vec<_> = result
                .modified_faces
                .iter()
                .filter(|(id, _)| *id == face.id())
                .collect();
            assert_eq!(replacements.len(), 1);
            assert!(accounted.insert(replacements[0].1.id()));
        }
    }
    assert_eq!(result.generated_faces.len(), 2);
    for face in &result.generated_faces {
        assert!(accounted.insert(face.id()));
    }
    assert_eq!(accounted.len(), result.solid.face_iter().count());
    assert!(result
        .solid
        .face_iter()
        .all(|face| accounted.contains(&face.id())));
}

#[test]
fn plane_cone_rims_preserve_radius_and_distances() {
    for slope in [-0.25, 0.25] {
        for analytic in [true, false] {
            for transform in [
                Matrix4::identity(),
                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
            ] {
                let input = builder::transformed(&frustum(slope, analytic), transform);
                assert!(input.is_geometric_consistent());
                common::assert_solid(&input, input_volume(slope), &[0], TOL);
                let before = serde_json::to_string(&input.compress()).unwrap();
                let inverse = transform.invert().unwrap();
                let wire = input
                    .face_iter()
                    .find(|face| {
                        face.vertex_iter()
                            .all(|vertex| inverse.transform_point(vertex.point()).z.near(&8.))
                    })
                    .unwrap()
                    .boundaries()[0]
                    .clone();
                for reversed in [false, true] {
                    eprintln!("cone case: slope={slope}, analytic={analytic}, transformed={}, reversed={reversed}", transform != Matrix4::identity());
                    let wire = if reversed {
                        wire.inverse()
                    } else {
                        wire.clone()
                    };
                    let result = try_fillet_solid_along_wire(&input, &wire, 0.5, TOL)
                        .unwrap_or_else(|e| {
                            panic!("slope={slope}, analytic={analytic}, reversed={reversed}: {e:?}")
                        });
                    assert_dimensions(&result, slope, transform, true, false);
                    assert_history(&input, &result);
                    common::assert_solid(&result.solid, fillet_volume(slope, 0.5), &[0], TOL);
                    common::blend::assert_step(&result.solid, fillet_volume(slope, 0.5), TOL);
                    let (a, b) = if reversed { (0.3, 0.5) } else { (0.5, 0.3) };
                    let result = try_chamfer_solid_along_wire(&input, &wire, a, b, TOL)
                        .unwrap_or_else(|e| {
                            panic!("slope={slope}, analytic={analytic}, reversed={reversed}: {e:?}")
                        });
                    assert_dimensions(&result, slope, transform, false, false);
                    assert_history(&input, &result);
                    let q = (1. + slope * slope).sqrt();
                    let area = 0.5 * 0.5 * 0.3 / q;
                    let centroid = 5. + 8. * slope - (0.5 + slope * 0.3 / q) / 3.;
                    let expected = input_volume(slope) - 2. * PI * area * centroid;
                    common::assert_solid(&result.solid, expected, &[0], TOL);
                    common::blend::assert_step(&result.solid, expected, TOL);
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
        }
    }
}

#[test]
fn concave_plane_cone_rims_preserve_cavity_orientation() {
    for slope in [-0.25, 0.25] {
        let outer =
            common::modeling::cuboid(Point3::new(-12., -12., -1.), Point3::new(12., 12., 10.));
        let mut cavity = frustum(slope, true);
        cavity.not();
        let mut shells = outer.boundaries().clone();
        shells.extend(cavity.boundaries().clone());
        let input = Solid::new(shells);
        common::assert_solid(&input, 6336. - input_volume(slope), &[0, 0], TOL);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let wire = input
            .face_iter()
            .find(|face| face.vertex_iter().all(|vertex| vertex.point().z.near(&8.)))
            .unwrap()
            .boundaries()[0]
            .clone();
        for fillet in [true, false] {
            let result = if fillet {
                try_fillet_solid_along_wire(&input, &wire, 0.5, TOL)
            } else {
                try_chamfer_solid_along_wire(&input, &wire, 0.5, 0.3, TOL)
            }
            .unwrap();
            let cavity_volume = if fillet {
                fillet_volume(slope, 0.5)
            } else {
                let q = (1. + slope * slope).sqrt();
                let area = 0.5 * 0.5 * 0.3 / q;
                let centroid = 5. + 8. * slope - (0.5 + slope * 0.3 / q) / 3.;
                input_volume(slope) - 2. * PI * area * centroid
            };
            assert_dimensions(&result, slope, Matrix4::identity(), fillet, true);
            assert_history(&input, &result);
            common::assert_solid(&result.solid, 6336. - cavity_volume, &[0, 0], TOL);
            common::blend::assert_step(&result.solid, 6336. - cavity_volume, TOL);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
