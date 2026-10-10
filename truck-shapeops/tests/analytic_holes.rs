mod common;
use common::{assert_solid, modeling::*};
use std::f64::consts::PI;
use truck_modeling::*;

const TOL: f64 = 0.001;

fn retained(before: &Solid, after: &Solid) -> usize {
    after
        .face_iter()
        .filter(|f| before.face_iter().any(|old| old.id() == f.id()))
        .count()
}

fn compound(solids: &[Solid]) -> Solid {
    Solid::new(
        solids
            .iter()
            .flat_map(|s| s.boundaries().iter().cloned())
            .collect(),
    )
}

#[test]
fn batch_holes_preserve_faces_and_exact_curves_under_transforms() {
    for moved in [false, true] {
        let transform = |s: &Solid| {
            if moved {
                builder::transformed(
                    s,
                    Matrix4::from_translation(Vector3::new(100., -30., 5.))
                        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.71)),
                )
            } else {
                s.clone()
            }
        };
        let plate = transform(&cuboid(Point3::origin(), Point3::new(16., 12., 2.)));
        let mut tools = Vec::new();
        let mut removed = 0.;
        for i in 0..6 {
            let radius = 0.4 + 0.2 * (i % 3) as f64;
            let axis = if i % 2 == 0 {
                Vector3::unit_z()
            } else {
                -Vector3::unit_z()
            };
            let center = Point3::new(
                2. + (i % 3) as f64 * 4.,
                2. + (i / 3) as f64 * 4.,
                if axis.z > 0. { -1. } else { 3. },
            );
            let wire = primitive::circle(
                center + radius * Vector3::unit_x(),
                center,
                axis,
                2 + i % 3 * 2,
            );
            let disk = builder::try_attach_plane(&[wire]).unwrap();
            tools.push(transform(&builder::tsweep(&disk, 4. * axis)));
            removed += 2. * PI * radius * radius;
        }
        for reverse in [false, true] {
            if reverse {
                tools.reverse();
            }
            let batch = compound(&tools);
            let inputs = serde_json::to_string(&[plate.compress(), batch.compress()]).unwrap();
            let cut = truck_shapeops::try_subtract_with_effect(&plate, &batch, TOL).unwrap();
            assert!(cut.removed_material);
            assert_eq!(retained(&plate, &cut.solid), 4);
            assert!(cut
                .solid
                .edge_iter()
                .all(|e| matches!(e.curve(), Curve::Line(_) | Curve::Conic(_))));
            assert!(cut.solid.is_geometric_consistent());
            assert_solid(&cut.solid, 384. - removed, &[6], TOL);
            assert_eq!(
                inputs,
                serde_json::to_string(&[plate.compress(), batch.compress()]).unwrap()
            );
            if moved && reverse {
                common::blend::assert_step(&cut.solid, 384. - removed, TOL);
            }
        }
    }
}

#[test]
fn batch_combines_new_holes_and_empty_space_without_rebuilding_old_walls() {
    let plate = cuboid(Point3::origin(), Point3::new(16., 8., 2.));
    let bore = cylinder(Point3::new(2., 2., -1.), Vector3::unit_z(), 1., 4.);
    let plate = truck_shapeops::try_subtract(&plate, &bore, TOL).unwrap();
    let empty = [
        cylinder(Point3::new(2., 2., -1.), Vector3::unit_z(), 0.5, 4.),
        cylinder(Point3::new(20., 2., -1.), Vector3::unit_z(), 1., 4.),
    ];
    let unchanged =
        truck_shapeops::try_subtract_with_effect(&plate, &compound(&empty), TOL).unwrap();
    assert!(!unchanged.removed_material);
    assert_eq!(
        retained(&plate, &unchanged.solid),
        plate.face_iter().count()
    );
    let mut tools = empty.to_vec();
    tools.push(cylinder(
        Point3::new(6., 2., -1.),
        Vector3::unit_z(),
        1.,
        4.,
    ));
    tools.push(cylinder(
        Point3::new(10., 2., -1.),
        Vector3::unit_z(),
        0.5,
        4.,
    ));
    let cut = truck_shapeops::try_subtract_with_effect(&plate, &compound(&tools), TOL).unwrap();
    assert!(cut.removed_material);
    assert_eq!(retained(&plate, &cut.solid), plate.face_iter().count() - 2);
    assert_solid(&cut.solid, 256. - 4.5 * PI, &[3], TOL);
    assert!(cut.solid.is_geometric_consistent());
}

#[test]
fn mixed_through_and_blind_batch_preserves_general_boolean_behavior() {
    let plate = cuboid(Point3::origin(), Point3::new(12., 8., 2.));
    let tools = [
        cylinder(Point3::new(3., 3., -1.), Vector3::unit_z(), 1., 4.),
        cylinder(Point3::new(8., 3., 1.), Vector3::unit_z(), 0.5, 2.),
    ];
    let cut = truck_shapeops::try_subtract(&plate, &compound(&tools), TOL).unwrap();
    assert_solid(&cut, 192. - 2.25 * PI, &[1], TOL);
    assert!(cut.is_geometric_consistent());
}

#[test]
fn through_hole_preserves_remote_faces_and_exact_circles() {
    let plate = cuboid(Point3::origin(), Point3::new(12., 8., 2.));
    let cutter = cylinder(Point3::new(3., 3., -1.), Vector3::unit_z(), 1., 4.);
    let inputs = serde_json::to_string(&[plate.compress(), cutter.compress()]).unwrap();
    let cut = truck_shapeops::try_subtract_with_effect(&plate, &cutter, TOL).unwrap();
    assert!(cut.removed_material);
    assert_eq!(retained(&plate, &cut.solid), 4);
    assert!(cut
        .solid
        .edge_iter()
        .all(|e| matches!(e.curve(), Curve::Line(_) | Curve::Conic(_))));
    assert_solid(&cut.solid, 192. - 2. * PI, &[1], TOL);
    assert!(cut.solid.is_geometric_consistent());
    assert_eq!(
        inputs,
        serde_json::to_string(&[plate.compress(), cutter.compress()]).unwrap()
    );
}

#[test]
fn repeated_holes_under_rigid_transforms_keep_unchanged_faces() {
    let transform = |s: &Solid| {
        let s = builder::rotated(
            s,
            Point3::origin(),
            Vector3::new(1., 2., 3.).normalize(),
            Rad(0.71),
        );
        builder::translated(&s, Vector3::new(1000., -300., 50.))
    };
    let mut plate = transform(&cuboid(Point3::origin(), Point3::new(16., 8., 2.)));
    for i in 0..8 {
        let tool = transform(&cylinder(
            Point3::new(2. + (i % 4) as f64 * 4., 2. + (i / 4) as f64 * 4., -1.),
            Vector3::unit_z(),
            1.,
            4.,
        ));
        let cut = truck_shapeops::try_subtract(&plate, &tool, TOL).unwrap();
        assert_eq!(retained(&plate, &cut), plate.face_iter().count() - 2);
        assert_solid(&cut, 256. - (i + 1) as f64 * 2. * PI, &[i + 1], TOL);
        assert!(cut.is_geometric_consistent());
        plate = cut;
    }
}

#[test]
fn reversed_axis_and_many_arc_segments_make_exact_holes() {
    let plate = cuboid(Point3::origin(), Point3::new(12., 8., 2.));
    let base = Point3::new(3., 3., 3.);
    let circle = primitive::circle(base + Vector3::unit_x(), base, -Vector3::unit_z(), 6);
    let disk = builder::try_attach_plane(&[circle]).unwrap();
    let cutter: Solid = builder::tsweep(&disk, -4. * Vector3::unit_z());
    let cut = truck_shapeops::try_subtract(&plate, &cutter, TOL).unwrap();
    assert_eq!(retained(&plate, &cut), 4);
    assert_eq!(cut.face_iter().count(), 12);
    assert!(cut.is_geometric_consistent());
    assert_solid(&cut, 192. - 2. * PI, &[1], TOL);
    common::blend::assert_step(&cut, 192. - 2. * PI, TOL);
}

#[test]
fn thin_cylindrical_wall_and_empty_space_preserve_faces() {
    let body = cylinder(Point3::origin(), Vector3::unit_z(), 4., 8.);
    let tool = cylinder(Point3::new(0., 0., -1.), Vector3::unit_z(), 3.99, 10.);
    let cut = truck_shapeops::try_subtract(&body, &tool, TOL).unwrap();
    assert_eq!(retained(&body, &cut), 2);
    assert_solid(&cut, PI * (16. - 3.99_f64.powi(2)) * 8., &[1], 0.00001);
    for x in [0., 10.] {
        let empty = cylinder(Point3::new(x, 0., -1.), Vector3::unit_z(), 1., 10.);
        let next = truck_shapeops::try_subtract_with_effect(&cut, &empty, TOL).unwrap();
        assert!(!next.removed_material);
        assert_eq!(retained(&cut, &next.solid), cut.face_iter().count());
        assert_eq!(
            serde_json::to_string(&cut.compress()).unwrap(),
            serde_json::to_string(&next.solid.compress()).unwrap()
        );
    }
}

#[test]
fn overlapping_and_blind_cuts_after_an_exact_hole_use_the_general_solver() {
    let plate = cuboid(Point3::origin(), Point3::new(12., 8., 2.));
    let tool = cylinder(Point3::new(3., 3., -1.), Vector3::unit_z(), 1., 4.);
    let first = truck_shapeops::try_subtract(&plate, &tool, TOL).unwrap();
    let adjacent = builder::translated(&tool, Vector3::unit_x());
    let overlap = truck_shapeops::try_subtract(&first, &adjacent, TOL).unwrap();
    assert_solid(&overlap, 192. - 8. * PI / 3. - 3.0_f64.sqrt(), &[1], TOL);
    assert!(overlap.is_geometric_consistent());
    let blind = cylinder(Point3::new(8., 3., 1.), Vector3::unit_z(), 1., 2.);
    let pocket = truck_shapeops::try_subtract(&first, &blind, TOL).unwrap();
    assert_solid(&pocket, 192. - 3. * PI, &[1], TOL);
    assert!(pocket.is_geometric_consistent());
}

#[test]
fn concave_outline_uses_material_winding() {
    let vertices: Vec<_> = [(0., 0.), (8., 0.), (8., 2.), (2., 2.), (2., 8.), (0., 8.)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.)))
        .into();
    let wire: Wire = (0..6)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 6]))
        .collect();
    let face = builder::try_attach_plane(&[wire]).unwrap();
    let body: Solid = builder::tsweep(&face, 2. * Vector3::unit_z());
    let outside = cylinder(Point3::new(5., 5., -1.), Vector3::unit_z(), 0.5, 4.);
    let unchanged = truck_shapeops::try_subtract_with_effect(&body, &outside, TOL).unwrap();
    assert!(!unchanged.removed_material);
    assert_eq!(retained(&body, &unchanged.solid), 8);
    let tool = cylinder(Point3::new(1., 5., -1.), Vector3::unit_z(), 0.5, 4.);
    let cut = truck_shapeops::try_subtract(&body, &tool, TOL).unwrap();
    assert_eq!(retained(&body, &cut), 6);
    assert_solid(&cut, 56. - PI / 2., &[1], TOL);
    assert!(cut.is_geometric_consistent());
}

#[test]
fn cap_parameter_origins_do_not_change_the_cut_height() {
    let shear = Matrix4::from_cols(
        Vector4::new(1., 0., 1e-13, 0.),
        Vector4::unit_y(),
        Vector4::unit_z(),
        Vector4::unit_w(),
    );
    let body = builder::transformed(&cuboid(Point3::origin(), Point3::new(8., 8., 2.)), shear);
    let body = body.mapped(
        |p| *p,
        Clone::clone,
        |surface| {
            if let Surface::Plane(plane) = surface {
                if plane.normal().z.abs() > 0.99 {
                    let sign = if plane.origin().z < 1. { -1. } else { 1. };
                    return Surface::Plane(plane.transformed(Matrix4::from_translation(
                        sign * Vector3::new(1e9, 0., 1e-4),
                    )));
                }
            }
            surface.clone()
        },
    );
    assert!(body.is_geometric_consistent());
    let tool = cylinder(Point3::new(4., 4., -1.), Vector3::unit_z(), 1., 4.);
    let cut = truck_shapeops::try_subtract(&body, &tool, TOL).unwrap();
    assert_eq!(retained(&body, &cut), 4);
    assert!(cut.is_geometric_consistent());
    assert_solid(&cut, 128. - 2. * PI, &[1], TOL);
}
