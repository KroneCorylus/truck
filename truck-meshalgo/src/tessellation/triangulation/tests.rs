use super::*;
use proptest::prelude::*;

#[test]
fn polar_ellipsoid_boundaries_keep_the_large_angular_sector() {
    use std::f64::consts::{FRAC_PI_2, PI};
    use truck_geometry::prelude::*;
    let meridian = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), (0., FRAC_PI_2)),
        Matrix4::from_cols(
            Vector4::new(1.5, 0., 0., 0.),
            Vector4::new(0., 0., 1., 0.),
            Vector4::new(0., -1., 0., 0.),
            Vector4::unit_w(),
        ),
    );
    let reference = Processor::<_, Matrix4>::new(RevolutedCurve::by_revolution(
        meridian,
        Point3::origin(),
        Vector3::unit_z(),
    ))
    .inverse();
    let angle = 1.5 * PI;
    let lines = [
        Line(Point2::new(0., 0.), Point2::new(angle, 0.)),
        Line(Point2::new(angle, 0.), Point2::new(angle, FRAC_PI_2)),
        Line(Point2::new(0., FRAC_PI_2), Point2::new(0., 0.)),
    ];
    let wire: Vec<PolylineCurve> = lines
        .iter()
        .map(|line| {
            (0..=64)
                .map(|i| {
                    let p = line.subs(i as f64 / 64.);
                    reference.subs(p.x, p.y)
                })
                .collect()
        })
        .collect();
    for reversed in [false, true] {
        for start in 0..3 {
            let mut surface = reference;
            let mut wire = wire.clone();
            wire.rotate_left(start);
            if reversed {
                surface.invert();
                wire.reverse();
                for edge in &mut wire {
                    edge.0.reverse();
                }
            }
            let piece = PolyBoundaryPiece::try_new(&surface, wire.into_iter(), by_search_parameter)
                .unwrap();
            let boundary = PolyBoundary::new(vec![piece], &surface, 0.001).unwrap();
            let bounds: BoundingBox<Point2> =
                boundary.loops.iter().flatten().map(|p| p.uv).collect();
            let span = bounds.max() - bounds.min();
            let (major, minor) = if reversed {
                (span.y, span.x)
            } else {
                (span.x, span.y)
            };
            assert!(
                (major - angle).abs() < 1e-8,
                "start={start} reversed={reversed} bounds={bounds:?}"
            );
            assert!(
                (minor - FRAC_PI_2).abs() < 1e-8,
                "start={start} reversed={reversed} bounds={bounds:?}"
            );
            assert_eq!(boundary.loops.len(), 1);
            assert_eq!(loop_orientation(&boundary.loops[0]), Some(true));
        }
    }
}

#[test]
fn thin_planar_faces_keep_all_nonzero_triangles() {
    let plane = truck_geometry::prelude::Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    for (width, height) in [(2e-7, 1.0), (2e-7, 2e-7)] {
        let boundary = PolyBoundary::from_loops(vec![rectangle(0.0, 0.0, width, height)]);
        let mesh = trimming_tessellation(&plane, &boundary, 0.001);
        let area: f64 = mesh
            .faces()
            .triangle_iter()
            .map(|tri| {
                let [a, b, c] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos]);
                (b - a).cross(c - a).magnitude() / 2.0
            })
            .sum();
        assert!(
            (area - width * height).abs() <= width * height * 1e-9,
            "{area}"
        );
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn one_worker_and_parallel_tessellation_are_identical() {
    use truck_modeling::{builder, primitive, Face, Solid};
    let circle = builder::rsweep(
        &builder::vertex(Point3::new(1.0, 0.0, 0.0)),
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::TAU),
        2,
    );
    let face: Face = builder::try_attach_plane(vec![circle]).unwrap();
    let cylinder: Solid = builder::tsweep(&face, Vector3::unit_z() * 2.0);
    let cube: Solid = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(2.0, 2.0, 2.0),
    ]));
    let serial = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let parallel = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    for solid in [cube, cylinder] {
        for tol in [0.1, 0.01, 0.001] {
            let run = || solid.triangulation(tol).to_polygon();
            assert_eq!(serial.install(run), parallel.install(run));
        }
    }
}

fn polygon(points: &[[f64; 2]]) -> Vec<SurfacePoint> {
    points
        .iter()
        .map(|&[x, y]| (Point2::new(x, y), Point3::new(x, y, 0.0)).into())
        .collect()
}

fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Vec<SurfacePoint> {
    polygon(&[
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ])
}

fn without_culling(boundary: &PolyBoundary) -> PolyBoundary {
    let mut unculled = boundary.clone();
    let everywhere = BoundingBox::from_iter([
        Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        Point2::new(f64::INFINITY, f64::INFINITY),
    ]);
    unculled.bounds.fill(everywhere);
    unculled
}

#[test]
fn holes_islands_disconnected_and_concave_loops() {
    let mut hole = rectangle(1.0, 1.0, 4.0, 4.0);
    hole.reverse();
    let boundary = PolyBoundary::from_loops(vec![
        rectangle(0.0, 0.0, 6.0, 6.0),
        hole,
        rectangle(2.0, 2.0, 1.0, 1.0),
        polygon(&[
            [8.0, 0.0],
            [11.0, 0.0],
            [11.0, 1.0],
            [9.0, 1.0],
            [9.0, 3.0],
            [8.0, 3.0],
        ]),
    ]);
    for (point, expected) in [
        ([0.5, 0.5], true),
        ([1.5, 1.5], false),
        ([2.5, 2.5], true),
        ([7.0, 0.5], false),
        ([8.5, 2.0], true),
        ([10.0, 2.0], false),
    ] {
        assert_eq!(boundary.include(Point2::from(point)), expected);
    }
    let unculled = without_culling(&boundary);
    for x in -10..130 {
        for y in -10..80 {
            let point = Point2::new(x as f64 / 10.0, y as f64 / 10.0);
            assert_eq!(
                boundary.include(point),
                unculled.include(point),
                "{point:?}"
            );
        }
    }
}

#[test]
fn culling_preserves_near_boundary_rejection() {
    let mut hole = rectangle(1.0, 1.0, 2.0, 2.0);
    hole.reverse();
    let boundary = PolyBoundary::from_loops(vec![rectangle(0.0, 0.0, 4.0, 4.0), hole]);
    let unculled = without_culling(&boundary);
    for points in &boundary.loops {
        for (a, b) in points.iter().circular_tuple_windows() {
            let normal = Vector2::new(b.y - a.y, a.x - b.x).normalize();
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for distance in [-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
                    let point = a.uv + (b.uv - a.uv) * t + normal * distance * TOLERANCE;
                    assert_eq!(
                        boundary.include(point),
                        unculled.include(point),
                        "{point:?}"
                    );
                }
            }
        }
    }
}

proptest! {
    #[test]
    fn culling_matches_full_winding_scan(
        loops in prop::collection::vec(
            (prop::collection::vec((-20.0..20.0, -20.0..20.0), 3..24), any::<bool>()), 0..16),
        x in -30.0..30.0,
        y in -30.0..30.0,
    ) {
        let loops = loops.into_iter().map(|(points, reversed)| {
            let mut points = polygon(&points.into_iter().map(|(x, y)| [x, y]).collect::<Vec<_>>());
            if reversed {
                points.reverse();
            }
            points
        }).collect();
        let boundary = PolyBoundary::from_loops(loops);
        let point = Point2::new(x, y);
        prop_assert_eq!(boundary.include(point), without_culling(&boundary).include(point));
    }
}

#[test]
fn fitted_edge_endpoints_use_shared_topology_vertices() {
    use crate::analyzers::Topology;
    use crate::filters::OptimizingFilter;
    use truck_modeling::{primitive, Curve, Matrix4, Solid};
    let solid: Solid = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(2.0, 3.0, 4.0),
    ]));
    let solid = solid.mapped(
        |p| *p,
        |c: &Curve| c.transformed(Matrix4::from_translation((c.back() - c.front()) * 1e-7)),
        Clone::clone,
    );
    for compressed in [false, true] {
        let mut mesh = if compressed {
            solid.compress().robust_triangulation(0.001).to_polygon()
        } else {
            solid.robust_triangulation(0.001).to_polygon()
        };
        mesh.put_together_same_attrs(1e-9).remove_degenerate_faces();
        assert_eq!(mesh.shell_condition(), shell::ShellCondition::Closed);
    }
}

#[test]
fn normal_orientation_preserves_ambiguous_meridian_facets_and_flips_reliable_ones() {
    let positions = vec![
        Point3::new(1.8011179551484724, 2.8000098658731867, 3.124827034989286e-6),
        Point3::new(1.8022358753627803, 2.800019731438082, 1.2499210494482105e-5),
        Point3::new(1.8, 2.8000000000000003, -5.551115123125783e-16),
    ];
    let normals = vec![
        Vector3::new(
            0.005589775742362419,
            4.932936593385396e-5,
            -0.9999843758648252,
        ),
        Vector3::new(
            0.011179376813901243,
            9.8657190410345e-5,
            -0.9999375039475277,
        ),
        Vector3::new(0., 0., -1.),
    ];
    let triangle = [0, 1, 2].map(|pos| StandardVertex {
        pos,
        uv: None,
        nor: Some(pos),
    });
    let check = |positions, normals, expected| {
        let mut mesh = PolygonMesh::new(
            StandardAttributes {
                positions,
                normals,
                ..Default::default()
            },
            Faces::from_iter([triangle]),
        );
        orient_to_normals(&mut mesh);
        let face = mesh.faces().triangle_iter().next().unwrap();
        assert_eq!([face[0].pos, face[1].pos, face[2].pos], expected);
    };
    check(positions, normals, [0, 1, 2]);
    for height in [0.0001, 0.1] {
        check(
            vec![
                Point3::origin(),
                Point3::new(1., 0., 0.),
                Point3::new(0.5, height, 0.),
            ],
            vec![-Vector3::unit_z(); 3],
            [2, 1, 0],
        );
    }
}
