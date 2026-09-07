use truck_meshalgo::prelude::*;

/// Whether the ray from `point` along `toward` meets a triangle of `mesh` ahead of `point`.
///
/// Triangles the ray only grazes, lying in a plane through the ray, do not count. A hit on the
/// edge or vertex of a triangle does, so a ray leaving a box exactly along the border of a face
/// is still blocked.
pub(crate) fn blocked(mesh: &PolygonMesh, point: Point3, toward: Vector3) -> bool {
    const SLACK: f64 = 1.0e-9;
    let positions = mesh.positions();
    mesh.faces().triangle_iter().any(|triangle| {
        let [p0, p1, p2] = [
            positions[triangle[0].pos],
            positions[triangle[1].pos],
            positions[triangle[2].pos],
        ];
        let matrix = Matrix3::from_cols(p1 - p0, p2 - p0, -toward);
        if matrix.determinant().so_small() {
            return false;
        }
        let uvt = matrix.invert().unwrap() * (point - p0);
        uvt.x >= -SLACK && uvt.y >= -SLACK && uvt.x + uvt.y <= 1.0 + SLACK && uvt.z > TOLERANCE
    })
}
