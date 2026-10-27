//! Bounded procedural construction timings; setup, QA and destruction are untimed.
use std::{hint::black_box, time::Instant};
use truck_modeling::*;
use truck_shapeops::fillet::{
    try_chamfer_solid_along_wire_chord, try_chamfer_solid_edges_with_distances,
};
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

fn parabola(a: f64, b: f64, height: impl Fn(f64) -> f64) -> BSplineCurve<Point3> {
    let point = |t: f64| Point3::new(t, t * t / 10., height(t));
    let p = point(a);
    let q = point(b);
    BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            p,
            Point3::from_vec(point((a + b) / 2.).to_vec() * 2. - (p.to_vec() + q.to_vec()) / 2.),
            q,
        ],
    )
}

// Exact source: 0 <= z <= min(4, 3.5+x+y), y >= x²/10,
// x+y >= 0, x <= 4, y <= 4. Build the prior bevel without calling a blend operation.
fn source(width: f64) -> (Solid, Wire) {
    let a = 30_f64.sqrt() - 5.;
    let v = builder::vertices([
        Point3::new(0., 0., 0.),
        Point3::new(4., 1.6, 0.),
        Point3::new(4., width, 0.),
        Point3::new(-width, width, 0.),
        Point3::new(0., 0., 3.5),
        Point3::new(a, a * a / 10., 4.),
        Point3::new(4., 1.6, 4.),
        Point3::new(4., width, 4.),
        Point3::new(0.5 - width, width, 4.),
        Point3::new(-width, width, 3.5),
    ]);
    let bottom = Edge::new(&v[0], &v[1], parabola(0., 4., |_| 0.).into());
    let selected = Edge::new(&v[5], &v[6], parabola(a, 4., |_| 4.).into());
    let prior = Edge::new(
        &v[4],
        &v[5],
        parabola(0., a, |t| 3.5 + t + t * t / 10.).into(),
    );
    let pairs = [
        (1, 2),
        (2, 3),
        (3, 0),
        (0, 4),
        (1, 6),
        (2, 7),
        (3, 9),
        (6, 7),
        (7, 8),
        (8, 5),
        (8, 9),
        (9, 4),
    ];
    let e: Vec<_> = pairs
        .into_iter()
        .map(|(a, b)| builder::line(&v[a], &v[b]))
        .collect();
    let loops = [
        wire![
            bottom.inverse(),
            e[2].inverse(),
            e[1].inverse(),
            e[0].inverse()
        ],
        wire![e[0].clone(), e[5].clone(), e[7].inverse(), e[4].inverse()],
        wire![
            e[1].clone(),
            e[6].clone(),
            e[10].inverse(),
            e[8].inverse(),
            e[5].inverse()
        ],
        wire![e[2].clone(), e[3].clone(), e[11].inverse(), e[6].inverse()],
        wire![selected.clone(), e[7].clone(), e[8].clone(), e[9].clone()],
        wire![prior.clone(), e[9].inverse(), e[10].clone(), e[11].clone()],
    ];
    let mut faces: Shell = loops
        .into_iter()
        .map(|w| builder::try_attach_plane(&[w]).unwrap())
        .collect();
    let wall = builder::tsweep(&bottom, Vector3::unit_z() * 4.);
    faces.push(Face::new(
        vec![wire![
            bottom,
            e[4].clone(),
            selected.inverse(),
            prior.inverse(),
            e[3].inverse()
        ]],
        wall.oriented_surface(),
    ));
    (Solid::new(vec![faces]), wire![selected])
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let samples: usize = args[2].parse().unwrap();
    let family = args[1].as_str();
    let (input, wire) = if family == "C1" {
        source(4.)
    } else {
        (quarter(1., 1.), Wire::new())
    };
    let ids = if family == "A7" {
        let mut seen = std::collections::HashSet::new();
        input
            .edge_iter()
            .map(|e| e.id())
            .filter(|id| seen.insert(*id))
            .collect()
    } else {
        selection(&input, Point3::new(1., 0., 1.))
    };
    let distances = setbacks(&input, &ids, 1., [0.2; 5], Matrix4::identity(), 0.2);
    if family == "A2" {
        assert_eq!(ids.len(), 3);
    } else if family == "A7" {
        assert_eq!(ids.len(), 9);
    } else {
        assert_eq!(family, "C1");
    }
    assert!(input.is_geometric_consistent());
    if family == "A7" && args.len() > 3 {
        use truck_stepio::out::*;
        let prepared = prepare_for_step(&input.compress(), 0.00005).unwrap();
        let design = StepDesign::from_model(StepModel::from(&prepared));
        std::fs::write(
            format!("{}.input.step", args[3]),
            StepDisplay::new(Default::default(), design).to_string(),
        )
        .unwrap();
        for (id, d) in &distances {
            let edge = input.edge_iter().find(|e| e.id() == *id).unwrap();
            let curve = edge.curve();
            let (a, b) = curve.range_tuple();
            println!("SELECTION {:?} {:?}", curve.subs((a + b) / 2.), d);
        }
    }
    let before = serde_json::to_string(&input.compress()).unwrap();
    let mut reference = None;
    for i in 0..=samples {
        let start = Instant::now();
        let result = black_box(if family == "C1" {
            try_chamfer_solid_along_wire_chord(black_box(&input), black_box(&wire), 1., 1., 0.001)
                .unwrap()
        } else {
            try_chamfer_solid_edges_with_distances(black_box(&input), black_box(&distances), 0.001)
                .unwrap()
        });
        let ms = start.elapsed().as_secs_f64() * 1000.;
        assert!(result.solid.is_geometric_consistent());
        if i == samples && args.len() > 3 {
            use truck_stepio::out::*;
            let prepared = prepare_for_step(&result.solid.compress(), 0.00005).unwrap();
            let design = StepDesign::from_model(StepModel::from(&prepared));
            std::fs::write(
                &args[3],
                StepDisplay::new(Default::default(), design).to_string(),
            )
            .unwrap();
        }
        let serialized = serde_json::to_string(&result.solid.compress()).unwrap();
        if let Some(ref expected) = reference {
            assert_eq!(&serialized, expected);
        } else {
            reference = Some(serialized);
        }
        assert_eq!(before, serde_json::to_string(&input.compress()).unwrap());
        println!(
            "{}",
            serde_json::json!({"engine":"truck","family":family,"sample":i,"warmup":i==0,"ms":ms,"faces":result.solid.face_iter().count(),"consistent":true,"repeatable":true})
        );
    }
}
