mod common;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges_with_distances, try_fillet_solid_edges};

const TOL: f64 = 0.001;

fn cross(a: Vector2, b: Vector2) -> f64 { a.x * b.y - a.y * b.x }

fn circle_integral(a: Vector2, b: Vector2, center: Vector2, radius: f64) -> f64 {
    let u = a - center;
    let v = b - center;
    cross(center, b - a) + radius * radius * cross(u, v).atan2(u.dot(v))
}

fn expected_volume(convex: bool, height: f64, fillet: bool, size: f64) -> f64 {
    let h = height - 10.;
    let x = (36. - h * h).sqrt();
    let start = Vector2::new(x, h);
    let sign = if convex { -1. } else { 1. };
    let (plane_contact, cylinder_contact, round) = if fillet {
        let cy = h - size;
        let cx = ((6. + sign * size).powi(2) - cy * cy).sqrt();
        let center = Vector2::new(cx, cy);
        (
            Vector2::new(cx, h),
            center * 6. / (6. + sign * size),
            Some(center),
        )
    } else {
        (
            Vector2::new(x + sign * size, h),
            (start + size * Vector2::new(h, -x) / 6.).normalize() * 6.,
            None,
        )
    };
    // Green's theorem around the removed section: top line, blend, original cylinder arc.
    let area = (cross(start, plane_contact)
        + round.map_or_else(
            || cross(plane_contact, cylinder_contact),
            |center| circle_integral(plane_contact, cylinder_contact, center, size),
        )
        + circle_integral(cylinder_contact, start, Vector2::zero(), 6.))
    .abs()
        / 2.;
    let bore = (h * x + 36. * (h / 6.).asin() + 18. * std::f64::consts::PI) * 10.;
    (if convex { bore } else { 400. * height - bore }) - 10. * area
}

fn input(convex: bool, height: f64) -> Solid {
    let block =
        common::modeling::cuboid(Point3::new(-10., -10., 0.), Point3::new(10., 10., height));
    let mut tool = common::modeling::cylinder(Point3::new(0., 0., 10.), Vector3::unit_y(), 6., 11.);
    if !convex {
        tool.not();
    }
    truck_shapeops::and(&block, &tool, TOL).unwrap()
}

fn check_lip(convex: bool, height: f64) {
    let source = input(convex, height);
    for scale in [0.01_f64, 1., 100.] {
        let transform = Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let input = builder::transformed(&source, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let x = (36. - (height - 10.).powi(2)).sqrt();
        let id = common::blend::edge_through(
            &input.boundaries()[0],
            transform.transform_point(Point3::new(x, 5., height)),
        )
        .id();
        for fillet in [false, true] {
            let size = 0.2 * scale;
            let tol = TOL * scale;
            let result = if fillet {
                try_fillet_solid_edges(&input, &[id], size, tol)
            } else {
                try_chamfer_solid_edges_with_distances(&input, &[(id, [size; 2])], tol)
            }
            .unwrap_or_else(|error| {
                panic!(
                    "convex={convex}, height={height}, scale={scale}, fillet={fillet}: {error:?}"
                )
            });
            assert!(result.solid.is_geometric_consistent());
            let volume = expected_volume(convex, height, fillet, 0.2) * scale.powi(3);
            common::assert_solid(&result.solid, volume, &[0], tol);
            let normalized = builder::transformed(&result.solid, transform.invert().unwrap());
            common::assert_solid(
                &normalized,
                expected_volume(convex, height, fillet, 0.2),
                &[0],
                TOL / 20.0,
            );
            if scale == 1. {
                common::blend::assert_step(&result.solid, volume, tol);
            }
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}

macro_rules! lip_case {
    ($name:ident, $convex:expr, $height:expr) => {
        #[test]
        fn $name() { check_lip($convex, $height); }
    };
}

lip_case!(concave_lip_below_axis_at_multiple_scales, false, 7.0);
lip_case!(concave_lip_through_axis_at_multiple_scales, false, 10.0);
lip_case!(concave_lip_above_axis_at_multiple_scales, false, 13.0);
lip_case!(convex_lip_below_axis_at_multiple_scales, true, 7.0);
lip_case!(convex_lip_through_axis_at_multiple_scales, true, 10.0);
lip_case!(convex_lip_above_axis_at_multiple_scales, true, 13.0);

#[test]
fn shallow_convex_cylinder_cap_rejects_nonfitting_fillet() {
    let input = input(true, 5.);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let id =
        common::blend::edge_through(&input.boundaries()[0], Point3::new(11_f64.sqrt(), 5., 5.))
            .id();
    for radius in [0.5, 1.] {
        assert!(try_fillet_solid_edges(&input, &[id], radius, TOL).is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}
