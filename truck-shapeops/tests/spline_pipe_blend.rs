mod common;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/spline-pipe/input.step")).unwrap();
    let (mut compressed, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    compressed.split_closed_edges_and_faces(TOL / 100.);
    Solid::extract(
        compressed
            .try_mapped(
                |p| Some(*p),
                |c| Curve::try_from(c).ok(),
                |s| Surface::try_from(s).ok(),
            )
            .unwrap(),
    )
    .unwrap()
}

fn check_blend(radius: f64, scale: f64, reverse: bool, volume: f64) {
    let original = input();
    let body = original
        .face_iter()
        .find(|f| matches!(f.surface(), Surface::NurbsSurface(_)))
        .unwrap()
        .oriented_surface();

    let matrix = (if reverse {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    }) * Matrix4::from_scale(scale);
    let inverse = matrix.invert().unwrap();
    let input = builder::transformed(&original, matrix);
    let cap = common::blend::face_through(
        &input.boundaries()[0],
        matrix.transform_point(Point3::origin()),
    );
    let wire = input.boundaries()[0][cap].boundaries()[0].clone();
    let wire = if reverse { wire.inverse() } else { wire };
    if reverse {
        let Curve::NurbsCurve(mut curve) = wire[0].curve() else {
            panic!("expected a spline rim");
        };
        curve.parameter_transform(2.5, -7.0);
        wire[0].set_curve(Curve::NurbsCurve(curve));
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_fillet_solid_along_wire(&input, &wire, radius * scale, TOL * scale)
        .expect("closed rim contacts must reach the displaced side-face seams");
    assert!(result.solid.is_geometric_consistent());
    assert!(!result.generated_faces.is_empty());
    for face in &result.generated_faces {
        let Surface::Fillet(surface) = face.surface() else {
            panic!("expected a rolling-ball patch");
        };
        let ((a, b), (c, d)) = surface.entity().range_tuple();
        let parameters = |u, v| {
            if surface.orientation() {
                (u, v)
            } else {
                (v, u)
            }
        };
        let point = |u, v| {
            let (u, v) = parameters(u, v);
            inverse.transform_point(surface.subs(u, v))
        };
        let normal = |u, v| {
            let (u, v) = parameters(u, v);
            let sign = if face.orientation() { 1. } else { -1. };
            inverse
                .transform_vector(sign * surface.normal(u, v))
                .normalize()
        };
        for j in 0..=16 {
            let v = c + (d - c) * j as f64 / 16.;
            let endpoints = [point(a, v), point(b, v)];
            let cap_side = usize::from(endpoints[1].x.abs() < endpoints[0].x.abs());
            let cap_point = endpoints[cap_side];
            let body_point = endpoints[1 - cap_side];
            assert!(cap_point.x.abs() < TOL);
            let center = Point3::new(radius, cap_point.y, cap_point.z);
            let uv = body.search_parameter(body_point, None, 100).unwrap();
            let support_normal = body.normal(uv.0, uv.1);
            let body_u = if cap_side == 0 { b } else { a };
            assert!(normal(body_u, v).dot(support_normal) > 1. - 1e-5);
            for i in 0..=8 {
                let u = a + (b - a) * i as f64 / 8.;
                let radial = point(u, v) - center;
                assert!((radial.magnitude() - radius).abs() < TOL);
                assert!(normal(u, v).dot(radial.normalize()) > 1. - 1e-5);
            }
        }
    }
    common::assert_solid(
        &input,
        1_815_077.757_129_17 * scale.powi(3),
        &[0],
        TOL * scale,
    );
    common::assert_solid(&result.solid, volume * scale.powi(3), &[0], TOL * scale);
    common::blend::assert_step(&result.solid, volume * scale.powi(3), TOL * scale);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

#[test]
fn closed_spline_pipe_rims_follow_the_support_seams_at_their_contacts() {
    check_blend(10., 1., false, 1_808_730.962_398_931_8);
}

#[test]
fn transformed_reversed_spline_pipe_rims_follow_the_support_seams() {
    check_blend(5., 0.01, true, 1_813_443.836_001_673_5);
}
