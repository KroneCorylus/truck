mod common;

use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;
const SIZE: f64 = 10.;

fn cube() -> Solid { common::modeling::cuboid(Point3::origin(), Point3::new(SIZE, SIZE, SIZE)) }

fn edge(solid: &Solid, point: Point3) -> EdgeID {
    common::blend::edge_through(&solid.boundaries()[0], point).id()
}

fn bottom_corner(solid: &Solid) -> Wire { bottom_corner_at(solid, Matrix4::identity()) }

fn bottom_corner_at(solid: &Solid, transform: Matrix4) -> Wire {
    let inverse = transform.invert().unwrap();
    let bottom = solid
        .face_iter()
        .find(|f| {
            f.vertex_iter()
                .all(|v| inverse.transform_point(v.point()).z.abs() < TOLERANCE)
        })
        .unwrap();
    let boundary = bottom.boundaries().remove(0);
    let chosen = |e: &Edge| {
        let c = e.curve();
        let (a, b) = c.range_tuple();
        let p = inverse.transform_point(c.subs((a + b) / 2.));
        p.x < SIZE - TOLERANCE && p.y < SIZE - TOLERANCE
    };
    let start = (0..boundary.len())
        .find(|&i| {
            chosen(&boundary[i]) && !chosen(&boundary[(i + boundary.len() - 1) % boundary.len()])
        })
        .unwrap();
    (0..boundary.len())
        .map(|i| boundary[(start + i) % boundary.len()].clone())
        .take_while(chosen)
        .collect()
}

#[test]
fn collapsed_chamfers_preserve_distance_sides_under_rigid_transforms() {
    let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73));
    let input = builder::transformed(&cube(), transform);
    let rounded = try_fillet_solid_edges(
        &input,
        &[edge(
            &input,
            transform.transform_point(Point3::new(0., 0., 5.)),
        )],
        1.,
        TOL,
    )
    .unwrap()
    .solid;
    let wire = bottom_corner_at(&rounded, transform);
    for (inset, depth) in [(1., 1.), (2., 0.5), (2., 3.)] {
        let base = SIZE.powi(3) - (1. - PI / 4.) * SIZE;
        let volume = base + (chamfered_round_volume(1., inset) - base) * depth / inset;
        for (wire, a, b) in [(wire.clone(), inset, depth), (wire.inverse(), depth, inset)] {
            let result = try_chamfer_solid_along_wire(&rounded, &wire, a, b, TOL).unwrap();
            common::assert_solid(&result.solid, volume, &[0], TOL);
            assert!(result.solid.is_geometric_consistent());
            common::blend::assert_step(&result.solid, volume, TOL);
        }
    }
}

#[test]
fn closed_rim_chamfers_can_consume_all_four_rounds() {
    let input = cube();
    let selected = [(0., 0.), (SIZE, 0.), (SIZE, SIZE), (0., SIZE)]
        .map(|(x, y)| edge(&input, Point3::new(x, y, 5.)));
    let rounded = try_fillet_solid_edges(&input, &selected, 1., TOL)
        .unwrap()
        .solid;
    let wire = rounded
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| v.point().z.abs() < TOLERANCE))
        .unwrap()
        .boundaries()
        .remove(0);
    for distance in [1., 2.] {
        let result =
            try_chamfer_solid_along_wire(&rounded, &wire, distance, distance, TOL).unwrap();
        let corner = 1. - PI / 4.;
        let volume = SIZE.powi(3) - 4. * corner * SIZE - 2. * SIZE * distance * distance
            + 4. / 3. * distance.powi(3)
            + 4. * corner * (distance - 1. / 3.);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        assert!(result.solid.is_geometric_consistent());
        common::blend::assert_step(&result.solid, volume, TOL);
    }
    let before = serde_json::to_string(&rounded.compress()).unwrap();
    for size in [5., 10., 20.] {
        assert!(try_chamfer_solid_along_wire(&rounded, &wire, size, size, TOL).is_err());
        assert_eq!(before, serde_json::to_string(&rounded.compress()).unwrap());
    }
}

#[test]
fn one_rim_can_mix_collapsed_and_regular_circular_contacts() {
    let mut rounded = cube();
    let radii = [1_f64, 2., 3., 1.];
    for ((x, y), radius) in [(0., 0.), (SIZE, 0.), (SIZE, SIZE), (0., SIZE)]
        .into_iter()
        .zip(radii)
    {
        let selected = edge(&rounded, Point3::new(x, y, 5.));
        rounded = try_fillet_solid_edges(&rounded, &[selected], radius, TOL)
            .unwrap()
            .solid;
    }
    let wire = rounded
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| v.point().z.abs() < TOLERANCE))
        .unwrap()
        .boundaries()
        .remove(0);
    let distance = 2_f64;
    let result = try_chamfer_solid_along_wire(&rounded, &wire, distance, distance, TOL).unwrap();
    let corner = 1. - PI / 4.;
    let volume = SIZE.powi(3)
        - corner * SIZE * radii.iter().map(|r| r * r).sum::<f64>()
        - 2. * SIZE * distance * distance
        + 4. / 3. * distance.powi(3)
        + corner
            * radii
                .iter()
                .map(|r| r * r * distance - (r.powi(3) - (r - distance).max(0.).powi(3)) / 3.)
                .sum::<f64>();
    common::assert_solid(&result.solid, volume, &[0], TOL);
    assert!(result.solid.is_geometric_consistent());
    common::blend::assert_step(&result.solid, volume, TOL);
}

fn chamfered_round_volume(radius: f64, distance: f64) -> f64 {
    let corner = 1. - PI / 4.;
    SIZE.powi(3) - corner * radius * radius * SIZE - SIZE * distance * distance
        + distance.powi(3) / 3.
        + corner
            * (radius * radius * distance
                - (radius.powi(3) - (radius - distance).max(0.).powi(3)) / 3.)
}

#[test]
fn chamfer_consumes_a_smaller_corner_fillet() {
    let input = cube();
    let rounded = try_fillet_solid_edges(&input, &[edge(&input, Point3::new(0., 0., 5.))], 1., TOL)
        .unwrap()
        .solid;
    let before = serde_json::to_string(&rounded.compress()).unwrap();
    let wire = bottom_corner(&rounded);
    assert_eq!(wire.len(), 3);
    for distance in [0.5, 1., 2.] {
        let result = try_chamfer_solid_along_wire(&rounded, &wire, distance, distance, TOL)
            .unwrap_or_else(|e| panic!("distance {distance}: {e}"));
        let volume = chamfered_round_volume(1., distance);
        common::assert_solid(&result.solid, volume, &[0], TOL);
        assert!(result.solid.is_geometric_consistent());
        common::blend::assert_step(&result.solid, volume, TOL);
    }
    assert_eq!(before, serde_json::to_string(&rounded.compress()).unwrap());
}

#[test]
fn fillet_ends_on_two_chamfer_faces() {
    let input = cube();
    let edges = [
        edge(&input, Point3::new(5., 0., 0.)),
        edge(&input, Point3::new(0., 5., 0.)),
    ];
    let bevelled = try_chamfer_solid_edges(&input, &edges, 2., TOL)
        .unwrap()
        .solid;
    let before = serde_json::to_string(&bevelled.compress()).unwrap();
    let result = try_fillet_solid_edges(
        &bevelled,
        &[edge(&bevelled, Point3::new(0., 0., 5.))],
        1.,
        TOL,
    )
    .unwrap();
    // Integrate min(x,y) over the part removed by the unit quarter-circle.
    let moment = (1. + 2_f64.sqrt()) / 3. - PI / 4.;
    let volume = SIZE.powi(3) - SIZE * 4. + 8. / 3. - (SIZE - 2.) * (1. - PI / 4.) - moment;
    common::assert_solid(&result.solid, volume, &[0], TOL);
    assert!(result.solid.is_geometric_consistent());
    common::blend::assert_step(&result.solid, volume, TOL);
    assert_eq!(before, serde_json::to_string(&bevelled.compress()).unwrap());
}

#[test]
fn chamfer_then_fillet_keeps_the_opposite_three_edge_spherical_corner() {
    for transform in [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ] {
        let input = builder::transformed(&cube(), transform);
        let edges = [Point3::new(5., 0., 0.), Point3::new(0., 5., 0.)]
            .map(|p| edge(&input, transform.transform_point(p)));
        let bevelled = try_chamfer_solid_edges(&input, &edges, 2., TOL)
            .unwrap()
            .solid;
        let mut selected = [
            Point3::new(0., 0., 5.),
            Point3::new(5., 0., 10.),
            Point3::new(0., 5., 10.),
            Point3::new(10., 5., 10.),
            Point3::new(5., 10., 10.),
        ]
        .map(|p| edge(&bevelled, transform.transform_point(p)));
        let corner = 1. - PI / 4.;
        let top = 4. * SIZE * corner - 4. * (5. / 3. - PI / 2.);
        let moment = (1. + 2_f64.sqrt()) / 3. - PI / 4.;
        let volume =
            SIZE.powi(3) - SIZE * 4. + 8. / 3. - top - corner * (SIZE - 2. - 1. / 3.) - moment;
        let before = serde_json::to_string(&bevelled.compress()).unwrap();
        let mut geometry = None;
        for _ in 0..2 {
            let result = try_fillet_solid_edges(&bevelled, &selected, 1., TOL).unwrap();
            common::assert_solid(&result.solid, volume, &[0], TOL);
            assert!(result.solid.is_geometric_consistent());
            assert!(result
                .generated_faces
                .iter()
                .any(|f| matches!(f.surface(), Surface::Sphere(_))));
            common::blend::assert_step(&result.solid, volume, TOL);
            let compressed = serde_json::to_string(&result.solid.compress()).unwrap();
            if let Some(previous) = &geometry {
                assert_eq!(previous, &compressed);
            }
            geometry = Some(compressed);
            selected.reverse();
        }
        assert_eq!(before, serde_json::to_string(&bevelled.compress()).unwrap());
    }
}
