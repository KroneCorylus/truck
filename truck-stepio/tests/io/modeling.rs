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
    let (shell, skipped) = table.to_compressed_shell(shell).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
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
    let (solid, skipped) = table.to_compressed_solid(step_solid).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    solid
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
        let (compressed, skipped) = table.to_compressed_solid(step_solid).unwrap();
        assert!(skipped.is_empty(), "{name}: {skipped:?}");
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

/// A profile in the `xz`-plane that revolves into a cylinder, a cone, a piece of a sphere and a
/// piece of a torus, besides planes, oriented for `rsweep` about `z`.
fn vase_profile() -> Face {
    let point = |x: f64, z: f64| Point3::new(x, 0.0, z);
    let s60 = f64::sqrt(3.0) / 2.0;
    let p = [
        point(0.5, 0.0),
        point(0.5, 4.0 + 2.0 * s60 + 0.5),
        point(1.5, 4.0 + 2.0 * s60 + 0.5),
        point(1.0, 4.0 + 2.0 * s60),
        point(2.0, 4.0),
        point(3.0, 2.0),
        point(3.0, 0.0),
    ];
    let v: Vec<Vertex> = p.iter().map(|p| builder::vertex(*p)).collect();
    let torus_transit = point(
        1.5 - 0.5 / f64::sqrt(2.0),
        4.0 + 2.0 * s60 + 0.5 / f64::sqrt(2.0),
    );
    let sphere_transit = point(2.0 * s60, 4.0 + 1.0);
    let wire: Wire = vec![
        builder::line(&v[0], &v[1]),
        builder::line(&v[1], &v[2]),
        builder::circle_arc(&v[2], &v[3], torus_transit),
        builder::circle_arc(&v[3], &v[4], sphere_transit),
        builder::line(&v[4], &v[5]),
        builder::line(&v[5], &v[6]),
        builder::line(&v[6], &v[0]),
    ]
    .into_iter()
    .collect();
    builder::try_attach_plane(&[wire]).unwrap()
}

/// Volume of the revolution of `profile` about the `z` axis: `2π ∮ x²/2 dz` around the
/// profile, by composite Simpson on each edge.
fn revolved_volume(profile: &Face) -> f64 {
    const N: usize = 2000;
    let mut integral = 0.0;
    for edge in profile.boundaries()[0].edge_iter() {
        let curve = edge.oriented_curve();
        let (t0, t1) = curve.range_tuple();
        let h = (t1 - t0) / N as f64;
        for i in 0..=N {
            let weight = match (i == 0 || i == N, i % 2) {
                (true, _) => 1.0,
                (false, 1) => 4.0,
                (false, _) => 2.0,
            };
            let t = t0 + h * i as f64;
            let (p, d) = (curve.subs(t), curve.der(t));
            integral += weight * p.x * p.x / 2.0 * d.z * h / 3.0;
        }
    }
    2.0 * PI * integral.abs()
}

fn elementary_kinds(solid: &CompressedSolid<Point3, Curve, Surface>) -> Vec<(Elementary, bool)> {
    solid.boundaries[0]
        .faces
        .iter()
        .map(|face| {
            let mut surface = face.surface.clone();
            if !face.orientation {
                surface.invert();
            }
            surface
                .elementary()
                .unwrap_or_else(|| panic!("not elementary: {surface:?}"))
        })
        .collect()
}

/// Whether two faces are on the same elementary surface with the same normal. A plane's
/// `outward` is relative to the plane's own normal, which the round trip need not keep, so
/// planes are compared by their oriented normal.
fn same_elementary(
    (a, outward_a): &(Elementary, bool),
    (b, outward_b): &(Elementary, bool),
) -> bool {
    const EPS: f64 = 1e-9;
    let near = |x: f64, y: f64| (x - y).abs() < EPS;
    let same_point = |p: Point3, q: Point3| p.distance(q) < EPS;
    let same_axis = |u: Vector3, v: Vector3| u.cross(v).magnitude() < EPS;
    let on_axis = |p: Point3, q: Point3, axis: Vector3| (p - q).cross(axis).magnitude() < EPS;
    let sign = |outward: bool| if outward { 1.0 } else { -1.0 };
    if let (Elementary::Plane(p), Elementary::Plane(q)) = (a, b) {
        let coplanar = (q.origin() - p.origin()).dot(p.normal()).abs() < EPS;
        return coplanar
            && (p.normal() * sign(*outward_a)).distance(q.normal() * sign(*outward_b)) < EPS;
    }
    if outward_a != outward_b {
        return false;
    }
    match (a, b) {
        (
            Elementary::Cylinder {
                origin: o,
                axis: u,
                radius: r,
            },
            Elementary::Cylinder {
                origin: p,
                axis: v,
                radius: s,
            },
        ) => same_axis(*u, *v) && on_axis(*o, *p, *u) && near(*r, *s),
        (
            Elementary::Cone {
                apex: o,
                axis: u,
                half_angle: a,
            },
            Elementary::Cone {
                apex: p,
                axis: v,
                half_angle: b,
            },
        ) => same_point(*o, *p) && u.distance(*v) < EPS && near(a.0, b.0),
        (
            Elementary::Sphere {
                center: o,
                radius: r,
            },
            Elementary::Sphere {
                center: p,
                radius: s,
            },
        ) => same_point(*o, *p) && near(*r, *s),
        (
            Elementary::Torus {
                center: o,
                axis: u,
                major: r0,
                minor: r1,
            },
            Elementary::Torus {
                center: p,
                axis: v,
                major: s0,
                minor: s1,
            },
        ) => same_point(*o, *p) && same_axis(*u, *v) && near(*r0, *s0) && near(*r1, *s1),
        _ => false,
    }
}

#[test]
fn revolved_solid_is_written_with_elementary_surfaces() {
    let profile = vase_profile();
    let vase: Solid = builder::rsweep(&profile, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    let compressed = vase.compress();
    let step = to_step(&compressed);
    for entity in [
        "CYLINDRICAL_SURFACE",
        "CONICAL_SURFACE",
        "SPHERICAL_SURFACE",
        "TOROIDAL_SURFACE",
        "PLANE",
    ] {
        assert!(step.contains(entity), "no {entity}");
    }
    assert!(!step.contains("B_SPLINE"), "{step}");
    assert!(!step.contains("SURFACE_OF_REVOLUTION"), "{step}");

    let read = read_back(&step);
    let original = elementary_kinds(&compressed);
    let reimported = elementary_kinds(&read);
    assert_eq!(original.len(), reimported.len());
    for face in &reimported {
        assert!(
            original.iter().any(|k| same_elementary(k, face)),
            "{face:?} has no original face"
        );
    }

    let expected = revolved_volume(&profile);
    for (name, solid) in [("original", &compressed), ("reimported", &read)] {
        let volume = solid.triangulation(0.001).to_polygon().volume();
        assert!(
            (volume - expected).abs() < 5e-3 * expected,
            "{name}: volume {volume}, expected {expected}"
        );
    }
}

#[test]
fn conic_torus_round_trips_with_positive_volume() {
    let (major, minor) = (3.0, 0.75);
    let vertex = builder::vertex(Point3::new(major + minor, 0.0, 0.0));
    let tube: Wire = builder::rsweep(
        &vertex,
        Point3::new(major, 0.0, 0.0),
        Vector3::unit_y(),
        Rad(7.0),
        3,
    );
    let tube = builder::try_attach_plane(&[tube]).unwrap();
    let torus: Solid = builder::rsweep(&tube, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    let step = to_step(&torus.compress());
    assert!(step.contains("TOROIDAL_SURFACE"), "{step}");
    assert!(!step.contains("SURFACE_OF_REVOLUTION"), "{step}");

    let volume = read_back_volume(&step);
    let expected = 2.0 * PI * PI * major * minor * minor;
    assert!(
        volume > 0.0 && (volume - expected).abs() < 5e-3 * expected,
        "volume {volume}, expected {expected}"
    );
}

#[test]
fn elliptical_arc_is_written_as_ellipse() {
    let transform = Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0))
        * Matrix4::from_nonuniform_scale(2.0, 1.0, 1.0);
    let arc = Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0)),
        transform,
    ));
    let (front, back) = (arc.front(), arc.back());
    let edge = Edge::new(&builder::vertex(front), &builder::vertex(back), arc);
    let face: Face = builder::tsweep(&edge, Vector3::unit_z());
    let shell: truck_modeling::Shell = vec![face].into();
    let compressed = shell.compress();
    let design = StepDesign::from_model(StepModel::from(&compressed));
    let step = StepDisplay::new(Default::default(), design).to_string();
    assert!(step.contains("ELLIPSE"), "{step}");
    assert!(!step.contains("B_SPLINE"), "{step}");
}
