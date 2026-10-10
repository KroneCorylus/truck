use super::*;

fn circle(count: usize, radius: f64, center: Vector3) -> Vec<Point3> {
    (0..count)
        .map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / count as f64;
            Point3::new(radius * angle.cos(), radius * angle.sin(), 0.0) + center
        })
        .collect()
}

#[test]
fn bounds_preserve_exhaustive_boundary_decisions() {
    for count in [8, 64, 97] {
        let outer = circle(count, 5.0, Vector3::zero());
        let mut concave = outer.clone();
        for p in concave.iter_mut().take(count / 4).skip(1) {
            *p = Point3::from_vec(p.to_vec() * 0.1);
        }
        let mut crossing = outer.clone();
        crossing.swap(1, count / 2);
        let mut repeated = outer.clone();
        repeated[1] = repeated[2];
        for transform in [
            Matrix4::identity(),
            Matrix4::from_translation(Vector3::new(80.0, -50.0, 13.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7)),
        ] {
            let normal = transform.transform_vector(Vector3::unit_z());
            let map = |points: &[Point3]| {
                points
                    .iter()
                    .map(|&p| transform.transform_point(p))
                    .collect::<Vec<_>>()
            };
            let outer = map(&outer);
            for points in [
                outer.clone(),
                map(&concave),
                map(&crossing),
                map(&repeated),
                map(&circle(count, 3.0, Vector3::zero())),
                map(&circle(count, 3.0, Vector3::new(4.0, 0.0, 0.0))),
                map(&circle(count, 3.0, Vector3::new(9.0, 0.0, 0.0))),
            ] {
                for epsilon in [0.0, TOLERANCE, 0.05] {
                    assert_eq!(
                        valid_polygon(&points, normal, epsilon),
                        exhaustive::valid_polygon(&points, normal, epsilon)
                    );
                    assert_eq!(
                        contained(&points, &outer, normal, epsilon),
                        exhaustive::contained(&points, &outer, normal, epsilon)
                    );
                    assert_eq!(
                        loops_intersect(&points, &outer, normal, epsilon),
                        exhaustive::loops_intersect(&points, &outer, normal, epsilon)
                    );
                    assert_eq!(
                        loops_touch(&points, &outer, normal, epsilon),
                        exhaustive::loops_touch(&points, &outer, normal, epsilon)
                    );
                    for i in 0..points.len() {
                        let p = points[i];
                        let mid = p + (points[(i + 1) % points.len()] - p) / 2.0;
                        for p in [p, mid] {
                            assert_eq!(
                                inside(p, &outer, normal, epsilon),
                                exhaustive::inside(p, &outer, normal, epsilon)
                            );
                        }
                    }
                }
            }
            for gap in [
                -2.0 * TOLERANCE,
                -0.5 * TOLERANCE,
                0.0,
                0.5 * TOLERANCE,
                2.0 * TOLERANCE,
            ] {
                let inner = map(&circle(count, 5.0 - gap, Vector3::zero()));
                assert_eq!(
                    contained(&inner, &outer, normal, TOLERANCE),
                    exhaustive::contained(&inner, &outer, normal, TOLERANCE)
                );
                assert_eq!(
                    loops_touch(&inner, &outer, normal, TOLERANCE),
                    exhaustive::loops_touch(&inner, &outer, normal, TOLERANCE)
                );
            }
        }
    }
}

#[test]
fn bounds_retain_every_contact_pair() {
    let mut seed = 123_u64;
    let mut number = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) as f64 / u32::MAX as f64 * 10.0
    };
    let a: Vec<_> = (0..80)
        .map(|_| Point3::new(number(), number(), number() * 1e-6))
        .collect();
    let b: Vec<_> = (0..70)
        .map(|_| Point3::new(number(), number(), number() * 1e-6))
        .collect();
    for normal in [Vector3::unit_z(), Vector3::new(1.0, 2.0, 3.0).normalize()] {
        for other in [None, Some(b.as_slice())] {
            for clearance in [TOLERANCE, 0.1, 1.0] {
                let contact = |i, j| {
                    let b = other.unwrap_or(&a);
                    exhaustive::loops_touch(
                        &[a[i], a[(i + 1) % a.len()]],
                        &[b[j], b[(j + 1) % b.len()]],
                        normal,
                        clearance,
                    )
                };
                let mut expected = HashSet::default();
                for i in 0..a.len() {
                    for j in other.map_or(i + 1..a.len(), |b| 0..b.len()) {
                        if contact(i, j) {
                            expected.insert((i, j));
                        }
                    }
                }
                let mut actual = HashSet::default();
                any_segment_pair(&a, other, normal, clearance, |i, j| {
                    if contact(i, j) {
                        actual.insert((i, j));
                    }
                    false
                });
                assert_eq!(actual, expected);
            }
        }
    }
}

#[test]
fn dense_circle_discards_distant_segment_pairs() {
    let points = circle(4096, 10.0, Vector3::zero());
    let mut checked = 0;
    assert!(!any_segment_pair(
        &points,
        None,
        Vector3::unit_z(),
        TOLERANCE,
        |_, _| {
            checked += 1;
            false
        }
    ));
    assert!(
        checked < points.len() * 4,
        "{checked} pairs survived bounds filtering"
    );
    assert!(valid_polygon(&points, Vector3::unit_z(), TOLERANCE));
}

// Exhaustive predicates retained to verify that spatial filtering preserves decisions.
mod exhaustive {
    use super::*;
    pub(super) fn valid_polygon(points: &[Point3], normal: Vector3, epsilon: f64) -> bool {
        if points.len() < 3 {
            return false;
        }
        polygon_area(points).dot(normal) > epsilon * epsilon
            && (0..points.len()).all(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                a.distance(b) > epsilon
                    && (i + 1..points.len()).all(|j| {
                        if j == i + 1 || (i == 0 && j == points.len() - 1) {
                            return true;
                        }
                        let (c, d) = (points[j], points[(j + 1) % points.len()]);
                        !crosses(a, b, c, d, normal, epsilon)
                            && [(a, c, d), (b, c, d), (c, a, b), (d, a, b)].iter().all(
                                |&(p, q, r)| {
                                    let axis = r - q;
                                    let t = ((p - q).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
                                    p.distance(q + t * axis) > epsilon
                                },
                            )
                    })
            })
    }

    pub(super) fn loops_intersect(
        a: &[Point3],
        b: &[Point3],
        normal: Vector3,
        epsilon: f64,
    ) -> bool {
        (0..a.len()).any(|i| {
            (0..b.len()).any(|j| {
                crosses(
                    a[i],
                    a[(i + 1) % a.len()],
                    b[j],
                    b[(j + 1) % b.len()],
                    normal,
                    epsilon,
                )
            })
        })
    }

    pub(super) fn inside(p: Point3, original: &[Point3], normal: Vector3, epsilon: f64) -> bool {
        let x = (original[1] - original[0]).normalize();
        let y = normal.cross(x);
        let project = |p: Point3| Point2::new((p - original[0]).dot(x), (p - original[0]).dot(y));
        let p = project(p);
        let mut winding = false;
        for j in 0..original.len() {
            let (a, b) = (
                project(original[j]),
                project(original[(j + 1) % original.len()]),
            );
            let axis = b - a;
            let t = ((p - a).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
            if p.distance(a + t * axis) <= epsilon {
                return true;
            }
            if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) * axis.x / axis.y {
                winding = !winding;
            }
        }
        winding
    }

    pub(super) fn contained(
        points: &[Point3],
        original: &[Point3],
        normal: Vector3,
        epsilon: f64,
    ) -> bool {
        (0..points.len()).all(|i| {
            inside(points[i], original, normal, epsilon)
                && inside(
                    points[i] + (points[(i + 1) % points.len()] - points[i]) / 2.0,
                    original,
                    normal,
                    epsilon,
                )
        }) && !loops_intersect(points, original, normal, epsilon)
    }

    pub(super) fn loops_touch(a: &[Point3], b: &[Point3], normal: Vector3, clearance: f64) -> bool {
        let distance = |p: Point3, q: Point3, r: Point3| {
            let axis = r - q;
            let t = ((p - q).dot(axis) / axis.magnitude2()).clamp(0.0, 1.0);
            p.distance(q + t * axis)
        };
        (0..a.len()).any(|i| {
            (0..b.len()).any(|j| {
                let (a, b, c, d) = (a[i], a[(i + 1) % a.len()], b[j], b[(j + 1) % b.len()]);
                crosses(a, b, c, d, normal, TOLERANCE)
                    || [(a, c, d), (b, c, d), (c, a, b), (d, a, b)]
                        .iter()
                        .any(|&(p, q, r)| distance(p, q, r) <= clearance)
            })
        })
    }
}
