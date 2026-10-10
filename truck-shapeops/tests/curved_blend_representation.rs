mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

#[derive(Clone, Copy)]
enum Representation {
    Analytic,
    Nurbs,
    ReparameterizedNurbs,
}

fn cylinder(representation: Representation) -> Solid {
    let solid = common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 5., 8.);
    if matches!(representation, Representation::Analytic) {
        return solid;
    }
    solid.mapped(
        |point| *point,
        |curve| Curve::NurbsCurve(NurbsCurve::try_from(curve).unwrap()),
        |surface| match surface {
            Surface::Extruded(surface) => {
                let bottom = NurbsCurve::try_from(surface.entity_curve()).unwrap();
                let top = bottom.transformed(Matrix4::from_translation(surface.extruding_vector()));
                let mut surface = NurbsSurface::new(BSplineSurface::homotopy(
                    bottom.into_non_rationalized(),
                    top.into_non_rationalized(),
                ));
                if matches!(representation, Representation::ReparameterizedNurbs) {
                    surface
                        .elevate_udegree()
                        .elevate_vdegree()
                        .knot_translate(8., -3.);
                }
                surface.into()
            }
            other => other.clone(),
        },
    )
}

fn rim(solid: &Solid) -> Wire {
    solid
        .face_iter()
        .find(|face| face.vertex_iter().all(|vertex| vertex.point().z.near(&8.)))
        .unwrap()
        .boundaries()[0]
        .clone()
}

#[test]
fn analytic_cylinder_rim() { check(Representation::Analytic); }

#[test]
fn nurbs_cylinder_rim() { check(Representation::Nurbs); }

fn check(representation: Representation) {
    let input = cylinder(representation);
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, 200. * PI, &[0], TOL);
    let wire = rim(&input);
    let radius: f64 = 0.5;
    let fillet = try_fillet_solid_along_wire(&input, &wire, radius, TOL).unwrap();
    assert_rim_dimensions(&fillet, true);
    let area = radius.powi(2) * (1. - PI / 4.);
    let centroid = radius * (10. - 3. * PI) / (12. - 3. * PI);
    let volume = 200. * PI - 2. * PI * (5. - centroid) * area;
    common::assert_solid(&fillet.solid, volume, &[0], TOL);
    common::blend::assert_step(&fillet.solid, volume, TOL);
    let chamfer = try_chamfer_solid_along_wire(&input, &wire, 0.5, 0.3, TOL).unwrap();
    assert_rim_dimensions(&chamfer, false);
    let volume = 200. * PI - 2. * PI * (5. - 0.5 / 3.) * 0.5 * 0.3 / 2.;
    common::assert_solid(&chamfer.solid, volume, &[0], TOL);
    common::blend::assert_step(&chamfer.solid, volume, TOL);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

fn assert_rim_dimensions(result: &BlendResult, fillet: bool) {
    assert!(!result.generated_faces.is_empty());
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let (u, v) = surface.try_range_tuple();
        let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
        for i in 0..=4 {
            for j in 0..=4 {
                let (u, v) = (a + (b - a) * i as f64 / 4., c + (d - c) * j as f64 / 4.);
                let p = surface.subs(u, v);
                let rho = p.x.hypot(p.y);
                if fillet {
                    assert!(((rho - 4.5).hypot(p.z - 7.5) - 0.5).abs() <= TOL);
                    let expected =
                        Vector3::new(p.x / rho * (rho - 4.5), p.y / rho * (rho - 4.5), p.z - 7.5)
                            .normalize();
                    assert!(surface.normal(u, v).dot(expected) > 1. - 1e-4);
                } else {
                    let residual = (5. - rho) / 0.5 + (8. - p.z) / 0.3 - 1.;
                    assert!(residual.abs() / 2.0_f64.hypot(1. / 0.3) <= TOL);
                }
            }
        }
    }
}

#[test]
fn reparameterized_nurbs_cylinder_rim() { check(Representation::ReparameterizedNurbs); }

#[test]
#[cfg(feature = "step-test")]
fn short_fillet_parameter_domains_do_not_panic_on_step_export() {
    let input = cylinder(Representation::Nurbs);
    let result = try_fillet_solid_along_wire(&input, &rim(&input), 0.5, TOL).unwrap();
    let mut solid = result.solid.compress();
    for face in solid
        .boundaries
        .iter_mut()
        .flat_map(|shell| &mut shell.faces)
    {
        if let Surface::Fillet(processor) = &face.surface {
            let mut entity = serde_json::to_value(processor.entity()).unwrap();
            for knot in entity["knot_vec"].as_array_mut().unwrap() {
                *knot = serde_json::json!(knot.as_f64().unwrap() * 1e-9);
            }
            let mut scaled = Processor::with_transform(
                serde_json::from_value::<ApproxFilletSurface<Box<Surface>, Box<Surface>>>(entity)
                    .unwrap(),
                *processor.transform(),
            );
            if !processor.orientation() {
                scaled.invert();
            }
            face.surface = Surface::Fillet(scaled);
        }
    }
    let before = serde_json::to_string(&solid).unwrap();
    let prepared = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        truck_stepio::out::prepare_for_step(&solid, TOL / 20.0)
    }));
    assert!(prepared.is_ok(), "an unresolved fit must return None");
    assert_eq!(before, serde_json::to_string(&solid).unwrap());
}
