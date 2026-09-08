mod common;
use common::{modeling::*, *};
use truck_modeling::*;
use truck_shapeops::subtract_with_effect;

fn cube(a: [f64; 3], b: [f64; 3]) -> Solid { cuboid(a.into(), b.into()) }

fn check(target: &Solid, cutter: &Solid, removed: bool, volume: f64) {
    let before = serde_json::to_string(&[target.compress(), cutter.compress()]).unwrap();
    let result = subtract_with_effect(target, cutter, 0.01).unwrap();
    assert_eq!(result.removed_material, removed);
    if volume == 0.0 {
        assert!(result.solid.boundaries().is_empty());
    } else {
        assert_volume(&result.solid, volume, 0.001);
        assert_mesh_closed(&result.solid, 0.001);
    }
    assert_eq!(
        before,
        serde_json::to_string(&[target.compress(), cutter.compress()]).unwrap()
    );
}

#[test]
fn removal_and_contact() {
    let target = cube([0.0; 3], [1.0; 3]);
    for (a, b) in [
        ([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
        ([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
        ([1.0, 1.0, 0.0], [2.0, 2.0, 1.0]),
        ([1.0; 3], [2.0; 3]),
        ([1.0, 0.2, 0.2], [2.0, 0.8, 0.8]),
    ] {
        check(&target, &cube(a, b), false, 1.0);
    }
    check(&target, &target.clone(), true, 0.0);
    check(&target, &cube([-1.0; 3], [2.0; 3]), true, 0.0);
    check(&target, &cube([0.5, -0.5, -0.5], [1.5; 3]), true, 0.5);
    check(&target, &cube([0.25; 3], [0.75; 3]), true, 0.875);
    check(&target, &cube([0.0; 3], [0.5, 0.5, 1.0]), true, 0.75);
}

#[test]
fn empty_operands_and_invalid_tolerance() {
    let target = cube([0.0; 3], [1.0; 3]);
    let empty = Solid::new(vec![]);
    check(&target, &empty, false, 1.0);
    check(&empty, &target, false, 0.0);
    check(&empty, &empty, false, 0.0);
    for tol in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(subtract_with_effect(&target, &empty, tol).is_none());
        assert!(subtract_with_effect(&target, &target, tol).is_none());
    }
}

#[test]
fn disconnected_solids_and_cavity_shells() {
    let outer = cube([0.0; 3], [4.0; 3]);
    let hole = cube([1.0; 3], [3.0; 3]);
    let hollow = subtract_with_effect(&outer, &hole, 0.01).unwrap().solid;
    check(&hollow, &cube([1.5; 3], [2.5; 3]), false, 56.0);
    check(&hollow, &hole, false, 56.0);
    check(&hollow, &cube([0.5; 3], [3.5; 3]), true, 37.0);
    check(&outer, &hollow, true, 8.0);
    let remote = cube([5.0; 3], [6.0; 3]);
    let disconnected = Solid::new(
        [
            hollow.boundaries().as_slice(),
            remote.boundaries().as_slice(),
        ]
        .concat(),
    );
    check(&disconnected, &hole, false, 57.0);
    check(&disconnected, &remote, true, 56.0);
    let cutters =
        Solid::new([hole.boundaries().as_slice(), remote.boundaries().as_slice()].concat());
    check(&disconnected, &cutters, true, 56.0);
}
