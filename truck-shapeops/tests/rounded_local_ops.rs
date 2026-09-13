mod common;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::{
    fillet::try_fillet_solid_edges,
    local::{try_draft, try_shell, try_shell_outward},
};
const TOL: f64 = 0.001;
fn rounded(concave: bool) -> Solid {
    let points = if concave {
        vec![
            (0., 0.),
            (20., 0.),
            (20., 10.),
            (10., 10.),
            (10., 20.),
            (0., 20.),
        ]
    } else {
        vec![(0., 0.), (20., 0.), (20., 20.), (0., 20.)]
    };
    let vertices = builder::vertices(points.into_iter().map(|(x, y)| Point3::new(x, y, 0.)));
    let wire: Wire = (0..vertices.len())
        .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % vertices.len()]))
        .collect();
    let solid = builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_z() * 10.,
    );
    let mut ids: Vec<_> = solid
        .edge_iter()
        .filter(|e| (e.front().point().z - e.back().point().z).abs() > 9.)
        .map(|e| e.id())
        .collect();
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| seen.insert(*id));
    try_fillet_solid_edges(&solid, &ids, 1., TOL).unwrap().solid
}
fn area_perimeter(concave: bool) -> (f64, f64) {
    let k = 1. - PI / 4.;
    (
        (if concave { 300. } else { 400. }) - 4. * k,
        80. - (if concave { 12. } else { 8. }) * k,
    )
}
fn transforms() -> [Matrix4; 2] {
    [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ]
}
#[test]
fn rounded_prisms_shell_inward_and_outward_with_constant_wall_thickness() {
    for concave in [false, true] {
        for transform in transforms() {
            let input = builder::transformed(&rounded(concave), transform);
            let inverse = transform.invert().unwrap();
            let before = serde_json::to_string(&input.compress()).unwrap();
            let top = input
                .face_iter()
                .find(|f| {
                    f.vertex_iter()
                        .all(|v| (inverse.transform_point(v.point()).z - 10.).abs() < 1e-6)
                })
                .unwrap()
                .id();
            let (area, perimeter) = area_perimeter(concave);
            common::assert_solid(&input, area * 10., &[0], TOL);
            for outward in [false, true] {
                let t = 0.5;
                let result = if outward {
                    try_shell_outward(&input, &[top], t)
                } else {
                    try_shell(&input, &[top], t)
                }
                .unwrap();
                let expected = if outward {
                    (area + perimeter * t + PI * t * t) * (10. + t) - area * 10.
                } else {
                    area * 10. - (area - perimeter * t + PI * t * t) * (10. - t)
                };
                assert!(result.is_geometric_consistent());
                common::assert_solid(&result, expected, &[0], TOL);
                common::blend::assert_step(&result, expected, TOL);
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}
#[test]
fn drafting_planar_walls_propagates_across_tangent_corner_rounds() {
    for concave in [false, true] {
        for transform in transforms() {
            let input = builder::transformed(&rounded(concave), transform);
            let inverse = transform.invert().unwrap();
            let before = serde_json::to_string(&input.compress()).unwrap();
            let height = |v: &Vertex| inverse.transform_point(v.point()).z;
            let walls = input
                .face_iter()
                .filter(|f| {
                    matches!(f.surface(), Surface::Plane(_))
                        && f.vertex_iter().any(|v| height(&v) < 1.)
                        && f.vertex_iter().any(|v| height(&v) > 9.)
                })
                .map(|f| f.id())
                .collect::<Vec<_>>();
            let bottom = input
                .face_iter()
                .find(|f| f.vertex_iter().all(|v| height(&v).abs() < 1e-6))
                .unwrap();
            let (area, perimeter) = area_perimeter(concave);
            for sign in [-1., 1.] {
                let angle = Rad(sign * PI / 60.);
                let result = try_draft(
                    &input,
                    &walls,
                    &Plane::xy().transformed(transform),
                    transform.transform_vector(Vector3::unit_z()),
                    angle,
                )
                .unwrap();
                let tangent = angle.0.tan();
                let expected =
                    area * 10. + perimeter * tangent * 50. + PI * tangent * tangent * 1000. / 3.;
                assert!(result.is_geometric_consistent());
                common::assert_solid(&result, expected, &[0], TOL);
                common::blend::assert_step(&result, expected, TOL);
                let neutral = result
                    .face_iter()
                    .find(|f| f.vertex_iter().all(|v| height(&v).abs() < 1e-6))
                    .unwrap();
                assert_eq!(
                    serde_json::to_string(&bottom.oriented_surface()).unwrap(),
                    serde_json::to_string(&neutral.oriented_surface()).unwrap()
                );
                for vertex in bottom.vertex_iter() {
                    assert!(neutral.vertex_iter().any(|v| v.id() == vertex.id()));
                }
                for edge in bottom.edge_iter() {
                    let curve = edge.oriented_curve();
                    let (a, b) = curve.range_tuple();
                    for t in [0., 0.25, 0.5, 0.75, 1.] {
                        let p = curve.subs(a + t * (b - a));
                        assert!(neutral
                            .edge_iter()
                            .any(|e| e.curve().search_parameter(p, None, 100).is_some()));
                    }
                }
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}
#[test]
fn draft_propagation_stops_at_non_tangent_walls() {
    let input = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(20., 20., 10.),
    ]));
    let selected = input
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| (v.point().x - 20.).abs() < 1e-6))
        .unwrap()
        .id();
    let untouched = input
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| v.point().x.abs() < 1e-6))
        .unwrap();
    let result = try_draft(
        &input,
        &[selected],
        &Plane::xy(),
        Vector3::unit_z(),
        Rad(PI / 60.),
    )
    .unwrap();
    assert!(result.face_iter().any(|f| f.id() == untouched.id()));
    assert!(result.is_geometric_consistent());
    common::assert_solid(&result, 4000. + 1000. * (PI / 60.).tan(), &[0], TOL);
}
