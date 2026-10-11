mod common;

use std::collections::HashMap;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_shapeops::fillet::*;

const TOL: f64 = 0.001;

fn pyramid(count: usize, apothem: f64, height: f64) -> Solid {
    let radius = apothem / (PI / count as f64).cos();
    let mut points: Vec<_> = (0..count)
        .map(|i| {
            let angle = 2. * PI * i as f64 / count as f64;
            Point3::new(radius * angle.cos(), radius * angle.sin(), 0.)
        })
        .collect();
    points.push(Point3::new(0., 0., height));
    let vertices = builder::vertices(points);
    let mut edges = HashMap::new();
    let faces = std::iter::once((0..count).rev().collect::<Vec<_>>())
        .chain((0..count).map(|i| vec![i, (i + 1) % count, count]));
    let shell = faces
        .map(|indices| {
            let wire: Wire = (0..indices.len())
                .map(|i| {
                    let (a, b) = (indices[i], indices[(i + 1) % indices.len()]);
                    let edge = edges
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
        })
        .collect();
    Solid::new(vec![shell])
}

fn volume(count: usize, apothem: f64, height: f64, radius: f64) -> f64 {
    // Divergence theorem about the rolling-ball center: p.n = r on every
    // non-base face. Integrate the cylindrical normal arcs and spherical polygon
    // analytically; their vertical flux gives the area of the clipped base.
    let n = count as f64;
    let alpha = PI / n;
    let slope = apothem / height;
    let hypotenuse = (1. + slope * slope).sqrt();
    let nz = slope / hypotenuse;
    let axis_z = 1. / (1. + (slope / alpha.cos()).powi(2)).sqrt();
    let beta = ((2. * alpha).cos() / hypotenuse.powi(2) + nz * nz).acos();
    let center_z = height - radius / nz;
    let contact_z = center_z + radius * nz;
    let planar_area = n * contact_z.powi(2) * slope * alpha.tan() * hypotenuse;
    let middle_z = nz / (beta / 2.).cos();
    let integral_z = 2. * middle_z * (beta / 2.).sin();
    let integral_z2 = middle_z.powi(2) * (beta + beta.sin()) / 2.;
    let cylinder_area = n * radius / axis_z * (center_z * beta + radius * integral_z);
    let face_angle = 2. * (slope * alpha.tan() * axis_z).asin();
    let sphere_area = radius.powi(2) * (2. * PI - n * face_angle);
    let base_area = nz * planar_area
        + n * radius / axis_z * (center_z * integral_z + radius * integral_z2)
        + radius.powi(2) * n * axis_z * beta / 2.;
    (radius * (planar_area + cylinder_area + sphere_area) + center_z * base_area) / 3.
}

fn snapshot(solid: &Solid) -> String { serde_json::to_string(&solid.compress()).unwrap() }

#[test]
fn equal_radius_fillets_share_high_valence_spherical_corners() {
    let transforms = [
        Matrix4::identity(),
        Matrix4::from_translation(Vector3::new(13., -7., 23.))
            * Matrix4::from_axis_angle(Vector3::new(1., 2., 3.).normalize(), Rad(0.73)),
    ];
    for count in [4, 5, 6, 8] {
        for transform in transforms {
            let input = builder::transformed(&pyramid(count, 5., 10.), transform);
            let before = snapshot(&input);
            let apex = transform.transform_point(Point3::new(0., 0., 10.));
            let mut seen = std::collections::HashSet::new();
            let selected: Vec<_> = input
                .edge_iter()
                .filter(|edge| edge.front().point().near(&apex) || edge.back().point().near(&apex))
                .map(|edge| edge.id())
                .filter(|id| seen.insert(*id))
                .collect();
            assert_eq!(selected.len(), count);
            for radius in [0.2, 1.2] {
                let result = try_fillet_solid_edges(&input, &selected, radius, TOL)
                    .unwrap_or_else(|e| panic!("count={count}, radius={radius}: {e:?}"));
                let expected = volume(count, 5., 10., radius);
                assert!(result.solid.is_geometric_consistent());
                common::assert_solid(&result.solid, expected, &[0], TOL);
                assert_eq!(result.generated_faces.len(), count + 1);
                assert_eq!(result.modified_faces.len(), count + 1);
                let corner = result
                    .generated_faces
                    .iter()
                    .find(|face| matches!(face.surface(), Surface::Sphere(_)))
                    .expect("exact spherical corner");
                assert_eq!(corner.edge_iter().count(), count);
                let center =
                    transform.transform_point(Point3::new(0., 0., 10. - radius * 5.0_f64.sqrt()));
                for edge in corner.edge_iter() {
                    let neighbour = result
                        .solid
                        .face_iter()
                        .find(|face| {
                            face.id() != corner.id()
                                && face.edge_iter().any(|e| e.id() == edge.id())
                        })
                        .expect("adjacent cylindrical fillet");
                    let curve = edge.curve();
                    let (a, b) = curve.range_tuple();
                    for i in 0..=16 {
                        let p = curve.subs(a + (b - a) * i as f64 / 16.);
                        assert!((p.distance(center) - radius).abs() < 1e-8);
                        let normals = [corner, neighbour].map(|face| {
                            let surface = face.oriented_surface();
                            let (u, v) = surface.search_parameter(p, None, 100).unwrap();
                            surface.normal(u, v).normalize()
                        });
                        assert!(
                            normals[0].near(&normals[1]),
                            "G1 spherical/cylindrical contact"
                        );
                    }
                }
                let mut reversed = selected.clone();
                reversed.reverse();
                let again = try_fillet_solid_edges(&input, &reversed, radius, TOL).unwrap();
                assert_eq!(snapshot(&result.solid), snapshot(&again.solid));
                common::blend::assert_step(&result.solid, expected, TOL);
                if count == 4 && radius == 1.2 {
                    let cap_height = radius * 0.2;
                    let clip_z = 10. - radius * 5.0_f64.sqrt() + radius - cap_height;
                    let cutter = builder::transformed(
                        &common::modeling::cuboid(
                            Point3::new(-10., -10., -1.),
                            Point3::new(10., 10., clip_z),
                        ),
                        transform,
                    );
                    let cut = truck_shapeops::and(&result.solid, &cutter, TOL)
                        .expect("Boolean through the spherical corner");
                    let clipped_volume =
                        expected - PI * cap_height.powi(2) * (radius - cap_height / 3.);
                    common::assert_solid(&cut, clipped_volume, &[0], TOL);
                    common::blend::assert_step(&cut, clipped_volume, TOL);
                }
                assert_eq!(before, snapshot(&input));
            }
        }
    }
}

#[test]
fn incompatible_corner_offsets_and_excessive_radii_preserve_input() {
    use truck_base::diagnostics::Code;
    for (transform, radius, expected) in [
        (
            Matrix4::from_nonuniform_scale(1.3, 1., 1.) * Matrix4::from_angle_z(Rad(PI / 4.)),
            1.,
            Code::UnsupportedGeometry,
        ),
        (Matrix4::identity(), 4., Code::OutsideNeighbour),
    ] {
        let input = builder::transformed(&pyramid(4, 5., 10.), transform);
        let before = snapshot(&input);
        let apex = Point3::new(0., 0., 10.);
        let mut seen = std::collections::HashSet::new();
        let ids: Vec<_> = input
            .edge_iter()
            .filter(|edge| edge.front().point().near(&apex) || edge.back().point().near(&apex))
            .map(|edge| edge.id())
            .filter(|id| seen.insert(*id))
            .collect();
        assert_eq!(
            try_fillet_solid_edges(&input, &ids, radius, TOL)
                .unwrap_err()
                .code,
            expected
        );
        assert_eq!(snapshot(&input), before);
    }
}
