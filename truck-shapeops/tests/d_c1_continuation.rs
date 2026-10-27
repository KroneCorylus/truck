mod common;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_along_wire, try_chamfer_solid_along_wire_chord};

fn parabola(a: f64, b: f64, height: impl Fn(f64) -> f64) -> BSplineCurve<Point3> {
    let point = |t: f64| Point3::new(t, t * t / 10., height(t));
    let p = point(a);
    let q = point(b);
    BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            p,
            Point3::from_vec(point((a + b) / 2.).to_vec() * 2. - (p.to_vec() + q.to_vec()) / 2.),
            q,
        ],
    )
}

// Exact source: 0 <= z <= min(4, 3.5+x+y), y >= x²/10,
// x+y >= 0, x <= 4, y <= 4. Build the prior bevel without calling a blend operation.
fn source(width: f64) -> (Solid, Wire) {
    let a = 30_f64.sqrt() - 5.;
    let v = builder::vertices([
        Point3::new(0., 0., 0.),
        Point3::new(4., 1.6, 0.),
        Point3::new(4., width, 0.),
        Point3::new(-width, width, 0.),
        Point3::new(0., 0., 3.5),
        Point3::new(a, a * a / 10., 4.),
        Point3::new(4., 1.6, 4.),
        Point3::new(4., width, 4.),
        Point3::new(0.5 - width, width, 4.),
        Point3::new(-width, width, 3.5),
    ]);
    let bottom = Edge::new(&v[0], &v[1], parabola(0., 4., |_| 0.).into());
    let selected = Edge::new(&v[5], &v[6], parabola(a, 4., |_| 4.).into());
    let prior = Edge::new(
        &v[4],
        &v[5],
        parabola(0., a, |t| 3.5 + t + t * t / 10.).into(),
    );
    let pairs = [
        (1, 2),
        (2, 3),
        (3, 0),
        (0, 4),
        (1, 6),
        (2, 7),
        (3, 9),
        (6, 7),
        (7, 8),
        (8, 5),
        (8, 9),
        (9, 4),
    ];
    let e: Vec<_> = pairs
        .into_iter()
        .map(|(a, b)| builder::line(&v[a], &v[b]))
        .collect();
    let loops = [
        wire![
            bottom.inverse(),
            e[2].inverse(),
            e[1].inverse(),
            e[0].inverse()
        ],
        wire![e[0].clone(), e[5].clone(), e[7].inverse(), e[4].inverse()],
        wire![
            e[1].clone(),
            e[6].clone(),
            e[10].inverse(),
            e[8].inverse(),
            e[5].inverse()
        ],
        wire![e[2].clone(), e[3].clone(), e[11].inverse(), e[6].inverse()],
        wire![selected.clone(), e[7].clone(), e[8].clone(), e[9].clone()],
        wire![prior.clone(), e[9].inverse(), e[10].clone(), e[11].clone()],
    ];
    let mut faces: Shell = loops
        .into_iter()
        .map(|w| builder::try_attach_plane(&[w]).unwrap())
        .collect();
    let wall = builder::tsweep(&bottom, Vector3::unit_z() * 4.);
    faces.push(Face::new(
        vec![wire![
            bottom,
            e[4].clone(),
            selected.inverse(),
            prior.inverse(),
            e[3].inverse()
        ]],
        wall.oriented_surface(),
    ));
    (Solid::new(vec![faces]), wire![selected])
}

fn integrate(f: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let n = 512;
    let h = (b - a) / n as f64;
    (f(a)
        + f(b)
        + (1..n)
            .map(|i| f(a + h * i as f64) * if i % 2 == 0 { 2. } else { 4. })
            .sum::<f64>())
        * h
        / 3.
}

#[test]
fn parabolic_prior_bevel_requires_original_support_continuation() {
    let (input, wire) = source(4.);
    let volume =
        4. * 328. / 15. - integrate(|s| (0.5 - s) * ((25. + 10. * s).sqrt() - 1. - s), 0., 0.5);
    assert!(input.is_geometric_consistent());
    common::assert_solid(&input, volume, &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_along_wire_chord(&input, &wire, 1., 1., 0.001);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    let result =
        result.expect("continued original support must meet the prior bevel on the end seam");
    assert!(result.solid.is_geometric_consistent());
    let expected = volume - removed_volume(1., 1.);
    common::assert_solid(&result.solid, expected, &[0], 0.001);
    common::assert_volume(&result.solid, expected, 1e-5);
    common::blend::assert_step(&result.solid, expected, 0.001);
    for (old, new) in input.face_iter().zip(result.solid.face_iter()) {
        assert_eq!(
            serde_json::to_string(&old.oriented_surface()).unwrap(),
            serde_json::to_string(&new.oriented_surface()).unwrap()
        );
    }
    let mesh = result.solid.triangulation(0.001).to_polygon();
    for p in mesh.positions() {
        assert!(p.z >= -1e-6 && p.z <= 4. + 1e-6 && p.z <= 3.5 + p.x + p.y + 1e-6);
        assert!(
            p.y >= p.x * p.x / 10. - 1e-6
                && p.x + p.y >= -1e-6
                && p.x <= 4. + 1e-6
                && p.y <= 4. + 1e-6
        );
    }
    let bevel = result.solid.face_iter().last().unwrap();
    let mut continued_boundary = false;
    for edge in bevel.edge_iter() {
        let curve = edge.oriented_curve();
        let (a, b) = curve.range_tuple();
        for i in 0..=64 {
            let p = curve.subs(a + (b - a) * i as f64 / 64.);
            // Invert the original parabola's normal chart independently of the fitter.
            let t = root(|t| t - p.x + (t * t / 10. - p.y) * t / 5.);
            let q = (1. + t * t / 25.).sqrt();
            let lambda = (p.y - t * t / 10.) * q;
            assert!((p.z - 3. - lambda).abs() < 1e-6);
            assert!((-1e-6..=1. + 1e-6).contains(&lambda));
            assert!(p.x + p.y >= -1e-6 && p.z <= 3.5 + p.x + p.y + 1e-6);
            assert!(p.y >= p.x * p.x / 10. - 1e-6);
            if t < -0.1 {
                continued_boundary = true;
                assert!(lambda > 0.);
            }
        }
    }
    assert!(continued_boundary);
    let mut a = -1.;
    let mut b = 0.;
    for _ in 0..60 {
        let t = (a + b) / 2.;
        let q = (1_f64 + t * t / 25.).sqrt();
        if t + t * t / 10. + 0.5 * (1. - t / 5.) / q < 0. {
            a = t;
        } else {
            b = t;
        }
    }
    let t = (a + b) / 2.;
    let q = (1. + t * t / 25.).sqrt();
    let joint = Point3::new(t - t / (10. * q), t * t / 10. + 0.5 / q, 3.5);
    assert!(result
        .solid
        .vertex_iter()
        .any(|v| v.point().distance(joint) < 1e-6));
}

fn root(f: impl Fn(f64) -> f64) -> f64 {
    let (mut a, mut b) = (-2., 6.);
    assert!(f(a) < 0. && f(b) > 0.);
    for _ in 0..60 {
        let t = (a + b) / 2.;
        if f(t) < 0. {
            a = t;
        } else {
            b = t;
        }
    }
    (a + b) / 2.
}

// Integrate the removed depth in the independent exact normal chart, split at
// the old-bevel/top transition and at the seam/bevel change of boundary owner.
fn removed_volume(dc: f64, dw: f64) -> f64 {
    let section = |l: f64| {
        let xy = |t: f64| {
            let q = (1. + t * t / 25.).sqrt();
            (t - dc * l * t / (5. * q), t * t / 10. + dc * l / q)
        };
        let sum = |t: f64| {
            let (x, y) = xy(t);
            x + y
        };
        let start = root(|t| sum(t) - 0_f64.max(0.5 - dw * (1. - l)));
        let end = root(|t| xy(t).0 - 4.);
        let split = root(|t| sum(t) - 0.5).clamp(start, end);
        let jacobian = |t: f64| {
            let q = (1. + t * t / 25.).sqrt();
            dc * (q - dc * l / (5. * q * q))
        };
        integrate(
            |t| (dw * (1. - l) - 0.5 + sum(t)) * jacobian(t),
            start,
            split,
        ) + integrate(|t| dw * (1. - l) * jacobian(t), split, end)
    };
    let split = (1. - 0.5 / dw).clamp(0., 1.);
    integrate(section, 0., split) + integrate(section, split, 1.)
}

#[test]
fn continuation_rejects_remote_wall_and_curvature_overlap() {
    for (width, distance) in [(1.8, 1.), (4., 3.)] {
        let (input, wire) = source(width);
        assert!(input.is_geometric_consistent());
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(
            try_chamfer_solid_along_wire_chord(&input, &wire, distance, distance, 0.001).is_err()
        );
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn continued_chamfer_preserves_reversed_unequal_setbacks() {
    let (input, wire) = source(4.);
    let volume =
        4. * 328. / 15. - integrate(|s| (0.5 - s) * ((25. + 10. * s).sqrt() - 1. - s), 0., 0.5);
    for reversed in [false, true] {
        let wire = if reversed {
            wire.inverse()
        } else {
            wire.clone()
        };
        let (dc, dw) = (0.8, 1.2);
        let distances = if reversed { (dw, dc) } else { (dc, dw) };
        let result =
            try_chamfer_solid_along_wire_chord(&input, &wire, distances.0, distances.1, 0.001)
                .unwrap();
        common::assert_solid(&result.solid, volume - removed_volume(dc, dw), &[0], 0.001);
        common::assert_volume(&result.solid, volume - removed_volume(dc, dw), 1e-5);
    }
}

#[test]
fn continued_chamfer_is_independent_of_world_frame() {
    let (input, wire) = source(4.);
    let matrix = Matrix4::from_translation(Vector3::new(7., -3., 11.))
        * Matrix4::from_angle_x(Rad(0.4))
        * Matrix4::from_scale(2.);
    let witness = matrix.transform_point(wire[0].oriented_curve().subs(0.5));
    let input = builder::transformed(&input, matrix);
    let edge = common::blend::edge_through(&input.boundaries()[0], witness);
    let result = try_chamfer_solid_along_wire(&input, &wire![edge], 2., 2., 0.001).unwrap();
    let volume =
        4. * 328. / 15. - integrate(|s| (0.5 - s) * ((25. + 10. * s).sqrt() - 1. - s), 0., 0.5);
    common::assert_solid(
        &result.solid,
        8. * (volume - removed_volume(1., 1.)),
        &[0],
        0.001,
    );
}
