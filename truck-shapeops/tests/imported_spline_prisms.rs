mod common;
use truck_modeling::*;
use truck_shapeops::SplitClosedEdgesAndFaces;
use truck_stepio::r#in::Table;

fn input(case: &str) -> Solid {
    let text = match case {
        "G3" => include_str!("fixtures/spline-prisms/G3.step"),
        "G7" => include_str!("fixtures/spline-prisms/G7.step"),
        _ => unreachable!(),
    };
    let table = Table::from_step(text).unwrap();
    let (mut raw, skipped) = table
        .to_compressed_solid(table.manifold_solid_brep.values().next().unwrap())
        .unwrap();
    assert!(skipped.is_empty());
    raw.split_closed_edges_and_faces(0.00001);
    Solid::extract(
        raw.try_mapped(
            |p| Some(*p),
            |c| Curve::try_from(c).ok(),
            |s| Surface::try_from(s).ok(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn verify(scale: f64, transformed: bool) {
    let matrix = if transformed {
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
    } else {
        Matrix4::identity()
    } * Matrix4::from_scale(scale);
    for (case, volume) in [("G3", 2_300_000.), ("G7", 372_000.)] {
        let solid = builder::transformed(&input(case), matrix);
        assert!(solid.is_geometric_consistent(), "{case}");
        for face in solid.face_iter() {
            let surface = face.oriented_surface();
            if !matches!(surface, Surface::Extruded(_)) {
                continue;
            }
            let (u, v) = surface.try_range_tuple();
            let ((a, b), (c, d)) = (u.unwrap(), v.unwrap());
            for edge in face.edge_iter() {
                let curve = edge.curve();
                let (t0, t1) = curve.range_tuple();
                for i in 0..=64 {
                    let p = curve.subs((t0 + (t1 - t0) * i as f64 / 64.).clamp(t0, t1));
                    let (u, v) = surface.search_nearest_parameter(p, None, 100).unwrap();
                    assert!((a - 1e-9..=b + 1e-9).contains(&u), "{case}: u={u}");
                    assert!((c - 1e-9..=d + 1e-9).contains(&v), "{case}: v={v}");
                    assert!(surface.subs(u, v).distance(p) < 1e-7);
                }
            }
        }
        let expected = volume * scale.powi(3);
        common::assert_solid(&solid, expected, &[0], 0.001);
        common::blend::assert_step(&solid, expected, 0.001);
    }
}

#[test]
fn imported_spline_prisms_preserve_trimmed_material_and_step() {
    verify(0.01, false);
    verify(0.01, true);
}

#[test]
fn original_scale_spline_prisms_preserve_trimmed_material_and_step() { verify(1., false); }
