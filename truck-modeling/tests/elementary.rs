//! `Surface::elementary` on solids built with `tsweep` and `rsweep`.

use std::f64::consts::PI;
use truck_modeling::*;

const EPS: f64 = 1e-12;

fn polygon_face(points: &[Point3]) -> Face {
    let vertices: Vec<Vertex> = points.iter().map(|p| builder::vertex(*p)).collect();
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::try_attach_plane(&[wire]).unwrap()
}

fn disk(center: Point3, normal: Vector3, radius: f64) -> Face {
    let radial = normal.cross(Vector3::unit_x());
    let radial = if radial.so_small() {
        Vector3::unit_y()
    } else {
        radial.normalize()
    };
    let vertex = builder::vertex(center + radial * radius);
    let circle: Wire = builder::rsweep(&vertex, center, normal, Rad(7.0), 3);
    builder::try_attach_plane(&[circle]).unwrap()
}

/// The elementary surface of `face`, checked against a sampled normal: `outward` must say
/// whether the oriented surface's normal at the middle of the parameter range points along
/// `canonical`, evaluated at the sampled point.
fn classified(face: &Face, canonical: impl Fn(Point3) -> Vector3) -> Elementary {
    let surface = face.oriented_surface();
    let (elementary, outward) = surface
        .elementary()
        .unwrap_or_else(|| panic!("not elementary: {surface:?}"));
    let mid = |range: Option<(f64, f64)>| range.map_or(0.5, |(a, b)| (a + b) / 2.0);
    let (u, v) = surface.try_range_tuple();
    let (u, v) = (mid(u), mid(v));
    let sampled = surface.normal(u, v).dot(canonical(surface.subs(u, v))) > 0.0;
    assert_eq!(outward, sampled, "{elementary:?}");
    elementary
}

fn assert_unit_parallel(v: Vector3, w: Vector3) {
    assert!(
        (v.magnitude() - 1.0).abs() < EPS,
        "{v:?} is not a unit vector"
    );
    assert!(
        v.cross(w).magnitude() < EPS,
        "{v:?} is not parallel to {w:?}"
    );
}

fn radial(p: Point3, origin: Point3, axis: Vector3) -> Vector3 {
    let r = p - origin;
    r - axis * r.dot(axis)
}

/// A ring with a conical outer face, revolved about an axis parallel to `z` through
/// `(0, 0, -1)`; the profile lies in the `xz`-plane.
#[test]
fn revolved_profile_gives_planes_cylinder_and_cone() {
    let (r0, r1, r2, z0, h) = (1.0, 2.0, 3.0, 0.5, 2.0);
    let profile = polygon_face(&[
        Point3::new(r0, 0.0, z0),
        Point3::new(r0, 0.0, z0 + h),
        Point3::new(r2, 0.0, z0 + h),
        Point3::new(r1, 0.0, z0),
    ]);
    let origin = Point3::new(0.0, 0.0, -1.0);
    let axis = Vector3::unit_z();
    let ring: Solid = builder::rsweep(&profile, origin, axis, Rad(7.0), 2);
    assert!(ring.is_geometric_consistent());
    let expected_apex = Point3::new(0.0, 0.0, z0 - r1 * h / (r2 - r1));

    let (mut planes, mut cylinders, mut cones) = (0, 0, 0);
    for face in ring.boundaries()[0].iter() {
        let surface = face.oriented_surface();
        let (elementary, outward) = surface.elementary().unwrap();
        match elementary {
            Elementary::Plane(plane) => {
                classified(face, |_| plane.normal());
                assert_unit_parallel(plane.normal(), axis);
                let z = plane.origin().z;
                assert!((z - z0).abs() < EPS || (z - z0 - h).abs() < EPS, "{z}");
                planes += 1;
            }
            Elementary::Cylinder {
                origin: o,
                axis: a,
                radius,
            } => {
                classified(face, |p| radial(p, o, a));
                assert_unit_parallel(a, axis);
                assert!(radial(o, origin, axis).magnitude() < EPS, "{o:?}");
                assert!((radius - r0).abs() < EPS, "{radius}");
                assert!(!outward, "the bore faces the axis");
                cylinders += 1;
            }
            Elementary::Cone {
                apex,
                axis: a,
                half_angle,
            } => {
                classified(face, |p| radial(p, apex, a));
                assert!(apex.distance(expected_apex) < EPS, "{apex:?}");
                assert!(a.distance(axis) < EPS, "{a:?}");
                let expected = f64::atan((r2 - r1) / h);
                assert!((half_angle.0 - expected).abs() < EPS, "{half_angle:?}");
                assert!(outward, "the outer cone faces away from the axis");
                cones += 1;
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!((planes, cylinders, cones), (4, 2, 2));
}

#[test]
fn revolved_half_circle_gives_sphere() {
    let (center, radius) = (Point3::new(1.0, -2.0, 3.0), 2.5);
    let vertex = builder::vertex(center + Vector3::unit_y() * radius);
    let meridian: Wire = builder::rsweep(&vertex, center, Vector3::unit_x(), Rad(PI), 3);
    let shell = builder::cone(&meridian, Vector3::unit_y(), Rad(7.0), 4);
    let sphere = Solid::new(vec![shell]);
    assert!(sphere.is_geometric_consistent());

    for face in sphere.boundaries()[0].iter() {
        match classified(face, |p| p - center) {
            Elementary::Sphere {
                center: c,
                radius: r,
            } => {
                assert!(c.distance(center) < EPS, "{c:?}");
                assert!((r - radius).abs() < EPS, "{r}");
                assert!(face.oriented_surface().elementary().unwrap().1);
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn revolved_disk_gives_torus() {
    let (major, minor) = (3.0, 0.75);
    let (origin, axis) = (Point3::new(0.0, 0.0, -1.0), Vector3::unit_z());
    let tube = disk(Point3::new(major, 0.0, 0.5), Vector3::unit_y(), minor);
    let torus: Solid = builder::rsweep(&tube, origin, axis, Rad(7.0), 2);
    assert!(torus.is_geometric_consistent());
    let expected_center = Point3::new(0.0, 0.0, 0.5);

    for face in torus.boundaries()[0].iter() {
        let canonical =
            |p: Point3| p - (expected_center + major * radial(p, origin, axis).normalize());
        match classified(face, canonical) {
            Elementary::Torus {
                center,
                axis: a,
                major: rr,
                minor: r,
            } => {
                assert!(center.distance(expected_center) < EPS, "{center:?}");
                assert_unit_parallel(a, axis);
                assert!(
                    (rr - major).abs() < EPS && (r - minor).abs() < EPS,
                    "{rr} {r}"
                );
                assert!(face.oriented_surface().elementary().unwrap().1);
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn extruded_disk_gives_cylinder() {
    let (center, radius, height) = (Point3::new(1.0, -2.0, 0.5), 1.5, 2.0);
    let cylinder: Solid = builder::tsweep(
        &disk(center, Vector3::unit_z(), radius),
        Vector3::unit_z() * height,
    );
    let mut sides = 0;
    for face in cylinder.boundaries()[0].iter() {
        let surface = face.oriented_surface();
        let Surface::Extruded(_) = surface else {
            assert!(matches!(
                surface.elementary(),
                Some((Elementary::Plane(_), true))
            ));
            continue;
        };
        match classified(face, |p| radial(p, center, Vector3::unit_z())) {
            Elementary::Cylinder {
                origin,
                axis,
                radius: r,
            } => {
                assert!(radial(origin, center, Vector3::unit_z()).magnitude() < EPS);
                assert_unit_parallel(axis, Vector3::unit_z());
                assert!((r - radius).abs() < EPS, "{r}");
                assert!(surface.elementary().unwrap().1);
                sides += 1;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(sides > 0);
}

/// A surface as the STEP reader builds it: a similarity transform and inverted orientation.
#[test]
fn transformed_and_inverted_surface_keeps_its_kind() {
    let line = Curve::Line(Line(Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 2.0)));
    let revolved = RevolutedCurve::by_revolution(line.clone(), Point3::origin(), Vector3::unit_z());
    let transform = Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0))
        * Matrix4::from_axis_angle(Vector3::new(1.0, 1.0, 0.0).normalize(), Rad(0.7))
        * Matrix4::from_scale(2.0);
    let upright = Surface::RevolutedCurve(Processor::new(revolved.clone()));
    let mut placed = Surface::RevolutedCurve(Processor::with_transform(revolved, transform));

    let (
        Elementary::Cone {
            apex,
            axis,
            half_angle,
        },
        outward,
    ) = upright.elementary().unwrap()
    else {
        panic!()
    };
    assert!(apex.distance(Point3::new(0.0, 0.0, -2.0)) < EPS);
    assert!(axis.distance(Vector3::unit_z()) < EPS);

    let (
        Elementary::Cone {
            apex: a,
            axis: ax,
            half_angle: h,
        },
        o,
    ) = placed.elementary().unwrap()
    else {
        panic!()
    };
    assert!(a.distance(transform.transform_point(apex)) < EPS);
    assert!(ax.distance(transform.transform_vector(axis).normalize()) < EPS);
    assert!((h.0 - half_angle.0).abs() < EPS);
    assert_eq!(o, outward);

    placed.invert();
    let (Elementary::Cone { apex: b, .. }, flipped) = placed.elementary().unwrap() else {
        panic!()
    };
    assert!(b.distance(a) < EPS);
    assert_eq!(flipped, !outward);

    let squashed = Surface::RevolutedCurve(Processor::with_transform(
        RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_z()),
        Matrix4::from_nonuniform_scale(1.0, 2.0, 1.0),
    ));
    assert!(squashed.elementary().is_none());
}

#[test]
fn other_surfaces_are_not_elementary() {
    let skewed: Solid = builder::tsweep(
        &disk(Point3::origin(), Vector3::unit_z(), 1.0),
        Vector3::new(0.0, 0.5, 2.0),
    );
    for face in skewed.boundaries()[0].iter() {
        if let Surface::Extruded(_) = face.surface() {
            assert!(face.surface().elementary().is_none());
        }
    }

    let v0 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
    let v1 = builder::vertex(Point3::new(2.0, 0.0, 3.0));
    let spline = builder::bezier(
        &v0,
        &v1,
        vec![Point3::new(1.5, 0.0, 1.0), Point3::new(1.2, 0.0, 2.0)],
    );
    let revolved: Shell =
        builder::rsweep(&spline, Point3::origin(), Vector3::unit_z(), Rad(1.0), 1);
    assert!(revolved[0].surface().elementary().is_none());

    let v2 = builder::vertex(Point3::new(2.0, 1.0, 3.0));
    let skew_line = builder::line(&v0, &v2);
    let hyperboloid: Shell =
        builder::rsweep(&skew_line, Point3::origin(), Vector3::unit_z(), Rad(1.0), 1);
    assert!(hyperboloid[0].surface().elementary().is_none());

    let ellipse = Curve::Conic(Processor::with_transform(
        TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)),
        Matrix4::from_nonuniform_scale(2.0, 1.0, 1.0),
    ));
    let elliptic = Surface::Extruded(ExtrudedCurve::by_extrusion(ellipse, Vector3::unit_z()));
    assert!(elliptic.elementary().is_none());
}
