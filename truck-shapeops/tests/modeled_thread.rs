mod common;
use truck_modeling::*;
use truck_shapeops::thread::{thread_groove, ThreadGroove};
fn spec() -> ThreadGroove {
    ThreadGroove {
        origin: Point3::new(0.0, 0.0, -1.25),
        axis: Vector3::unit_z(),
        radius: 3.4,
        depth: 0.6,
        pitch: 1.25,
        length: 5.5,
        internal: true,
        left_handed: false,
    }
}
#[test]
fn helical_cutter_is_closed_and_cuts_a_bore() {
    let cutter = thread_groove(spec()).unwrap();
    common::assert_topology(&cutter, &[0]);
    common::assert_mesh_closed(&cutter, 0.01);
    let block = common::modeling::cuboid(Point3::new(-8.0, -8.0, 0.0), Point3::new(8.0, 8.0, 3.0));
    let bore = common::modeling::cylinder(Point3::new(0.0, 0.0, -1.0), Vector3::unit_z(), 3.4, 5.0);
    let block = truck_shapeops::subtract(&block, &bore, 0.01).unwrap();
    let threaded = truck_shapeops::try_subtract(&block, &cutter, 0.01).unwrap();
    common::assert_topology(&threaded, &[1]);
    common::assert_mesh_closed(&threaded, 0.01);
}
#[test]
fn invalid_thread_dimensions_are_rejected() {
    for pitch in [0.0, -1.0, f64::NAN] {
        assert!(thread_groove(ThreadGroove { pitch, ..spec() }).is_err());
    }
}

fn groove_removed(radius: f64, depth: f64, pitch: f64, length: f64, internal: bool) -> f64 {
    let sign = if internal { 1.0 } else { -1.0 };
    std::f64::consts::TAU * length / pitch
        * (radius * pitch / 8.0 * depth
            + sign * pitch / 16.0 * depth * depth
            + radius * depth * depth / 3.0_f64.sqrt()
            + sign * depth.powi(3) / (3.0 * 3.0_f64.sqrt()))
}

#[test]
fn internal_and_external_threads_match_helical_volume_in_both_hands() {
    for internal in [true, false] {
        for left_handed in [false, true] {
            let radius = if internal { 3.4 } else { 4.0 };
            let spec = ThreadGroove {
                radius,
                internal,
                left_handed,
                ..spec()
            };
            let cutter = thread_groove(spec).unwrap();
            let d = spec.depth + spec.pitch * 0.1;
            let r0 = radius
                + if internal {
                    -spec.pitch * 0.1
                } else {
                    spec.pitch * 0.1
                };
            let expected = groove_removed(r0, d, spec.pitch, spec.length, internal);
            common::assert_solid(&cutter, expected, &[0], 0.003);
            let cylinder =
                common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), radius, 3.0);
            let target = if internal {
                let block = common::modeling::cuboid(
                    Point3::new(-8.0, -8.0, 0.0),
                    Point3::new(8.0, 8.0, 3.0),
                );
                truck_shapeops::subtract(&block, &cylinder, 0.01).unwrap()
            } else {
                cylinder
            };
            let threaded = truck_shapeops::try_subtract(&target, &cutter, 0.01).unwrap();
            let original = if internal {
                768.0 - std::f64::consts::PI * radius * radius * 3.0
            } else {
                std::f64::consts::PI * radius * radius * 3.0
            };

            common::assert_solid(
                &threaded,
                original - groove_removed(radius, spec.depth, spec.pitch, 3.0, internal),
                &[if internal { 1 } else { 0 }],
                0.003,
            );
        }
    }
}

#[test]
fn rotated_helix_keeps_pitch_radius_and_handedness() {
    for left_handed in [false, true] {
        let axis = Vector3::new(1.0, 2.0, 3.0).normalize();
        let origin = Point3::new(12.0, -5.0, 8.0);
        let spec = ThreadGroove {
            axis,
            origin,
            left_handed,
            ..spec()
        };
        let cutter = thread_groove(spec).unwrap();
        common::assert_mesh_closed(&cutter, 0.003);
        for edge in cutter.edge_iter() {
            let Curve::NurbsCurve(curve) = edge.oriented_curve() else {
                continue;
            };
            let (a, b) = curve.range_tuple();
            let start = curve.subs(a) - origin;
            let r = start - axis * start.dot(axis);
            if (r.magnitude() - 4.0).abs() > 1e-6 {
                continue;
            }
            let x = r.normalize();
            let y = axis.cross(x);
            for i in 0..=16 {
                let p = curve.subs(a + (b - a) * i as f64 / 16.0) - origin;
                let radial = p - axis * p.dot(axis);
                assert!((radial.magnitude() - 4.0).abs() < 1e-8);
                let angle = radial.dot(y).atan2(radial.dot(x));
                let rise = angle * spec.pitch / std::f64::consts::TAU
                    * if left_handed { -1.0 } else { 1.0 };
                assert!(((p - start).dot(axis) - rise).abs() < 0.00001);
            }
        }
    }
}
