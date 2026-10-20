mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::*;
fn fixture(angle: f64) -> (Solid, Wire) {
    let h = 0.782539615254126;
    let k = angle.tan();
    let points = [(0., 0.), (0.6, 0.), (1., 0.4), (1., 1.), (0., 1.)];
    let bottom: Vec<_> = points
        .iter()
        .map(|&(x, y)| Vertex::new(Point3::new(x, y, 0.)))
        .collect();
    let top: Vec<_> = points
        .iter()
        .map(|&(x, y)| Vertex::new(Point3::new(x, y, h - k * x)))
        .collect();
    let mut transform = Matrix4::identity();
    transform.x.z = -k;
    transform.w.z = h;
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    let vertical: Vec<_> = (0..5).map(|i| builder::line(&bottom[i], &top[i])).collect();
    for i in 0..5 {
        let j = (i + 1) % 5;
        let edge: Edge = if i == 1 {
            builder::circle_arc(
                &bottom[i],
                &bottom[j],
                Point3::new(0.6 + 0.4 / 2_f64.sqrt(), 0.4 - 0.4 / 2_f64.sqrt(), 0.),
            )
        } else {
            builder::line(&bottom[i], &bottom[j])
        };
        upper.push(Edge::new(
            &top[i],
            &top[j],
            edge.oriented_curve().transformed(transform),
        ));
        lower.push(edge);
    }
    let mut shell = Shell::new();
    shell.push(
        Face::new(
            vec![lower.iter().cloned().collect()],
            Plane::new(
                Point3::origin(),
                Point3::new(1., 0., 0.),
                Point3::new(0., 1., 0.),
            )
            .into(),
        )
        .inverse(),
    );
    shell.push(Face::new(
        vec![upper.iter().cloned().collect()],
        Plane::new(
            Point3::new(0., 0., h),
            Point3::new(1., 0., h - k),
            Point3::new(0., 1., h),
        )
        .into(),
    ));
    for i in 0..5 {
        let j = (i + 1) % 5;
        let surface: Surface = if i == 1 {
            ExtrudedCurve::by_extrusion(lower[i].oriented_curve(), Vector3::unit_z()).into()
        } else {
            Plane::new(bottom[i].point(), bottom[j].point(), top[j].point()).into()
        };
        shell.push(Face::new(
            vec![vec![
                lower[i].clone(),
                vertical[j].clone(),
                upper[i].inverse(),
                vertical[i].inverse(),
            ]
            .into()],
            surface,
        ));
    }
    let wire = upper[..3].iter().cloned().collect();
    (Solid::new(vec![shell]), wire)
}
fn assert_history(input: &Solid, result: &BlendResult, generated: usize) {
    assert_eq!(result.generated_faces.len(), generated);
    let mut accounted = HashSet::new();
    for original in input.face_iter() {
        let replacements: Vec<_> = result
            .modified_faces
            .iter()
            .filter(|(id, _)| *id == original.id())
            .collect();
        assert!(replacements.len() <= 1);
        let face = replacements.first().map_or(original, |(_, face)| face);
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        assert!(accounted.insert(face.id()));
    }
    for (id, _) in &result.modified_faces {
        assert!(input.face_iter().any(|face| face.id() == *id));
    }
    for face in &result.generated_faces {
        assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        assert!(accounted.insert(face.id()));
    }
    assert_eq!(accounted.len(), result.solid.face_iter().count());
}

fn input_volume(angle: f64) -> f64 {
    let a = 1. - std::f64::consts::PI / 4.;
    let c = 0.4 * (10. - 3. * std::f64::consts::PI) / (12. - 3. * std::f64::consts::PI);
    0.782539615254126 * (1. - 0.16 * a) - angle.tan() * (0.5 - 0.16 * a * (1. - c))
}
fn setback_error(input: &Solid, wire: &Wire, result: &BlendResult, distance: f64) -> (f64, usize) {
    let supports: Vec<_> = input
        .face_iter()
        .filter(|f| f.edge_iter().any(|e| wire.iter().any(|q| q.id() == e.id())))
        .map(|f| {
            result
                .modified_faces
                .iter()
                .find(|(id, _)| *id == f.id())
                .map_or(f, |(_, f)| f)
        })
        .collect();
    let mut seen = HashSet::new();
    let mut maximum = 0_f64;
    let mut samples = 0;
    for face in &result.generated_faces {
        for edge in face.edge_iter() {
            if !supports
                .iter()
                .any(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                || !seen.insert(edge.id())
            {
                continue;
            }
            let curve = edge.oriented_curve();
            let (a, b) = curve.range_tuple();
            for i in 0..=32 {
                let p = curve.subs(a + (b - a) * i as f64 / 32.);
                let nearest = wire
                    .iter()
                    .map(|e| {
                        let c = e.oriented_curve();
                        let (a, b) = c.range_tuple();
                        let t = c
                            .search_nearest_parameter(p, None, 100)
                            .unwrap()
                            .clamp(a, b);
                        p.distance(c.subs(t))
                    })
                    .fold(f64::INFINITY, f64::min);
                maximum = maximum.max((nearest - distance).abs());
                samples += 1;
            }
        }
    }
    (maximum, samples)
}

fn check(solid: &Solid, wire: &Wire, distance: f64, expected: f64) {
    let before = serde_json::to_string(&solid.compress()).unwrap();
    let result = try_chamfer_solid_along_wire(solid, wire, distance, distance, 0.001);
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    let result = result.expect("tangent rounded rim under a tilted planar cap");
    assert_history(solid, &result, 3);
    let (error, samples) = setback_error(solid, wire, &result, distance);
    assert_eq!(samples, 198);
    assert!(error < 0.001, "setback error {error}");
    common::assert_solid(&result.solid, expected, &[0], 0.001);
    common::blend::assert_step(&result.solid, expected, 0.001);
    assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
}
#[test]
fn tilted_cap_chamfers_preserve_material_history_and_step() {
    // Independent native control volumes; input material is also checked analytically.
    for (degrees, expected) in [
        (2_f64, 0.7304309859056308),
        (-2., 0.7631585483063213),
        (10., 0.664180532704892),
    ] {
        let (solid, wire) = fixture(degrees.to_radians());
        common::assert_solid(&solid, input_volume(degrees.to_radians()), &[0], 0.001);
        for wire in [wire.clone(), wire.inverse()] {
            check(&solid, &wire, 0.1, expected);
        }
    }
}
#[test]
fn tilted_cap_chamfers_survive_similarity_transforms() {
    let (original, wire) = fixture(2_f64.to_radians());
    for scale in [0.5, 5.] {
        let matrix = Matrix4::from_translation(Vector3::new(7., -11., 13.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let solid = builder::transformed(&original, matrix);
        let selected: Wire = wire
            .iter()
            .map(|edge| {
                let a = matrix.transform_point(edge.front().point());
                let b = matrix.transform_point(edge.back().point());
                solid
                    .edge_iter()
                    .find_map(|e| {
                        if (e.front().point() - a).magnitude() < 1e-8
                            && (e.back().point() - b).magnitude() < 1e-8
                        {
                            Some(e.clone())
                        } else if (e.front().point() - b).magnitude() < 1e-8
                            && (e.back().point() - a).magnitude() < 1e-8
                        {
                            Some(e.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        common::assert_solid(
            &solid,
            input_volume(2_f64.to_radians()) * scale.powi(3),
            &[0],
            0.001,
        );
        for selected in [selected.clone(), selected.inverse()] {
            check(
                &solid,
                &selected,
                0.1 * scale,
                0.7304309859056308 * scale.powi(3),
            );
        }
    }
}
#[test]
fn horizontal_cap_retains_its_chamfer_contract() {
    let (solid, wire) = fixture(0.);
    common::assert_solid(&solid, input_volume(0.), &[0], 0.001);
    for wire in [wire.clone(), wire.inverse()] {
        check(&solid, &wire, 0.1, 0.7467903118046886);
    }
}

#[test]
fn tight_contact_tolerance_and_folded_offsets_remain_rejected() {
    let (solid, wire) = fixture(10_f64.to_radians());
    let before = serde_json::to_string(&solid.compress()).unwrap();
    for selected in [wire.clone(), wire.inverse()] {
        assert!(try_chamfer_solid_along_wire(&solid, &selected, 0.1, 0.1, 1e-7).is_err());
        assert!(try_chamfer_solid_along_wire(&solid, &selected, 0.6, 0.6, 0.001).is_err());
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}
