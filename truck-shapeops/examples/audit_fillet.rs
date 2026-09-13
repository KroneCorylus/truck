//! Characterizes ordinary modeling coverage; observed errors are gaps, not desired contracts.
#[path = "../tests/common/mod.rs"]
mod common;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::*;
const TOL: f64 = 0.001;
fn prism(points: &[(f64, f64)]) -> Solid {
    let v = builder::vertices(points.iter().map(|&(x, y)| Point3::new(x, y, 0.)));
    let w: Wire = (0..v.len())
        .map(|i| builder::line(&v[i], &v[(i + 1) % v.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[w]).unwrap(),
        Vector3::unit_z() * 10.,
    )
}
fn probe(label: &str, input: &Solid, points: &[Point3], radius: f64) {
    assert!(input.is_geometric_consistent(), "input geometry");
    let before = serde_json::to_string(&input.compress()).unwrap();
    let ids: Vec<_> = points
        .iter()
        .map(|&p| common::blend::edge_through(&input.boundaries()[0], p).id())
        .collect();
    let result = try_fillet_solid_edges(input, &ids, radius, TOL);
    match result {
        Ok(r) => {
            eprintln!(
                "AUDIT {label}: OK geometric={} faces={}",
                r.solid.is_geometric_consistent(),
                r.generated_faces.len()
            );
            inspect(&r.solid, 0);
        }
        Err(e) => eprintln!("AUDIT {label}: {e}"),
    };
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
fn characterize_planar_junctions() {
    let l = prism(&[
        (0., 0.),
        (20., 0.),
        (20., 10.),
        (10., 10.),
        (10., 20.),
        (0., 20.),
    ]);
    for n in [1, 2, 3] {
        probe(
            &format!("L corner count {n}"),
            &l,
            &[
                Point3::new(10., 10., 5.),
                Point3::new(15., 10., 10.),
                Point3::new(10., 15., 10.),
            ][..n],
            1.,
        );
    }
    let skew = prism(&[(0., 0.), (20., 0.), (25., 10.), (5., 10.)]);
    for n in [1, 2, 3] {
        probe(
            &format!("oblique convex count {n}"),
            &skew,
            &[
                Point3::new(0., 0., 5.),
                Point3::new(10., 0., 10.),
                Point3::new(2.5, 5., 10.),
            ][..n],
            1.,
        );
    }
    let skew_l = prism(&[
        (0., 0.),
        (20., 0.),
        (25., 10.),
        (15., 10.),
        (20., 20.),
        (10., 20.),
    ]);
    for n in [1, 2, 3] {
        probe(
            &format!("oblique concave count {n}"),
            &skew_l,
            &[
                Point3::new(15., 10., 5.),
                Point3::new(20., 10., 10.),
                Point3::new(17.5, 15., 10.),
            ][..n],
            1.,
        );
    }
}
fn rim(input: &Solid, z: f64) -> Wire {
    let plane = input
        .face_iter()
        .find(|f| {
            matches!(f.surface(), Surface::Plane(_))
                && f.vertex_iter().all(|v| (v.point().z - z).abs() < 1e-6)
        })
        .unwrap();
    plane
        .boundaries()
        .iter()
        .find(|w| w.iter().all(|e| !matches!(e.curve(), Curve::Line(_))))
        .unwrap()
        .clone()
}
fn probe_wire(label: &str, input: &Solid, wire: &Wire, radius: f64, genus: usize) {
    assert!(input.is_geometric_consistent(), "input geometry");
    let before = serde_json::to_string(&input.compress()).unwrap();
    match try_fillet_solid_along_wire(input, wire, radius, TOL) {
        Ok(r) => {
            eprintln!(
                "AUDIT {label}: OK geometric={} faces={}",
                r.solid.is_geometric_consistent(),
                r.generated_faces.len()
            );
            inspect(&r.solid, genus);
        }
        Err(e) => eprintln!("AUDIT {label}: {e}"),
    };
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
fn characterize_hole_pocket_and_boss_rims() {
    let block = common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., 10.));
    for blind in [false, true] {
        let mut cutter = common::modeling::cylinder(
            Point3::new(0., 0., if blind { 5. } else { -1. }),
            Vector3::unit_z(),
            3.,
            12.,
        );
        cutter.not();
        let input = truck_shapeops::and(&block, &cutter, TOL).unwrap();
        for z in if blind { vec![10., 5.] } else { vec![10., 0.] } {
            let w = rim(&input, z);
            probe_wire(
                &format!("{} rim z{z}", if blind { "pocket" } else { "through hole" }),
                &input,
                &w,
                0.5,
                usize::from(!blind),
            );
            probe_wire(
                &format!(
                    "reversed {} rim z{z}",
                    if blind { "pocket" } else { "through hole" }
                ),
                &input,
                &w.inverse(),
                0.5,
                usize::from(!blind),
            );
        }
    }
    let cylinder = common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 3., 10.);
    probe_wire("cylinder top rim", &cylinder, &rim(&cylinder, 10.), 0.5, 0);
    let boss = truck_shapeops::or(
        &block,
        &common::modeling::cylinder(Point3::new(0., 0., 9.), Vector3::unit_z(), 3., 6.),
        TOL,
    )
    .unwrap();
    probe_wire("boss foot rim", &boss, &rim(&boss, 10.), 0.5, 0);
}

fn inspect(solid: &Solid, genus: usize) {
    common::assert_topology(solid, &[genus]);
    for (i, face) in solid
        .face_iter()
        .enumerate()
        .filter(|(_, f)| !f.is_geometric_consistent())
    {
        let kind = match face.surface() {
            Surface::Plane(_) => "Plane",
            Surface::NurbsSurface(_) => "NurbsSurface",
            Surface::Fillet(_) => "Fillet",
            _ => "other",
        };
        eprintln!("  inconsistent face {i} {kind}");
    }
    let mut mesh = solid.triangulation(TOL).to_polygon();
    mesh.put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    eprintln!(
        "  mesh={:?} volume={}",
        mesh.shell_condition(),
        mesh.volume()
    );
}
fn characterize_large_valid_radius() {
    let cube = common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.));
    for radius in [4.9, 5., 5.1, 8.] {
        for n in [1, 3] {
            probe(
                &format!("cube count {n} radius {radius}"),
                &cube,
                &[
                    Point3::new(0., 0., 5.),
                    Point3::new(5., 0., 10.),
                    Point3::new(0., 5., 10.),
                ][..n],
                radius,
            );
        }
        let e = common::blend::edge_through(&cube.boundaries()[0], Point3::new(0., 0., 5.));
        probe_wire(
            &format!("cube single wire radius {radius}"),
            &cube,
            &vec![e].into(),
            radius,
            0,
        );
    }
}
fn main() {
    characterize_large_valid_radius();
    characterize_planar_junctions();
    characterize_hole_pocket_and_boss_rims();
}
