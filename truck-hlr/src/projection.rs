use crate::Curve2;
use truck_modeling::*;

/// The affine map of a parallel projection onto a plane, in the plane's coordinates.
pub(crate) struct Projection {
    origin: Point3,
    rows: [Vector3; 2],
}

impl Projection {
    /// # Panics
    ///
    /// `direction` must not lie in `plane`.
    pub fn new(plane: &Plane, direction: Vector3) -> Self {
        let normal = plane.normal();
        let to_plane_coords = Matrix3::from_cols(plane.u_axis(), plane.v_axis(), normal)
            .invert()
            .expect("degenerate plane");
        let slope = to_plane_coords * direction;
        assert!(
            !slope.z.so_small(),
            "the view direction lies in the projection plane"
        );
        let rows = [
            to_plane_coords.row(0) - to_plane_coords.row(2) * (slope.x / slope.z),
            to_plane_coords.row(1) - to_plane_coords.row(2) * (slope.y / slope.z),
        ];
        Self {
            origin: plane.origin(),
            rows,
        }
    }

    pub fn point(&self, point: Point3) -> Point2 {
        Point2::from_vec(self.vector(point - self.origin))
    }

    pub fn vector(&self, vector: Vector3) -> Vector2 {
        Vector2::new(self.rows[0].dot(vector), self.rows[1].dot(vector))
    }

    /// The projection of `curve`, `None` when it fits in a circle of radius `tol`.
    pub fn curve(&self, curve: &Curve, tol: f64) -> Option<Curve2> {
        let (params, points) = curve.parameter_division(curve.range_tuple(), tol);
        let points: Vec<Point2> = points.into_iter().map(|p| self.point(p)).collect();
        if points.iter().all(|p| p.distance(points[0]) < tol) {
            return None;
        }
        let projected = match curve {
            Curve::Line(Line(p, q)) => Line(self.point(*p), self.point(*q)).into(),
            Curve::BSplineCurve(curve) => {
                let control_points = curve.control_points().iter().map(|p| self.point(*p));
                BSplineCurve::new(curve.knot_vec().clone(), control_points.collect()).into()
            }
            Curve::NurbsCurve(curve) => {
                let control_points = curve.control_points().iter().map(|p| {
                    let weighted = p.truncate() - self.origin.to_vec() * p.w;
                    self.vector(weighted).extend(p.w)
                });
                let curve = BSplineCurve::new(curve.knot_vec().clone(), control_points.collect());
                NurbsCurve::new(curve).into()
            }
            Curve::Conic(conic) => {
                let matrix = conic.transform();
                let axis = |i: usize| self.vector(matrix[i].truncate()).extend(0.0);
                let center = self.point(Point3::from_homogeneous(matrix[3]));
                let transform = Matrix3::from_cols(axis(0), axis(1), center.to_vec().extend(1.0));
                let unit_arc = TrimmedCurve::new(UnitCircle::<Point2>::new(), conic.range_tuple());
                let mut arc = Processor::with_transform(unit_arc, transform);
                if !conic.orientation() {
                    arc.invert();
                }
                arc.into()
            }
            Curve::IntersectionCurve(_) | Curve::PCurve(_) => interpolate(params, points).into(),
        };
        Some(projected)
    }
}

/// A cubic B-spline through `points` at `params`, with the knots averaged from the parameters.
fn interpolate(params: Vec<f64>, points: Vec<Point2>) -> BSplineCurve<Point2> {
    let n = points.len();
    let degree = 3.min(n - 1);
    let mut knots = vec![params[0]; degree + 1];
    knots
        .extend((1..n - degree).map(|j| params[j..j + degree].iter().sum::<f64>() / degree as f64));
    knots.extend(vec![params[n - 1]; degree + 1]);
    let parameter_points: Vec<_> = params.into_iter().zip(points).collect();
    BSplineCurve::interpole(KnotVec::from(knots), parameter_points)
}
