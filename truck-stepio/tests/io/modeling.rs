//! Solids modelled with `truck-modeling`, written to STEP and read back.

use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_stepio::{out::*, r#in::*};
use truck_topology::compress::CompressedSolid;

fn to_step(solid: &CompressedSolid<Point3, Curve, Surface>) -> String {
    let design = StepDesign::from_model(StepModel::from(solid));
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
    let step = to_step(&cylinder.compress());
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
    let step = to_step(&skewed.compress());
    assert!(step.contains("SURFACE_OF_LINEAR_EXTRUSION"), "{step}");
    assert!(!step.contains("CYLINDRICAL_SURFACE"), "{step}");
}

/// The first solid of `step`, mapped onto the `truck-modeling` enums.
fn read_back(step: &str) -> CompressedSolid<Point3, Curve, Surface> {
    let table = Table::from_step(step).unwrap();
    let step_solid = table.manifold_solid_brep.values().next().unwrap();
    table
        .to_compressed_solid(step_solid)
        .unwrap()
        .try_mapped(
            |p| Some(*p),
            |c| c.try_into().map_err(|e| eprintln!("{e}")).ok(),
            |s| s.try_into().map_err(|e| eprintln!("{e}")).ok(),
        )
        .unwrap()
}

#[test]
fn cylinder_survives_a_round_trip() {
    let (radius, height) = (1.5, 2.0);
    let disk = disk(Point3::new(1.0, -2.0, 0.5), radius);
    let cylinder: Solid = builder::tsweep(&disk, Vector3::unit_z() * height);
    let read = read_back(&to_step(&cylinder.compress()));

    let shell = &read.boundaries[0];
    for face in &shell.faces {
        match &face.surface {
            Surface::Plane(_) => {}
            Surface::Extruded(extruded) => {
                assert!(matches!(extruded.entity_curve(), Curve::Conic(_)))
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(shell
        .faces
        .iter()
        .any(|face| matches!(face.surface, Surface::Extruded(_))));
    for edge in &shell.edges {
        assert!(matches!(edge.curve, Curve::Line(_) | Curve::Conic(_)));
    }

    let step = to_step(&read);
    assert!(step.contains("CYLINDRICAL_SURFACE"), "{step}");
    assert!(!step.contains("B_SPLINE"), "{step}");

    let volume = read.triangulation(0.001).to_polygon().volume();
    let expected = PI * radius * radius * height;
    assert!(
        (volume - expected).abs() < 0.05,
        "volume {volume}, expected {expected}"
    );
}

/// Every elementary surface OCC writes maps onto an exact surface with the same orientation.
#[test]
fn occt_primitives_map_onto_modeling() {
    use truck_topology::shell::ShellCondition;
    for name in [
        "occt-cone",
        "occt-cube",
        "occt-cylinder",
        "occt-sphere",
        "occt-torus",
    ] {
        let path = format!(
            "{}/../resources/step/{name}.step",
            env!("CARGO_MANIFEST_DIR")
        );
        let step = std::fs::read_to_string(path).unwrap();
        let table = Table::from_step(&step).unwrap();
        let step_solid = table.manifold_solid_brep.values().next().unwrap();
        let compressed = table.to_compressed_solid(step_solid).unwrap();
        let expected = compressed.triangulation(0.01).to_polygon().volume();

        let solid = read_back(&step);
        let mut mesh = solid.triangulation(0.01).to_polygon();
        mesh.put_together_same_attrs(TOLERANCE * 50.0)
            .remove_degenerate_faces();
        assert_eq!(mesh.shell_condition(), ShellCondition::Closed, "{name}");
        let volume = mesh.volume();
        assert!(
            expected > 0.0 && (volume - expected).abs() < 0.02 * expected,
            "{name}: volume {volume}, expected {expected}"
        );
    }
}
