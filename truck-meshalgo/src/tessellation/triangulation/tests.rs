use super::*;
use proptest::prelude::*;

fn polygon(points: &[[f64; 2]]) -> Vec<SurfacePoint> {
    points
        .iter()
        .map(|&[x, y]| (Point2::new(x, y), Point3::new(x, y, 0.0)).into())
        .collect()
}

fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Vec<SurfacePoint> {
    polygon(&[
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ])
}

fn without_culling(boundary: &PolyBoundary) -> PolyBoundary {
    let mut unculled = boundary.clone();
    let everywhere = BoundingBox::from_iter([
        Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        Point2::new(f64::INFINITY, f64::INFINITY),
    ]);
    unculled.bounds.fill(everywhere);
    unculled
}

#[test]
fn holes_islands_disconnected_and_concave_loops() {
    let mut hole = rectangle(1.0, 1.0, 4.0, 4.0);
    hole.reverse();
    let boundary = PolyBoundary::from_loops(vec![
        rectangle(0.0, 0.0, 6.0, 6.0),
        hole,
        rectangle(2.0, 2.0, 1.0, 1.0),
        polygon(&[
            [8.0, 0.0],
            [11.0, 0.0],
            [11.0, 1.0],
            [9.0, 1.0],
            [9.0, 3.0],
            [8.0, 3.0],
        ]),
    ]);
    for (point, expected) in [
        ([0.5, 0.5], true),
        ([1.5, 1.5], false),
        ([2.5, 2.5], true),
        ([7.0, 0.5], false),
        ([8.5, 2.0], true),
        ([10.0, 2.0], false),
    ] {
        assert_eq!(boundary.include(Point2::from(point)), expected);
    }
    let unculled = without_culling(&boundary);
    for x in -10..130 {
        for y in -10..80 {
            let point = Point2::new(x as f64 / 10.0, y as f64 / 10.0);
            assert_eq!(
                boundary.include(point),
                unculled.include(point),
                "{point:?}"
            );
        }
    }
}

#[test]
fn culling_preserves_near_boundary_rejection() {
    let mut hole = rectangle(1.0, 1.0, 2.0, 2.0);
    hole.reverse();
    let boundary = PolyBoundary::from_loops(vec![rectangle(0.0, 0.0, 4.0, 4.0), hole]);
    let unculled = without_culling(&boundary);
    for points in &boundary.loops {
        for (a, b) in points.iter().circular_tuple_windows() {
            let normal = Vector2::new(b.y - a.y, a.x - b.x).normalize();
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for distance in [-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0] {
                    let point = a.uv + (b.uv - a.uv) * t + normal * distance * TOLERANCE;
                    assert_eq!(
                        boundary.include(point),
                        unculled.include(point),
                        "{point:?}"
                    );
                }
            }
        }
    }
}

proptest! {
    #[test]
    fn culling_matches_full_winding_scan(
        loops in prop::collection::vec(
            (prop::collection::vec((-20.0..20.0, -20.0..20.0), 3..24), any::<bool>()), 0..16),
        x in -30.0..30.0,
        y in -30.0..30.0,
    ) {
        let loops = loops.into_iter().map(|(points, reversed)| {
            let mut points = polygon(&points.into_iter().map(|(x, y)| [x, y]).collect::<Vec<_>>());
            if reversed {
                points.reverse();
            }
            points
        }).collect();
        let boundary = PolyBoundary::from_loops(loops);
        let point = Point2::new(x, y);
        prop_assert_eq!(boundary.include(point), without_culling(&boundary).include(point));
    }
}
