use truck_modeling::*;
fn source() -> serde_json::Value {
    let input: serde_json::Value = serde_json::from_str(include_str!(
        "../../truck-shapeops/tests/R10-first-fillet-solid.json"
    ))
    .unwrap();
    input["boundaries"][0]["faces"][6]["surface"].clone()
}
#[test]
fn translated_procedural_circle_is_an_analytic_cylinder() {
    for shifted in [false, true] {
        let mut raw = source();
        if shifted {
            for t in raw["Fillet"]["entity"]["knot_vec"].as_array_mut().unwrap() {
                *t = serde_json::json!(17. + 3.5 * t.as_f64().unwrap());
            }
        }
        let input: Surface = serde_json::from_value(raw).unwrap();
        for scale in [0.2, 1., 7.] {
            let matrix = Matrix4::from_translation(Vector3::new(7., -11., 17.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                * Matrix4::from_scale(scale);
            for inverse in [false, true] {
                let mut surface = input.transformed(matrix);
                if inverse {
                    surface.invert();
                }
                let before = serde_json::to_string(&surface).unwrap();
                let Some((
                    Elementary::Cylinder {
                        origin,
                        axis,
                        radius,
                    },
                    outward,
                )) = surface.elementary()
                else {
                    panic!("translated circular section must be recognized");
                };
                assert_eq!(outward, !inverse);
                assert!((radius - scale).abs() < 1e-8);
                let expected_axis = matrix.transform_vector(Vector3::unit_z()).normalize();
                assert!(axis.cross(expected_axis).magnitude() < 1e-8);
                let center = matrix.transform_point(Point3::new(1., 1., 0.));
                assert!((origin - center).cross(axis).magnitude() < 1e-8);
                let (u, v) = surface.try_range_tuple();
                let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
                for i in 0..=8 {
                    for j in 0..=8 {
                        let (u, v) = (a + (b - a) * i as f64 / 8., c + (d - c) * j as f64 / 8.);
                        let p = surface.subs(u, v);
                        let q = p - origin;
                        let radial = q - axis * q.dot(axis);
                        assert!((radial.magnitude() - scale).abs() < 1e-8);
                        let alignment = surface.normal(u, v).normalize().dot(radial.normalize());
                        assert!(alignment * if inverse { -1. } else { 1. } > 1. - 1e-8);
                    }
                }
                assert_eq!(before, serde_json::to_string(&surface).unwrap());
            }
        }
    }
}
#[test]
fn varying_or_non_circular_procedural_sections_are_not_classified() {
    for mode in 0..8 {
        let mut raw = source();
        let data = &mut raw["Fillet"]["entity"];
        match mode {
            0 => {
                let x = data["tangent_vecs0"][1]["x"].as_f64().unwrap();
                data["tangent_vecs0"][1]["x"] = serde_json::json!(x + 1e-4);
            }
            1 => {
                for p in data["tangent_vecs0"].as_array_mut().unwrap() {
                    p["x"] = serde_json::json!(p["x"].as_f64().unwrap() * 1.01);
                }
            }
            2 => {
                for w in data["weights"].as_array_mut().unwrap() {
                    *w = serde_json::json!(w.as_f64().unwrap() + 0.001);
                }
            }
            3 => {
                let x = data["side_control_points0"][1]["x"].as_f64().unwrap();
                data["side_control_points0"][1]["x"] = serde_json::json!(x + 0.001);
            }
            4 => {
                for side in ["side_control_points0", "side_control_points1"] {
                    data[side][1]["y"] = serde_json::json!(1.1);
                }
            }
            5 => {
                for knot in data["knot_vec"].as_array_mut().unwrap() {
                    *knot = serde_json::json!(0.);
                }
            }
            6 => {
                data["knot_vec"][2] = serde_json::json!(2.);
            }
            7 => {
                data["tangent_vecs0"].as_array_mut().unwrap().truncate(1);
            }
            _ => unreachable!(),
        }
        let decoded: std::result::Result<Surface, _> = serde_json::from_value(raw);
        if mode == 6 {
            assert!(
                decoded.is_err(),
                "unordered knots must fail deserialization"
            );
            continue;
        }
        let surface = decoded.unwrap();
        let before = serde_json::to_string(&surface).unwrap();
        assert!(surface.elementary().is_none(), "mode {mode}");
        assert_eq!(before, serde_json::to_string(&surface).unwrap());
    }
    let input: Surface = serde_json::from_value(source()).unwrap();
    assert!(input
        .transformed(Matrix4::from_nonuniform_scale(2., 1., 1.))
        .elementary()
        .is_none());
}
