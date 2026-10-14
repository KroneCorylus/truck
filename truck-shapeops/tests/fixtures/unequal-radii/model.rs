#![allow(dead_code)]
use std::f64::consts::{FRAC_PI_2, PI};
use truck_modeling::*;

pub struct Case {
    pub step: &'static str,
    pub edges: [(Point3, Point3, f64); 2],
    pub axes: [(Point3, Vector3, f64); 2],
    pub volume: f64,
}

fn corner_area(radius: f64, angle: f64) -> f64 {
    radius * radius * (1. / (angle / 2.).tan() - (PI - angle) / 2.)
}

fn circular_corner_area(support_radius: f64, radius: f64, inner: bool) -> f64 {
    let offset = support_radius + if inner { radius } else { -radius };
    let angle = (radius / offset).asin();
    let center = (offset * offset - radius * radius).sqrt();
    let (cx, end, original) = if inner {
        (-center, -angle, -support_radius * support_radius * angle)
    } else {
        (center, angle, support_radius * support_radius * angle)
    };
    // Green's theorem: replace the line/circle corner by its tangent fillet arc.
    let arc = radius * cx * (end.sin() + 1.) - radius * radius * end.cos()
        + radius * radius * (end + FRAC_PI_2);
    (original - arc) / 2.
}

pub fn case(name: &str) -> Case {
    match name {
        "A2" => Case {
            step: include_str!("A2.step"),
            edges: [
                (Point3::new(0., 0., 30.), Point3::new(10., 0., 30.), 2.),
                (Point3::new(0., 20., 0.), Point3::new(10., 20., 0.), 3.),
            ],
            axes: [
                (Point3::new(0., 2., 28.), Vector3::unit_x(), 2.),
                (Point3::new(0., 17., 3.), Vector3::unit_x(), 3.),
            ],
            volume: 6000. - 10. * (corner_area(2., FRAC_PI_2) + corner_area(3., FRAC_PI_2)),
        },
        "B5" => {
            let angle = 0.25_f64.atan();
            Case {
                step: include_str!("B5.step"),
                edges: [
                    (Point3::new(5., 1., 0.), Point3::new(5., 1., 5.), 0.2),
                    (Point3::new(0., 0., 0.), Point3::new(0., 0., 5.), 0.5),
                ],
                axes: [
                    (
                        Point3::new(5. - 0.2 / (angle / 2.).tan(), 0.8, 0.),
                        Vector3::unit_z(),
                        0.2,
                    ),
                    (Point3::new(0.5, 0.5, 0.), Vector3::unit_z(), 0.5),
                ],
                volume: 15. - 5. * (corner_area(0.2, angle) + corner_area(0.5, FRAC_PI_2)),
            }
        }
        "C4" => Case {
            step: include_str!("C4.step"),
            edges: [
                (Point3::new(10., 0., 0.), Point3::new(10., 0., 20.), 2.),
                (Point3::new(-2., 0., 0.), Point3::new(-2., 0., 20.), 1.),
            ],
            axes: [
                (Point3::new(60.0_f64.sqrt(), 2., 0.), Vector3::unit_z(), 2.),
                (Point3::new(-8.0_f64.sqrt(), 1., 0.), Vector3::unit_z(), 1.),
            ],
            volume: 20.
                * (48. * PI
                    - circular_corner_area(10., 2., false)
                    - circular_corner_area(2., 1., true)),
        },
        _ => panic!("unknown fixture"),
    }
}
