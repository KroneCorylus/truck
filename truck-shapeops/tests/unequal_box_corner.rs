mod common;
use truck_modeling::*;
use truck_shapeops::fillet::try_fillet_solid_edges_with_radii;

#[test]
fn rational_linear_box_edges_require_pole_free_weights() {
    for weights in [[1., -1.], [1., 2.], [-1., -2.]] {
        let input = common::modeling::cuboid(Point3::origin(), Point3::new(5., 5., 5.));
        let corner = Point3::new(5., 0., 5.);
        let selected: Vec<_> = [Vector3::unit_y(), Vector3::unit_x()]
            .into_iter()
            .zip([1., 0.5])
            .map(|(axis, radius)| {
                let edge = input
                    .edge_iter()
                    .find(|edge| {
                        (edge.front().point().near(&corner) || edge.back().point().near(&corner))
                            && (edge.back().point() - edge.front().point()).dot(axis).abs() > 1.
                    })
                    .unwrap();
                (edge.id(), radius)
            })
            .collect();
        let edge = input
            .edge_iter()
            .find(|edge| edge.id() == selected[0].0)
            .unwrap();
        let controls = [edge.absolute_front().point(), edge.absolute_back().point()]
            .into_iter()
            .zip(weights)
            .map(|(p, w)| p.to_homogeneous() * w)
            .collect();
        edge.set_curve(
            NurbsCurve::new(BSplineCurve::new(
                KnotVec::from(vec![0., 0., 1., 1.]),
                controls,
            ))
            .into(),
        );
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_edges_with_radii(&input, &selected, 0.001);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if weights[0].signum() != weights[1].signum() {
            assert!(
                result.is_err(),
                "opposite weights must not be reconstructed as a box"
            );
        } else {
            let result = result.unwrap();
            common::assert_solid(&result.solid, 123.69042579279512, &[0], 0.001);
            common::blend::assert_step(&result.solid, 123.69042579279512, 0.001);
        }
    }
}

#[test]
fn unequal_box_crease_material_and_step() {
    for scale in [0.1, 1., 3.] {
        let transform = Matrix4::from_translation(Vector3::new(2., -3., 5.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
            * Matrix4::from_scale(scale);
        let input = common::modeling::cuboid(Point3::origin(), Point3::new(5., 5., 5.));
        let input = builder::transformed(&input, transform);
        let before = serde_json::to_string(&input.compress()).unwrap();
        let edge = |p: Point3, q: Point3| {
            let (p, q) = (transform.transform_point(p), transform.transform_point(q));
            input
                .edge_iter()
                .find(|e| {
                    (e.front().point().near(&p) && e.back().point().near(&q))
                        || (e.front().point().near(&q) && e.back().point().near(&p))
                })
                .unwrap()
                .id()
        };
        let a = edge(Point3::new(5., 0., 5.), Point3::new(5., 5., 5.));
        let b = edge(Point3::new(5., 0., 5.), Point3::new(0., 0., 5.));
        for selection in [
            [(a, scale), (b, 0.5 * scale)],
            [(b, 0.5 * scale), (a, scale)],
        ] {
            let result =
                try_fillet_solid_edges_with_radii(&input, &selection, 0.001 * scale).unwrap();
            assert_eq!(result.solid.face_iter().count(), 8);
            assert_eq!(result.generated_faces.len(), 2);
            assert!(result.solid.is_geometric_consistent());
            let faces = &result.generated_faces;
            let shared: Vec<_> = faces[0]
                .edge_iter()
                .filter(|e| faces[1].edge_iter().any(|f| e.id() == f.id()))
                .collect();
            assert_eq!(shared.len(), 1);
            let seam = &shared[0];
            let other = faces[1].edge_iter().find(|e| e.id() == seam.id()).unwrap();
            assert_ne!(seam.orientation(), other.orientation());
            let inverse = transform.invert().unwrap();
            let curve = seam.curve();
            let (lo, hi) = curve.range_tuple();
            let mut last = -1.;
            for i in 0..=256 {
                let t = lo + (hi - lo) * i as f64 / 256.;
                let p = inverse.transform_point(curve.subs(t));
                let depth = 5. - p.z;
                assert!(depth >= last - 1e-9 && (-1e-9..=0.5 + 1e-9).contains(&depth));
                last = depth;
                assert!((p.x - 4.).powi(2) + (p.z - 4.).powi(2) > 0.);
                assert!(((p.x - 4.).powi(2) + (p.z - 4.).powi(2) - 1.).abs() < 1e-7);
                assert!(((p.y - 0.5).powi(2) + (p.z - 4.5).powi(2) - 0.25).abs() < 1e-7);
                assert!(p.x >= 4. - 1e-8 && p.y <= 0.5 + 1e-8);
                let normals: Vec<_> = faces
                    .iter()
                    .map(|f| {
                        let surface = f.oriented_surface();
                        let uv = surface.search_parameter(curve.subs(t), None, 100).unwrap();
                        assert!(surface.subs(uv.0, uv.1).distance(curve.subs(t)) < 1e-7);
                        inverse
                            .transform_vector(surface.normal(uv.0, uv.1))
                            .normalize()
                    })
                    .collect();
                assert!(
                    (normals[0].dot(normals[1]) - (1. - depth) * (1. - 2. * depth)).abs() < 1e-6
                );
            }
            let q = transform.transform_point(Point3::new(4. + 3_f64.sqrt() / 2., 0., 4.5));
            let r = transform.transform_point(Point3::new(5., 0., 4.));
            assert!(result.solid.edge_iter().any(|e| (e.front().point().near(&q)
                && e.back().point().near(&r))
                || (e.front().point().near(&r) && e.back().point().near(&q))));
            let bottom = transform.transform_point(Point3::new(5., 0., 0.));
            assert!(result
                .solid
                .edge_iter()
                .any(
                    |e| (e.front().point().near(&bottom) && e.back().point().near(&r))
                        || (e.front().point().near(&r) && e.back().point().near(&bottom))
                ));

            common::assert_solid(
                &result.solid,
                123.69042579279512 * scale.powi(3),
                &[0],
                0.001 * scale,
            );
            if scale == 1. {
                common::assert_solid(&result.solid, 123.69042579279512, &[0], 0.00025);
            }
            common::blend::assert_step(
                &result.solid,
                123.69042579279512 * scale.powi(3),
                0.001 * scale,
            );
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn unequal_box_guards_preserve_input() {
    let input = common::modeling::cuboid(Point3::origin(), Point3::new(5., 6., 7.));
    let corner = Point3::new(5., 0., 7.);
    let edges: Vec<_> = input
        .edge_iter()
        .filter(|e| e.front().point().near(&corner) || e.back().point().near(&corner))
        .collect();
    let a = edges
        .iter()
        .find(|e| (e.back().point() - e.front().point()).y.abs() > 1.)
        .unwrap()
        .id();
    let b = edges
        .iter()
        .find(|e| (e.back().point() - e.front().point()).x.abs() > 1.)
        .unwrap()
        .id();
    let before = serde_json::to_string(&input.compress()).unwrap();
    for (ra, rb) in [
        (0., 0.5),
        (-1., 0.5),
        (f64::NAN, 0.5),
        (f64::INFINITY, 0.5),
        (1., 1. + 1e-8),
        (1., 1e-8),
        (5., 0.5),
        (1., 6.),
        (7., 0.5),
    ] {
        for order in [[(a, ra), (b, rb)], [(b, rb), (a, ra)]] {
            assert!(try_fillet_solid_edges_with_radii(&input, &order, 0.001).is_err());
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        }
    }
    // Non-square boxes use the same geometry-derived frame and independent sections.
    let result = try_fillet_solid_edges_with_radii(&input, &[(a, 1.), (b, 0.5)], 0.001).unwrap();
    let volume = 210. - (6. + 5. * 0.25) * (1. - std::f64::consts::PI / 4.) + 0.03168727156106706;
    common::assert_solid(&result.solid, volume, &[0], 0.001);
    common::blend::assert_step(&result.solid, volume, 0.001);
    for (large, small, volume) in [
        (2., 0.3, 204.78269772177754),
        (0.5, 0.4, 209.51492707686393),
    ] {
        let result =
            try_fillet_solid_edges_with_radii(&input, &[(a, large), (b, small)], 0.001).unwrap();
        common::assert_solid(&result.solid, volume, &[0], 0.001);
        common::blend::assert_step(&result.solid, volume, 0.001);
    }
    let cavity = common::modeling::cuboid(Point3::new(1., 1., 1.), Point3::new(2., 2., 2.));
    let mut boundaries = input.boundaries().clone();
    boundaries.push(cavity.boundaries()[0].inverse().into());
    let hollow = Solid::try_new(boundaries).unwrap();
    let before_hollow = serde_json::to_string(&hollow.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(&hollow, &[(a, 1.), (b, 0.5)], 0.001).is_err());
    assert_eq!(
        before_hollow,
        serde_json::to_string(&hollow.compress()).unwrap()
    );
    let shear = Matrix4::from_cols(
        Vector4::new(1., 0., 0., 0.),
        Vector4::new(0.2, 1., 0., 0.),
        Vector4::new(0., 0., 1., 0.),
        Vector4::new(0., 0., 0., 1.),
    );
    let skew = builder::transformed(&input, shear);
    let corner = shear.transform_point(corner);
    let mut ids = std::collections::HashSet::new();
    let adjacent: Vec<_> = skew
        .edge_iter()
        .filter(|e| {
            (e.front().point().near(&corner) || e.back().point().near(&corner))
                && ids.insert(e.id())
        })
        .collect();
    let selected: Vec<_> = adjacent
        .iter()
        .filter(|e| (e.back().point() - e.front().point()).z.abs() < 1e-6)
        .map(|e| e.id())
        .collect();
    let before = serde_json::to_string(&skew.compress()).unwrap();
    assert!(try_fillet_solid_edges_with_radii(
        &skew,
        &[(selected[0], 1.), (selected[1], 0.5)],
        0.001
    )
    .is_err());
    assert_eq!(before, serde_json::to_string(&skew.compress()).unwrap());
}

#[test]
fn translated_unequal_box_numerical_admission_is_immutable() {
    for (offset, accepted) in [(10., true), (1e9, false)] {
        let input = common::modeling::cuboid(
            Point3::new(offset, 0., 0.),
            Point3::new(offset + 5., 5., 5.),
        );
        let corner = Point3::new(offset + 5., 0., 5.);
        let selected: Vec<_> = [Vector3::unit_y(), Vector3::unit_x()]
            .into_iter()
            .zip([1., 0.5])
            .map(|(axis, radius)| {
                let edge = input
                    .edge_iter()
                    .find(|e| {
                        (e.front().point() == corner || e.back().point() == corner)
                            && (e.back().point() - e.front().point()).dot(axis).abs() > 1.
                    })
                    .unwrap();
                (edge.id(), radius)
            })
            .collect();
        let before = serde_json::to_string(&input.compress()).unwrap();
        let result = try_fillet_solid_edges_with_radii(&input, &selected, 0.001);
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        if let Ok(result) = result {
            common::assert_solid(&result.solid, 123.69042579279512, &[0], 0.001);
        }
    }
}
