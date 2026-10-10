//! Native counterpart of occt/main.cpp. See scripts/compare_occt.py.
#[allow(dead_code)]
#[path = "../benches/fixtures.rs"]
mod fixtures;

use serde_json::{json, Value};
use std::{
    f64::consts::{PI, TAU},
    hint::black_box,
    path::Path,
    time::Instant,
};
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_shapeops::{
    fillet::try_fillet_solid_along_wire,
    thread::{thread_groove, ThreadGroove},
};

enum Output {
    Solid(Solid),
    Mesh(PolygonMesh),
}

struct Case {
    run: Box<dyn Fn() -> Output>,
    expected: f64,
    qa_tol: f64,
}

fn cylinder(radius: f64, height: f64, center: Point3) -> Solid {
    let disc = builder::try_attach_plane(vec![fixtures::circle(center, radius)]).unwrap();
    builder::tsweep(&disc, Vector3::unit_z() * height)
}

fn compound(solids: &[Solid]) -> Solid {
    Solid::new(
        solids
            .iter()
            .flat_map(|s| s.boundaries().iter().cloned())
            .collect(),
    )
}

fn thread_cutter(turns: usize) -> Solid {
    thread_groove(ThreadGroove {
        origin: Point3::new(0.0, 0.0, -1.25),
        axis: Vector3::unit_z(),
        radius: 4.0,
        depth: 0.4,
        pitch: 1.25,
        length: (turns + 2) as f64 * 1.25,
        internal: false,
        left_handed: false,
    })
    .expect("thread cutter")
}

fn thread_volume(turns: usize) -> f64 {
    let (r, root, pitch): (f64, f64, f64) = (4.0, 3.6, 1.25);
    let radial_area = (r * r - root * root) / 2.0;
    let radial_moment = (r.powi(3) - root.powi(3)) / 3.0 - root * radial_area;
    turns as f64
        * (PI * r * r * pitch
            - 4.0 * PI * (pitch / 16.0 * radial_area + radial_moment / 3.0_f64.sqrt()))
}

fn knurl(count: usize) -> (Solid, Solid, f64) {
    let (r, depth, height) = (12.0, 0.35, 8.0);
    let half = PI / (2.0 * count as f64);
    let outer_x = r - depth + 1.25 * (r * half.cos() - (r - depth));
    let outer_y = 1.25 * r * half.sin();
    assert!(outer_x > r && outer_y.atan2(outer_x) < PI / count as f64);
    let p = Point3::new(r - depth, 0.0, -1.0);
    let cutters: Vec<_> = (0..count)
        .map(|i| {
            let points = [
                p,
                p + (Point3::new(r * half.cos(), -r * half.sin(), -1.0) - p) * 1.25,
                p + (Point3::new(r * half.cos(), r * half.sin(), -1.0) - p) * 1.25,
            ];
            let vertices = points.map(builder::vertex);
            let wire: Wire = (0..3)
                .map(|j| builder::line(&vertices[j], &vertices[(j + 1) % 3]))
                .collect();
            let face = builder::try_attach_plane(vec![wire]).unwrap();
            builder::rotated(
                &builder::tsweep(&face, Vector3::unit_z() * 10.0),
                Point3::origin(),
                Vector3::unit_z(),
                Rad(TAU * i as f64 / count as f64),
            )
        })
        .collect();
    let removed = r * r * half - (r - depth) * r * half.sin();
    (
        cylinder(r, height, Point3::origin()),
        compound(&cutters),
        height * (PI * r * r - count as f64 * removed),
    )
}

fn fixture(name: &str, tol: f64, qa_tol: f64) -> Case {
    let (kind, parameter) = name.split_once('/').expect("case/parameter");
    let n = || parameter.parse::<usize>().expect("integer parameter");
    let scalar = || parameter.parse::<f64>().expect("numeric parameter");
    let (run, expected, qa_tol): (Box<dyn Fn() -> Output>, f64, f64) = match kind {
        "holes_batch" | "holes_sequential" => {
            let (plate, cutters) = fixtures::plate_and_cutters(n());
            let batch = compound(&cutters);
            let sequential = kind == "holes_sequential";
            (
                Box::new(move || {
                    Output::Solid(if sequential {
                        cutters.iter().fold(plate.clone(), |s, c| {
                            truck_shapeops::try_subtract(&s, c, tol).expect("subtract")
                        })
                    } else {
                        truck_shapeops::try_subtract(&plate, &batch, tol).expect("subtract")
                    })
                }),
                fixtures::plate_volume(n()),
                qa_tol,
            )
        }
        "knurl" => {
            let (body, tool, volume) = knurl(n());
            (
                Box::new(move || {
                    Output::Solid(
                        truck_shapeops::try_subtract(&body, &tool, tol).expect("knurl subtraction"),
                    )
                }),
                volume,
                qa_tol,
            )
        }
        "thread" => {
            let body = cylinder(4.0, n() as f64 * 1.25, Point3::origin());
            let tool = thread_cutter(n());
            (
                Box::new(move || {
                    Output::Solid(
                        truck_shapeops::try_subtract(&body, &tool, tol)
                            .expect("thread subtraction"),
                    )
                }),
                thread_volume(n()),
                qa_tol,
            )
        }
        "near_tangent" => {
            let overlap = scalar();
            let distance = 2.0 - overlap;
            let a = cylinder(1.0, 2.0, Point3::origin());
            let b = cylinder(1.0, 2.0, Point3::new(distance, 0.0, 0.0));
            let area =
                2.0 * (distance / 2.0).acos() - distance / 2.0 * (4.0 - distance * distance).sqrt();
            (
                Box::new(move || {
                    Output::Solid(
                        truck_shapeops::try_and(&a, &b, tol).expect("near tangent intersection"),
                    )
                }),
                area * 2.0,
                qa_tol.min(overlap / 10000.0).max(1e-6),
            )
        }
        "thin_wall" => {
            let wall = scalar();
            let body = cylinder(4.0, 8.0, Point3::origin());
            let tool = cylinder(4.0 - wall, 10.0, Point3::new(0.0, 0.0, -1.0));
            (
                Box::new(move || {
                    Output::Solid(
                        truck_shapeops::try_subtract(&body, &tool, tol).expect("thin wall"),
                    )
                }),
                PI * (16.0 - (4.0 - wall).powi(2)) * 8.0,
                qa_tol.min(wall / 1000.0),
            )
        }
        "fillet_bore" => {
            let radius = scalar();
            let block = fixtures::cuboid(
                Point3::new(-10.0, -10.0, 0.0),
                Point3::new(10.0, 10.0, 10.0),
            );
            let tool = cylinder(3.0, 12.0, Point3::new(0.0, 0.0, -1.0));
            let body = truck_shapeops::try_subtract(&block, &tool, tol).unwrap();
            let rim = body
                .face_iter()
                .find(|f| {
                    matches!(f.surface(), Surface::Plane(_))
                        && f.vertex_iter().all(|v| (v.point().z - 10.0).abs() < 1e-6)
                })
                .unwrap()
                .boundaries()
                .into_iter()
                .find(|w| w.iter().all(|e| !matches!(e.curve(), Curve::Line(_))))
                .unwrap();
            let removed = 2.0
                * PI
                * (3.0 * radius.powi(2) * (1.0 - PI / 4.0)
                    + radius.powi(3) * (5.0 / 6.0 - PI / 4.0));
            (
                Box::new(move || {
                    Output::Solid(
                        try_fillet_solid_along_wire(&body, &rim, radius, tol)
                            .expect("bore fillet")
                            .solid,
                    )
                }),
                4000.0 - 90.0 * PI - removed,
                qa_tol,
            )
        }
        "mesh_plate" => {
            let body = fixtures::perforated_plate(n());
            (
                Box::new(move || {
                    let mesh = body.triangulation(tol);
                    assert!(
                        mesh.face_iter().all(|f| f.surface().is_some()),
                        "partial timed mesh"
                    );
                    Output::Mesh(mesh.to_polygon())
                }),
                fixtures::plate_volume(n()),
                tol,
            )
        }
        _ => panic!("unknown case {name}"),
    };
    Case {
        run,
        expected,
        qa_tol,
    }
}

fn check(output: &Output, expected: f64, tol: f64) -> Value {
    let (mut mesh, faces, geometric, topology) = match output {
        Output::Solid(solid) => {
            let topology = solid.boundaries().len() == 1
                && Solid::try_new(solid.boundaries().clone()).is_ok();
            let geometric = solid.is_geometric_consistent();
            let meshed = solid.triangulation(tol);
            assert!(
                meshed.face_iter().all(|f| f.surface().is_some()),
                "partial QA tessellation"
            );
            (
                meshed.to_polygon(),
                Some(solid.face_iter().count()),
                geometric,
                topology,
            )
        }
        Output::Mesh(mesh) => (mesh.clone(), None, true, true),
    };
    mesh.put_together_same_attrs(1e-8)
        .remove_degenerate_faces()
        .remove_unused_attrs();
    let closed = format!("{:?}", mesh.shell_condition()) == "Closed";
    let volume = mesh.volume();
    let error = (volume - expected).abs() / expected;
    let pass =
        topology && geometric && closed && volume.is_finite() && volume > 0.0 && error <= 0.002;
    json!({"pass":pass,"topology":topology,"geometric":geometric,"mesh_closed":closed,
        "faces":faces,"triangles":mesh.faces().triangle_iter().count(),"volume":volume,
        "expected_volume":expected,"relative_volume_error":error,"qa_tolerance":tol})
}

fn peak_rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|s| s.split_whitespace().next()?.parse().ok())
        })
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--export-threads") {
        let directory = Path::new(&args[2]);
        std::fs::create_dir_all(directory).unwrap();
        for turns in [1, 4] {
            let tool = thread_cutter(turns).compress();
            use truck_stepio::out::*;
            let design = StepDesign::from_model(StepModel::from(&tool));
            std::fs::write(
                directory.join(format!("thread-{turns}.step")),
                StepDisplay::new(Default::default(), design).to_string(),
            )
            .unwrap();
        }
        return;
    }
    assert_eq!(args.len(), 6, "CASE SAMPLES TOL QA_TOL FIXTURE_DIR");
    let samples: usize = args[2].parse().unwrap();
    let tol: f64 = args[3].parse().unwrap();
    let qa_tol: f64 = args[4].parse().unwrap();
    assert!(tol.is_finite() && tol > 0.0 && qa_tol.is_finite() && qa_tol > 0.0);
    let result = std::panic::catch_unwind(|| {
        let case = fixture(&args[1], tol, qa_tol);
        let mut output = black_box((case.run)());
        let mut times = Vec::with_capacity(samples);
        for _ in 0..samples {
            let start = Instant::now();
            let next = black_box((case.run)());
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            output = next;
        }
        let rss = peak_rss_kib();
        // Persist timing evidence even if independent postflight QA panics or times out.
        println!(
            "RESULT {}",
            json!({"phase":"timing","samples_ms":times,"peak_rss_kib":rss})
        );
        let qa = check(&output, case.expected, case.qa_tol);
        println!("RESULT {}", json!({"phase":"qa","qa":qa}));
    });
    if let Err(error) = result {
        let message = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap_or("panic");
        println!("RESULT {}", json!({"phase":"failure","error":message}));
        std::process::exit(1);
    }
}
