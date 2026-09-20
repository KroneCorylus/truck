//! Helical, single-start 60-degree thread cutting tools.
use std::f64::consts::TAU;
use truck_base::diagnostics::{Code, Diagnostic};
use truck_modeling::*;

/// Parameters of a helical groove. All lengths use the solid's units.
#[derive(Clone, Copy, Debug)]
pub struct ThreadGroove {
    /// Point on the axis at the middle of the initial tooth.
    pub origin: Point3,
    /// Axis and positive axial travel direction.
    pub axis: Vector3,
    /// Existing cylinder radius.
    pub radius: f64,
    /// Radial extent of the groove beyond the existing cylinder.
    pub depth: f64,
    /// Axial distance per revolution.
    pub pitch: f64,
    /// Axial travel of the groove centre.
    pub length: f64,
    /// Cut outwards into a bore, rather than inwards into a shaft.
    pub internal: bool,
    /// Reverse the helix handedness.
    pub left_handed: bool,
}

fn error(stage: &'static str) -> Diagnostic {
    Diagnostic::new(Code::InvalidParameter, "thread_groove", stage)
}

/// Constructs a closed B-rep cutter with 60-degree flanks and a pitch/8 root flat.
///
/// Rational quadratic spans retain exact circular radial sections; their axial
/// helix approximation is sample-checked within 0.00001 units.
/// The cutter overlaps the original cylinder by pitch/10. Callers must extend the
/// travel beyond both ends of the cylinder before subtraction. Up to 100 turns
/// are supported. Invalid dimensions return a diagnostic without changing input.
pub fn thread_groove(spec: ThreadGroove) -> std::result::Result<Solid, Diagnostic> {
    let ThreadGroove {
        origin,
        axis,
        radius,
        depth,
        pitch,
        length,
        internal,
        left_handed,
    } = spec;
    if !origin.to_vec().is_finite()
        || !axis.is_finite()
        || (!axis.magnitude2().is_finite() || axis.magnitude2() < 1e-12)
        || [radius, depth, pitch, length]
            .iter()
            .any(|v| !v.is_finite() || *v <= TOLERANCE * 10.0)
        || depth >= radius
        || length / pitch > 100.0
        || depth / 3.0_f64.sqrt() * 2.0 + pitch / 8.0 >= pitch
    {
        return Err(error("validate_dimensions"));
    }
    let axis = axis.normalize();
    let x = if axis.x.abs() < 0.9 {
        axis.cross(Vector3::unit_x())
    } else {
        axis.cross(Vector3::unit_y())
    }
    .normalize();
    let y = axis.cross(x);
    let sign = if left_handed { -1.0 } else { 1.0 };
    let overlap = pitch * 0.1;
    let r0 = radius + if internal { -overlap } else { overlap };
    let r1 = radius + if internal { depth } else { -depth };
    if r0 <= 0.0 || r1 <= 0.0 {
        return Err(error("validate_radii"));
    }
    let half_root = pitch / 16.0;
    let half_base = half_root + (depth + overlap) / 3.0_f64.sqrt();
    if 2.0 * half_base >= pitch {
        return Err(error("overlapping_turns"));
    }
    let mut profile = vec![
        (r0, -half_base),
        (r1, -half_root),
        (r1, half_root),
        (r0, half_base),
    ];
    if !internal {
        profile.reverse();
    }
    if left_handed {
        profile.reverse();
    }
    let total = length / pitch * TAU;
    let sections = (total / (TAU / 4.0)).ceil() as usize;
    let step = total / sections as f64;
    let subdivisions = (step / 0.1_f64.min((0.00001 / pitch).cbrt())).ceil() as usize;
    if subdivisions == 0 || sections.saturating_mul(subdivisions) > 100_000 {
        return Err(error("thread_resolution_limit"));
    }
    let point = |r: f64, z: f64, angle: f64| {
        origin
            + (x * (sign * angle).cos() + y * (sign * angle).sin()) * r
            + axis * (z + angle / TAU * pitch)
    };
    let stations: Vec<Vec<Vertex>> = (0..=sections)
        .map(|i| {
            profile
                .iter()
                .map(|&(r, z)| builder::vertex(point(r, z, i as f64 * step)))
                .collect()
        })
        .collect();
    let rings: Vec<Vec<Edge>> = stations
        .iter()
        .map(|v| {
            (0..4)
                .map(|j| builder::line(&v[j], &v[(j + 1) % 4]))
                .collect()
        })
        .collect();
    let mut shell = Shell::new();
    shell.push(
        builder::try_attach_plane(&[rings[0].iter().cloned().collect()])
            .map_err(|_| error("start_cap"))?,
    );
    for i in 0..sections {
        let curves: Vec<NurbsCurve<Vector4>> = profile
            .iter()
            .map(|&(r, z)| {
                let mut knots = vec![0.0; 3];
                let mut controls = vec![point(r, z, i as f64 * step).to_homogeneous()];
                for j in 0..subdivisions {
                    let a = i as f64 * step + j as f64 * step / subdivisions as f64;
                    let b = i as f64 * step + (j + 1) as f64 * step / subdivisions as f64;
                    let mid = (a + b) * 0.5;
                    let weight = ((b - a) * 0.5).cos();
                    let middle = origin
                        + (x * (sign * mid).cos() + y * (sign * mid).sin()) * (r / weight)
                        + axis * (z + mid / TAU * pitch);
                    controls.extend([
                        middle.to_homogeneous() * weight,
                        point(r, z, b).to_homogeneous(),
                    ]);
                    knots.extend(std::iter::repeat_n(
                        (j + 1) as f64 / subdivisions as f64,
                        if j + 1 == subdivisions { 3 } else { 2 },
                    ));
                }
                NurbsCurve::new(BSplineCurve::new(KnotVec::from(knots), controls))
            })
            .collect();
        for (curve, &(r, z)) in curves.iter().zip(&profile) {
            for j in 0..=subdivisions * 4 {
                let t = j as f64 / (subdivisions * 4) as f64;
                if curve.subs(t).distance(point(r, z, (i as f64 + t) * step)) > 0.00001 {
                    return Err(Diagnostic::new(
                        Code::CurveFittingFailed,
                        "thread_groove",
                        "verify_helix",
                    ));
                }
            }
        }
        let helices: Vec<Edge> = (0..4)
            .map(|j| {
                Edge::new(
                    &stations[i][j],
                    &stations[i + 1][j],
                    curves[j].clone().into(),
                )
            })
            .collect();
        for j in 0..4 {
            let k = (j + 1) % 4;
            let wire: Wire = [
                helices[j].clone(),
                rings[i + 1][j].clone(),
                helices[k].inverse(),
                rings[i][j].inverse(),
            ]
            .into_iter()
            .collect();
            shell.push(Face::new(
                vec![wire],
                NurbsSurface::new(BSplineSurface::homotopy(
                    curves[j].non_rationalized().clone(),
                    curves[k].non_rationalized().clone(),
                ))
                .into(),
            ));
        }
    }
    shell.push(
        builder::try_attach_plane(&[rings[sections].iter().cloned().collect()])
            .map_err(|_| error("end_cap"))?
            .inverse(),
    );
    Solid::try_new(vec![shell]).map_err(|e| {
        Diagnostic::new(Code::InvalidOutputTopology, "thread_groove", "close_solid")
            .with_coded_source(e)
    })
}
