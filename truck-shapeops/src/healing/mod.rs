use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_geometry::prelude::*;
use truck_meshalgo::rexport_polymesh::*;
use truck_topology::compress::*;

type Edge<C> = CompressedEdge<C>;
type EdgeIndex = CompressedEdgeIndex;
type Wire = Vec<EdgeIndex>;
type Face<S> = CompressedFace<S>;
type Shell<P, C, S> = CompressedShell<P, C, S>;

trait SP<S>: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}
impl<S, F> SP<S> for F where F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}

mod split_closed_edges;
use split_closed_edges::split_closed_edges;

mod split_closed_faces;
use split_closed_faces::split_closed_faces;

fn connect_coincident_wire_ends<C, S>(shell: &mut Shell<Point3, C, S>) {
    fn root(parents: &[usize], mut i: usize) -> usize {
        while parents[i] != i {
            i = parents[i];
        }
        i
    }
    let mut parents: Vec<_> = (0..shell.vertices.len()).collect();
    for boundary in shell.faces.iter().flat_map(|face| &face.boundaries) {
        for (a, b) in boundary
            .iter()
            .zip(boundary.iter().cycle().skip(1))
            .take(boundary.len())
        {
            let edge_a = &shell.edges[a.index];
            let edge_b = &shell.edges[b.index];
            let end = if a.orientation {
                edge_a.vertices.1
            } else {
                edge_a.vertices.0
            };
            let start = if b.orientation {
                edge_b.vertices.0
            } else {
                edge_b.vertices.1
            };
            if end != start && shell.vertices[end] == shell.vertices[start] {
                let (a, b) = (root(&parents, end), root(&parents, start));
                parents[a.max(b)] = a.min(b);
            }
        }
    }
    for edge in &mut shell.edges {
        edge.vertices = (
            root(&parents, edge.vertices.0),
            root(&parents, edge.vertices.1),
        );
    }
}

/// Splits closed edges and faces
///
/// # Details
/// The topology of the shapes handled by truck has the following rules
/// - The endpoints of the edges must be different.
/// - The boundaries of the faces must be a simple wire.
///
/// Shapes created in other CAD systems do not necessarily follow these rules.
/// When such shapes are handled by truck, this method is applied at the stage
/// of `CompressedShell` and `CompressedSolid`, which are intermediate forms.
///
/// # Remarks
/// Boundary reconstruction handles regular periodic seams and isolated retraced seams ending
/// at a surface pole. More general singular or intersecting boundaries may remain unsupported.
pub trait SplitClosedEdgesAndFaces {
    /// Splits closed edges and faces
    fn split_closed_edges_and_faces(&mut self, tol: f64);
}

impl<C, S> SplitClosedEdgesAndFaces for CompressedShell<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    fn split_closed_edges_and_faces(&mut self, tol: f64) {
        fn sp<S>(surface: &S, point: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)>
        where S: SearchParameter<D2, Point = Point3> {
            surface.search_parameter(point, hint, 100)
        }
        connect_coincident_wire_ends(self);
        split_closed_edges(self);
        split_closed_faces(self, tol, sp);
    }
}

impl<C, S> SplitClosedEdgesAndFaces for CompressedSolid<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    fn split_closed_edges_and_faces(&mut self, tol: f64) {
        self.boundaries
            .iter_mut()
            .for_each(|shell| shell.split_closed_edges_and_faces(tol))
    }
}

/// robust version of splitting closed edges and faces.
///
/// # Details
/// Robust version of [`SplitClosedEdgesAndFaces`] based on [`SearchNearestParameter`].
pub trait RobustSplitClosedEdgesAndFaces {
    /// Splits closed edges and faces
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64);
}

impl<C, S> RobustSplitClosedEdgesAndFaces for CompressedShell<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64) {
        fn sp<S>(surface: &S, point: Point3, hint: Option<(f64, f64)>) -> Option<(f64, f64)>
        where S: SearchParameter<D2, Point = Point3> + SearchNearestParameter<D2, Point = Point3>
        {
            surface
                .search_parameter(point, hint, 100)
                .or_else(|| surface.search_parameter(point, None, 100))
                .or_else(|| surface.search_nearest_parameter(point, hint, 100))
                .or_else(|| surface.search_nearest_parameter(point, None, 100))
        }
        connect_coincident_wire_ends(self);
        split_closed_edges(self);
        split_closed_faces(self, tol, sp);
    }
}

impl<C, S> RobustSplitClosedEdgesAndFaces for CompressedSolid<Point3, C, S>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    fn robust_split_closed_edges_and_faces(&mut self, tol: f64) {
        let fs = RobustSplitClosedEdgesAndFaces::robust_split_closed_edges_and_faces;
        self.boundaries.iter_mut().for_each(|shell| fs(shell, tol))
    }
}

#[cfg(test)]
mod tests;
