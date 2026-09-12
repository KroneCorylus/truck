use super::*;
use crate::alternative::Alternative;
use shell::ShellCondition;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use truck_geometry::prelude::*;
use truck_topology::Vertex;
const TOL: f64 = 0.05;

type Curve<S> = AltCurve<BSplineCurve<Point3>, S>;

fn line<S>(v0: &Vertex<Point3>, v1: &Vertex<Point3>) -> Edge<Point3, Curve<S>> {
    let curve = BSplineCurve::new(KnotVec::bezier_knot(1), vec![v0.point(), v1.point()]);
    Edge::new(v0, v1, Alternative::FirstType(curve))
}

fn parabola<S>(v0: &Vertex<Point3>, v1: &Vertex<Point3>, pt: Point3) -> Edge<Point3, Curve<S>> {
    let curve = BSplineCurve::new(KnotVec::bezier_knot(2), vec![v0.point(), pt, v1.point()]);
    Edge::new(v0, v1, Alternative::FirstType(curve))
}

#[test]
fn divide_plane_test() {
    let v = Vertex::news([
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 4.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
        Point3::new(-1.0, 3.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 3.0, 0.0),
    ]);
    let edge: [Edge<Point3, Curve<Plane>>; 7] = [
        parabola(&v[0], &v[1], Point3::new(-4.0, 2.0, 0.0)),
        parabola(&v[0], &v[1], Point3::new(4.0, 2.0, 0.0)),
        line(&v[0], &v[1]),
        parabola(&v[2], &v[3], Point3::new(-3.0, 2.0, 0.0)),
        parabola(&v[2], &v[3], Point3::new(-1.0, 2.0, 0.0)),
        parabola(&v[4], &v[5], Point3::new(1.0, 2.0, 0.0)),
        parabola(&v[4], &v[5], Point3::new(3.0, 2.0, 0.0)),
    ];
    let wire = [
        wire![edge[1].clone(), edge[0].inverse()],
        wire![edge[2].clone(), edge[0].inverse()],
        wire![edge[1].clone(), edge[2].inverse()],
        wire![edge[3].clone(), edge[4].inverse()],
        wire![edge[5].clone(), edge[6].inverse()],
    ];
    let face = Face::new(
        vec![wire[0].clone(), wire[3].clone(), wire[4].clone()],
        Plane::new(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ),
    );
    let loops: Loops<_, _> = vec![
        BoundaryWire::new(wire[1].clone(), ShapesOpStatus::Or),
        BoundaryWire::new(wire[2].clone(), ShapesOpStatus::And),
        BoundaryWire::new(wire[3].clone(), ShapesOpStatus::Unknown),
        BoundaryWire::new(wire[4].clone(), ShapesOpStatus::Unknown),
    ]
    .into_iter()
    .collect();
    let res = divide_one_face(&face, &loops, 0.01).unwrap();
    assert_eq!(res.len(), 2);
    let (mut or, mut and) = (true, true);
    for (face, status) in res {
        let bdd = face.absolute_boundaries();
        match status {
            ShapesOpStatus::Or => {
                assert_eq!(bdd.len(), 2);
                assert!(bdd[0] == wire[1] || bdd[0] == wire[3]);
                assert!(bdd[1] == wire[1] || bdd[1] == wire[3]);
                assert_ne!(bdd[0], bdd[1]);
                assert!(or);
                or = false;
            }
            ShapesOpStatus::And => {
                assert_eq!(bdd.len(), 2);
                assert!(bdd[0] == wire[2] || bdd[0] == wire[4]);
                assert!(bdd[1] == wire[2] || bdd[1] == wire[4]);
                assert_ne!(bdd[0], bdd[1]);
                assert!(and);
                and = false;
            }
            _ => panic!("There must be no unknown!"),
        }
    }
}

type AlternativeIntersection = Alternative<
    NurbsCurve<Vector4>,
    IntersectionCurve<PolylineCurve<Point3>, AlternativeSurface, AlternativeSurface>,
>;
type AlternativeSurface = Alternative<BSplineSurface<Point3>, Plane>;

crate::impl_from!(
    NurbsCurve<Vector4>,
    IntersectionCurve<PolylineCurve<Point3>, AlternativeSurface, AlternativeSurface>
);
crate::impl_from!(BSplineSurface<Point3>, Plane);

fn parabola_surfaces() -> (AlternativeSurface, AlternativeSurface) {
    // define surfaces
    #[rustfmt::skip]
	let ctrl0 = vec![
		vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
		vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
		vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
	];
    #[rustfmt::skip]
	let ctrl1 = vec![
		vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
		vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
		vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
	];
    (
        BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0).into(),
        BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1).into(),
    )
}

#[test]
fn independent_intersection() {
    // prepare geoetries
    let arc00: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(1.0, 0.0, 1.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-1.0, 0.0, 1.0, 1.0),
        ],
    ))
    .into();
    let arc01: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(-1.0, 0.0, 1.0, 1.0),
            Vector4::new(0.0, -1.0, 0.0, 0.0),
            Vector4::new(1.0, 0.0, 1.0, 1.0),
        ],
    ))
    .into();
    let arc10: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(1.0, 0.0, -1.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-1.0, 0.0, -1.0, 1.0),
        ],
    ))
    .into();
    let arc11: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(-1.0, 0.0, -1.0, 1.0),
            Vector4::new(0.0, -1.0, 0.0, 0.0),
            Vector4::new(1.0, 0.0, -1.0, 1.0),
        ],
    ))
    .into();
    let (surface0, surface1) = parabola_surfaces();
    let plane0: AlternativeSurface = Plane::new(
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    )
    .into();
    let plane1: AlternativeSurface = Plane::new(
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(1.0, 0.0, -1.0),
        Point3::new(0.0, 1.0, -1.0),
    )
    .into();

    // prepare topologies
    let v00 = Vertex::new(Point3::new(1.0, 0.0, 1.0));
    let v01 = Vertex::new(Point3::new(-1.0, 0.0, 1.0));
    let v10 = Vertex::new(Point3::new(1.0, 0.0, -1.0));
    let v11 = Vertex::new(Point3::new(-1.0, 0.0, -1.0));
    let wire0: Wire<_, _> = vec![Edge::new(&v00, &v01, arc00), Edge::new(&v01, &v00, arc01)].into();
    let wire1: Wire<_, _> = vec![Edge::new(&v10, &v11, arc10), Edge::new(&v11, &v10, arc11)].into();
    let shell0: Shell<_, _, _> = vec![
        Face::new(vec![wire0.clone()], plane0),
        Face::new(vec![wire0], surface0).inverse(),
    ]
    .into();
    assert_eq!(shell0.shell_condition(), ShellCondition::Closed);
    let shell1: Shell<_, _, _> = vec![
        Face::new(vec![wire1.clone()], plane1).inverse(),
        Face::new(vec![wire1], surface1),
    ]
    .into();
    assert_eq!(shell1.shell_condition(), ShellCondition::Closed);
    let poly_shell0 = shell0.triangulation(TOL);
    let poly_shell1 = shell1.triangulation(TOL);

    let LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = create_loops_stores(&shell0, &poly_shell0, &shell1, &poly_shell1, TOL).unwrap();
    let [and0, or0, unknown0] = divide_faces(&shell0, &loops_store0, TOL)
        .unwrap()
        .and_or_unknown();
    let [and1, or1, unknown1] = divide_faces(&shell1, &loops_store1, TOL)
        .unwrap()
        .and_or_unknown();
    assert_eq!(and0.len(), 1);
    assert_eq!(or0.len(), 1);
    assert_eq!(unknown0.len(), 1);
    assert_eq!(and1.len(), 1);
    assert_eq!(or1.len(), 1);
    assert_eq!(unknown1.len(), 1);

    match unknown0[0].surface() {
        AlternativeSurface::FirstType(_) => panic!("This face must plane!"),
        AlternativeSurface::SecondType(_) => {}
    }
    match unknown1[0].surface() {
        AlternativeSurface::FirstType(_) => panic!("This face must plane!"),
        AlternativeSurface::SecondType(_) => {}
    }

    let and_shell: Shell<_, _, _> = vec![and0[0].clone(), and1[0].clone()].into();
    assert_eq!(and_shell.shell_condition(), ShellCondition::Closed);
}

/// A plane counting its nearest-point searches, which every exact intersection-curve
/// evaluation starts with.
#[derive(Clone, Debug)]
struct CountedPlane {
    plane: Plane,
    nearest: Arc<AtomicUsize>,
}

impl ParametricSurface for CountedPlane {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64, v: f64) -> Point3 { self.plane.subs(u, v) }
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.plane.uder(u, v) }
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.plane.vder(u, v) }
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.plane.uuder(u, v) }
    fn uvder(&self, u: f64, v: f64) -> Vector3 { self.plane.uvder(u, v) }
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.plane.vvder(u, v) }
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        self.plane.der_mn(m, n, u, v)
    }
}
impl ParametricSurface3D for CountedPlane {}
impl Invertible for CountedPlane {
    fn invert(&mut self) { self.plane.invert() }
}
impl SearchParameter<D2> for CountedPlane {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        self.plane.search_parameter(point, hint, trials)
    }
}
impl SearchNearestParameter<D2> for CountedPlane {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        self.nearest.fetch_add(1, Ordering::Relaxed);
        self.plane.search_nearest_parameter(point, hint, trials)
    }
}

#[test]
fn division_traces_cut_edges_without_solving_them() {
    let nearest = Arc::new(AtomicUsize::new(0));
    let counted = |plane| CountedPlane {
        plane,
        nearest: nearest.clone(),
    };
    let floor = counted(Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ));
    let wall = counted(Plane::new(
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 0.0, 1.0),
    ));
    let v = Vertex::news([
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ]);
    let edge: Vec<_> = (0..6).map(|i| line(&v[i], &v[(i + 1) % 6])).collect();
    let leader = PolylineCurve(vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 0.5, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ]);
    let cut = Edge::new(
        &v[1],
        &v[4],
        Alternative::SecondType(IntersectionCurve::new(floor.clone(), wall, leader)),
    );
    let face = Face::new(vec![edge.iter().cloned().collect()], floor);
    let left = wire![
        edge[0].clone(),
        cut.clone(),
        edge[4].clone(),
        edge[5].clone()
    ];
    let right = wire![
        edge[1].clone(),
        edge[2].clone(),
        edge[3].clone(),
        cut.inverse()
    ];
    let loops: Loops<_, _> = vec![
        BoundaryWire::new(left.clone(), ShapesOpStatus::And),
        BoundaryWire::new(right.clone(), ShapesOpStatus::Or),
    ]
    .into_iter()
    .collect();

    let res = divide_one_face(&face, &loops, 0.01).unwrap();

    assert_eq!(nearest.load(Ordering::Relaxed), 0);
    assert_eq!(res.len(), 2);
    for (face, status) in res {
        let expected = match status {
            ShapesOpStatus::And => &left,
            ShapesOpStatus::Or => &right,
            _ => panic!("every piece is decided by its cut"),
        };
        assert_eq!(
            &face.absolute_boundaries()[..],
            std::slice::from_ref(expected)
        );
    }
}
