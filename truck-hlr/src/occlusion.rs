use truck_meshalgo::prelude::*;
use truck_modeling::*;

/// `|n · e|` below which the ray runs along a face of an edge.
const GRAZE: f64 = 1.0e-3;
/// Bending below which a face is straight along the ray.
const FLAT: f64 = 1.0e-6;

/// A face the tested point lies on: its surface and whether the face keeps its orientation.
pub(crate) struct Support<'a> {
    pub surface: &'a Surface,
    pub orientation: bool,
}

/// Whether the body hides `point`, which lies on `faces`, from a viewer in direction `toward`.
///
/// A face the ray grazes is decided by how the face bends along the ray. The faces of a
/// silhouette piece are all grazed; a face of an edge is grazed when its normal at the point
/// is within `GRAZE` of perpendicular to the ray. Bending towards the outside of the face puts
/// the ray inside the body: hidden. Bending away leaves the ray outside; the point is moved by
/// `2 tol` out of the body along the normal, past the chord sag of the tessellation, which
/// overhangs the exact surface where the face is concave across the ray, and the ray test
/// against `mesh` decides. A face straight along the ray, a plane or a ruling, decides nothing,
/// and the point is moved by `2 tol` into the body instead, so the far side of the body
/// answers. A silhouette point is first put back onto its surface, since its curve is only
/// fitted.
pub(crate) fn hidden(
    mesh: &Occluders,
    point: Point3,
    toward: Vector3,
    faces: &[Support<'_>],
    silhouette: bool,
    tol: f64,
) -> bool {
    let mut origin = point;
    let mut nudge = Vector3::zero();
    for face in faces {
        let Some((u, v)) = face
            .surface
            .search_nearest_parameter(point, SPHint2D::None, 100)
        else {
            continue;
        };
        let sign = if face.orientation { 1.0 } else { -1.0 };
        let normal = face.surface.normal(u, v) * sign;
        if silhouette {
            origin = face.surface.subs(u, v);
        } else if normal.dot(toward).abs() >= GRAZE {
            continue;
        }
        let bend = bending(face.surface, u, v, normal, toward);
        if bend > FLAT {
            return true;
        }
        let outward = if bend < -FLAT { 1.0 } else { -1.0 };
        nudge += normal * (outward * 2.0 * tol);
    }
    mesh.blocked(origin + nudge)
}

/// The second derivative of the surface along the tangent direction `e` at `(u, v)`, signed
/// along `normal`: positive bends towards the normal.
fn bending(surface: &Surface, u: f64, v: f64, normal: Vector3, e: Vector3) -> f64 {
    let (su, sv) = (surface.uder(u, v), surface.vder(u, v));
    let gram = Matrix2::new(su.dot(su), su.dot(sv), su.dot(sv), sv.dot(sv));
    let Some(inverse) = gram.invert() else {
        return 0.0;
    };
    let ab = inverse * Vector2::new(su.dot(e), sv.dot(e));
    let (a, b) = (ab.x, ab.y);
    let curvature = surface.uuder(u, v) * (a * a)
        + surface.uvder(u, v) * (2.0 * a * b)
        + surface.vvder(u, v) * (b * b);
    curvature.dot(normal)
}

/// Whether the ray from `point` along `toward` meets a triangle of `mesh` ahead of `point`.
///
/// Triangles the ray only grazes, lying in a plane through the ray, do not count. A hit on the
/// edge or vertex of a triangle does, so a ray leaving a box exactly along the border of a face
/// is still blocked.
pub(crate) struct Occluders {
    triangles: Vec<(Point3, Matrix3, [Point2; 2])>,
    axes: [Vector3; 2],
}

impl Occluders {
    pub(crate) fn new(mesh: &PolygonMesh, toward: Vector3) -> Self {
        let helper = if toward.x.abs() < 0.9 {
            Vector3::unit_x()
        } else {
            Vector3::unit_y()
        };
        let u = helper.cross(toward).normalize();
        let axes = [u, toward.cross(u)];
        let positions = mesh.positions();
        let triangles = mesh
            .faces()
            .triangle_iter()
            .filter_map(|triangle| {
                let [p0, p1, p2] = triangle.map(|vertex| positions[vertex.pos]);
                let matrix = Matrix3::from_cols(p1 - p0, p2 - p0, -toward);
                if matrix.determinant().so_small() {
                    return None;
                }
                let points = [p0, p1, p2].map(|point| {
                    Point2::new(axes[0].dot(point.to_vec()), axes[1].dot(point.to_vec()))
                });
                let bounds: BoundingBox<Point2> = points.into_iter().collect();
                // Include the barycentric slack and roundoff in the projected bounding box.
                let extent = (p1 - p0).magnitude() + (p2 - p0).magnitude();
                let roundoff = points
                    .iter()
                    .flat_map(|point| [point.x.abs(), point.y.abs()])
                    .fold(1.0, f64::max)
                    * (32.0 * f64::EPSILON);
                let pad = Vector2::new(1.0, 1.0) * (extent * 2.0e-9 + roundoff);
                Some((
                    p0,
                    matrix.invert().unwrap(),
                    [bounds.min() - pad, bounds.max() + pad],
                ))
            })
            .collect();
        Self { triangles, axes }
    }

    pub(crate) fn blocked(&self, point: Point3) -> bool {
        const SLACK: f64 = 1.0e-9;
        let projected = Point2::new(
            self.axes[0].dot(point.to_vec()),
            self.axes[1].dot(point.to_vec()),
        );
        self.triangles.iter().any(|(origin, inverse, [min, max])| {
            if projected.x < min.x
                || projected.y < min.y
                || projected.x > max.x
                || projected.y > max.y
            {
                return false;
            }
            let uvt = inverse * (point - origin);
            uvt.x >= -SLACK && uvt.y >= -SLACK && uvt.x + uvt.y <= 1.0 + SLACK && uvt.z > TOLERANCE
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(mesh: &PolygonMesh, point: Point3, toward: Vector3) -> bool {
        mesh.faces().triangle_iter().any(|triangle| {
            let [p0, p1, p2] = triangle.map(|vertex| mesh.positions()[vertex.pos]);
            let matrix = Matrix3::from_cols(p1 - p0, p2 - p0, -toward);
            if matrix.determinant().so_small() {
                return false;
            }
            let uvt = matrix.invert().unwrap() * (point - p0);
            uvt.x >= -1.0e-9
                && uvt.y >= -1.0e-9
                && uvt.x + uvt.y <= 1.0 + 1.0e-9
                && uvt.z > TOLERANCE
        })
    }

    #[test]
    fn prepared_rays_match_direct_tests_at_interiors_edges_and_grazing_angles() {
        let solid: Solid = primitive::cuboid(BoundingBox::from_iter([
            Point3::origin(),
            Point3::new(2.0, 2.0, 2.0),
        ]));
        let mesh = solid.triangulation(0.01).to_polygon();
        for toward in [
            Vector3::unit_x(),
            Vector3::unit_y(),
            Vector3::unit_z(),
            Vector3::new(1.0, 2.0, 3.0).normalize(),
            Vector3::new(1.0, 1.0e-8, 0.0).normalize(),
        ] {
            let prepared = Occluders::new(&mesh, toward);
            for x in [-1.0, 0.0, 1.0e-10, 1.0, 2.0, 3.0] {
                for y in [-1.0, 0.0, 1.0, 2.0, 3.0] {
                    for z in [-1.0, 0.0, 1.0, 2.0, 3.0] {
                        let point = Point3::new(x, y, z);
                        assert_eq!(
                            prepared.blocked(point),
                            reference(&mesh, point, toward),
                            "{point:?} {toward:?}"
                        );
                    }
                }
            }
        }
    }
}
