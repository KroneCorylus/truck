use truck_meshalgo::prelude::*;
use truck_modeling::*;

fn check(robust: bool, split: bool) {
    let solid = Solid::extract(
        serde_json::from_str(if split {
            include_str!("cylindrical-terminal-fan-transformed.json")
        } else {
            include_str!("cylindrical-terminal-fan.json")
        })
        .unwrap(),
    )
    .unwrap();
    let (radius, halfwidth, level, height): (f64, f64, f64, f64) = (5., 2., 5., 10.);
    let primitive = |y: f64| {
        (y * (radius * radius - y * y).sqrt() + radius * radius * (y / radius).asin()) / 2.
    };
    let moment = |y: f64| -(radius * radius - y * y).powf(1.5) / 3.;
    let lower = halfwidth - 0.7;
    let expected = std::f64::consts::PI * radius * radius * height
        - 2. * level * primitive(halfwidth)
        + moment(halfwidth)
        - moment(lower)
        - lower * (primitive(halfwidth) - primitive(lower));
    for moved in [false, true] {
        let scale: f64 = if moved { 0.2 } else { 1. };
        let transform = if moved {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        } * Matrix4::from_scale(scale);
        let input = builder::transformed(&solid, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(input.is_geometric_consistent());
        let tol = 0.001;
        let meshed = if robust {
            input.robust_triangulation(tol)
        } else {
            input.triangulation(tol)
        };
        assert!(meshed.face_iter().all(|face| face.surface().is_some()));
        let mut mesh = meshed.to_polygon();
        mesh.put_together_same_attrs(TOLERANCE)
            .remove_degenerate_faces()
            .remove_unused_attrs();
        assert_eq!(mesh.shell_condition(), ShellCondition::Closed);
        let volume = mesh.volume();
        let scale = scale * if split { 0.2 } else { 1. };
        // All curved faces lie on the original cylinder, so its complete area bounds chord error.
        let allowed = std::f64::consts::TAU * radius * height * scale * scale * tol;
        assert!(
            (volume - expected * scale.powi(3)).abs() <= allowed,
            "robust={robust}, moved={moved}: volume {volume}, expected {}, allowed {allowed}",
            expected * scale.powi(3)
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn cylindrical_terminal_fan_preserves_mesh_material() { check(false, false); }

#[test]
fn cylindrical_terminal_fan_preserves_robust_mesh_material() { check(true, false); }

#[test]
fn subdivided_terminal_fan_preserves_mesh_material() { check(false, true); }
#[test]
fn subdivided_terminal_fan_preserves_robust_mesh_material() { check(true, true); }
