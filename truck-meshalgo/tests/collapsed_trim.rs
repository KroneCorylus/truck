use truck_meshalgo::prelude::*;
use truck_modeling::*;

fn circular_segment(angle: f64) -> Shell {
    let a = builder::vertex(Point3::new(12.0, 0.0, -10.8));
    let b = builder::vertex(Point3::new(12.0 * angle.cos(), 12.0 * angle.sin(), -10.8));
    let arc = builder::circle_arc(&a, &b, Vector3::unit_y());
    let chord = builder::line(&b, &a);
    let origin = Point3::new(11.975386691676, 0.588313022933, -10.8);
    let plane = Plane::new(
        origin,
        origin + Vector3::unit_x(),
        origin + Vector3::unit_y(),
    );
    Shell::from(vec![Face::new(vec![vec![arc, chord].into()], plane.into())])
}

fn check_segment(mesh: &PolygonMesh, angle: f64) {
    let area: f64 = mesh
        .faces()
        .triangle_iter()
        .map(|triangle| {
            let [a, b, c] = triangle.map(|v| mesh.positions()[v.pos]);
            (b - a).cross(c - a).magnitude() / 2.0
        })
        .sum();
    let exact = 72.0 * (angle - angle.sin());
    assert!(
        area > exact / 2.0 && area <= exact * (1.0 + 1e-8),
        "area {area}, exact {exact}"
    );
    for point in mesh.positions() {
        assert!(
            point.x.hypot(point.y) <= 12.0 + 1e-8,
            "outside circle: {point:?}"
        );
        let inward = Vector3::new((angle / 2.0).cos(), (angle / 2.0).sin(), 0.0);
        assert!(point.to_vec().dot(inward) >= 12.0 * (angle / 2.0).cos() - 1e-8);
    }
}

#[test]
fn coarse_tessellation_preserves_arc_chord_faces() {
    for angle in [std::f64::consts::TAU / 64.0, std::f64::consts::TAU / 256.0] {
        let shell = circular_segment(angle);
        for tolerance in [0.05, 0.01, 0.001] {
            for mesh in [
                shell.triangulation(tolerance).to_polygon(),
                shell.robust_triangulation(tolerance).to_polygon(),
                shell.compress().triangulation(tolerance).to_polygon(),
                shell
                    .compress()
                    .robust_triangulation(tolerance)
                    .to_polygon(),
            ] {
                check_segment(&mesh, angle);
            }
        }
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn collapsed_trim_refinement_agrees_between_serial_and_parallel() {
    let angle = std::f64::consts::TAU / 64.0;
    let shell = circular_segment(angle);
    let run = |workers| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap()
            .install(|| shell.triangulation(0.05).to_polygon())
    };
    let serial = run(1);
    check_segment(&serial, angle);
    assert_eq!(serial, run(4));
}

#[test]
fn collapsed_inner_trim_is_refined_instead_of_filling_the_hole() {
    let angle = std::f64::consts::TAU / 64.0;
    let segment = circular_segment(angle);
    let points = [(11.0, -1.0), (13.0, -1.0), (13.0, 2.0), (11.0, 2.0)]
        .map(|(x, y)| builder::vertex(Point3::new(x, y, -10.8)));
    let outer = (0..4)
        .map(|i| builder::line(&points[i], &points[(i + 1) % 4]))
        .collect();
    let hole = segment[0].boundaries()[0].inverse();
    let shell = Shell::from(vec![Face::new(vec![outer, hole], segment[0].surface())]);
    let mesh = shell.triangulation(0.05).to_polygon();
    let area: f64 = mesh
        .faces()
        .triangle_iter()
        .map(|triangle| {
            let [a, b, c] = triangle.map(|v| mesh.positions()[v.pos]);
            (b - a).cross(c - a).magnitude() / 2.0
        })
        .sum();
    let exact_hole = 72.0 * (angle - angle.sin());
    assert!(6.0 - area > exact_hole / 2.0 && 6.0 - area <= exact_hole);
}

#[test]
fn irrecoverable_trim_is_reported_without_inventing_a_face() {
    let a = builder::vertex(Point3::new(5.0, 0.0, 0.0));
    let b = builder::vertex(Point3::new(6.0, 0.0, 0.0));
    let boundary = vec![builder::line(&a, &b), builder::line(&b, &a)].into();
    let plane = Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let shell: Shell = vec![Face::new(vec![boundary], plane.into())].into();
    assert_eq!(shell.triangulation(0.05).missing_faces(), vec![(0, 0)]);
    assert_eq!(
        shell.compress().triangulation(0.05).missing_faces(),
        vec![(0, 0)]
    );
    assert!(try_triangulation(&shell, 0.05).is_err());
}

#[test]
fn sub_tolerance_contours_do_not_depend_on_a_global_retry_budget() {
    let angle = std::f64::consts::TAU / 4096.0;
    let shell = circular_segment(angle);
    for tolerance in [0.05, 100.0] {
        for mesh in [
            shell.triangulation(tolerance).to_polygon(),
            shell.robust_triangulation(tolerance).to_polygon(),
            shell.compress().triangulation(tolerance).to_polygon(),
            shell
                .compress()
                .robust_triangulation(tolerance)
                .to_polygon(),
        ] {
            check_segment(&mesh, angle);
        }
    }
}

#[test]
fn recovering_a_contour_does_not_resample_an_unrelated_face() {
    let circle = builder::rsweep(
        &builder::vertex(Point3::new(2.0, 0.0, 0.0)),
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::TAU),
        2,
    );
    let disk = builder::try_attach_plane(&[circle]).unwrap();
    let reference = Shell::from(vec![disk.clone()]).triangulation(0.05);
    let mut shell = circular_segment(std::f64::consts::TAU / 64.0);
    shell.push(disk);
    let mesh = shell.triangulation(0.05);
    check_segment(&mesh[0].surface().unwrap(), std::f64::consts::TAU / 64.0);
    assert_eq!(mesh[1].surface(), reference[0].surface());
    let compressed = shell.compress().triangulation(0.05);
    assert_eq!(compressed.faces[1].surface, reference[0].surface());
}

#[test]
fn a_clockwise_planar_trim_does_not_mean_the_whole_surface() {
    let segment = circular_segment(std::f64::consts::TAU / 64.0);
    let boundary = segment[0].boundaries()[0].inverse();
    let shell = Shell::from(vec![Face::new(vec![boundary], segment[0].surface())]);
    assert_eq!(shell.triangulation(0.001).missing_faces(), vec![(0, 0)]);
    assert_eq!(
        shell.compress().triangulation(0.001).missing_faces(),
        vec![(0, 0)]
    );
}

#[test]
fn a_tiny_hole_is_recovered_without_resampling_its_outer_wire() {
    let angle = std::f64::consts::TAU / 4096.0;
    let segment = circular_segment(angle);
    let center = Point3::new(12.0, 0.0, -10.8);
    let outer = builder::rsweep(
        &builder::vertex(center + Vector3::unit_x() * 3.0),
        center,
        Vector3::unit_z(),
        Rad(std::f64::consts::TAU),
        2,
    );
    let disk = Face::new(vec![outer.clone()], segment[0].surface());
    let reference = Shell::from(vec![disk]).triangulation(0.05);
    let hole = segment[0].boundaries()[0].inverse();
    let shell = Shell::from(vec![Face::new(vec![outer, hole], segment[0].surface())]);
    let mesh = shell.triangulation(0.05);
    for (actual, expected) in mesh[0].absolute_boundaries()[0]
        .iter()
        .zip(reference[0].absolute_boundaries()[0].iter())
    {
        assert_eq!(actual.curve(), expected.curve());
    }
    let area = |mesh: PolygonMesh| {
        mesh.faces()
            .triangle_iter()
            .map(|triangle| {
                let [a, b, c] = triangle.map(|v| mesh.positions()[v.pos]);
                (b - a).cross(c - a).magnitude() / 2.0
            })
            .sum::<f64>()
    };
    let removed = area(reference.to_polygon()) - area(mesh.to_polygon());
    let exact = 72.0 * (angle - angle.sin());
    assert!(
        removed > exact / 2.0 && removed <= exact,
        "hole area {removed}, exact {exact}"
    );
}

#[test]
fn curved_lenses_survive_scale_rotation_and_coarse_tolerance() {
    for height in [1e-3, 1e-6] {
        let a = builder::vertex(Point3::new(-1.0, 0.0, 0.0));
        let b = builder::vertex(Point3::new(1.0, 0.0, 0.0));
        let edge = |height| {
            let curve = BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![a.point(), Point3::new(0.0, height, 0.0), b.point()],
            );
            Edge::new(&a, &b, Curve::from(curve))
        };
        let plane = Plane::new(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );
        let face = Face::new(
            vec![vec![edge(-height), edge(height).inverse()].into()],
            Surface::from(plane),
        );
        let shell = Shell::from(vec![face]);
        for scale in [0.1, 1.0, 10.0] {
            let transform = Matrix4::from_translation(Vector3::new(10.0, -3.0, 7.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale);
            let shell = builder::transformed(&shell, transform);
            let mesh = shell.triangulation(100.0).to_polygon();
            let area: f64 = mesh
                .faces()
                .triangle_iter()
                .map(|triangle| {
                    let [a, b, c] = triangle.map(|v| mesh.positions()[v.pos]);
                    (b - a).cross(c - a).magnitude() / 2.0
                })
                .sum();
            let exact = 4.0 * height * scale * scale / 3.0;
            assert!(
                area > exact / 2.0 && area <= exact * (1.0 + 1e-7),
                "area {area}, exact {exact}"
            );
        }
    }
}
