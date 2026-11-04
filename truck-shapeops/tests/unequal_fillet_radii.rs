mod common;
#[path = "fixtures/unequal-radii/model.rs"]
mod fixture;
use truck_base::diagnostics::Code;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

fn input(text: &str) -> Solid {
    let table = Table::from_step(text).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(1e-5);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn selected(
    solid: &Solid,
    case: &fixture::Case,
    inverse: Matrix4,
    scale: f64,
) -> Vec<(EdgeID, f64)> {
    case.edges
        .iter()
        .map(|&(a, b, r)| {
            let edge = solid
                .edge_iter()
                .find(|e| {
                    let (p, q) = (
                        inverse.transform_point(e.front().point()),
                        inverse.transform_point(e.back().point()),
                    );
                    (p.near(&a) && q.near(&b)) || (p.near(&b) && q.near(&a))
                })
                .unwrap();
            (edge.id(), r * scale)
        })
        .collect()
}

#[test]
fn unequal_straight_radii_preserve_material_normals_history_and_step() {
    for name in ["A2", "B5", "C4"] {
        let case = fixture::case(name);
        for (scale, moved) in [(1., false), (1., true), (0.1, true)] {
            let transform = if moved {
                Matrix4::from_translation(Vector3::new(2., -3., 5.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(scale)
            } else {
                Matrix4::identity()
            };
            let inverse = transform.invert().unwrap();
            let solid = builder::transformed(&input(case.step), transform);
            let before = serde_json::to_string(&solid.compress()).unwrap();
            let selection = selected(&solid, &case, inverse, scale);
            let mut reference = None;
            for order in [selection.clone(), selection.into_iter().rev().collect()] {
                let result = try_fillet_solid_edges_with_radii(&solid, &order, 0.001).unwrap();
                assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
                assert!(result.solid.is_geometric_consistent());
                assert_eq!(result.generated_faces.len(), 2);
                let old: Vec<_> = solid.face_iter().collect();
                let new: Vec<_> = result.solid.face_iter().collect();
                let expected: Vec<_> = old
                    .iter()
                    .zip(&new)
                    .filter(|(a, b)| a.id() != b.id())
                    .map(|(a, b)| (a.id(), b.id()))
                    .collect();
                assert_eq!(
                    result
                        .modified_faces
                        .iter()
                        .map(|(a, b)| (*a, b.id()))
                        .collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    result
                        .generated_faces
                        .iter()
                        .map(|f| f.id())
                        .collect::<Vec<_>>(),
                    new.iter()
                        .skip(old.len())
                        .map(|f| f.id())
                        .collect::<Vec<_>>()
                );
                let compressed = serde_json::to_string(&result.solid.compress()).unwrap();
                if let Some(expected) = &reference {
                    assert_eq!(&compressed, expected);
                } else {
                    reference = Some(compressed);
                }
                let mut observed = std::collections::HashSet::new();
                for face in &result.generated_faces {
                    let surface = face.oriented_surface();
                    let (u, v) = surface.try_range_tuple();
                    let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                    let middle = inverse.transform_point(surface.subs((a + b) / 2., (c + d) / 2.));
                    let index = case
                        .axes
                        .iter()
                        .position(|&(center, axis, radius)| {
                            ((middle - center).cross(axis).magnitude() - radius).abs() < 1e-6
                        })
                        .unwrap();
                    assert!(observed.insert(index));
                    let (center, axis, radius) = case.axes[index];
                    for i in 0..=8 {
                        for j in 0..=8 {
                            let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 8.);
                            let p = inverse.transform_point(surface.subs(u, v));
                            let delta = p - center;
                            let radial = delta - axis * delta.dot(axis);
                            assert!(
                                (radial.magnitude() - radius).abs() * scale < 1e-6,
                                "{name}: {p:?}"
                            );
                            let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                            assert!(
                                normal.dot(radial.normalize()) > 1. - 1e-6,
                                "{name}: {p:?} {normal:?}"
                            );
                        }
                    }
                }
                common::assert_solid(&result.solid, case.volume * scale.powi(3), &[0], 0.001);
                common::blend::assert_step(&result.solid, case.volume * scale.powi(3), 0.001);
            }
        }
    }
}

#[test]
fn unequal_radii_validate_every_assignment_and_preserve_input_on_failure() {
    let case = fixture::case("A2");
    let solid = input(case.step);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let selection = selected(&solid, &case, Matrix4::identity(), 1.);
    for radius in [0., -1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut invalid = selection.clone();
        invalid[1].1 = radius;
        let error = try_fillet_solid_edges_with_radii(&solid, &invalid, 0.001).unwrap_err();
        assert_eq!(error.code, Code::InvalidParameter);
        assert_eq!(error.context.selection_index, Some(1));
    }
    let duplicate = [(selection[0].0, 2.), (selection[0].0, 3.)];
    assert_eq!(
        try_fillet_solid_edges_with_radii(&solid, &duplicate, 0.001)
            .unwrap_err()
            .code,
        Code::DuplicateSelection
    );
    let unknown = common::modeling::cuboid(Point3::origin(), Point3::new(1., 1., 1.))
        .edge_iter()
        .next()
        .unwrap()
        .id();
    assert_eq!(
        try_fillet_solid_edges_with_radii(&solid, &[(selection[0].0, 2.), (unknown, 3.)], 0.001)
            .unwrap_err()
            .code,
        Code::UnknownEdge
    );
    let mut oversized = selection.clone();
    oversized[1].1 = 100.;
    assert!(try_fillet_solid_edges_with_radii(&solid, &oversized[..1], 0.001).is_ok());
    assert!(try_fillet_solid_edges_with_radii(&solid, &oversized, 0.001).is_err());
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}

#[test]
fn unequal_box_crease_junctions_and_parallel_round_guards() {
    let solid = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(10., 10., 10.),
    ]));
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let edge = |a: Point3, b: Point3| {
        solid
            .edge_iter()
            .find(|e| {
                (e.front().point().near(&a) && e.back().point().near(&b))
                    || (e.front().point().near(&b) && e.back().point().near(&a))
            })
            .unwrap()
            .id()
    };
    let first = edge(Point3::origin(), Point3::new(0., 0., 10.));
    let connected = edge(Point3::origin(), Point3::new(10., 0., 0.));
    // A rectangular convex corner has a two-cylinder crease miter.
    for order in [
        [(first, 1.), (connected, 2.)],
        [(connected, 2.), (first, 1.)],
    ] {
        let result = try_fillet_solid_edges_with_radii(&solid, &order, 0.001).unwrap();
        common::assert_solid(&result.solid, 8. * 123.69042579279512, &[0], 0.001);
        common::blend::assert_step(&result.solid, 8. * 123.69042579279512, 0.001);
    }
    let opposite = edge(Point3::new(10., 0., 0.), Point3::new(10., 0., 10.));
    for order in [[(first, 6.), (opposite, 5.)], [(opposite, 5.), (first, 6.)]] {
        assert!(try_fillet_solid_edges_with_radii(&solid, &order, 0.001).is_err());
    }
    let result =
        try_fillet_solid_edges_with_radii(&solid, &[(first, 1.), (opposite, 2.)], 0.001).unwrap();
    let volume = 1000. - 50. * (1. - std::f64::consts::PI / 4.);
    common::assert_solid(&result.solid, volume, &[0], 0.001);
    common::blend::assert_step(&result.solid, volume, 0.001);
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}

#[test]
fn equal_and_empty_assignments_retain_existing_solid_behavior() {
    let solid = common::modeling::cuboid(Point3::origin(), Point3::new(10., 10., 10.));
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<_> = solid
        .edge_iter()
        .map(|e| e.id())
        .filter(|id| seen.insert(*id))
        .collect();
    let equal: Vec<_> = ids.iter().map(|&id| (id, 1.)).collect();
    let a = try_fillet_solid_edges(&solid, &ids, 1., 0.001).unwrap();
    let b = try_fillet_solid_edges_with_radii(&solid, &equal, 0.001).unwrap();
    assert_eq!(
        serde_json::to_string(&a.solid.compress()).unwrap(),
        serde_json::to_string(&b.solid.compress()).unwrap()
    );
    assert_eq!(a.modified_faces.len(), b.modified_faces.len());
    assert_eq!(a.generated_faces.len(), b.generated_faces.len());
    let empty = try_fillet_solid_edges_with_radii(&solid, &[], 0.001).unwrap();
    assert!(empty.modified_faces.is_empty() && empty.generated_faces.is_empty());
    assert_eq!(
        serde_json::to_string(&empty.solid.compress()).unwrap(),
        serde_json::to_string(&solid.compress()).unwrap()
    );
}
