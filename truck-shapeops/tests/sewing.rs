mod common;
use common::{assert_solid, blend::from_modeling};
use truck_modeling::*;
fn separated_box(missing: bool) -> CompressedShell {
    let v = builder::vertex(Point3::origin());
    let e = builder::tsweep(&v, Vector3::unit_x());
    let face = builder::tsweep(&e, Vector3::unit_y());
    let solid = builder::tsweep(&face, Vector3::unit_z());
    let mut result = CompressedShell {
        vertices: Vec::new(),
        edges: Vec::new(),
        faces: Vec::new(),
    };
    for (i, face) in solid.face_iter().enumerate() {
        if missing && i == 0 {
            continue;
        }
        let face = if i == 1 {
            builder::translated(face, Vector3::new(0.000005, 0.0, 0.0))
        } else {
            face.clone()
        };
        let shell = Shell::from(vec![face]).compress();
        let vertices = result.vertices.len();
        let edges = result.edges.len();
        result.vertices.extend(shell.vertices);
        result.edges.extend(shell.edges.into_iter().map(|mut e| {
            e.vertices.0 += vertices;
            e.vertices.1 += vertices;
            e
        }));
        result.faces.extend(shell.faces.into_iter().map(|mut f| {
            for e in f.boundaries.iter_mut().flatten() {
                e.index += edges;
            }
            f
        }));
    }
    result
}
#[test]
fn tolerance_sewing_joins_near_seams_and_preserves_missing_faces() {
    let input = separated_box(false);
    let before = serde_json::to_string(&input).unwrap();
    let sewn = truck_shapeops::sew_shell(&input, 0.00002).unwrap();
    assert_eq!(serde_json::to_string(&input).unwrap(), before);
    let shell = Shell::extract(sewn).unwrap();
    let solid = Solid::try_new(vec![shell]).unwrap();
    let replacements = solid
        .face_iter()
        .map(|face| (face.id(), face.oriented_surface()))
        .collect::<Vec<_>>();
    let solid = truck_shapeops::local::replace_surfaces(&solid, &replacements).unwrap();
    assert_solid(&from_modeling(&solid), 1.0, &[0], 1e-4);
    let missing = truck_shapeops::sew_shell(&separated_box(true), 0.00002).unwrap();
    let shell = Shell::extract(missing).unwrap();
    assert!(!shell.extract_boundaries().is_empty());
    assert!(Solid::try_new(vec![shell]).is_err());
    assert!(truck_shapeops::sew_shell(&input, 0.0).is_none());
    let separate = truck_shapeops::sew_shell(&input, 0.000001).unwrap();
    let shell = Shell::extract(separate).unwrap();
    assert!(!shell.extract_boundaries().is_empty());
}
