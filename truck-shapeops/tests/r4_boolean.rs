//! R4: split cuts, composable booleans, and oriented boundary components.
mod common;

use common::{assert_solid, modeling::cuboid};
use truck_meshalgo::prelude::*;
use truck_modeling::*;

const TOL: f64 = 0.01;

fn swept_box(x0: f64, y0: f64, x1: f64, y1: f64, height: f64) -> Solid {
    let vertices = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, 0.0)));
    let wire: Wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let face = builder::try_attach_plane(&[wire]).unwrap();
    builder::tsweep(&face, Vector3::unit_z() * height)
}

fn split_operands() -> (Solid, Solid) {
    (
        swept_box(0.0, 0.0, 10.0, 10.0, 10.0),
        swept_box(4.0, -1.0, 6.0, 31.0, 11.0),
    )
}

fn cube(low: f64, high: f64) -> Solid {
    cuboid(Point3::new(low, low, low), Point3::new(high, high, high))
}

fn combine_shells(solids: impl IntoIterator<Item = Solid>) -> Solid {
    Solid::new(
        solids
            .into_iter()
            .flat_map(Solid::into_boundaries)
            .collect(),
    )
}

fn subtract(a: &Solid, b: Solid) -> Option<Solid> { truck_shapeops::subtract(a, &b, TOL) }

fn cavity() -> Solid {
    let mut inner = cube(2.0, 8.0);
    inner.not();
    combine_shells([cube(0.0, 10.0), inner])
}

fn assert_shell_volumes(solid: &Solid, expected: &[f64]) {
    assert_solid(solid, expected.iter().sum(), &vec![0; expected.len()], TOL);
    let mut actual: Vec<_> = solid
        .boundaries()
        .iter()
        .map(|shell| shell.triangulation(TOL).to_polygon().volume())
        .collect();
    actual.sort_by(f64::total_cmp);
    let mut expected = expected.to_vec();
    expected.sort_by(f64::total_cmp);
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
    }
}

#[test]
fn split_cut_intersection_probe() {
    let (a, b) = split_operands();
    for (left, right) in [(&a, &b), (&b, &a)] {
        let overlap = truck_shapeops::and(left, right, TOL).expect("intersection probe");
        assert_shell_volumes(&overlap, &[200.0]);
    }
}

#[test]
fn split_cut_subtraction_and_subsequent_boolean() {
    let (a, b) = split_operands();
    let split = subtract(&a, b).expect("split subtraction");
    assert_shell_volumes(&split, &[400.0, 400.0]);
    assert_components(&split, &[&[400.0], &[400.0]]);
    let mut bounds: Vec<_> = truck_shapeops::solid_components(&split, TOL)
        .unwrap()
        .iter()
        .map(|body| {
            let bbox: BoundingBox<Point3> = body.vertex_iter().map(|v| v.point()).collect();
            (bbox.min().x, bbox.max().x)
        })
        .collect();
    bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(bounds, [(0.0, 4.0), (6.0, 10.0)]);
    let cutter = cuboid(Point3::new(-1.0, -1.0, -1.0), Point3::new(2.0, 11.0, 11.0));
    let cut_again = subtract(&split, cutter).expect("boolean on disconnected result");
    assert_shell_volumes(&cut_again, &[200.0, 400.0]);
    assert_components(&cut_again, &[&[200.0], &[400.0]]);
}

#[test]
fn split_cut_union() {
    let (a, b) = split_operands();
    let joined = truck_shapeops::or(&a, &b, TOL).expect("union");
    assert_shell_volumes(&joined, &[1504.0]);
}

#[test]
fn primitive_boxes_also_reproduce_the_intersection_failure() {
    let a = cube(0.0, 10.0);
    let b = cuboid(Point3::new(4.0, -1.0, 0.0), Point3::new(6.0, 31.0, 11.0));
    let overlap = truck_shapeops::and(&a, &b, TOL).expect("primitive intersection");
    assert_shell_volumes(&overlap, &[200.0]);
}

#[test]
fn disconnected_components_intersect_enclosure() {
    let pieces = combine_shells([cube(0.0, 1.0), cube(3.0, 4.0)]);
    let result = truck_shapeops::and(&pieces, &cube(-1.0, 5.0), TOL).unwrap();
    assert_shell_volumes(&result, &[1.0, 1.0]);
}

#[test]
fn cavity_union_remote_cube() {
    let result = truck_shapeops::or(&cavity(), &cube(20.0, 21.0), TOL).unwrap();
    assert_shell_volumes(&result, &[-216.0, 1000.0, 1.0]);
}

#[test]
fn cavity_intersect_enclosure_control() {
    let result = truck_shapeops::and(&cavity(), &cube(-1.0, 11.0), TOL).unwrap();
    assert_shell_volumes(&result, &[-216.0, 1000.0]);
}

#[test]
fn island_inside_cavity_intersect_enclosure() {
    let nested = combine_shells([cavity(), cube(3.0, 4.0)]);
    let result = truck_shapeops::and(&nested, &cube(-1.0, 11.0), TOL).unwrap();
    assert_shell_volumes(&result, &[-216.0, 1000.0, 1.0]);
}

#[test]
fn empty_intersection() {
    let empty = Solid::new(vec![]);
    let result = truck_shapeops::and(&empty, &cube(0.0, 1.0), TOL).unwrap();
    assert!(result.boundaries().is_empty());
}

#[test]
fn empty_union() {
    let empty = Solid::new(vec![]);
    let result = truck_shapeops::or(&empty, &cube(0.0, 1.0), TOL).unwrap();
    assert_shell_volumes(&result, &[1.0]);
}

#[cfg(feature = "step-test")]
fn assert_step(solid: &Solid, expected: &[f64]) {
    use truck_stepio::{out::*, r#in::Table};
    let compressed = solid.compress();
    let prepared = prepare_for_step(&compressed, TOL / 20.0).unwrap();
    let design = StepDesign::from_model(StepModel::from(&prepared));
    let text = StepDisplay::new(Default::default(), design).to_string();
    let table = Table::from_step(&text).unwrap();
    assert_eq!(table.manifold_solid_brep.len(), 1);
    let entity = table.manifold_solid_brep.values().next().unwrap();
    let (read, skipped) = table.to_compressed_solid(entity).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    let read = read
        .try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap();
    assert_shell_volumes(&Solid::extract(read).unwrap(), expected);
}

#[test]
#[cfg(feature = "step-test")]
fn cavity_step_round_trip_control() { assert_step(&cavity(), &[-216.0, 1000.0]); }

fn shell_orders(solid: &Solid) -> Vec<Solid> {
    let shells = solid.boundaries();
    (0..shells.len())
        .flat_map(|shift| {
            let mut rotated = shells.clone();
            rotated.rotate_left(shift);
            let mut reversed = rotated.clone();
            reversed.reverse();
            [Solid::new(rotated), Solid::new(reversed)]
        })
        .collect()
}

fn assert_components(solid: &Solid, expected: &[&[f64]]) {
    let mut bodies = truck_shapeops::solid_components(solid, TOL).expect("body grouping");
    bodies.sort_by(|a, b| {
        let volume = |solid: &Solid| solid.triangulation(TOL).to_polygon().volume();
        volume(a).total_cmp(&volume(b))
    });
    let mut expected = expected.to_vec();
    expected.sort_by(|a, b| a.iter().sum::<f64>().total_cmp(&b.iter().sum::<f64>()));
    assert_eq!(bodies.len(), expected.len());
    for (body, volumes) in bodies.iter().zip(expected) {
        assert_shell_volumes(body, volumes);
        let shell_volumes: Vec<_> = body
            .boundaries()
            .iter()
            .map(|shell| shell.triangulation(TOL).to_polygon().volume())
            .collect();
        assert!(shell_volumes[0] > 0.0);
        assert!(shell_volumes[1..].iter().all(|v| *v < 0.0));
        #[cfg(feature = "step-test")]
        assert_step(body, volumes);
    }
}

#[test]
fn body_grouping_and_booleans_ignore_shell_and_operand_order() {
    for (solid, expected) in [
        (
            combine_shells([cavity(), cube(20.0, 21.0)]),
            vec![vec![1000.0, -216.0], vec![1.0]],
        ),
        (
            combine_shells([cavity(), cube(3.0, 4.0)]),
            vec![vec![1000.0, -216.0], vec![1.0]],
        ),
    ] {
        let expected: Vec<_> = expected.iter().map(Vec::as_slice).collect();
        for reordered in shell_orders(&solid) {
            assert_components(&reordered, &expected);
            let enclosure = cube(-1.0, 22.0);
            for (a, b) in [(&reordered, &enclosure), (&enclosure, &reordered)] {
                let result = truck_shapeops::and(a, b, TOL).unwrap();
                assert_components(&result, &expected);
            }
        }
    }
    for reordered in shell_orders(&cavity()) {
        let remote = cube(20.0, 21.0);
        for (a, b) in [(&reordered, &remote), (&remote, &reordered)] {
            let result = truck_shapeops::or(a, b, TOL).unwrap();
            assert_components(&result, &[&[1000.0, -216.0], &[1.0]]);
        }
    }
}

#[test]
fn split_ignores_face_order() {
    let (a, b) = split_operands();
    for i in 0..6 {
        let reorder = |solid: &Solid, reverse: bool| {
            let mut faces = solid.boundaries()[0].clone();
            faces.rotate_left(i);
            if reverse {
                faces.reverse();
            }
            Solid::new(vec![faces])
        };
        let a = reorder(&a, false);
        let b = reorder(&b, true);
        assert_shell_volumes(&truck_shapeops::and(&a, &b, TOL).unwrap(), &[200.0]);
        assert_shell_volumes(&truck_shapeops::or(&b, &a, TOL).unwrap(), &[1504.0]);
        assert_shell_volumes(
            &truck_shapeops::subtract(&a, &b, TOL).unwrap(),
            &[400.0, 400.0],
        );
    }
}

#[test]
fn cut_crosses_both_exterior_and_cavity() {
    let cutter = swept_box(4.0, -1.0, 6.0, 11.0, 11.0);
    for hollow in shell_orders(&cavity()) {
        let split = truck_shapeops::subtract(&hollow, &cutter, TOL).unwrap();
        assert_components(&split, &[&[328.0], &[328.0]]);
    }
}

#[test]
fn disconnected_coplanar_cutters() {
    let cutters = combine_shells([
        swept_box(2.0, -1.0, 3.0, 11.0, 11.0),
        swept_box(6.0, -1.0, 7.0, 11.0, 11.0),
    ]);
    let a = cube(0.0, 10.0);
    for cutters in shell_orders(&cutters) {
        assert_components(
            &truck_shapeops::subtract(&a, &cutters, TOL).unwrap(),
            &[&[200.0], &[300.0], &[300.0]],
        );
        for (a, b) in [(&a, &cutters), (&cutters, &a)] {
            assert_components(
                &truck_shapeops::and(a, b, TOL).unwrap(),
                &[&[100.0], &[100.0]],
            );
            assert_shell_volumes(&truck_shapeops::or(a, b, TOL).unwrap(), &[1064.0]);
        }
    }
}

#[test]
fn nested_island_with_its_own_cavity() {
    let mut inner = cube(3.2, 3.8);
    inner.not();
    let nested = combine_shells([cavity(), cube(3.0, 4.0), inner]);
    for solid in shell_orders(&nested) {
        assert_components(&solid, &[&[1000.0, -216.0], &[1.0, -0.216]]);
    }
}

#[test]
fn empty_results_are_composable() {
    let a = cube(0.0, 1.0);
    let b = cube(3.0, 4.0);
    let empty = truck_shapeops::and(&a, &b, TOL).unwrap();
    assert!(empty.boundaries().is_empty());
    assert!(truck_shapeops::solid_components(&empty, TOL)
        .unwrap()
        .is_empty());
    for (a, b) in [(&a, &empty), (&empty, &a)] {
        assert!(truck_shapeops::and(a, b, TOL)
            .unwrap()
            .boundaries()
            .is_empty());
        assert_shell_volumes(&truck_shapeops::or(a, b, TOL).unwrap(), &[1.0]);
    }
    assert_shell_volumes(&truck_shapeops::subtract(&a, &empty, TOL).unwrap(), &[1.0]);
    assert!(truck_shapeops::subtract(&empty, &a, TOL)
        .unwrap()
        .boundaries()
        .is_empty());
    for op in [
        truck_shapeops::and,
        truck_shapeops::or,
        truck_shapeops::subtract,
    ] {
        assert!(op(&empty, &empty, TOL).unwrap().boundaries().is_empty());
    }
}

#[test]
fn complements_of_multiple_shells() {
    let pieces = combine_shells([cube(0.0, 1.0), cube(3.0, 4.0)]);
    for pieces in shell_orders(&pieces) {
        let result = truck_shapeops::subtract(&cube(-1.0, 5.0), &pieces, TOL).unwrap();
        assert_components(&result, &[&[216.0, -1.0, -1.0]]);
        let mut complement = pieces.clone();
        complement.not();
        assert!(truck_shapeops::solid_components(&complement, TOL).is_none());
        assert!(truck_shapeops::or(&pieces, &complement, TOL).is_none());
        assert!(truck_shapeops::and(&pieces, &complement, TOL)
            .unwrap()
            .boundaries()
            .is_empty());
    }
}

#[test]
fn invalid_shell_sets_return_none() {
    let mut remote_void = cube(20.0, 21.0);
    remote_void.not();
    let a = cube(0.0, 2.0);
    let invalid = [
        combine_shells([cube(0.0, 10.0), cube(2.0, 8.0)]),
        combine_shells([cube(0.0, 10.0), remote_void]),
        combine_shells([a.clone(), cube(1.0, 3.0)]),
        combine_shells([a.clone(), a]),
        Solid::new_unchecked(vec![Shell::default()]),
    ];
    for solid in invalid {
        assert!(truck_shapeops::solid_components(&solid, TOL).is_none());
        for op in [
            truck_shapeops::and,
            truck_shapeops::or,
            truck_shapeops::subtract,
        ] {
            assert!(op(&solid, &cube(-1.0, 11.0), TOL).is_none());
            assert!(op(&cube(-1.0, 11.0), &solid, TOL).is_none());
        }
    }
}

#[test]
fn invalid_tolerances_return_none() {
    let a = cube(0.0, 1.0);
    let empty = Solid::new(vec![]);
    for tol in [0.0, -0.01, f64::NAN, f64::INFINITY] {
        assert!(truck_shapeops::solid_components(&a, tol).is_none());
        for op in [
            truck_shapeops::and,
            truck_shapeops::or,
            truck_shapeops::subtract,
        ] {
            assert!(op(&a, &a, tol).is_none());
            assert!(op(&a, &empty, tol).is_none());
        }
    }
}
