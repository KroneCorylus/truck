#[path = "../tests/common/mod.rs"]
mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{
    try_chamfer_solid_along_wire, try_chamfer_solid_edge, try_chamfer_solid_edges,
    try_chamfer_solid_edges_with_distances,
};

const TOL: f64 = 0.001;

fn prism(points: &[[f64; 2]], height: f64) -> Solid {
    let vertices = builder::vertices(points.iter().map(|p| Point3::new(p[0], p[1], 0.0)));
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * height,
    )
}

fn at(input: &Solid, p: [f64; 3]) -> EdgeID {
    common::blend::edge_through(&input.boundaries()[0], p.into()).id()
}

fn all_box_edges_at_once() {
    let input = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    let ids: std::collections::HashSet<_> = input.edge_iter().map(|e| e.id()).collect();
    let result =
        try_chamfer_solid_edges(&input, &ids.into_iter().collect::<Vec<_>>(), 0.2, TOL).unwrap();
    common::assert_solid(
        &result.solid,
        1000.0 - 60.0 * 0.2_f64.powi(2) + 6.0 * 0.2_f64.powi(3),
        &[0],
        TOL,
    );
    assert!(result.solid.is_geometric_consistent());
}

fn oblique_prism_single_and_top_rim() {
    let input = prism(&[[0.0, 0.0], [20.0, 0.0], [25.0, 10.0], [5.0, 10.0]], 10.0);
    for (name, points) in [
        ("single", vec![[10.0, 0.0, 10.0]]),
        ("two", vec![[10.0, 0.0, 10.0], [22.5, 5.0, 10.0]]),
        (
            "rim",
            vec![
                [10.0, 0.0, 10.0],
                [22.5, 5.0, 10.0],
                [15.0, 10.0, 10.0],
                [2.5, 5.0, 10.0],
            ],
        ),
        ("two_vertical", vec![[10.0, 0.0, 10.0], [0.0, 0.0, 5.0]]),
        (
            "three",
            vec![[10.0, 0.0, 10.0], [2.5, 5.0, 10.0], [0.0, 0.0, 5.0]],
        ),
    ] {
        let ids: Vec<_> = points.into_iter().map(|p| at(&input, p)).collect();
        let result = try_chamfer_solid_edges(&input, &ids, 0.2, TOL)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        common::assert_topology(&result.solid, &[0]);
        common::assert_mesh_closed(&result.solid, TOL);
        assert!(result.solid.is_geometric_consistent(), "{name}");
    }
}

fn unequal_two_edge_box_corner() {
    let input = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    let ids = [at(&input, [5.0, 0.0, 10.0]), at(&input, [0.0, 5.0, 10.0])];
    for sizes in [[0.2, 0.3], [0.3, 0.2]] {
        let selected: Vec<_> = ids.iter().map(|&id| (id, sizes)).collect();
        let result = try_chamfer_solid_edges_with_distances(&input, &selected, TOL).unwrap();
        common::assert_topology(&result.solid, &[0]);
        common::assert_mesh_closed(&result.solid, TOL);
        assert!(result.solid.is_geometric_consistent());
    }
}

fn concave_and_convex_two_edge_junction() {
    let input = prism(
        &[
            [0.0, 0.0],
            [20.0, 0.0],
            [20.0, 10.0],
            [10.0, 10.0],
            [10.0, 20.0],
            [0.0, 20.0],
        ],
        10.0,
    );
    let ids = [
        at(&input, [15.0, 10.0, 10.0]),
        at(&input, [10.0, 10.0, 5.0]),
    ];
    let result = try_chamfer_solid_edges(&input, &ids, 0.2, TOL).unwrap();
    common::assert_topology(&result.solid, &[0]);
    common::assert_mesh_closed(&result.solid, TOL);
    assert!(result.solid.is_geometric_consistent());
}

fn pocket_floor_complete_rim_and_vertical_corners() {
    let block = common::modeling::cuboid(Point3::origin(), Point3::new(20.0, 20.0, 10.0));
    let cutter =
        common::modeling::cuboid(Point3::new(5.0, 5.0, 5.0), Point3::new(15.0, 15.0, 11.0));
    let input = truck_shapeops::subtract(&block, &cutter, TOL).unwrap();
    for points in [
        vec![
            [10.0, 5.0, 5.0],
            [15.0, 10.0, 5.0],
            [10.0, 15.0, 5.0],
            [5.0, 10.0, 5.0],
        ],
        vec![[10.0, 5.0, 5.0], [5.0, 10.0, 5.0], [5.0, 5.0, 7.5]],
    ] {
        let ids: Vec<_> = points.into_iter().map(|p| at(&input, p)).collect();
        let result = try_chamfer_solid_edges(&input, &ids, 0.2, TOL).unwrap();
        common::assert_topology(&result.solid, &[0]);
        common::assert_mesh_closed(&result.solid, TOL);
        assert!(result.solid.is_geometric_consistent());
    }
}

fn open_tangent_chamfer_chain() {
    let input = prism(
        &[
            [0.0, 0.0],
            [10.0, 0.0],
            [20.0, 0.0],
            [20.0, 10.0],
            [0.0, 10.0],
        ],
        10.0,
    );
    let face = input
        .face_iter()
        .find(|f| {
            f.edge_iter()
                .any(|e| e.id() == at(&input, [5.0, 0.0, 10.0]))
                && f.edge_iter()
                    .any(|e| e.id() == at(&input, [15.0, 0.0, 10.0]))
        })
        .unwrap();
    let a = face
        .edge_iter()
        .find(|e| e.id() == at(&input, [5.0, 0.0, 10.0]))
        .unwrap();
    let b = face
        .edge_iter()
        .find(|e| e.id() == at(&input, [15.0, 0.0, 10.0]))
        .unwrap();
    let wire: Wire = if a.back() == b.front() {
        vec![a, b].into()
    } else {
        vec![b, a].into()
    };
    let result = try_chamfer_solid_along_wire(&input, &wire, 0.2, 0.2, TOL).unwrap();
    common::assert_solid(&result.solid, 1999.6, &[0], TOL);
}

fn circular_blind_hole_top_and_floor_rims() {
    let block = common::modeling::cuboid(
        Point3::new(-10.0, -10.0, 0.0),
        Point3::new(10.0, 10.0, 10.0),
    );
    let cutter =
        common::modeling::cylinder(Point3::new(0.0, 0.0, 5.0), Vector3::unit_z(), 3.0, 6.0);
    let input = truck_shapeops::subtract(&block, &cutter, TOL).unwrap();
    for z in [10.0, 5.0] {
        let face = input
            .face_iter()
            .find(|f| {
                matches!(f.surface(), Surface::Plane(_))
                    && f.vertex_iter().all(|v| (v.point().z - z).abs() < 1e-6)
                    && f.edge_iter().any(|e| !matches!(e.curve(), Curve::Line(_)))
            })
            .unwrap();
        let wire = face
            .boundaries()
            .into_iter()
            .find(|w| w.iter().all(|e| !matches!(e.curve(), Curve::Line(_))))
            .unwrap();
        let result = try_chamfer_solid_along_wire(&input, &wire, 0.2, 0.2, TOL)
            .unwrap_or_else(|e| panic!("z={z}: {e}"));
        common::assert_topology(&result.solid, &[0]);
        common::assert_mesh_closed(&result.solid, TOL);
        assert!(result.solid.is_geometric_consistent(), "z={z}");
    }
}

fn sequential_chamfer_on_new_bevel_edge() {
    let input = common::modeling::cuboid(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
    let first =
        try_chamfer_solid_edge(&input, at(&input, [5.0, 0.0, 10.0]), 1.0, 1.0, TOL).unwrap();
    let second = try_chamfer_solid_edge(
        &first.solid,
        at(&first.solid, [5.0, 1.0, 10.0]),
        0.2,
        0.2,
        TOL,
    )
    .unwrap();
    common::assert_topology(&second.solid, &[0]);
    common::assert_mesh_closed(&second.solid, TOL);
    assert!(second.solid.is_geometric_consistent());
}

fn pocket_floor_with_equivalent_native_lines() {
    let block = common::modeling::cuboid(Point3::origin(), Point3::new(20.0, 20.0, 10.0));
    let cutter =
        common::modeling::cuboid(Point3::new(5.0, 5.0, 5.0), Point3::new(15.0, 15.0, 11.0));
    let original = truck_shapeops::subtract(&block, &cutter, TOL).unwrap();
    let mut compressed = original.compress();
    for shell in &mut compressed.boundaries {
        for edge in &mut shell.edges {
            let [a, b] = [
                shell.vertices[edge.vertices.0],
                shell.vertices[edge.vertices.1],
            ];
            let curve = &edge.curve;
            let (start, end) = curve.range_tuple();
            for i in 0..=10 {
                let p = curve.subs(start + (end - start) * i as f64 / 10.0);
                assert!((p - a).cross(b - a).magnitude() < 1e-6);
            }
            edge.curve = Curve::Line(Line(a, b));
        }
    }
    let input = Solid::extract(compressed).unwrap();
    common::assert_solid(&input, 3500.0, &[0], TOL);
    assert!(input.is_geometric_consistent());
    let ids = [
        [10.0, 5.0, 5.0],
        [15.0, 10.0, 5.0],
        [10.0, 15.0, 5.0],
        [5.0, 10.0, 5.0],
    ]
    .map(|p| at(&input, p));
    let result = try_chamfer_solid_edges(&input, &ids, 0.2, TOL).unwrap();
    common::assert_topology(&result.solid, &[0]);
    common::assert_mesh_closed(&result.solid, TOL);
    assert!(result.solid.is_geometric_consistent());
}

fn cylinder_outer_rim() {
    let input = common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 3.0, 10.0);
    let face = input
        .face_iter()
        .find(|f| {
            matches!(f.surface(), Surface::Plane(_))
                && f.vertex_iter().all(|v| (v.point().z - 10.0).abs() < 1e-6)
        })
        .unwrap();
    let wire = face.boundaries()[0].clone();
    let result = try_chamfer_solid_along_wire(&input, &wire, 0.2, 0.2, TOL).unwrap();
    let expected = 90.0 * std::f64::consts::PI
        - std::f64::consts::PI * (3.0 * 0.2_f64.powi(2) - 0.2_f64.powi(3) / 3.0);
    common::assert_solid(&result.solid, expected, &[0], TOL);
    assert!(result.solid.is_geometric_consistent());
}

fn main() {
    let mut failed = 0;
    if std::panic::catch_unwind(pocket_floor_with_equivalent_native_lines).is_err() {
        eprintln!("AUDIT FAIL pocket_native_lines");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS pocket_native_lines");
    }
    if std::panic::catch_unwind(cylinder_outer_rim).is_err() {
        eprintln!("AUDIT FAIL cylinder_outer_rim");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS cylinder_outer_rim");
    }
    if std::panic::catch_unwind(all_box_edges_at_once).is_err() {
        eprintln!("AUDIT FAIL all_box_edges_at_once");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS all_box_edges_at_once");
    }
    if std::panic::catch_unwind(oblique_prism_single_and_top_rim).is_err() {
        eprintln!("AUDIT FAIL oblique_prism_single_and_top_rim");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS oblique_prism_single_and_top_rim");
    }
    if std::panic::catch_unwind(unequal_two_edge_box_corner).is_err() {
        eprintln!("AUDIT FAIL unequal_two_edge_box_corner");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS unequal_two_edge_box_corner");
    }
    if std::panic::catch_unwind(concave_and_convex_two_edge_junction).is_err() {
        eprintln!("AUDIT FAIL concave_and_convex_two_edge_junction");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS concave_and_convex_two_edge_junction");
    }
    if std::panic::catch_unwind(pocket_floor_complete_rim_and_vertical_corners).is_err() {
        eprintln!("AUDIT FAIL pocket_floor_complete_rim_and_vertical_corners");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS pocket_floor_complete_rim_and_vertical_corners");
    }
    if std::panic::catch_unwind(open_tangent_chamfer_chain).is_err() {
        eprintln!("AUDIT FAIL open_tangent_chamfer_chain");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS open_tangent_chamfer_chain");
    }
    if std::panic::catch_unwind(circular_blind_hole_top_and_floor_rims).is_err() {
        eprintln!("AUDIT FAIL circular_blind_hole_top_and_floor_rims");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS circular_blind_hole_top_and_floor_rims");
    }
    if std::panic::catch_unwind(sequential_chamfer_on_new_bevel_edge).is_err() {
        eprintln!("AUDIT FAIL sequential_chamfer_on_new_bevel_edge");
        failed += 1;
    } else {
        eprintln!("AUDIT PASS sequential_chamfer_on_new_bevel_edge");
    }
    eprintln!("AUDIT failures: {failed}");
    if failed != 0 {
        std::process::exit(1);
    }
}
