mod common;

use std::f64::consts::PI;
use truck_modeling::*;

const TOL: f64 = 0.001;

fn plate_cut(x: f64, depth: f64) -> Solid {
    let plate = common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., 10.));
    let origin = Point3::new(x, 0., 10.);
    let start = builder::vertex(origin + Vector3::unit_x() * 3.);
    let rim: Wire = builder::rsweep(&start, origin, -Vector3::unit_z(), Rad(2. * PI), 4);
    let cutter = builder::tsweep(
        &builder::try_attach_plane(&[rim]).unwrap(),
        -Vector3::unit_z() * depth,
    );
    truck_shapeops::try_subtract(&plate, &cutter, TOL).unwrap()
}

#[test]
fn pocket_shell_reintersects_the_cylindrical_floor_join() {
    let input = plate_cut(0., 5.);
    let top = input
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| v.point().z.near(&10.))
                && matches!(face.surface(), Surface::Plane(_))
        })
        .unwrap()
        .id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for outward in [false, true] {
        let output = if outward {
            truck_shapeops::local::try_shell_outward(&input, &[top], 0.5)
        } else {
            truck_shapeops::local::try_shell(&input, &[top], 0.5)
        }
        .unwrap();
        let outer_volume = if outward { 630.5 } else { 570.5 };
        let cup_volume = if outward { 16.875 * PI } else { 22.375 * PI };
        assert!(output.is_geometric_consistent());
        // Removing the annular top separates the outer container from the pocket's cup.
        common::assert_solid(&output, outer_volume + cup_volume, &[0, 0], TOL);
        let parts = truck_shapeops::solid_components(&output, TOL).unwrap();
        assert_eq!(parts.len(), 2);
        let mut volumes = Vec::new();
        for part in &parts {
            let outer = part.vertex_iter().any(|v| v.point().x.abs() > 5.0);
            let volume = if outer { outer_volume } else { cup_volume };
            common::assert_solid(part, volume, &[0], TOL);
            common::blend::assert_step(part, volume, TOL);
            volumes.push(volume);
        }
        assert!(volumes.contains(&outer_volume));
        assert!(volumes.contains(&cup_volume));
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn semicircular_notch_chamfer_trims_both_open_ends() {
    let input = plate_cut(10., 10.);
    let top = input
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| v.point().z.near(&10.))
                && matches!(face.surface(), Surface::Plane(_))
        })
        .unwrap();
    let selected: Vec<_> = top
        .edge_iter()
        .filter(|edge| !matches!(edge.curve(), Curve::Line(_)))
        .collect();
    assert_eq!(selected.len(), 2);
    let mut wire: Wire = selected.into();
    if !wire.is_continuous() {
        wire.swap(0, 1);
    }
    assert!(wire.is_continuous());
    let before = serde_json::to_string(&input.compress()).unwrap();
    for wire in [wire.clone(), wire.inverse()] {
        let output =
            truck_shapeops::fillet::try_chamfer_solid_along_wire(&input, &wire, 0.5, 0.5, TOL)
                .unwrap()
                .solid;
        assert!(output.is_geometric_consistent());
        common::assert_solid(
            &output,
            4000. - 45. * PI - PI / 2. * (3. * 0.5_f64.powi(2) + 0.5_f64.powi(3) / 3.),
            &[0],
            TOL,
        );
    }
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn notch_fillet_meets_perimeter_fillets_at_collapsed_contact_edges() {
    let input = plate_cut(10., 10.);
    let top = input
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| v.point().z.near(&10.))
                && matches!(face.surface(), Surface::Plane(_))
        })
        .unwrap();
    let ids: Vec<_> = top
        .edge_iter()
        .filter(|edge| matches!(edge.curve(), Curve::Line(_)))
        .map(|edge| edge.id())
        .collect();
    let radius = 0.5;
    let straight = truck_shapeops::fillet::try_fillet_solid_edges(&input, &ids, radius, TOL)
        .unwrap()
        .solid;
    let top = straight
        .face_iter()
        .find(|face| {
            face.vertex_iter().all(|v| v.point().z.near(&10.))
                && matches!(face.surface(), Surface::Plane(_))
        })
        .unwrap();
    let mut wire: Wire = top
        .edge_iter()
        .filter(|edge| matches!(edge.curve(), Curve::Conic(_)))
        .collect();
    assert_eq!(wire.len(), 2);
    if !wire.is_continuous() {
        wire.swap(0, 1);
    }
    // Integrate the exact horizontal section: inset rectangle minus a circular segment.
    let section = |angle: f64| {
        let inset = radius * (1. - angle.cos());
        let bore = 3. + inset;
        let segment =
            bore * bore * (inset / bore).acos() - inset * (bore * bore - inset * inset).sqrt();
        ((20. - 2. * inset).powi(2) - segment) * radius * angle.cos()
    };
    let n = 4096;
    let step = PI / (2. * n as f64);
    let cap = (0..=n)
        .map(|i| {
            section(i as f64 * step)
                * if i == 0 || i == n {
                    1.
                } else if i % 2 == 0 {
                    2.
                } else {
                    4.
                }
        })
        .sum::<f64>()
        * step
        / 3.;
    let expected = (10. - radius) * (400. - 4.5 * PI) + cap;
    for wire in [wire.clone(), wire.inverse()] {
        let output =
            truck_shapeops::fillet::try_fillet_solid_along_wire(&straight, &wire, radius, TOL)
                .unwrap()
                .solid;
        assert!(output.is_geometric_consistent());
        common::assert_solid(&output, expected, &[0], TOL);
    }
}
