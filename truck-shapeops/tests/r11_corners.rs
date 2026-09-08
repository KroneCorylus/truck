mod common;

use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

fn fixture() -> Solid {
    let vertices = builder::vertices([
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 0.0),
        Point3::new(0.0, 10.0, 0.0),
    ]);
    let wire: Wire = (0..4)
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::new(0.0, 0.0, 10.0),
    )
}

fn edge_between(solid: &Solid, a: Point3, b: Point3) -> Edge {
    solid
        .edge_iter()
        .find(|e| {
            (e.front().point().near(&a) && e.back().point().near(&b))
                || (e.front().point().near(&b) && e.back().point().near(&a))
        })
        .unwrap()
        .clone()
}

fn selection_at(solid: &Solid, transform: Matrix4) -> [EdgeID; 3] {
    let v = Point3::new(0.0, 0.0, 10.0);
    [
        Point3::new(10.0, 0.0, 10.0),
        Point3::new(0.0, 10.0, 10.0),
        Point3::origin(),
    ]
    .map(|p| {
        edge_between(
            solid,
            transform.transform_point(v),
            transform.transform_point(p),
        )
        .id()
    })
}

fn selection(solid: &Solid) -> [EdgeID; 3] { selection_at(solid, Matrix4::identity()) }

fn snapshot(solid: &Solid) -> String { serde_json::to_string(&solid.compress()).unwrap() }

fn assert_history(input: &Solid, result: &BlendResult, generated: usize) {
    assert_eq!(result.generated_faces.len(), generated);
    let mut accounted = std::collections::HashSet::new();
    for face in input.face_iter() {
        let replacements: Vec<_> = result
            .modified_faces
            .iter()
            .filter(|(id, _)| *id == face.id())
            .collect();
        assert_eq!(replacements.len(), 1);
        assert!(accounted.insert(replacements[0].1.id()));
    }
    for face in &result.generated_faces {
        assert!(accounted.insert(face.id()));
    }
    assert_eq!(accounted.len(), result.solid.face_iter().count());
    assert!(result
        .solid
        .face_iter()
        .all(|f| accounted.contains(&f.id())));
}

fn transforms() -> [Matrix4; 2] {
    [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13.0, -7.0, 23.0))
            * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.73)),
    ]
}

fn fillet_volume(count: usize) -> f64 {
    use std::f64::consts::PI;
    let overlap = match count {
        1 => 0.0,
        // Integral from 0 to 1 of (1 - sqrt(1 - (1-z)^2))^2 dz.
        2 => 5.0 / 3.0 - PI / 2.0,
        3 => 2.0 - 7.0 * PI / 12.0,
        _ => unreachable!(),
    };
    1000.0 - count as f64 * 10.0 * (1.0 - PI / 4.0) + overlap
}

fn chamfer_volume(count: usize) -> f64 {
    let d: f64 = 0.2;
    // Inclusion-exclusion of triangular prisms: pair overlaps d^3/3, triple d^3/4.
    let overlap = match count {
        1 => 0.0,
        2 => d.powi(3) / 3.0,
        3 => 3.0 * d.powi(3) / 4.0,
        _ => unreachable!(),
    };
    1000.0 - count as f64 * 10.0 * d * d / 2.0 + overlap
}

fn check_downstream(result: &BlendResult, expected: f64, fillet: bool, transform: Matrix4) {
    let before = snapshot(&result.solid);
    common::blend::assert_step(&result.solid, expected, TOL);
    let cutter = builder::transformed(
        &common::modeling::cuboid(Point3::new(5.0, -1.0, -1.0), Point3::new(11.0, 11.0, 11.0)),
        transform,
    );
    let cut = truck_shapeops::and(&result.solid, &cutter, TOL).expect("boolean after corner blend");
    let section = if fillet {
        1.0 - std::f64::consts::PI / 4.0
    } else {
        0.2 * 0.2 / 2.0
    };
    common::assert_solid(&cut, 500.0 - 5.0 * section, &[0], TOL);
    let height = if fillet { 9.5 } else { 9.9 };
    let cutter = builder::transformed(
        &common::modeling::cuboid(
            Point3::new(-1.0, -1.0, -1.0),
            Point3::new(11.0, 11.0, height),
        ),
        transform,
    );
    let cut =
        truck_shapeops::and(&result.solid, &cutter, TOL).expect("boolean through corner junction");
    let expected_cut = if fillet {
        let t: f64 = 0.5;
        let circular = t * (1.0 - t * t).sqrt() + t.asin();
        950.0 - 20.0 * (t - circular / 2.0) + 2.0 * t - t.powi(3) / 3.0 - circular
    } else if result.generated_faces.len() == 2 {
        990.0 - 20.0 * 0.1_f64.powi(2) / 2.0 + 0.1_f64.powi(3) / 3.0
    } else {
        990.0 - 9.9 * 0.2_f64.powi(2) / 2.0 - 19.6 * 0.1_f64.powi(2) / 2.0 - 0.1_f64.powi(3) / 3.0
    };
    common::assert_solid(&cut, expected_cut, &[0], TOL);
    common::blend::assert_step(&cut, expected_cut, TOL);
    assert_eq!(
        before,
        snapshot(&result.solid),
        "export and boolean preserve modeling geometry"
    );
}

fn assert_unselected_edge(result: &BlendResult, transform: Matrix4, amount: f64) {
    let edge = edge_between(
        &result.solid,
        transform.transform_point(Point3::origin()),
        transform.transform_point(Point3::new(0.0, 0.0, 10.0 - amount)),
    );
    assert!(matches!(edge.curve(), Curve::Line(_)));
    let faces: Vec<_> = result
        .solid
        .face_iter()
        .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
        .collect();
    assert_eq!(faces.len(), 2);
    assert!(faces
        .iter()
        .all(|f| matches!(f.surface(), Surface::Plane(_))));
    assert!(result
        .generated_faces
        .iter()
        .all(|f| f.edge_iter().all(|e| e.id() != edge.id())));
}

fn assert_convex_geometry(result: &BlendResult) {
    use truck_meshalgo::prelude::*;
    let points: Vec<_> = result
        .solid
        .edge_iter()
        .flat_map(|edge| {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            (0..=8).map(move |k| curve.subs(a + (b - a) * k as f64 / 8.0))
        })
        .collect();
    let meshed = result.solid.triangulation(TOL);
    for (face, tessellated) in result.solid.face_iter().zip(meshed.face_iter()) {
        let surface = face.oriented_surface();
        let mesh = tessellated.surface().expect("face tessellation");
        let area: f64 = mesh
            .faces()
            .triangle_iter()
            .map(|tri| {
                let [p, q, r] = [0, 1, 2].map(|i| mesh.positions()[tri[i].pos]);
                (q - p).cross(r - p).magnitude() / 2.0
            })
            .sum();
        assert!(area > 0.001, "no degenerate corner faces");
        for edge in face.edge_iter() {
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            for k in 1..8 {
                let p = curve.subs(a + (b - a) * k as f64 / 8.0);
                let (u, v) = surface
                    .search_parameter(p, None, 100)
                    .expect("boundary on face");
                assert!(surface.subs(u, v).distance(p) < TOLERANCE);
                let normal = surface.normal(u, v).normalize();
                assert!(normal.magnitude().is_finite());
                assert!(
                    points.iter().all(|&q| normal.dot(q - p) <= TOLERANCE),
                    "every sampled boundary lies inside each outward supporting plane"
                );
            }
        }
    }
}

fn assert_miter_and_contacts(result: &BlendResult, transform: Matrix4) {
    let [a, b] = result.generated_faces.as_slice() else {
        panic!("two cylindrical strips")
    };
    let seam = a
        .edge_iter()
        .find(|e| b.edge_iter().any(|f| e.id() == f.id()))
        .unwrap();
    let curve = seam.curve();
    let (t0, t1) = curve.range_tuple();
    let inverse = transform.invert().unwrap();
    for i in 0..=16 {
        let p = inverse.transform_point(curve.subs(t0 + (t1 - t0) * i as f64 / 16.0));
        assert!((p.x - p.y).abs() < TOLERANCE);
        assert!(((p.x - 1.0).powi(2) + (9.0 - p.z).powi(2) - 1.0).abs() < TOLERANCE);
    }
    for face in [a, b] {
        for edge in face
            .edge_iter()
            .filter(|e| matches!(e.curve(), Curve::Line(_)))
        {
            let neighbour = result
                .solid
                .face_iter()
                .find(|f| f.id() != face.id() && f.edge_iter().any(|e| e.id() == edge.id()))
                .unwrap();
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            for i in 1..8 {
                let p = curve.subs(t0 + (t1 - t0) * i as f64 / 8.0);
                let normals = [face, neighbour].map(|f| {
                    let surface = f.oriented_surface();
                    let (u, v) = surface.search_parameter(p, None, 100).unwrap();
                    surface.normal(u, v).normalize()
                });
                assert!(
                    normals[0].near(&normals[1]),
                    "tangent contact with original plane"
                );
            }
        }
    }
}

#[test]
fn fillet_fixture_table_and_order() {
    for transform in transforms() {
        for count in [1, 2, 3] {
            let input = builder::transformed(&fixture(), transform);
            let before = snapshot(&input);
            let mut selected = selection_at(&input, transform)[..count].to_vec();
            let result = try_fillet_solid_edges(&input, &selected, 1.0, TOL).unwrap();
            common::assert_solid(&result.solid, fillet_volume(count), &[0], TOL);
            assert_history(&input, &result, count + usize::from(count == 3));
            assert_convex_geometry(&result);
            selected.reverse();
            let reversed = try_fillet_solid_edges(&input, &selected, 1.0, TOL).unwrap();
            assert_eq!(snapshot(&result.solid), snapshot(&reversed.solid));
            if count == 2 {
                assert_unselected_edge(&result, transform, 1.0);
                assert_miter_and_contacts(&result, transform);
                check_downstream(&result, fillet_volume(count), true, transform);
            }
            assert_eq!(before, snapshot(&input));
        }
    }
}

#[test]
fn chamfer_fixture_table_and_order() {
    for transform in transforms() {
        for count in [1, 2, 3] {
            let input = builder::transformed(&fixture(), transform);
            let before = snapshot(&input);
            let mut selected = selection_at(&input, transform)[..count].to_vec();
            let result = try_chamfer_solid_edges(&input, &selected, 0.2, TOL).unwrap();
            common::assert_solid(&result.solid, chamfer_volume(count), &[0], TOL);
            assert_history(&input, &result, count);
            assert_convex_geometry(&result);
            selected.reverse();
            let reversed = try_chamfer_solid_edges(&input, &selected, 0.2, TOL).unwrap();
            assert_eq!(snapshot(&result.solid), snapshot(&reversed.solid));
            if count == 1 {
                let control = chamfer_solid_edge(&input, selected[0], 0.2, 0.2, TOL).unwrap();
                common::assert_solid(&control.solid, chamfer_volume(1), &[0], TOL);
            } else {
                if count == 2 {
                    assert_unselected_edge(&result, transform, 0.2);
                }
                check_downstream(&result, chamfer_volume(count), false, transform);
            }
            assert_eq!(before, snapshot(&input));
        }
    }
}

#[test]
fn sequential_chamfers_reproduce_kernel_gap() {
    use truck_base::diagnostics::Code;
    for count in [2, 3] {
        for reverse in [false, true] {
            let input = fixture();
            let mut axes: Vec<_> =
                [Vector3::unit_x(), Vector3::unit_y(), -Vector3::unit_z()][..count].to_vec();
            if reverse {
                axes.reverse();
            }
            let edge_at = |solid: &Solid, axis| {
                let midpoint = Point3::new(0.0, 0.0, 10.0) + 5.0 * axis;
                common::blend::edge_through(&solid.boundaries()[0], midpoint).id()
            };
            let first =
                try_chamfer_solid_edge(&input, edge_at(&input, axes[0]), 0.2, 0.2, TOL).unwrap();
            common::assert_solid(&first.solid, chamfer_volume(1), &[0], TOL);
            // Resolve the shortened remaining edge on the new solid, independently of app references.
            let before = snapshot(&first.solid);
            let error =
                try_chamfer_solid_edge(&first.solid, edge_at(&first.solid, axes[1]), 0.2, 0.2, TOL)
                    .unwrap_err();
            assert_eq!(error.code, Code::BlendConstructionFailed);
            assert_eq!(before, snapshot(&first.solid));
        }
    }
}

#[test]
fn invalid_parameters_and_selections_leave_input_unchanged() {
    use truck_base::diagnostics::Code;
    let input = fixture();
    let before = snapshot(&input);
    let ids = selection(&input);
    for count in [2, 3] {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                try_fillet_solid_edges(&input, &ids[..count], bad, TOL)
                    .unwrap_err()
                    .code,
                Code::InvalidParameter
            );
            assert_eq!(
                try_chamfer_solid_edges(&input, &ids[..count], bad, TOL)
                    .unwrap_err()
                    .code,
                Code::InvalidParameter
            );
        }
        for bad in [10.0, 100.0, f64::MAX] {
            assert_eq!(
                try_fillet_solid_edges(&input, &ids[..count], bad, TOL)
                    .unwrap_err()
                    .code,
                Code::OutsideNeighbour
            );
            assert_eq!(
                try_chamfer_solid_edges(&input, &ids[..count], bad, TOL)
                    .unwrap_err()
                    .code,
                Code::OutsideNeighbour
            );
        }
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                try_fillet_solid_edges(&input, &ids[..count], 1.0, bad)
                    .unwrap_err()
                    .code,
                Code::InvalidTolerance
            );
            assert_eq!(
                try_chamfer_solid_edges(&input, &ids[..count], 0.2, bad)
                    .unwrap_err()
                    .code,
                Code::InvalidTolerance
            );
        }
    }
    for (selected, code) in [
        (vec![ids[0], ids[0]], Code::DuplicateSelection),
        (selection(&fixture()).to_vec(), Code::UnknownEdge),
    ] {
        assert_eq!(
            try_fillet_solid_edges(&input, &selected, 1.0, TOL)
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(
            try_chamfer_solid_edges(&input, &selected, 0.2, TOL)
                .unwrap_err()
                .code,
            code
        );
    }
    let no_op = try_chamfer_solid_edges(&input, &[], 0.2, TOL).unwrap();
    assert!(no_op.modified_faces.is_empty() && no_op.generated_faces.is_empty());
    assert_eq!(snapshot(&no_op.solid), before);
    assert_eq!(snapshot(&input), before);
}

#[test]
fn unsupported_junctions_have_distinct_diagnostics() {
    use truck_base::diagnostics::Code;
    let vertices = builder::vertices([
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [5.0, 5.0, 10.0],
    ]);
    let mut edges = std::collections::HashMap::new();
    let shell = [
        vec![3, 2, 1, 0],
        vec![0, 1, 4],
        vec![1, 2, 4],
        vec![2, 3, 4],
        vec![3, 0, 4],
    ]
    .into_iter()
    .map(|indices| {
        let wire: Wire = (0..indices.len())
            .map(|i| {
                let (a, b) = (indices[i], indices[(i + 1) % indices.len()]);
                let e = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| builder::line(&vertices[a.min(b)], &vertices[a.max(b)]));
                if a < b {
                    e.clone()
                } else {
                    e.inverse()
                }
            })
            .collect();
        builder::try_attach_plane(&[wire]).unwrap()
    })
    .collect();
    let input = Solid::new(vec![shell]);
    let before = snapshot(&input);
    let ids = [edges[&(0, 4)].id(), edges[&(1, 4)].id()];
    assert_eq!(
        try_fillet_solid_edges(&input, &ids, 1.0, TOL)
            .unwrap_err()
            .code,
        Code::UnsupportedTopology
    );
    assert_eq!(
        try_chamfer_solid_edges(&input, &ids, 0.2, TOL)
            .unwrap_err()
            .code,
        Code::UnsupportedTopology
    );
    assert_eq!(before, snapshot(&input));

    let transform = Matrix4::from_cols(
        Vector4::new(1.0, 0.2, 0.0, 0.0),
        Vector4::unit_y(),
        Vector4::unit_z(),
        Vector4::unit_w(),
    );
    let skew = builder::transformed(&fixture(), transform);
    let ids = selection_at(&skew, transform);
    let before = snapshot(&skew);
    assert_eq!(
        try_fillet_solid_edges(&skew, &ids[..2], 1.0, TOL)
            .unwrap_err()
            .code,
        Code::UnsupportedGeometry
    );
    assert_eq!(before, snapshot(&skew));
}

#[test]
fn miters_at_both_ends_and_next_to_a_spherical_corner() {
    use std::f64::consts::PI;
    for third in [false, true] {
        let input = fixture();
        let ids = selection(&input);
        let mut selected = ids[..2].to_vec();
        for end in [Point3::new(10.0, 0.0, 10.0), Point3::new(0.0, 10.0, 10.0)] {
            selected.push(edge_between(&input, Point3::new(10.0, 10.0, 10.0), end).id());
        }
        if third {
            selected.push(ids[2]);
        }
        let corners = if third { 3.0 } else { 4.0 };
        let round = try_fillet_solid_edges(&input, &selected, 1.0, TOL).unwrap();
        let expected = 1000.0 - selected.len() as f64 * 10.0 * (1.0 - PI / 4.0)
            + corners * (5.0 / 3.0 - PI / 2.0)
            + if third { 2.0 - 7.0 * PI / 12.0 } else { 0.0 };
        common::assert_solid(&round.solid, expected, &[0], TOL);
        assert_convex_geometry(&round);
        let bevel = try_chamfer_solid_edges(&input, &selected, 0.2, TOL).unwrap();
        let expected = 1000.0 - selected.len() as f64 * 10.0 * 0.2_f64.powi(2) / 2.0
            + corners * 0.2_f64.powi(3) / 3.0
            + if third {
                3.0 * 0.2_f64.powi(3) / 4.0
            } else {
                0.0
            };
        common::assert_solid(&bevel.solid, expected, &[0], TOL);
        assert_convex_geometry(&bevel);
    }
}
