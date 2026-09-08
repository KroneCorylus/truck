use crate::{wasm_bindgen, AbstractShape, Edge, Face, IntoWasm, Vertex, Wire};
use truck_modeling::*;

macro_rules! intopt {
    ($type: ty, $slice: ident) => {
        assert!(
            $slice.len() == 3,
            "{} is not a 3-dimensional!",
            stringify!($slice)
        );
        let $slice = <$type>::new($slice[0], $slice[1], $slice[2]);
    };
    ($type: ty, $slice: ident, $($a: ty, $b: ident),*) => {
        intopt!($type, $slice); intopt!($($a, $b),*);
    }
}

/// Creates and returns a vertex by a three dimensional point.
#[wasm_bindgen]
pub fn vertex(x: f64, y: f64, z: f64) -> Vertex { builder::vertex(Point3::new(x, y, z)).into() }
/// Returns a line from `vertex0` to `vertex1`.
#[wasm_bindgen]
pub fn line(vertex0: &Vertex, vertex1: &Vertex) -> Edge { builder::line(vertex0, vertex1).into() }
/// Returns a circle arc from `vertex0` to `vertex1` via `transit`.
#[wasm_bindgen]
pub fn circle_arc(vertex0: &Vertex, vertex1: &Vertex, transit: &[f64]) -> Edge {
    intopt!(Point3, transit);
    builder::circle_arc(vertex0, vertex1, transit).into()
}
/// Returns a Bezier curve from `vertex0` to `vertex1` with inter control points `inter_points`.
#[wasm_bindgen]
pub fn bezier(vertex0: &Vertex, vertex1: &Vertex, inter_points: &[f64]) -> Edge {
    assert!(
        inter_points.len().is_multiple_of(3),
        "inter_points cannot convert to 3-dimensional points!"
    );
    let inter_points = inter_points
        .chunks(3)
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    builder::bezier(vertex0, vertex1, inter_points).into()
}
/// Returns a homotopic face from `edge0` to `edge1`.
#[wasm_bindgen]
pub fn homotopy(edge0: &Edge, edge1: &Edge) -> Face { builder::homotopy(edge0, edge1).into() }
/// Try attatiching a plane whose boundary is `wire`.
#[wasm_bindgen]
pub fn try_attach_plane(wire: &Wire) -> Option<Face> {
    builder::try_attach_plane(&[wire.as_ref().clone()])
        .map(|face| face.into())
        .map_err(|e| gloo::console::error!(format!("{e}")))
        .ok()
}

macro_rules! transform_if_chain {
    ($shape: expr, $function: expr, ($($arg: expr),*), $exception: expr, $member: ident) => {
        if let Some(entity) = AbstractShape::$member($shape) {
            $function(entity.as_ref(), $($arg),*).into_wasm().upcast()
        } else {
            $exception
        }
    };
    ($shape: expr, $function: expr, ($($arg: expr),*), $exception: expr, $member: ident, $($a: ident),*) => {
        if let Some(entity) = AbstractShape::$member($shape) {
            $function(entity.as_ref(), $($arg),*).into_wasm().upcast()
        } else {
            transform_if_chain!($shape, $function, ($($arg),*), $exception, $($a),*)
        }
    }
}

macro_rules! derive_all_shape {
    ($shape: expr, $function: expr, ($($arg: expr),*)) => {
        transform_if_chain!(
            $shape,
            $function,
            ($($arg),*),
            unreachable!(),
            as_vertex,
            as_edge,
            as_wire,
            as_face,
            as_shell,
            as_solid
        )
    };
}

/// Returns a translated vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn translated(shape: &AbstractShape, vector: &[f64]) -> AbstractShape {
    intopt!(Vector3, vector);
    derive_all_shape!(shape, builder::translated, (vector))
}

/// Returns a rotated vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn rotated(shape: &AbstractShape, origin: &[f64], axis: &[f64], angle: f64) -> AbstractShape {
    intopt!(Point3, origin, Vector3, axis);
    derive_all_shape!(shape, builder::rotated, (origin, axis, Rad(angle)))
}

/// Returns a scaled vertex, edge, wire, face, shell or solid.
#[wasm_bindgen]
pub fn scaled(shape: &AbstractShape, origin: &[f64], scalars: &[f64]) -> AbstractShape {
    intopt!(Point3, origin);
    if scalars.len() == 1 {
        let s = Vector3::new(scalars[0], scalars[0], scalars[0]);
        derive_all_shape!(shape, builder::scaled, (origin, s))
    } else if scalars.len() == 3 {
        let s = Vector3::new(scalars[0], scalars[1], scalars[2]);
        derive_all_shape!(shape, builder::scaled, (origin, s))
    } else {
        panic!("The length of scalars is not 1 or 3.");
    }
}

macro_rules! derive_all_sweepable{
    ($shape: expr, $function: expr, ($($arg: expr),*)) => {
        transform_if_chain!(
            $shape,
            $function,
            ($($arg),*),
            panic!("sweep is only implemented to Vertex, Edge, Wire and Face."),
            as_vertex,
            as_edge,
            as_wire,
            as_face
        )
    };
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by a vector.
#[wasm_bindgen]
pub fn tsweep(shape: &AbstractShape, vector: &[f64]) -> AbstractShape {
    intopt!(Vector3, vector);
    derive_all_sweepable!(shape, builder::tsweep, (vector))
}

/// Sweeps a vertex, an edge, a wire, a face, or a shell by the rotation.
#[wasm_bindgen]
pub fn rsweep(
    shape: &AbstractShape,
    origin: &[f64],
    axis: &[f64],
    angle: f64,
    division: usize,
) -> AbstractShape {
    intopt!(Point3, origin, Vector3, axis);
    derive_all_sweepable!(shape, builder::rsweep, (origin, axis, Rad(angle), division))
}

fn checked_vector(
    values: &[f64],
    parameter: &'static str,
    operation: &'static str,
) -> std::result::Result<Vector3, wasm_bindgen::JsValue> {
    if values.len() != 3 || values.iter().any(|x| !x.is_finite()) {
        return Err(invalid_parameter(
            operation,
            parameter,
            format!("{values:?}"),
        ));
    }
    Ok(Vector3::new(values[0], values[1], values[2]))
}
fn invalid_parameter(
    operation: &'static str,
    parameter: &'static str,
    value: impl std::fmt::Display,
) -> wasm_bindgen::JsValue {
    use truck_base::diagnostics::{Code, Diagnostic};
    crate::diagnostics::js_error(
        Diagnostic::new(Code::InvalidParameter, operation, "validate_input")
            .parameter(parameter, value),
    )
}

/// Translates with finite 3D-vector validation and structured errors.
#[wasm_bindgen]
pub fn try_translated(
    shape: &AbstractShape,
    vector: &[f64],
) -> std::result::Result<AbstractShape, wasm_bindgen::JsValue> {
    checked_vector(vector, "vector", "translated")?;
    Ok(translated(shape, vector))
}
/// Rotates with finite parameter and unit-axis validation.
#[wasm_bindgen]
pub fn try_rotated(
    shape: &AbstractShape,
    origin: &[f64],
    axis: &[f64],
    angle: f64,
) -> std::result::Result<AbstractShape, wasm_bindgen::JsValue> {
    checked_vector(origin, "origin", "rotated")?;
    let direction = checked_vector(axis, "axis", "rotated")?;
    if !direction.magnitude2().near(&1.0) {
        return Err(invalid_parameter(
            "rotated",
            "axis",
            "expected a unit vector",
        ));
    }
    if !angle.is_finite() {
        return Err(invalid_parameter("rotated", "angle", angle));
    }
    Ok(rotated(shape, origin, axis, angle))
}
/// Scales with finite origin and nonzero scalar validation.
#[wasm_bindgen]
pub fn try_scaled(
    shape: &AbstractShape,
    origin: &[f64],
    scalars: &[f64],
) -> std::result::Result<AbstractShape, wasm_bindgen::JsValue> {
    checked_vector(origin, "origin", "scaled")?;
    if !matches!(scalars.len(), 1 | 3) || scalars.iter().any(|x| !x.is_finite() || *x == 0.0) {
        return Err(invalid_parameter(
            "scaled",
            "scalars",
            format!("{scalars:?}"),
        ));
    }
    Ok(scaled(shape, origin, scalars))
}
/// Attaches a plane, preserving the modeling error in a structured exception.
#[wasm_bindgen]
pub fn attach_plane_with_diagnostics(
    wire: &Wire,
) -> std::result::Result<Face, wasm_bindgen::JsValue> {
    use truck_base::diagnostics::{Code, Diagnostic};
    builder::try_attach_plane(&[wire.as_ref().clone()])
        .map(IntoWasm::into_wasm)
        .map_err(|e| {
            crate::diagnostics::js_error(
                Diagnostic::new(Code::UnsupportedGeometry, "attach_plane", "construct_face")
                    .with_coded_source(e),
            )
        })
}
/// Creates an edge, preserving topology errors instead of panicking on identical vertices.
#[wasm_bindgen]
pub fn try_line(
    vertex0: &Vertex,
    vertex1: &Vertex,
) -> std::result::Result<Edge, wasm_bindgen::JsValue> {
    use truck_base::diagnostics::{Code, Diagnostic};
    truck_modeling::Edge::try_new(
        vertex0,
        vertex1,
        Curve::Line(Line(vertex0.point(), vertex1.point())),
    )
    .map(IntoWasm::into_wasm)
    .map_err(|e| {
        crate::diagnostics::js_error(
            Diagnostic::new(Code::InvalidInputTopology, "line", "validate_input")
                .with_coded_source(e),
        )
    })
}
/// Creates a Bezier edge after validating its point array and endpoints.
#[wasm_bindgen]
pub fn try_bezier(
    vertex0: &Vertex,
    vertex1: &Vertex,
    inter_points: &[f64],
) -> std::result::Result<Edge, wasm_bindgen::JsValue> {
    if !inter_points.len().is_multiple_of(3) || inter_points.iter().any(|x| !x.is_finite()) {
        return Err(invalid_parameter(
            "bezier",
            "inter_points",
            format!("{inter_points:?}"),
        ));
    }
    if vertex0.id() == vertex1.id() {
        return Err(invalid_parameter(
            "bezier",
            "vertices",
            "endpoints must be distinct vertices",
        ));
    }
    Ok(bezier(vertex0, vertex1, inter_points))
}
/// Creates a circle arc after checking its transit point and non-collinear endpoints.
#[wasm_bindgen]
pub fn try_circle_arc(
    vertex0: &Vertex,
    vertex1: &Vertex,
    transit: &[f64],
) -> std::result::Result<Edge, wasm_bindgen::JsValue> {
    let point = Point3::from_vec(checked_vector(transit, "transit", "circle_arc")?);
    let chord = vertex1.point() - vertex0.point();
    if vertex0.id() == vertex1.id()
        || !chord.magnitude2().is_finite()
        || chord.cross(point - vertex0.point()).so_small()
    {
        return Err(invalid_parameter(
            "circle_arc",
            "points",
            "points must be finite and non-collinear",
        ));
    }
    Ok(circle_arc(vertex0, vertex1, transit))
}
fn check_sweep_shape(
    shape: &AbstractShape,
    operation: &'static str,
) -> std::result::Result<(), wasm_bindgen::JsValue> {
    if shape.as_shell().is_some() || shape.as_solid().is_some() {
        use truck_base::diagnostics::{Code, Diagnostic};
        return Err(crate::diagnostics::js_error(Diagnostic::new(
            Code::UnsupportedTopology,
            operation,
            "validate_input",
        )));
    }
    Ok(())
}
/// Translational sweep with vector and shape-kind validation.
#[wasm_bindgen]
pub fn try_tsweep(
    shape: &AbstractShape,
    vector: &[f64],
) -> std::result::Result<AbstractShape, wasm_bindgen::JsValue> {
    let vector3 = checked_vector(vector, "vector", "tsweep")?;
    if vector3.so_small() {
        return Err(invalid_parameter(
            "tsweep",
            "vector",
            "sweep vector must be nonzero",
        ));
    }
    check_sweep_shape(shape, "tsweep")?;
    Ok(tsweep(shape, vector))
}
/// Rotational sweep with axis, angle, subdivision and shape-kind validation.
#[wasm_bindgen]
pub fn try_rsweep(
    shape: &AbstractShape,
    origin: &[f64],
    axis: &[f64],
    angle: f64,
    division: usize,
) -> std::result::Result<AbstractShape, wasm_bindgen::JsValue> {
    checked_vector(origin, "origin", "rsweep")?;
    let direction = checked_vector(axis, "axis", "rsweep")?;
    if !direction.magnitude2().near(&1.0) {
        return Err(invalid_parameter(
            "rsweep",
            "axis",
            "expected a unit vector",
        ));
    }
    if !angle.is_finite() || angle == 0.0 {
        return Err(invalid_parameter("rsweep", "angle", angle));
    }
    let minimum = if angle.abs() >= 2.0 * std::f64::consts::PI {
        2
    } else {
        1
    };
    if division < minimum {
        return Err(invalid_parameter("rsweep", "division", division));
    }
    check_sweep_shape(shape, "rsweep")?;
    Ok(rsweep(shape, origin, axis, angle, division))
}
