//! Solids modelled with `truck-modeling`, written to STEP and read back.

use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_stepio::{out::*, r#in::*};

fn to_step(solid: &Solid) -> String {
    let compressed = solid.compress();
    let design = StepDesign::from_model(StepModel::from(&compressed));
    StepDisplay::new(Default::default(), design).to_string()
}

/// Signed volume of the mesh of the first shell in `step`, positive when the faces read back
/// with outward normals.
fn read_back_volume(step: &str) -> f64 {
    let table = Table::from_step(step).unwrap();
    let shell = table.shell.values().next().unwrap();
    let shell = table.to_compressed_shell(shell).unwrap();
    shell.triangulation(0.001).to_polygon().volume()
}

fn disk(center: Point3, radius: f64) -> Face {
    let vertex = builder::vertex(center + Vector3::unit_x() * radius);
    let circle: Wire = builder::rsweep(&vertex, center, Vector3::unit_z(), Rad(7.0), 3);
    builder::try_attach_plane(&[circle]).unwrap()
}

#[test]
fn cylinder_is_written_as_cylindrical_surface() {
    let (radius, height) = (1.5, 2.0);
    let disk = disk(Point3::new(1.0, -2.0, 0.5), radius);
    let cylinder: Solid = builder::tsweep(&disk, Vector3::unit_z() * height);
    let step = to_step(&cylinder);
    assert!(step.contains("CYLINDRICAL_SURFACE"), "{step}");
    assert!(step.contains("CIRCLE"), "{step}");
    assert!(!step.contains("B_SPLINE"), "{step}");
    assert!(!step.contains("SURFACE_OF_LINEAR_EXTRUSION"), "{step}");

    let volume = read_back_volume(&step);
    let expected = PI * radius * radius * height;
    assert!(
        (volume - expected).abs() < 0.05,
        "volume {volume}, expected {expected}"
    );
}

#[test]
fn skewed_extrusion_stays_a_linear_extrusion() {
    let disk = disk(Point3::origin(), 1.0);
    let skewed: Solid = builder::tsweep(&disk, Vector3::new(0.0, 0.5, 2.0));
    let step = to_step(&skewed);
    assert!(step.contains("SURFACE_OF_LINEAR_EXTRUSION"), "{step}");
    assert!(!step.contains("CYLINDRICAL_SURFACE"), "{step}");
}
