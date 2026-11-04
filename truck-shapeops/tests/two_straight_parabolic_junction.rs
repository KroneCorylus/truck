mod common;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges_with_radii;
fn body() -> (Solid, Vec<EdgeID>) { body_variant(0., -50.) }

fn body_variant(middle_y: f64, middle_z: f64) -> (Solid, Vec<EdgeID>) {
    body_with_hole(middle_y, middle_z, false)
}

fn body_with_hole(middle_y: f64, middle_z: f64, hole: bool) -> (Solid, Vec<EdgeID>) {
    let controls = vec![
        Point3::new(0., -100., 50.),
        Point3::new(0., middle_y, middle_z),
        Point3::new(0., 100., 50.),
    ];
    let a = Vertex::new(controls[0]);
    let b = Vertex::new(controls[2]);
    let c = BSplineCurve::new(KnotVec::bezier_knot(2), controls);
    let e = Edge::new(&a, &b, Curve::BSplineCurve(c));
    let mut boundaries = vec![wire![e, builder::line(&b, &a)]];
    if hole {
        let vertices = [
            Point3::new(0., -2., 20.),
            Point3::new(0., -2., 24.),
            Point3::new(0., 2., 24.),
            Point3::new(0., 2., 20.),
        ]
        .map(Vertex::new);
        boundaries.push(
            (0..4)
                .map(|i| builder::line(&vertices[i], &vertices[(i + 1) % 4]))
                .collect(),
        );
    }
    let face = builder::try_attach_plane(boundaries.as_slice()).unwrap();
    let input: Solid = builder::tsweep(&face, Vector3::new(20., 0., 100.));
    for face in input.face_iter() {
        if let Surface::BSplineSurface(s) = face.surface() {
            let p = s.control_points();
            let v = p[0][1] - p[0][0];
            let c = BSplineCurve::new(s.uknot_vec().clone(), p.iter().map(|r| r[0]).collect());
            face.set_surface(ExtrudedCurve::by_extrusion(c.into(), v).into());
        }
    }
    let mids = [Point3::new(10., -100., 100.), Point3::new(20., 0., 150.)];
    let mut ids = vec![];
    for p in mids {
        let mut seen = std::collections::HashSet::new();
        let es: Vec<_> = input.boundaries()[0]
            .edge_iter()
            .filter(|e| seen.insert(e.id()))
            .filter(|e| {
                let c = e.curve();
                let (s, t) = c.range_tuple();
                c.subs((s + t) / 2.).distance(p) < 1e-7
            })
            .collect();
        assert_eq!(es.len(), 1);
        ids.push(es[0].id());
    }
    (input, ids)
}

#[test]
fn bounded_f9_positive() {
    let (input, ids) = body();
    common::assert_solid(&input, 400000. / 3., &[0], 0.001);
    let before = serde_json::to_string(&input.compress()).unwrap();
    for r in [0.5, 1., 1.5] {
        for rev in [false, true] {
            let assignments: Vec<_> = if rev {
                ids.iter().rev().map(|id| (*id, r)).collect()
            } else {
                ids.iter().map(|id| (*id, r)).collect()
            };
            let result = try_fillet_solid_edges_with_radii(&input, &assignments, 1e-5);
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            let output = result.expect("bounded F9 two-cylinder crease construction");
            assert!(output.solid.is_geometric_consistent());
            let expected = if r == 0.5 {
                132681.9329767928
            } else if r == 1. {
                130745.06675857291
            } else {
                127542.36410896911
            };
            check_structure(&input, &output.solid);
            common::assert_solid(&output.solid, expected, &[0], 0.001);
            common::blend::assert_step(&output.solid, expected, 0.001);
        }
    }
}

fn check_structure(input: &Solid, output: &Solid) {
    assert_eq!(output.face_iter().count(), 6);
    for (a, b) in input.face_iter().zip(output.face_iter()) {
        assert_eq!(
            serde_json::to_string(&a.oriented_surface()).unwrap(),
            serde_json::to_string(&b.oriented_surface()).unwrap()
        );
    }
    let mut uses = std::collections::HashMap::<_, Vec<_>>::new();
    for face in output.face_iter() {
        for edge in face.edge_iter() {
            uses.entry(edge.id()).or_default().push(edge.orientation());
        }
    }
    assert!(uses.values().all(|v| v.len() == 2 && v[0] != v[1]));
}

#[test]
fn bounded_f9_immutable_radius_guards() {
    let (input, ids) = body();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for radii in [[1., 0.5], [1.51, 1.51], [5., 5.], [1e-12, 1e-12]] {
        assert!(try_fillet_solid_edges_with_radii(
            &input,
            &[(ids[0], radii[0]), (ids[1], radii[1])],
            1e-5
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

fn contacts(r: f64) -> (f64, f64, f64, f64, f64) {
    let k = 26_f64.sqrt();
    let (mut a, mut b) = (-100., 0.);
    for _ in 0..64 {
        let t = (a + b) / 2.;
        let p = -t / (100. * k);
        if (50. - t * t / 200.) / k - r / (1. + p * p).sqrt() - r > 0. {
            b = t;
        } else {
            a = t;
        }
    }
    let t = (a + b) / 2.;
    let p = -t / (100. * k);
    (
        t,
        (50. - t * t / 200.) / k,
        t + r * p / (1. + p * p).sqrt(),
        770. / k - (5. + k) * r,
        r * (1. + 5. / k),
    )
}

#[test]
fn bounded_f9_short_cap_and_material() {
    let (input, ids) = body();
    let output = try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 1.)).collect::<Vec<_>>(),
        1e-5,
    )
    .unwrap()
    .solid;
    let (t, dt, cy, cx, dc) = contacts(1.);
    let k = 26_f64.sqrt();
    let point = |u: f64, y: f64, d: f64| {
        Point3::new(
            (u + 5. * d) / k - 250. / 26.,
            y,
            (5. * u - d) / k + 50. / 26.,
        )
    };
    let start = point(cx + 1. / k, cy - 1. / k, dc);
    let end = point(770. / k - 5. * dt, t, dt);
    let edge = output.boundaries()[0]
        .edge_iter()
        .find(|e| {
            (e.front().point().distance(start) < 1e-7 && e.back().point().distance(end) < 1e-7)
                || (e.back().point().distance(start) < 1e-7
                    && e.front().point().distance(end) < 1e-7)
        })
        .expect("separate short cap ellipse");
    let curve = edge.curve();
    let (a, b) = curve.range_tuple();
    for i in 0..=32 {
        let p = curve.subs(a + (b - a) * i as f64 / 32.);
        let d = (50. - p.z + 5. * p.x) / k;
        let u = (p.x + 5. * p.z) / k;
        assert!(d >= dc - 1e-8 && d <= dt + 1e-8);
        assert!((u + 5. * d - 770. / k).abs() < 1e-8);
        assert!(((p.y - cy).powi(2) + (d - 1.).powi(2) - 1.).abs() < 1e-8);
    }
    for (start, end, seam) in [
        (point(cx, cy, 0.), point(cx + 1. / k, cy - 1. / k, dc), true),
        (
            point(250. / k, cy, 0.),
            point(250. / k - 5. * dt, t, dt),
            false,
        ),
    ] {
        let edge = output.boundaries()[0]
            .edge_iter()
            .find(|e| {
                (e.front().point().distance(start) < 1e-7 && e.back().point().distance(end) < 1e-7)
                    || (e.back().point().distance(start) < 1e-7
                        && e.front().point().distance(end) < 1e-7)
            })
            .expect("shared seam or remote cap ellipse");
        let c = edge.curve();
        let (a, b) = c.range_tuple();
        for i in 0..=64 {
            let p = c.subs(a + (b - a) * i as f64 / 64.);
            let d = (50. - p.z + 5. * p.x) / k;
            let u = (p.x + 5. * p.z) / k;
            assert!(((p.y - cy).powi(2) + (d - 1.).powi(2) - 1.).abs() < 1e-8);
            if seam {
                assert!(((u - cx).powi(2) + (d - 1.).powi(2) - 1.).abs() < 1e-8);
                assert!((u + p.y - cx - cy).abs() < 1e-8);
                assert!(d >= -1e-8 && d <= dc + 1e-8);
            } else {
                assert!((u + 5. * d - 250. / k).abs() < 1e-8);
            }
        }
    }
    let remote_start = point(cx, 100., 0.);
    let remote_end = point(cx + 1. / k, (10000. - 200. * k * dc).sqrt(), dc);
    let remote = output.boundaries()[0]
        .edge_iter()
        .find(|e| {
            (e.front().point().distance(remote_start) < 1e-7
                && e.back().point().distance(remote_end) < 1e-7)
                || (e.back().point().distance(remote_start) < 1e-7
                    && e.front().point().distance(remote_end) < 1e-7)
        })
        .expect("positive profile/cylinder termination");
    let curve = remote.curve();
    let (a, b) = curve.range_tuple();
    for i in 0..=129 {
        let p = curve.subs(a + (b - a) * i as f64 / 129.);
        let d = (50. - p.z + 5. * p.x) / k;
        let u = (p.x + 5. * p.z) / k;
        assert!((d - (50. - p.y * p.y / 200.) / k).abs() < 1e-7);
        assert!(((u - cx).powi(2) + (d - 1.).powi(2) - 1.).abs() < 1e-7);
        assert!(p.y > 0. && d >= -1e-7 && d <= dc + 1e-7 && u >= cx - 1e-7);
    }
    let mesh = output.triangulation(0.0005).to_polygon();
    let check = |p: Point3, n: Vector3| {
        for delta in [-0.003, 0.003] {
            assert_eq!(mesh.inside(p + n * delta), delta < 0., "boundary {:?}", p);
        }
    };
    let uaxis = Vector3::new(1., 0., 5.) / k;
    let daxis = Vector3::new(5., 0., -1.) / k;
    for angle in [0.2_f64, 0.8, 1.5, 2.5, 2.9] {
        let d = 1. - angle.cos();
        let v = angle.sin();
        check(
            point(80., cy - v, d),
            -Vector3::unit_y() * v - daxis * angle.cos(),
        );
        check(point(cx + v, 0., d), uaxis * v - daxis * angle.cos());
    }
    for d in [0.5, 1.5, (dc + dt) / 2., 4.] {
        let low = 250. / k - 5. * d;
        let high = 770. / k - 5. * d;
        check(point(low, 0., d), (-uaxis - 5. * daxis) / k);
        if d > dc {
            check(point(high, 0., d), (uaxis + 5. * daxis) / k);
        }
    }
    check(point(80., 0., 0.), -daxis);
    for y in [-50., 0., 50.] {
        let d = (50. - y * y / 200.) / k;
        let n = (Vector3::unit_y() * (y / (100. * k)) + daxis).normalize();
        check(point(80., y, d), n);
    }
}

#[test]
fn bounded_f9_rigid_scale_and_reversal() {
    let (input, _) = body();
    for scale in [0.5, 2.] {
        let matrix = Matrix4::from_translation(Vector3::new(3., -8., 11.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7))
            * Matrix4::from_scale(scale);
        let body = builder::transformed(&input, matrix);
        for face in body.face_iter() {
            if let Surface::Extruded(e) = face.surface() {
                let v = e.extruding_vector();
                let mut c = e.entity_curve().clone();
                if let Curve::BSplineCurve(b) = &c {
                    c = BSplineCurve::new(
                        vec![2., 2., 2., 5., 5., 5.].into(),
                        b.control_points().to_vec(),
                    )
                    .into();
                }
                c.transform_by(Matrix4::from_translation(v * 1.001));
                c.invert();
                face.set_surface(ExtrudedCurve::by_extrusion(c, -v * 1.002).into());
            }
        }
        let selected = [Point3::new(10., -100., 100.), Point3::new(20., 0., 150.)].map(|p| {
            common::blend::edge_through(&body.boundaries()[0], matrix.transform_point(p)).id()
        });
        let out =
            try_fillet_solid_edges_with_radii(&body, &selected.map(|id| (id, scale)), 1e-5 * scale)
                .unwrap()
                .solid;
        check_structure(&body, &out);
        common::assert_solid(
            &out,
            130745.06675857291 * scale.powi(3),
            &[0],
            0.001 * scale,
        );
    }
}

#[test]
fn bounded_f9_immutable_profile_guards() {
    for (y, z) in [(0., -49.), (1., -50.), (-1., -50.), (0., -60.)] {
        let (input, ids) = body_variant(y, z);
        let before = serde_json::to_string(&input.compress()).unwrap();
        assert!(try_fillet_solid_edges_with_radii(
            &input,
            &ids.iter().map(|id| (*id, 1.)).collect::<Vec<_>>(),
            1e-5
        )
        .is_err());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn bounded_f9_immutable_hole_and_extra_supports() {
    let (input, ids) = body_with_hole(0., -50., true);
    assert!(input.is_geometric_consistent());
    assert!(Solid::try_new(input.boundaries().clone()).is_ok());
    let before = serde_json::to_string(&input.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(
        &input,
        &ids.iter().map(|id| (*id, 1.)).collect::<Vec<_>>(),
        1e-5
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}
