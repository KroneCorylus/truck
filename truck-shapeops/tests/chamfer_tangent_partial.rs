mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

fn rounded_rectangle(hole: bool) -> Solid { rounded_rectangle_shift(hole, 0., 0.) }

fn rounded_rectangle_shift(hole: bool, shift: f64, slant: f64) -> Solid {
    let p = [
        (-1., -1.),
        (0.1 + slant, -1.),
        (0.1, 0.),
        (0., 0.1),
        (-1., 0.1),
    ]
    .map(|(x, y)| Vertex::new(Point3::new(x, y, 0.)));
    let arc = builder::rsweep(
        &p[2],
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::FRAC_PI_2),
        1,
    );
    let edge = Edge::new(&p[2], &p[3], arc[0].curve());
    let wire = Wire::from(vec![
        builder::line(&p[0], &p[1]),
        builder::line(&p[1], &p[2]),
        edge,
        builder::line(&p[3], &p[4]),
        builder::line(&p[4], &p[0]),
    ]);
    let mut loops = vec![wire];
    if hole {
        let p = [(-0.7, -0.7), (-0.7, -0.5), (-0.5, -0.5), (-0.5, -0.7)]
            .map(|(x, y)| Vertex::new(Point3::new(x + shift, y + shift, 0.)));
        loops.push(
            (0..4)
                .map(|i| builder::line(&p[i], &p[(i + 1) % 4]))
                .collect(),
        );
    }
    builder::tsweep(
        &builder::try_attach_plane(loops).unwrap(),
        Vector3::unit_z(),
    )
}

fn check(b: f64) { check_variant(b, false, false); }

fn check_variant(b: f64, hole: bool, both: bool) {
    let input = rounded_rectangle(hole);
    let mut seen = std::collections::HashSet::new();
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| {
            (e.front().point().z == 1. && e.back().point().z == 1.
                || both && e.front().point().z == 0. && e.back().point().z == 0.)
                && (e.front().point().x >= 0.
                    || e.front().point().y >= 0.
                    || e.front().point().x == -1.
                    || e.front().point().y == -1.)
                && seen.insert(e.id())
        })
        .map(|e| {
            let faces: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|x| x.id() == e.id()))
                .collect();
            (
                e.id(),
                std::array::from_fn(|i| match faces[i].oriented_surface() {
                    Surface::Plane(p) if p.normal().z.abs() > 0.9 => b,
                    _ => 0.03,
                }),
            )
        })
        .collect();
    assert_eq!(selected.len(), if both { 10 } else { 5 });
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    let result = result.expect("bounded tangent contour must construct");
    let pi = std::f64::consts::PI;
    let area = 1.1 * 1.1 - (1. - pi / 4.) * 0.1 * 0.1;
    let perimeter = 4.2 + pi * 0.1 / 2.;
    let volume = area
        - if hole { 0.04 } else { 0. }
        - if both { 2. } else { 1. } * 0.03 * (perimeter * b / 2. - (3. + pi / 4.) * b * b / 3.);
    for angle in [0., 0.2, 0.7, std::f64::consts::FRAC_PI_2] {
        let n = Vector3::new(angle.cos(), angle.sin(), 0.);
        for f in [0.25, 0.5, 0.75] {
            let rho = 0.1 - b + b * f;
            let p = Point3::new(rho * n.x, rho * n.y, 1. - 0.03 * f);
            let (cone, uv) = result
                .solid
                .face_iter()
                .filter(|face| matches!(face.surface(), Surface::RevolutedCurve(_)))
                .find_map(|face| {
                    let cone = face.oriented_surface();
                    let uv = cone.search_parameter(p, None, 100)?;
                    Some((cone, uv))
                })
                .expect("analytic cone contact");
            assert!(cone.subs(uv.0, uv.1).distance(p) < 1e-8);
            let normal = (n + Vector3::unit_z() * (b / 0.03)).normalize();
            assert!(cone.normal(uv.0, uv.1).dot(normal) > 1. - 1e-8);
        }
    }
    for p in [
        Point3::new(0.1 - b, 0., 1.),
        Point3::new(0., 0.1 - b, 1.),
        Point3::new(0.1, 0., 0.97),
        Point3::new(0., 0.1, 0.97),
    ] {
        assert!(result
            .solid
            .vertex_iter()
            .any(|v| v.point().distance(p) < 1e-9));
    }
    if !both {
        for edge in input
            .edge_iter()
            .filter(|e| matches!(e.curve(), Curve::Conic(_)) && e.front().point().z == 0.)
        {
            assert!(result.solid.edge_iter().any(|e| e.id() == edge.id()
                && serde_json::to_string(&e.curve()).unwrap()
                    == serde_json::to_string(&edge.curve()).unwrap()));
        }
    }
    common::assert_solid(&result.solid, volume, &[usize::from(hole)], 0.001);
    common::blend::assert_step(&result.solid, volume, 0.001);
}
#[test]
fn tangent_equal() { check(0.03); }
#[test]
fn tangent_dd() { check(0.04); }
#[test]
fn tangent_da() { check(0.03 * 3_f64.sqrt()); }

#[test]
fn tangent_remote_hole() { check_variant(0.04, true, false); }
#[test]
fn tangent_both_caps() { check_variant(0.04, false, true); }

#[test]
fn tangent_invalid_distances_are_immutable() {
    let input = rounded_rectangle(false);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().z == 1. && e.back().point().z == 1. && seen.insert(e.id()))
        .map(|e| e.id())
        .collect();
    for distance in [0., -0.03, 0.1, 1., f64::NAN, f64::INFINITY] {
        let selected: Vec<_> = ids.iter().map(|&id| (id, [distance; 2])).collect();
        assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
    let mut selected: Vec<_> = ids.iter().map(|&id| (id, [0.03; 2])).collect();
    let rim = input
        .edge_iter()
        .find(|e| ids.contains(&e.id()) && matches!(e.curve(), Curve::Conic(_)))
        .unwrap()
        .id();
    selected.iter_mut().find(|e| e.0 == rim).unwrap().1 = [0.04; 2];
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn tangent_rigid_scale_and_order() {
    for scale in [0.1, 1., 10.] {
        for reorder in [false, true] {
            let transform = Matrix4::from_translation(Vector3::new(3., -2., 4.))
                * Matrix4::from_angle_x(Rad(0.7))
                * Matrix4::from_scale(scale);
            let inverse = transform.invert().unwrap();
            let mut input = builder::transformed(&rounded_rectangle(false), transform);
            if reorder {
                input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
            }
            let mut seen = std::collections::HashSet::new();
            let mut selected: Vec<_> = input
                .edge_iter()
                .filter(|e| {
                    (inverse.transform_point(e.front().point()).z - 1.).abs() < 1e-10
                        && (inverse.transform_point(e.back().point()).z - 1.).abs() < 1e-10
                        && seen.insert(e.id())
                })
                .map(|e| {
                    let faces: Vec<_> = input
                        .face_iter()
                        .filter(|f| f.edge_iter().any(|x| x.id() == e.id()))
                        .collect();
                    (
                        e.id(),
                        std::array::from_fn(|i| match faces[i].oriented_surface() {
                            Surface::Plane(p)
                                if inverse.transform_vector(p.normal()).normalize().z.abs()
                                    > 0.9 =>
                            {
                                0.04 * scale
                            }
                            _ => 0.03 * scale,
                        }),
                    )
                })
                .collect();
            if reorder {
                selected.reverse();
            }
            assert_eq!(selected.len(), 5);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let result =
                try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001 * scale)
                    .unwrap();
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            common::assert_solid(
                &result.solid,
                1.2053003002249814 * scale.powi(3),
                &[0],
                0.001 * scale,
            );
            for local in [Point3::new(0.06, 0., 1.), Point3::new(0.1, 0., 0.97)] {
                assert!(result
                    .solid
                    .vertex_iter()
                    .any(|v| inverse.transform_point(v.point()).distance(local) < 1e-8));
            }
        }
    }
}

#[test]
fn tangent_reversed_shell_rejects_immutably() {
    let mut shell = rounded_rectangle(false).boundaries()[0].clone();
    for face in shell.iter_mut() {
        face.invert();
    }
    let input = Solid::new(vec![shell]);
    let mut seen = std::collections::HashSet::new();
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().z == 1. && e.back().point().z == 1. && seen.insert(e.id()))
        .map(|e| (e.id(), [0.03; 2]))
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn tangent_selection_and_overlap_rejections() {
    for kind in 0..4 {
        let transform = if kind == 3 {
            Matrix4::from_nonuniform_scale(1., 1., 0.05)
        } else {
            Matrix4::from_scale(1.)
        };
        let input = builder::transformed(&rounded_rectangle(false), transform);
        let top = if kind == 3 { 0.05 } else { 1. };
        let mut seen = std::collections::HashSet::new();
        let mut selected: Vec<_> = input
            .edge_iter()
            .filter(|e| {
                ((e.front().point().z == top && e.back().point().z == top)
                    || kind == 3 && e.front().point().z == 0. && e.back().point().z == 0.)
                    && seen.insert(e.id())
            })
            .map(|e| (e.id(), [0.03; 2]))
            .collect();
        let rim = input
            .edge_iter()
            .find(|e| e.front().point().z == top && matches!(e.curve(), Curve::Conic(_)))
            .unwrap();
        if kind == 0 {
            selected.retain(|e| e.0 != rim.id());
        }
        if kind == 1 {
            selected.retain(|e| e.0 == rim.id());
        }
        if kind == 2 {
            let seam = input
                .edge_iter()
                .find(|e| {
                    e.front().point().z != e.back().point().z
                        && e.front().point().x == 0.1
                        && e.front().point().y == 0.
                })
                .unwrap();
            selected.push((seam.id(), [0.03; 2]));
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(
            try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err(),
            "kind {kind}"
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn tangent_remote_hole_collision_rejects_immutably() {
    let input = rounded_rectangle_shift(true, 0.55, 0.);
    let cap = input
        .face_iter()
        .find(|f| matches!(f.oriented_surface(),Surface::Plane(p) if p.normal().z>0.9))
        .unwrap();
    let wire = cap
        .boundaries()
        .into_iter()
        .find(|w| w.vertex_iter().any(|v| v.point().x == -1.))
        .unwrap();
    let selected: Vec<_> = wire.iter().map(|e| (e.id(), [0.04; 2])).collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn tangent_reversed_edge_representation() {
    let original = rounded_rectangle(false);
    let mut edges = std::collections::HashMap::new();
    for e in original.edge_iter() {
        edges.entry(e.id()).or_insert_with(|| {
            let e = e.absolute_clone();
            Edge::new(e.back(), e.front(), e.curve().inverse())
        });
    }
    let shell = original
        .face_iter()
        .map(|face| {
            let boundaries = face
                .boundaries()
                .iter()
                .map(|w| {
                    w.iter()
                        .map(|e| {
                            let new = &edges[&e.id()];
                            if e.orientation() {
                                new.inverse()
                            } else {
                                new.clone()
                            }
                        })
                        .collect()
                })
                .collect();
            Face::new(boundaries, face.oriented_surface())
        })
        .collect();
    let input = Solid::new(vec![shell]);
    let mut seen = std::collections::HashSet::new();
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().z == 1. && e.back().point().z == 1. && seen.insert(e.id()))
        .map(|e| (e.id(), [0.03; 2]))
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    common::assert_solid(&result.solid, 1.2059273643827395, &[0], 0.001);
}

#[test]
fn tangent_coordinate_uncertainty_rejects_immutably() {
    let input = builder::translated(&rounded_rectangle(false), Vector3::new(1e12, 0., 0.));
    let mut seen = std::collections::HashSet::new();
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().z == 1. && e.back().point().z == 1. && seen.insert(e.id()))
        .map(|e| (e.id(), [0.03; 2]))
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn tangent_oblique_cap_rejects_immutably() {
    let transform = Matrix4::from_cols(
        Vector4::new(1., 0., 0.1, 0.),
        Vector4::unit_y(),
        Vector4::unit_z(),
        Vector4::unit_w(),
    );
    let input = builder::transformed(&rounded_rectangle(false), transform);
    let cap = input
        .face_iter()
        .find(|f| matches!(f.oriented_surface(),Surface::Plane(p) if p.normal().normalize().z>0.9))
        .unwrap();
    let selected: Vec<_> = cap.boundaries()[0]
        .iter()
        .map(|e| (e.id(), [0.03; 2]))
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn tangent_nontangent_support_rejects_immutably() {
    let input = rounded_rectangle_shift(false, 0., 0.02);
    let mut seen = std::collections::HashSet::new();
    let selected: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().z == 1. && e.back().point().z == 1. && seen.insert(e.id()))
        .map(|e| (e.id(), [0.03; 2]))
        .collect();
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
