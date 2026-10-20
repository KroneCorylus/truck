#[path = "/home/krone/Development/Pieza_WS/truck/truck-shapeops/tests/common/mod.rs"]
mod common;
use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::{
    fillet_solid_along_wire, try_fillet_solid_edges_with_radii, BlendResult,
};

fn input() -> Solid {
    let cube = primitive::cuboid(BoundingBox::from_iter([
        Point3::origin(),
        Point3::new(12., 10., 8.),
    ]));
    let mut seen = HashSet::new();
    let edges: Vec<_> = cube
        .edge_iter()
        .filter(|e| {
            let d = e.back().point() - e.front().point();
            d.x.abs() < 1e-8 && d.y.abs() < 1e-8 && seen.insert(e.id())
        })
        .map(|e| {
            let p = e.front().point();
            let r = if p.x < 6. {
                if p.y < 5. {
                    1.
                } else {
                    2.
                }
            } else if p.y < 5. {
                2.
            } else {
                3.
            };
            (e.id(), r)
        })
        .collect();
    try_fillet_solid_edges_with_radii(&cube, &edges, 0.001)
        .unwrap()
        .solid
}
fn rim(solid: &Solid) -> Wire {
    solid
        .face_iter()
        .find(|f| f.vertex_iter().all(|v| (v.point().z - 8.).abs() < 1e-8))
        .unwrap()
        .boundaries()[0]
        .clone()
}
fn expected(radius: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let initial = 960. - (1. - pi / 4.) * 8. * 18.;
    let area = radius * radius * (1. - pi / 4.);
    let centroid = radius * (10. - 3. * pi) / (12. - 3. * pi);
    initial - area * (28. + pi / 2. * (8. - 4. * centroid))
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

fn assert_rounds(result: &BlendResult, inverse: Matrix4, scale: f64) {
    let corners = [
        (Point3::new(1., 1., 7.), 0.),
        (Point3::new(10., 2., 7.), 1.),
        (Point3::new(9., 7., 7.), 2.),
        (Point3::new(2., 8., 7.), 1.),
    ];
    let mut counts = [0; 3];
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let kind = surface.elementary().expect("analytic rolling patch");
        match kind {
            (Elementary::Sphere { radius, .. }, true) => {
                assert!((radius - scale).abs() < 1e-8);
                counts[0] += 1;
            }
            (Elementary::Torus { minor, .. }, true) => {
                assert!((minor - scale).abs() < 1e-8);
                counts[1] += 1;
            }
            (Elementary::Cylinder { radius, .. }, true) => {
                assert!((radius - scale).abs() < 1e-8);
                counts[2] += 1;
            }
            _ => panic!("unexpected generated surface"),
        }
        if matches!(kind.0, Elementary::Sphere { .. } | Elementary::Torus { .. }) {
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            let middle = inverse.transform_point(surface.subs((a + b) / 2., (c + d) / 2.));
            let (center, major) = corners
                .iter()
                .min_by(|(ca, ra), (cb, rb)| {
                    let residual = |center: Point3, radius: f64| {
                        let p = middle - center;
                        ((p.x.hypot(p.y) - radius).hypot(p.z) - 1.).abs()
                    };
                    residual(*ca, *ra).total_cmp(&residual(*cb, *rb))
                })
                .copied()
                .unwrap();
            for i in 0..=8 {
                for j in 0..=8 {
                    let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 8.);
                    let p = inverse.transform_point(surface.subs(u, v));
                    let q = p - center;
                    let tube = if major == 0. {
                        center
                    } else {
                        center + Vector3::new(q.x, q.y, 0.).normalize() * major
                    };
                    assert!(((p - tube).magnitude() - 1.).abs() < 1e-8);
                    let normal = inverse.transform_vector(surface.normal(u, v)).normalize();
                    assert!(normal.dot((p - tube).normalize()) > 1. - 1e-8);
                }
            }
        }
    }
    assert_eq!(counts, [1, 3, 4]);
}
#[test]
fn mixed_collapsed_and_regular_corners_preserve_material_history_and_step() {
    let solid = input();
    let wire = rim(&solid);
    assert_eq!(wire.len(), 8);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    for offset in 0..wire.len() {
        let selected: Wire = (0..wire.len())
            .map(|j| wire[(j + offset) % wire.len()].clone())
            .collect();
        for selected in [selected.clone(), selected.inverse()] {
            let result = fillet_solid_along_wire(&solid, &selected, 1., 0.001)
                .expect("one sphere and three regular torus corners");
            assert_history(&solid, &result, 8);
            assert_rounds(&result, Matrix4::identity(), 1.);
            common::assert_solid(&result.solid, expected(1.), &[0], 0.001);
            common::blend::assert_step(&result.solid, expected(1.), 0.001);
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
#[test]
fn mixed_corner_continuation_survives_scale_rotation_and_reversed_wire() {
    let original = input();
    for scale in [0.25, 3.] {
        let matrix = Matrix4::from_translation(Vector3::new(13., -7., 19.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let solid = builder::transformed(&original, matrix);
        let wire = solid
            .face_iter()
            .find(|f| {
                f.vertex_iter().all(|v| {
                    (matrix.invert().unwrap().transform_point(v.point()).z - 8.).abs() < 1e-8
                })
            })
            .unwrap()
            .boundaries()[0]
            .clone();
        let before = serde_json::to_string(&solid.compress()).unwrap();
        for selected in [wire.clone(), wire.inverse()] {
            let result = fillet_solid_along_wire(&solid, &selected, scale, 0.001)
                .expect("transformed mixed corners");
            assert_history(&solid, &result, 8);
            assert_rounds(&result, matrix.invert().unwrap(), scale);
            common::assert_solid(&result.solid, expected(1.) * scale.powi(3), &[0], 0.001);
            common::blend::assert_step(&result.solid, expected(1.) * scale.powi(3), 0.001);
            assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
        }
    }
}
#[test]
fn regular_mixed_corners_and_larger_fold_keep_their_contracts() {
    let solid = input();
    let wire = rim(&solid);
    let before = serde_json::to_string(&solid.compress()).unwrap();
    for selected in [wire.clone(), wire.inverse()] {
        let result =
            fillet_solid_along_wire(&solid, &selected, 0.5, 0.001).expect("regular torus corners");
        assert_history(&solid, &result, 8);
        common::assert_solid(&result.solid, expected(0.5), &[0], 0.001);
        common::blend::assert_step(&result.solid, expected(0.5), 0.001);
        assert!(fillet_solid_along_wire(&solid, &selected, 1.2, 0.001).is_none());
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}

#[test]
fn export_mixed_controls(){
 use truck_stepio::out::*;
 let directory=std::path::Path::new("target/mixed-collapsed-controls");std::fs::create_dir_all(directory).unwrap();let mut records=Vec::new();
 for radius in [0.5,1.]{
  let solid=input();let before=serde_json::to_string(&solid.compress()).unwrap();let wire=rim(&solid);let result=fillet_solid_along_wire(&solid,&wire,radius,0.001).unwrap();
  assert_history(&solid,&result,8);if radius==1.{assert_rounds(&result,Matrix4::identity(),1.);}
  common::assert_solid(&result.solid,expected(radius),&[0],0.001);common::blend::assert_step(&result.solid,expected(radius),0.001);assert_eq!(before,serde_json::to_string(&solid.compress()).unwrap());
  let mode=format!("size12-radius{radius}");
  for (label,shape) in [("input",&solid),("result",&result.solid)]{
   std::fs::write(directory.join(format!("{mode}.{label}.json")),serde_json::to_string(&shape.compress()).unwrap()).unwrap();
   let prepared=prepare_for_step(&shape.compress(),5e-5).unwrap();let design=StepDesign::from_model(StepModel::from(&prepared));std::fs::write(directory.join(format!("{mode}.{label}.step")),StepDisplay::new(Default::default(),design).to_string()).unwrap();
  }
  records.push(serde_json::json!({"mode":mode,"radius":radius,"expected_volume":expected(radius),"input_unchanged":true,"geometry_consistent":result.solid.is_geometric_consistent(),"generated_faces":result.generated_faces.len(),"modified_faces":result.modified_faces.len()}));
 }
 std::fs::write(directory.join("summary.json"),serde_json::to_string_pretty(&records).unwrap()).unwrap();
}
