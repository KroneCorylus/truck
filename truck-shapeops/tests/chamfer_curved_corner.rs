mod common;

use truck_modeling::*;
use truck_shapeops::fillet::{try_chamfer_solid_edges, try_chamfer_solid_edges_with_distances};

fn quarter(radius: f64, length: f64) -> Solid {
    sector(radius, length, std::f64::consts::FRAC_PI_2)
}

fn sector(radius: f64, length: f64, angle: f64) -> Solid {
    let start = Vertex::new(Point3::new(0., radius, 0.));
    let arc = builder::rsweep(&start, Point3::origin(), Vector3::unit_x(), Rad(angle), 1);
    let center = Vertex::new(Point3::origin());
    let mut wire = arc.clone();
    wire.extend([
        builder::line(arc.back_vertex().unwrap(), &center),
        builder::line(&center, &start),
    ]);
    builder::tsweep(
        &builder::try_attach_plane(&[wire]).unwrap(),
        Vector3::unit_x() * length,
    )
}

fn selection(input: &Solid, corner: Point3) -> Vec<EdgeID> {
    let mut seen = std::collections::HashSet::new();
    input
        .edge_iter()
        .filter(|e| e.front().point().near(&corner) || e.back().point().near(&corner))
        .map(|e| e.id())
        .filter(|id| seen.insert(*id))
        .collect()
}

fn setbacks(
    input: &Solid,
    ids: &[EdgeID],
    radius: f64,
    distances: [f64; 5],
    inverse: Matrix4,
    other_axial: f64,
) -> Vec<(EdgeID, [f64; 2])> {
    let [axial, inset, flat, chord, side] = distances;
    ids.iter()
        .map(|&id| {
            let faces: Vec<_> = input
                .face_iter()
                .filter(|f| f.edge_iter().any(|e| e.id() == id))
                .collect();
            let is_cylinder = |f: &Face| {
                matches!(
                    f.surface().elementary(),
                    Some((Elementary::Cylinder { .. }, _))
                )
            };
            let is_cap = |f: &Face| match f.oriented_surface() {
                Surface::Plane(p) => inverse.transform_vector(p.normal()).x.abs() > 0.9,
                _ => false,
            };
            let rim = faces.iter().any(|f| is_cylinder(f)) && faces.iter().any(|f| is_cap(f));
            let lip = faces.iter().any(|f| is_cylinder(f)) && !rim;
            let d = std::array::from_fn(|i| {
                if rim {
                    if is_cylinder(faces[i]) {
                        axial
                    } else {
                        inset
                    }
                } else if lip {
                    if is_cylinder(faces[i]) {
                        radius * (2. * (chord / (2. * radius)).asin()).tan()
                    } else {
                        flat
                    }
                } else if is_cap(faces[i]) {
                    side
                } else {
                    other_axial
                }
            });
            (id, d)
        })
        .collect()
}

fn check(radius: f64, length: f64, distances: [f64; 5], transform: Matrix4, reverse: bool) {
    let axial = distances[0];
    let inverse = transform.invert().unwrap();
    let mut input = builder::transformed(&quarter(radius, length), transform);
    if reverse {
        input = Solid::new(vec![input.boundaries()[0].iter().rev().cloned().collect()]);
    }
    let mut ids = selection(
        &input,
        transform.transform_point(Point3::new(length, 0., radius)),
    );
    assert_eq!(ids.len(), 3);
    if reverse {
        ids.reverse();
    }
    let selected = setbacks(&input, &ids, radius, distances, inverse, axial);
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).unwrap();
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    assert_eq!(result.modified_faces.len(), 5);
    assert_eq!(result.generated_faces.len(), 4);
    verify(&result.solid, radius, length, distances, inverse);
}

fn verify(solid: &Solid, radius: f64, length: f64, distances: [f64; 5], inverse: Matrix4) {
    let [axial, inset, flat, chord, side] = distances;
    let p = Point3::new(
        length - axial,
        chord * (1. - chord * chord / (4. * radius * radius)).sqrt(),
        radius - chord * chord / (2. * radius),
    );
    let q = Point3::new(length - axial, 0., radius - flat);
    let r = Point3::new(
        length,
        side,
        ((radius - inset).powi(2) - side * side).sqrt(),
    );
    let normal = (q - p).cross(r - p);
    for point in [p, q, r] {
        assert!(
            solid
                .vertex_iter()
                .any(|v| inverse.transform_point(v.point()).distance(point) < 1e-8),
            "missing {point:?}"
        );
    }
    assert!((Point3::new(p.x, 0., radius).distance(p) - chord).abs() < 1e-12);
    let edge = solid
        .edge_iter()
        .find(|e| {
            let a = inverse.transform_point(e.front().point());
            let b = inverse.transform_point(e.back().point());
            (a.near(&p) && b.near(&r)) || (a.near(&r) && b.near(&p))
        })
        .unwrap();
    let curve = edge.curve();
    let (a, b) = curve.range_tuple();
    for i in 0..=32 {
        let v = inverse.transform_point(curve.subs(a + (b - a) * i as f64 / 32.));
        assert!(normal.dot(v - p).abs() < 1e-9);
        let rho = radius - inset * (v.x - (length - axial)) / axial;
        assert!((v.y * v.y + v.z * v.z - rho * rho).abs() < 1e-9);
    }
    assert!(
        inverse
            .transform_point(curve.subs((a + b) / 2.))
            .distance(p + (r - p) / 2.)
            > 1e-5
    );
    // Independent material integral: cylinder/cone, longitudinal bevel, radial bevel,
    // and corner-plane half-spaces. Midpoint quadrature is refined independently of tessellation.
    let integrate = |n: usize| {
        let mut volume = 0.;
        for i in 0..n {
            let x = length * (i as f64 + 0.5) / n as f64;
            let fraction = (x - (length - axial)).max(0.) / axial;
            let rho = radius - inset * fraction;
            let low = side * fraction;
            for j in 0..n {
                let y = low + (rho - low) * (j as f64 + 0.5) / n as f64;
                let circle = (rho * rho - y * y).sqrt();
                let lip = q.z + (p.z - q.z) * y / p.y;
                let corner = p.z - (normal.x * (x - p.x) + normal.y * (y - p.y)) / normal.z;
                volume +=
                    circle.min(lip).min(corner).max(0.) * length * (rho - low) / (n * n) as f64;
            }
        }
        volume
    };
    // Refine the material oracle for larger bodies without increasing the mesh tolerance.
    let subdivisions = if radius >= 10. { 6400 } else { 1600 };
    let volume = integrate(subdivisions);
    assert!((volume - integrate(subdivisions / 2)).abs() < 2e-5 * radius * radius * length);
    common::assert_solid(solid, volume, &[0], 0.0001);
    common::blend::assert_step(solid, volume, 0.0001);
}

#[test]
fn quarter_cylinder_three_chamfers() {
    for reverse in [false, true] {
        check(1., 1., [0.2; 5], Matrix4::identity(), reverse);
    }
}

#[test]
fn curved_corner_distances_and_orientation() {
    let transform = Matrix4::from_translation(Vector3::new(2., -3., 4.))
        * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.7));
    for reverse in [false, true] {
        check(2., 3., [0.3, 0.4, 0.35, 0.25, 0.4], transform, reverse);
    }
}

#[test]
fn oversized_curved_corner_leaves_input_unchanged() {
    let input = quarter(1., 1.);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    let before = serde_json::to_string(&input.compress()).unwrap();
    for distance in [0., 0.8, 1., 2.] {
        let error = try_chamfer_solid_edges(&input, &ids, distance, 0.001).unwrap_err();
        assert!(matches!(
            error.code,
            truck_base::diagnostics::Code::OutsideNeighbour
                | truck_base::diagnostics::Code::InvalidParameter
        ));
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    }
}

#[test]
fn curved_corner_normal_section_angle() {
    let alpha = 0.5_f64.atan();
    let flat: f64 = 0.2;
    let tangent = alpha.tan();
    // Reference is the radial plane: cy / (cz - (1-flat)) = tan(alpha),
    // with cy² + cz² = 1. The cylinder setback passed to check is a chord.
    let cz = (tangent * tangent * (1. - flat)
        + (1. + tangent * tangent - (tangent * (1. - flat)).powi(2)).sqrt())
        / (1. + tangent * tangent);
    let cy = tangent * (cz - (1. - flat));
    let chord = cy.hypot(cz - 1.);
    assert!((cy / (cz - (1. - flat)) - alpha.tan()).abs() < 1e-12);
    check(
        1.,
        1.,
        [flat * tangent, flat, flat, chord, flat],
        Matrix4::identity(),
        false,
    );
}

#[test]
fn equal_distance_api_uses_tangent_projection() {
    let input = quarter(1., 1.);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    let output = try_chamfer_solid_edges(&input, &ids, 0.2, 0.001).unwrap();
    let expected = Point3::new(0.8, 0.2 / 1.04_f64.sqrt(), 1. / 1.04_f64.sqrt());
    assert!(output
        .solid
        .vertex_iter()
        .any(|v| v.point().distance(expected) < 1e-9));
    let chord = 2. * (0.2_f64.atan() / 2.).sin();
    check(
        1.,
        1.,
        [0.2, 0.2, 0.2, chord, 0.2],
        Matrix4::identity(),
        false,
    );
}

#[test]
// This fixture fails through the public dispatcher today. The constructor guard itself
// does not forbid another constructor from supporting an elliptic four-contact corner.
fn unsupported_elliptic_corner_contract_leaves_input_unchanged() {
    let input = quarter(1., 1.);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    let selected = setbacks(
        &input,
        &ids,
        1.,
        [0.2, 0.1, 0.1, 0.05, 0.7],
        Matrix4::identity(),
        0.2,
    );
    let before = serde_json::to_string(&input.compress()).unwrap();
    let error = try_chamfer_solid_edges_with_distances(&input, &selected, &[], 0.001).unwrap_err();
    assert_eq!(
        error.code,
        truck_base::diagnostics::Code::UnsupportedGeometry
    );
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
}

fn immutable_attempt(input: &Solid, selected: &[(EdgeID, [f64; 2])]) -> Option<Solid> {
    let before = serde_json::to_string(&input.compress()).unwrap();
    let result = try_chamfer_solid_edges_with_distances(input, selected, &[], 0.001);
    assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
    match result {
        Ok(result) => Some(result.solid),
        Err(error) => {
            use truck_base::diagnostics::Code;
            assert!(
                matches!(
                    error.code,
                    Code::OutsideNeighbour
                        | Code::UnsupportedGeometry
                        | Code::IntersectionFailed
                        | Code::UnsupportedTopology
                ) || (error.code == Code::BlendConstructionFailed
                    && error.stage == "validate_geometry"),
                "{error:?}"
            );
            eprintln!("rejection: {error}");
            None
        }
    }
}

#[test]
fn curved_corner_unsupported_sector_and_unequal_axial_setbacks() {
    for angle in [1.2, 1.9] {
        let input = sector(1., 1., angle);
        let ids = selection(&input, Point3::new(1., angle.cos(), angle.sin()));
        assert_eq!(ids.len(), 3);
        let selected: Vec<_> = ids.into_iter().map(|id| (id, [0.2; 2])).collect();
        assert!(immutable_attempt(&input, &selected).is_none());
    }
    let input = quarter(1., 1.);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    for other in [0.1, 0.3] {
        let selected = setbacks(&input, &ids, 1., [0.2; 5], Matrix4::identity(), other);
        assert!(immutable_attempt(&input, &selected).is_none());
    }
}

#[test]
fn curved_corner_equivalent_spline_boundary() {
    // Same five supports, vertices and edges, but straight boundaries are degree-one splines.
    let input = quarter(1., 1.).mapped(
        |p| *p,
        |c| match c {
            Curve::Line(line) => Curve::BSplineCurve((*line).into()),
            _ => c.clone(),
        },
        Clone::clone,
    );
    common::assert_solid(&input, std::f64::consts::PI / 4., &[0], 0.0001);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    let selected = setbacks(&input, &ids, 1., [0.2; 5], Matrix4::identity(), 0.2);
    if let Some(solid) = immutable_attempt(&input, &selected) {
        verify(&solid, 1., 1., [0.2; 5], Matrix4::identity());
    }
}

#[test]
fn curved_corner_contact_limits() {
    let input = quarter(1., 1.);
    let ids = selection(&input, Point3::new(1., 0., 1.));
    let cy = 0.2 * (1. - 0.2_f64.powi(2) / 4.).sqrt();
    let cz = 1. - 0.2_f64.powi(2) / 2.;
    let rz = (0.8_f64.powi(2) - 0.2_f64.powi(2)).sqrt();
    for delta in [-1e-4, 0., 1e-4] {
        for (name, distances) in [
            ("cy-side", [0.2, 0.2, 0.2, 0.2, cy + delta]),
            ("cz-height", [0.2, 0.2, 1. - cz + delta, 0.2, 0.2]),
            ("rz-height", [0.2, 0.2, 1. - rz + delta, 0.2, 0.2]),
        ] {
            eprintln!("{name} {delta}");
            let selected = setbacks(&input, &ids, 1., distances, Matrix4::identity(), 0.2);
            let output = immutable_attempt(&input, &selected);
            if delta != 0. {
                assert_eq!(
                    output.is_some(),
                    (name == "cy-side" && delta > 0.) || (name == "rz-height" && delta < 0.)
                );
            }
            if let Some(solid) = output {
                verify(&solid, 1., 1., distances, Matrix4::identity());
            }
        }
    }
}

#[test]
fn curved_corner_parabolic_transition() {
    let cy = 0.05 * (1. - 0.05_f64.powi(2) / 4.).sqrt();
    let cz = 1. - 0.05_f64.powi(2) / 2.;
    // A cone section is parabolic when the plane normal is perpendicular to a generator.
    let discriminant = |side: f64| {
        let nx = cy * (0.9 - (0.9_f64.powi(2) - side * side).sqrt()) + side * (cz - 0.9);
        nx * nx - 0.1_f64.powi(2) * ((cz - 0.9).powi(2) + cy * cy)
    };
    let (mut lo, mut hi) = (0.1, 0.2);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.;
        if discriminant(mid) < 0. {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let transition = (lo + hi) / 2.;
    assert!(discriminant(transition).abs() < 1e-18);
    for length in [1., 100., 10000., 1e6] {
        let input = quarter(1., length);
        let ids = selection(&input, Point3::new(length, 0., 1.));
        for delta in [-1e-4, 0., 1e-4] {
            eprintln!("parabola {delta}, length {length}");
            let distances = [0.2, 0.1, 0.1, 0.05, transition + delta];
            let selected = setbacks(&input, &ids, 1., distances, Matrix4::identity(), 0.2);
            let output = immutable_attempt(&input, &selected);
            // Long sectors are currently rejected conservatively, despite valid local contacts.
            assert_eq!(output.is_some(), length <= 100. && delta <= 0.);
            if let Some(solid) = output {
                verify(&solid, 1., length, distances, Matrix4::identity());
            }
        }
    }
}

#[test]
fn curved_corner_other_cap_radial_corner_and_aspect_ratio() {
    let input = quarter(0.5, 8.);
    for opposite_cap in [false, true] {
        for other_radial in [false, true] {
            let inverse = Matrix4::from_cols(
                (Vector3::unit_x() * if opposite_cap { -1. } else { 1. }).extend(0.),
                if other_radial {
                    Vector3::unit_z()
                } else {
                    Vector3::unit_y()
                }
                .extend(0.),
                if other_radial {
                    Vector3::unit_y()
                } else {
                    Vector3::unit_z()
                }
                .extend(0.),
                Point3::new(if opposite_cap { 8. } else { 0. }, 0., 0.).to_homogeneous(),
            );
            let corner = inverse.transform_point(Point3::new(8., 0., 0.5));
            let ids = selection(&input, corner);
            assert_eq!(ids.len(), 3);
            let selected = setbacks(&input, &ids, 0.5, [0.1; 5], inverse, 0.1);
            let solid = immutable_attempt(&input, &selected).expect("all four radial/cap corners");
            verify(&solid, 0.5, 8., [0.1; 5], inverse);
        }
    }
    check(
        4.,
        0.5,
        [0.1, 0.8, 0.8, 0.8, 0.8],
        Matrix4::identity(),
        false,
    );
}

#[test]
fn curved_corner_parameter_guard_scales() {
    for scale in [0.01, 0.1, 1., 10., 100.] {
        eprintln!("scale {scale}");
        let input = quarter(scale, scale);
        let ids = selection(&input, Point3::new(scale, 0., scale));
        let distances = [0.2 * scale; 5];
        let selected = setbacks(
            &input,
            &ids,
            scale,
            distances,
            Matrix4::identity(),
            distances[0],
        );
        let solid = immutable_attempt(&input, &selected).expect("uniformly scaled corner");
        verify(&solid, scale, scale, distances, Matrix4::identity());
    }
}
