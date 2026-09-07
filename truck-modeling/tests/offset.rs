//! `Surface::offset` on the five elementary kinds, against exact parameters.

use std::f64::consts::PI;
use truck_modeling::{errors::Error, *};

const EPS: f64 = 1e-12;

fn polygon_face(points: &[Point3]) -> Face {
    let vertices: Vec<Vertex> = points.iter().map(|p| builder::vertex(*p)).collect();
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::try_attach_plane(&[wire]).unwrap()
}

/// The elementary kind of `surface` and whether its normal points the canonical way.
fn kind(surface: &Surface) -> (Elementary, bool) { surface.elementary().unwrap() }

#[test]
fn plane_moves_along_its_normal() {
    let plane = Plane::new(
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(2.0, 2.0, 3.0),
        Point3::new(1.0, 3.0, 4.0),
    );
    let Surface::Plane(moved) = Surface::Plane(plane).offset(0.7).unwrap() else {
        panic!()
    };
    assert!((moved.normal() - plane.normal()).magnitude() < EPS);
    assert!(((moved.origin() - plane.origin()).dot(plane.normal()) - 0.7).abs() < EPS);
    assert!((moved.origin() - plane.origin()).magnitude() - 0.7 < EPS);
}

#[test]
fn extruded_circle_changes_radius() {
    let (radius, height) = (1.5, 2.0);
    let vertex = builder::vertex(Point3::new(radius, 0.0, 0.0));
    let circle: Wire = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    let disc: Face = builder::try_attach_plane(&[circle]).unwrap();
    let cylinder: Solid = builder::tsweep(&disc, Vector3::unit_z() * height);
    for face in cylinder.face_iter() {
        let Surface::Extruded(_) = face.surface() else {
            continue;
        };
        for surface in [face.surface(), face.oriented_surface()] {
            let (Elementary::Cylinder { origin, axis, .. }, outward) = kind(&surface) else {
                panic!()
            };
            let sign = if outward { 1.0 } else { -1.0 };
            for d in [0.3, -0.3] {
                let offset = surface.offset(d).unwrap();
                assert!(matches!(offset, Surface::Extruded(_)));
                let (
                    Elementary::Cylinder {
                        origin: o,
                        axis: a,
                        radius: r,
                    },
                    out,
                ) = kind(&offset)
                else {
                    panic!()
                };
                assert!((r - (radius + sign * d)).abs() < EPS, "{r}");
                assert!(
                    a.cross(axis).magnitude() < EPS && (o - origin).cross(axis).magnitude() < EPS
                );
                assert_eq!(out, outward);
            }
            assert_eq!(
                surface.offset(-sign * radius).unwrap_err(),
                Error::OffsetRadiusNotPositive
            );
        }
    }
}

/// A ring with a conical outer face, as in the elementary tests: the offset cone keeps its
/// half angle and its apex moves along the axis by `d / sin`.
#[test]
fn revolved_line_moves_a_cylinder_and_a_cone() {
    let (r0, r1, r2, z0, h) = (1.0, 2.0, 3.0, 0.5, 2.0);
    let profile = polygon_face(&[
        Point3::new(r0, 0.0, z0),
        Point3::new(r0, 0.0, z0 + h),
        Point3::new(r2, 0.0, z0 + h),
        Point3::new(r1, 0.0, z0),
    ]);
    let ring: Solid = builder::rsweep(
        &profile,
        Point3::new(0.0, 0.0, -1.0),
        Vector3::unit_z(),
        Rad(7.0),
        2,
    );
    let d = 0.25;
    for face in ring.face_iter() {
        let surface = face.oriented_surface();
        match kind(&surface) {
            (Elementary::Cylinder { radius, .. }, outward) => {
                let (Elementary::Cylinder { radius: r, .. }, _) = kind(&surface.offset(d).unwrap())
                else {
                    panic!()
                };
                let sign = if outward { 1.0 } else { -1.0 };
                assert!((r - (radius + sign * d)).abs() < EPS, "{r}");
                assert!(matches!(
                    surface.offset(-sign * (radius + 0.1)),
                    Err(Error::OffsetRadiusNotPositive)
                ));
            }
            (
                Elementary::Cone {
                    apex,
                    axis,
                    half_angle,
                },
                outward,
            ) => {
                let (
                    Elementary::Cone {
                        apex: a,
                        axis: ax,
                        half_angle: half,
                    },
                    out,
                ) = kind(&surface.offset(d).unwrap())
                else {
                    panic!()
                };
                assert!((half.0 - half_angle.0).abs() < EPS && (ax - axis).magnitude() < EPS);
                assert_eq!(out, outward);
                let sign = if outward { -1.0 } else { 1.0 };
                let expected = apex + axis * (sign * d / half_angle.0.sin());
                assert!(a.distance(expected) < EPS, "{a:?} against {expected:?}");
            }
            (Elementary::Plane(plane), _) => {
                // the flat ends are revolved lines, and stay so
                let (Elementary::Plane(moved), _) = kind(&surface.offset(d).unwrap()) else {
                    panic!()
                };
                assert!(moved.normal().cross(plane.normal()).magnitude() < EPS);
                let shift = (moved.origin() - plane.origin()).dot(plane.normal());
                assert!((shift.abs() - d).abs() < EPS, "{shift}");
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn revolved_circles_give_a_sphere_and_a_torus() {
    let (center, radius) = (Point3::new(1.0, -2.0, 3.0), 2.5);
    let vertex = builder::vertex(center + Vector3::unit_y() * radius);
    let meridian: Wire = builder::rsweep(&vertex, center, Vector3::unit_x(), Rad(PI), 3);
    let shell: Shell = builder::cone(&meridian, Vector3::unit_y(), Rad(7.0), 4);
    for face in shell.iter() {
        let surface = face.oriented_surface();
        let (Elementary::Sphere { .. }, outward) = kind(&surface) else {
            panic!()
        };
        assert!(outward);
        let (
            Elementary::Sphere {
                center: c,
                radius: r,
            },
            _,
        ) = kind(&surface.offset(0.5).unwrap())
        else {
            panic!()
        };
        assert!(c.distance(center) < EPS && (r - 3.0).abs() < EPS);
        assert!(matches!(
            surface.offset(-3.0),
            Err(Error::OffsetRadiusNotPositive)
        ));
    }

    let (major, minor) = (3.0, 1.0);
    let vertex = builder::vertex(Point3::new(major + minor, 0.0, 0.0));
    let tube: Wire = builder::rsweep(
        &vertex,
        Point3::new(major, 0.0, 0.0),
        Vector3::unit_y(),
        Rad(7.0),
        2,
    );
    let torus: Shell = builder::rsweep(&tube, Point3::origin(), Vector3::unit_z(), Rad(7.0), 2);
    for face in torus.iter() {
        let surface = face.oriented_surface();
        let (Elementary::Torus { .. }, outward) = kind(&surface) else {
            panic!()
        };
        let sign = if outward { 1.0 } else { -1.0 };
        let (
            Elementary::Torus {
                major: big,
                minor: small,
                ..
            },
            _,
        ) = kind(&surface.offset(0.25).unwrap())
        else {
            panic!()
        };
        assert!((big - major).abs() < EPS && (small - (minor + sign * 0.25)).abs() < EPS);
    }
}

#[test]
fn splines_have_no_typed_offset() {
    let square = polygon_face(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ]);
    let Surface::Plane(plane) = square.surface() else {
        panic!()
    };
    let spline = Surface::BSplineSurface(BSplineSurface::from(plane));
    assert_eq!(spline.offset(0.1).unwrap_err(), Error::NoTypedOffset);
}
