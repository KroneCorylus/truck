mod common;
use std::f64::consts::PI;
use truck_base::diagnostics::Code;
use truck_modeling::*;
use truck_shapeops::{fillet::*, SplitClosedEdgesAndFaces};
use truck_stepio::r#in::Table;

const TOL: f64 = 0.001;

fn input() -> Solid {
    let table = Table::from_step(include_str!("fixtures/rounded-roof-post/input.step")).unwrap();
    let source = table.manifold_solid_brep.values().next().unwrap();
    let (mut compressed, skipped) = table.to_compressed_solid(source).unwrap();
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

fn root(solid: &Solid, inverse: Matrix4) -> Wire {
    solid
        .face_iter()
        .find_map(|face| {
            face.boundaries().into_iter().find(|wire| {
                wire.iter().all(|edge| {
                    let curve = edge.curve();
                    let (a, b) = curve.range_tuple();
                    [a, (a + b) / 2., b].iter().all(|&t| {
                        let point = inverse.transform_point(curve.subs(t));
                        (point.z - 5.).abs() < 1e-6
                            && ((point.x - 2.5).hypot(point.y - 2.5) - 1.).abs() < 1e-6
                    })
                })
            })
        })
        .unwrap()
}

fn input_volume() -> f64 { 125. - 5. * (1. - PI / 4.) + 7.5 * PI }

#[test]
fn rounded_roof_post_import_preserves_its_material() {
    let solid = input();
    assert!(solid.is_geometric_consistent());
    common::assert_solid(&solid, input_volume(), &[0], TOL);
}

#[test]
fn post_fillet_cannot_return_a_contact_loop_crossing_the_roof_boundary() {
    for transformed in [false, true] {
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(13., -7., 23.))
                * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73))
        } else {
            Matrix4::identity()
        };
        let solid = builder::transformed(&input(), matrix);
        let mut wire = root(&solid, matrix.invert().unwrap());
        if transformed {
            wire.invert();
        }
        let before = serde_json::to_string(&solid.compress()).unwrap();
        let result = try_fillet_solid_along_wire(&solid, &wire, 1., TOL);
        let error = result.expect_err("an overlapping planar contact loop is not a solid");
        assert_eq!(error.code, Code::OutsideNeighbour);
        assert_eq!(before, serde_json::to_string(&solid.compress()).unwrap());
    }
}

#[test]
fn post_fillet_that_fits_the_roof_remains_supported() {
    let solid = input();
    let radius = 0.2_f64;
    let result =
        try_fillet_solid_along_wire(&solid, &root(&solid, Matrix4::identity()), radius, TOL)
            .unwrap();
    let added = PI
        * (2. * radius.powi(2) + 5. * radius.powi(3) / 3.
            - PI / 2. * (1. + radius) * radius.powi(2));
    common::assert_solid(&result.solid, input_volume() + added, &[0], TOL);
    common::blend::assert_step(&result.solid, input_volume() + added, TOL);
}
