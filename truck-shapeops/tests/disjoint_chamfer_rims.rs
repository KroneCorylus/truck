mod common;

use std::collections::HashSet;
use truck_modeling::*;
use truck_shapeops::fillet::try_chamfer_solid_edges_with_distances;

const TOL: f64 = 0.001;

fn cylinder(full: bool) -> Solid {
    if full {
        return common::modeling::cylinder(Point3::origin(), Vector3::unit_z(), 5.0, 10.0);
    }
    let center = builder::vertex(Point3::origin());
    let start = builder::vertex(Point3::new(5.0, 0.0, 0.0));
    let mut boundary = builder::rsweep(
        &start,
        Point3::origin(),
        Vector3::unit_z(),
        Rad(std::f64::consts::FRAC_PI_2),
        2,
    );
    let closing = [
        builder::line(boundary.back_vertex().unwrap(), &center),
        builder::line(&center, boundary.front_vertex().unwrap()),
    ];
    boundary.extend(closing);
    builder::tsweep(
        &builder::try_attach_plane(&[boundary]).unwrap(),
        Vector3::new(0.0, 0.0, 10.0),
    )
}

fn check_rims(full: bool, sizes: [[f64; 2]; 2], reject: bool) {
    for transformed in [false, true] {
        let scale = if transformed { 0.2 } else { 1.0 };
        let matrix = if transformed {
            Matrix4::from_translation(Vector3::new(2.0, -3.0, 4.0))
                * Matrix4::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalize(), Rad(0.7))
                * Matrix4::from_scale(scale)
        } else {
            Matrix4::identity()
        };
        let inverse = matrix.invert().unwrap();
        let mut input = builder::transformed(&cylinder(full), matrix);
        if transformed {
            let mut faces: Vec<_> = input.boundaries()[0].iter().cloned().collect();
            faces.reverse();
            input = Solid::new(vec![faces.into()]);
        }
        let before = serde_json::to_string(&input.compress()).unwrap();
        let mut seen = HashSet::new();
        let mut selected = Vec::new();
        for edge in input.edge_iter().filter(|e| seen.insert(e.id())) {
            let curve = edge.curve();
            let samples: Vec<_> = curve
                .parameter_division(curve.range_tuple(), TOL)
                .1
                .into_iter()
                .map(|p| inverse.transform_point(p))
                .collect();
            let top = samples[0].z > 5.0;
            let z = if top { 10.0 } else { 0.0 };
            if !samples
                .iter()
                .all(|p| (p.z - z).abs() < 1e-7 && (p.x * p.x + p.y * p.y - 25.0).abs() < 1e-6)
            {
                continue;
            }
            let pair = sizes[usize::from(!top)];
            let distances: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == edge.id()))
                .map(|f| {
                    let cap = f
                        .oriented_surface()
                        .normal(0.0, 0.0)
                        .normalize()
                        .dot(matrix.transform_vector(Vector3::unit_z()).normalize())
                        .abs()
                        > 0.9;
                    pair[usize::from(!cap)] * scale
                })
                .collect();
            selected.push((edge.id(), distances.try_into().unwrap()));
        }
        assert!(selected.len() >= 2);
        if transformed {
            selected.reverse();
        }
        let result = try_chamfer_solid_edges_with_distances(&input, &selected, TOL);
        if reject {
            assert!(result.is_err(), "overlapping end cuts must fail");
            assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
            continue;
        }
        let result = result.unwrap();
        let theta = if full {
            std::f64::consts::TAU
        } else {
            std::f64::consts::FRAC_PI_2
        };
        let expected = (theta / 2.0 * 25.0 * 10.0
            - sizes
                .iter()
                .map(|[radial, axial]| theta / 2.0 * axial * (5.0 * radial - radial * radial / 3.0))
                .sum::<f64>())
            * scale.powi(3);
        common::assert_solid(&result.solid, expected, &[0], TOL);
        common::blend::assert_step(&result.solid, expected, TOL);
        assert!(result.solid.is_geometric_consistent());
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        assert!(!result.generated_faces.is_empty());
        for (id, face) in &result.modified_faces {
            assert!(input.face_iter().any(|f| f.id() == *id));
            assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        }
        for face in &result.generated_faces {
            assert!(!input.face_iter().any(|f| f.id() == face.id()));
            assert!(result.solid.face_iter().any(|f| f.id() == face.id()));
        }
        let modified: HashSet<_> = result.modified_faces.iter().map(|(id, _)| *id).collect();
        assert_eq!(modified.len(), result.modified_faces.len());
        let expected_faces: HashSet<_> = input
            .face_iter()
            .filter(|f| !modified.contains(&f.id()))
            .map(|f| f.id())
            .chain(result.modified_faces.iter().map(|(_, f)| f.id()))
            .chain(result.generated_faces.iter().map(|f| f.id()))
            .collect();
        assert_eq!(
            expected_faces,
            result.solid.face_iter().map(|f| f.id()).collect()
        );
        let mut contacts = [[false; 2]; 2];
        for face in &result.generated_faces {
            for vertex in face.vertex_iter() {
                let p = inverse.transform_point(vertex.point());
                let radius = (p.x * p.x + p.y * p.y).sqrt();
                let end = usize::from(p.z < 5.0);
                let [radial, axial] = sizes[end];
                let cap = if end == 0 { 10.0 } else { 0.0 };
                let side = if end == 0 { 10.0 - axial } else { axial };
                let on_cylinder = (radius - 5.0).abs() < 1e-6 && (p.z - side).abs() < 1e-6;
                let on_cap = (radius - (5.0 - radial)).abs() < 1e-6 && (p.z - cap).abs() < 1e-6;
                assert!(on_cylinder || on_cap, "unexpected rim contact {p:?}");
                contacts[end][0] |= on_cylinder;
                contacts[end][1] |= on_cap;
            }
        }
        assert_eq!(contacts, [[true; 2]; 2]);
    }
}

#[test]
fn quarter_cylinder_disjoint_rims() {
    for sizes in [
        [[0.7, 0.7]; 2],
        [[0.7, 0.5]; 2],
        [[0.7, 0.7 / 3.0_f64.sqrt()]; 2],
        [[0.7, 0.5], [0.4, 0.3]],
    ] {
        check_rims(false, sizes, false);
    }
}
#[test]
fn full_cylinder_disjoint_rims() {
    for sizes in [
        [[0.7, 0.7]; 2],
        [[0.7, 0.5]; 2],
        [[0.7, 0.7 / 3.0_f64.sqrt()]; 2],
        [[0.7, 0.5], [0.4, 0.3]],
    ] {
        check_rims(true, sizes, false);
    }
}

#[test]
fn overlapping_disjoint_rim_cuts_preserve_input() {
    for full in [false, true] {
        check_rims(full, [[0.7, 6.0]; 2], true);
    }
}
