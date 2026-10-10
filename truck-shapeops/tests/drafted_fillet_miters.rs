mod common;

use std::{collections::HashMap, f64::consts::PI};
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_fillet_solid_edges};

const TOL: f64 = 0.001;
const WIDTH: f64 = 12.;
const HEIGHT: f64 = 8.;

fn frustum(draft: f64) -> Solid {
    let [x, y] = [draft * HEIGHT; 2];
    let points = [
        Point3::new(-x, -y, 0.),
        Point3::new(WIDTH + x, -y, 0.),
        Point3::new(WIDTH + x, WIDTH + y, 0.),
        Point3::new(-x, WIDTH + y, 0.),
        Point3::new(0., 0., HEIGHT),
        Point3::new(WIDTH, 0., HEIGHT),
        Point3::new(WIDTH, WIDTH, HEIGHT),
        Point3::new(0., WIDTH, HEIGHT),
    ];
    let vertices = points.map(builder::vertex);
    let mut edges = HashMap::new();
    let faces = [
        [3, 2, 1, 0],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
        [4, 5, 6, 7],
    ]
    .map(|indices| {
        let wire: Wire = (0..4)
            .map(|i| {
                let (a, b) = (indices[i], indices[(i + 1) % 4]);
                let edge: &Edge = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| builder::line(&vertices[a.min(b)], &vertices[a.max(b)]));
                if a < b {
                    edge.clone()
                } else {
                    edge.inverse()
                }
            })
            .collect();
        builder::try_attach_plane(&[wire]).unwrap()
    });
    Solid::new(vec![faces.into_iter().collect()])
}

fn volume(draft: f64) -> f64 {
    WIDTH * WIDTH * HEIGHT
        + 2. * draft * WIDTH * HEIGHT.powi(2)
        + 4. / 3. * draft * draft * HEIGHT.powi(3)
}

fn as_spline(solid: &Solid, rational: bool) -> Solid {
    solid.mapped(
        |p| *p,
        |curve| {
            let Curve::Line(line) = curve else {
                panic!("straight fixture")
            };
            let spline: BSplineCurve<Point3> = (*line).into();
            if rational {
                Curve::NurbsCurve(spline.into())
            } else {
                Curve::BSplineCurve(spline)
            }
        },
        |surface| {
            let Surface::Plane(plane) = surface else {
                panic!("planar fixture")
            };
            let parameters: Vec<_> = solid
                .vertex_iter()
                .map(|v| plane.get_parameter(v.point()))
                .collect();
            let low = parameters
                .iter()
                .fold(Vector2::new(f64::INFINITY, f64::INFINITY), |p, q| {
                    Vector2::new(p.x.min(q.x), p.y.min(q.y))
                });
            let high = parameters.iter().fold(
                Vector2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
                |p, q| Vector2::new(p.x.max(q.x), p.y.max(q.y)),
            );
            let bounded = Plane::new(
                plane.subs(low.x, low.y),
                plane.subs(high.x, low.y),
                plane.subs(low.x, high.y),
            );
            let spline: BSplineSurface<Point3> = bounded.into();
            if rational {
                Surface::NurbsSurface(spline.into())
            } else {
                Surface::BSplineSurface(spline)
            }
        },
    )
}

fn assert_radius_and_contacts(result: &truck_shapeops::fillet::BlendResult, radius: f64) {
    for face in &result.generated_faces {
        let surface = face.oriented_surface();
        let Some((
            geometry::Elementary::Cylinder {
                radius: actual,
                axis,
                ..
            },
            _,
        )) = surface.elementary()
        else {
            panic!("expected a cylindrical round")
        };
        assert!((actual - radius).abs() < 1e-8);
        let mut contacts = 0;
        for edge in face.edge_iter() {
            let curve = edge.oriented_curve();
            let (a, b) = curve.range_tuple();
            if curve.der((a + b) / 2.).normalize().cross(axis).magnitude() > 1e-8 {
                continue;
            }
            let adjacent = result
                .solid
                .face_iter()
                .find(|other| {
                    other.id() != face.id() && other.edge_iter().any(|e| e.id() == edge.id())
                })
                .unwrap();
            let other = adjacent.oriented_surface();
            assert!(matches!(
                other.elementary(),
                Some((geometry::Elementary::Plane(_), _))
            ));
            for t in [0.2, 0.5, 0.8] {
                let p = curve.subs(a + t * (b - a));
                let (u, v) = surface.search_parameter(p, None, 100).unwrap();
                let (s, t) = other.search_parameter(p, None, 100).unwrap();
                assert!(
                    surface
                        .normal(u, v)
                        .normalize()
                        .dot(other.normal(s, t).normalize())
                        > 1. - 1e-8
                );
            }
            contacts += 1;
        }
        assert_eq!(contacts, 2);
    }
}

fn removed_volume(draft: f64, radius: f64, count: usize) -> f64 {
    // At depth y below the top, each rounded side removes width
    // d(y) = r*(sqrt(1+s²)-s) + s*y - sqrt(2*r*y-y²), until contact with the wall.
    // Integrate the square-section area loss: 2*W(y)*d-d² for two adjacent sides,
    // or 4*W(y)*d-4*d² for the complete rim, with W(y)=WIDTH+2*s*y.
    let q = (1. + draft * draft).sqrt();
    let a = radius * (q - draft);
    let depth = radius * (1. - draft / q);
    let u = depth - radius;
    let root = radius / q;
    let circle_area = (u * root + radius * radius * ((u / radius).asin() + PI / 2.)) / 2.;
    let circle_moment = radius * circle_area - root.powi(3) / 3.;
    let inset = a * depth + draft * depth * depth / 2. - circle_area;
    let inset_moment = a * depth * depth / 2. + draft * depth.powi(3) / 3. - circle_moment;
    let inset_square = a * a * depth
        + a * draft * depth * depth
        + (draft * draft - 1.) * depth.powi(3) / 3.
        + radius * depth * depth
        - 2. * a * circle_area
        - 2. * draft * circle_moment;
    if count == 2 {
        2. * WIDTH * inset + 4. * draft * inset_moment - inset_square
    } else {
        4. * WIDTH * inset + 8. * draft * inset_moment - 4. * inset_square
    }
}

fn check_drafted_rims(representation: Option<bool>, cavity: bool) {
    for draft in [-0.4, 0.3] {
        for (scale, transform) in [
            (1_f64, Matrix4::identity()),
            (
                2.,
                Matrix4::from_translation(Vector3::new(13., -7., 23.))
                    * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
                    * Matrix4::from_scale(2.),
            ),
        ] {
            let mut input = frustum(draft);
            let original_volume = if cavity {
                input.not();
                let outer = common::modeling::cuboid(
                    Point3::new(-8., -8., -4.),
                    Point3::new(20., 20., 12.),
                );
                input = Solid::new(vec![
                    outer.boundaries()[0].clone(),
                    input.boundaries()[0].clone(),
                ]);
                28. * 28. * 16. - volume(draft)
            } else {
                volume(draft)
            };
            if let Some(rational) = representation {
                input = as_spline(&input, rational);
            }
            let input = builder::transformed(&input, transform);
            assert!(
                input.is_geometric_consistent(),
                "input draft={draft}, scale={scale}, cavity={cavity}"
            );
            let genera: &[usize] = if cavity { &[0, 0] } else { &[0] };
            common::assert_solid(&input, original_volume * scale.powi(3), genera, TOL);
            let before = serde_json::to_string(&input.compress()).unwrap();
            let ids = [
                Point3::new(WIDTH / 2., 0., HEIGHT),
                Point3::new(0., WIDTH / 2., HEIGHT),
                Point3::new(WIDTH, WIDTH / 2., HEIGHT),
                Point3::new(WIDTH / 2., WIDTH, HEIGHT),
            ]
            .map(|p| {
                common::blend::edge_through(
                    &input.boundaries()[usize::from(cavity)],
                    transform.transform_point(p),
                )
                .id()
            });
            for radius in [0.2, 0.8] {
                for count in [2, 4] {
                    let result = try_fillet_solid_edges(&input, &ids[..count], radius * scale, TOL)
                        .unwrap_or_else(|e| panic!("draft={draft}, radius={radius}, count={count}, cavity={cavity}: {e:?}"));
                    assert!(result.solid.is_geometric_consistent(), "result draft={draft}, radius={radius}, count={count}, scale={scale}, cavity={cavity}");
                    assert_eq!(result.generated_faces.len(), count);
                    assert_radius_and_contacts(&result, radius * scale);
                    let loss = removed_volume(draft, radius, count);
                    let expected =
                        (original_volume + if cavity { loss } else { -loss }) * scale.powi(3);
                    common::assert_solid(&result.solid, expected, genera, TOL);
                    if radius == 0.8 {
                        common::blend::assert_step(&result.solid, expected, TOL);
                    }
                    let mut reversed = ids[..count].to_vec();
                    reversed.reverse();
                    let again =
                        try_fillet_solid_edges(&input, &reversed, radius * scale, TOL).unwrap();
                    assert_eq!(
                        serde_json::to_string(&result.solid.compress()).unwrap(),
                        serde_json::to_string(&again.solid.compress()).unwrap()
                    );
                    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
                }
            }
            for radius in [0., f64::NAN, 1e9] {
                assert!(try_fillet_solid_edges(&input, &ids, radius, TOL).is_err());
                assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            }
        }
    }
}

#[test]
fn analytic_drafted_rims_control() { check_drafted_rims(None, false); }

#[test]
fn nurbs_drafted_rims_match_analytic_geometry() {
    check_drafted_rims(Some(true), false);
    check_drafted_rims(Some(true), true);
}

#[test]
fn bspline_drafted_rims_match_analytic_geometry() {
    check_drafted_rims(Some(false), false);
    check_drafted_rims(Some(false), true);
}

#[test]
fn spline_cavity_rim_chamfers_preserve_the_same_setbacks() {
    for draft in [-0.4, 0.3] {
        for rational in [false, true] {
            let mut inner = frustum(draft);
            inner.not();
            let outer =
                common::modeling::cuboid(Point3::new(-8., -8., -4.), Point3::new(20., 20., 12.));
            let input = as_spline(
                &Solid::new(vec![
                    outer.boundaries()[0].clone(),
                    inner.boundaries()[0].clone(),
                ]),
                rational,
            );
            assert!(input.is_geometric_consistent());
            let before = serde_json::to_string(&input.compress()).unwrap();
            let ids = [
                Point3::new(WIDTH / 2., 0., HEIGHT),
                Point3::new(0., WIDTH / 2., HEIGHT),
                Point3::new(WIDTH, WIDTH / 2., HEIGHT),
                Point3::new(WIDTH / 2., WIDTH, HEIGHT),
            ]
            .map(|p| common::blend::edge_through(&input.boundaries()[1], p).id());
            let distance = 0.8_f64;
            let q = (1. + draft * draft).sqrt();
            // Integrate the linear section inset d-q*y over 0 <= y <= d/q.
            let inset = distance.powi(2) / (2. * q);
            let inset_moment = distance.powi(3) / (6. * q * q);
            let inset_square = distance.powi(3) / (3. * q);
            let loss = 4. * WIDTH * inset + 8. * draft * inset_moment - 4. * inset_square;
            let expected = 28. * 28. * 16. - volume(draft) + loss;
            let result = try_chamfer_solid_edges(&input, &ids, distance, TOL).unwrap();
            assert!(result.solid.is_geometric_consistent());
            common::assert_solid(&result.solid, expected, &[0, 0], TOL);
            common::blend::assert_step(&result.solid, expected, TOL);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
}
