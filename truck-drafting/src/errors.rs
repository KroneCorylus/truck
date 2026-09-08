use thiserror::Error;

/// Drafting errors
#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// cannot construct a circle arc from collinear points.
    #[error("cannot construct a circle arc from collinear points.")]
    CollinearArcPoints,
    /// cannot construct a circle arc when the tangent is parallel to the chord.
    #[error("cannot construct a circle arc when the tangent is parallel to the chord.")]
    ParallelArcTangent,
    /// the radius of circle must be positive.
    #[error("the radius of circle must be positive.")]
    NonPositiveRadius,
    /// the tangent vector at the specified parameter vanished.
    #[error("the tangent vector vanished near the specified corner.")]
    DegenerateTangent,
    /// two line directions are parallel and cannot determine a crossing point.
    #[error("two line directions are parallel and cannot determine a crossing point.")]
    ParallelLineDirections,
    /// the connection corner is too close to one of the vertices.
    #[error("the connection corner is too close to one of the vertices.")]
    DegenerateConnectionCorner,
    /// the requested vertices and tangents cannot be connected by the selected primitive sequence.
    #[error(
        "the requested vertices and tangents cannot be connected by the selected primitive sequence."
    )]
    NoConnection,
    /// the specified corner is degenerate and cannot define a fillet direction.
    #[error("the specified corner is degenerate and cannot define a fillet direction.")]
    DegenerateCorner,
    /// the Jacobian of fillet equations became degenerate.
    #[error("failed to solve fillet candidate because the Jacobian became degenerate. {0}")]
    DegenerateFilletJacobian(String),
    /// Newton method did not converge while solving the fillet equations.
    #[error("failed to solve fillet candidate because Newton method did not converge. {0}")]
    FilletNewtonNotConverged(String),
    /// the chamfer distance must be positive.
    #[error("the chamfer distance must be positive.")]
    NonPositiveChamferDistance,
    /// the requested curve length goes outside the curve parameter range.
    #[error("the requested curve length goes outside the curve parameter range.")]
    CurveLengthOutOfRange,
    /// corner operations require a continuous wire.
    #[error("corner operations require a continuous wire.")]
    NonContinuousWire,
    /// error from `truck_geometry::errors::Error`.
    #[error("{0}")]
    GeometricError(#[source] truck_geometry::errors::Error),
    /// error from `truck_topology::errors::Error`.
    #[error("{0}")]
    TopologicalError(#[source] truck_topology::errors::Error),
}

impl From<truck_geometry::errors::Error> for Error {
    fn from(value: truck_geometry::errors::Error) -> Self { Self::GeometricError(value) }
}

impl From<truck_topology::errors::Error> for Error {
    fn from(value: truck_topology::errors::Error) -> Self { Self::TopologicalError(value) }
}

impl Error {
    /// Stable external error code. Display messages are not a machine-readable contract.
    pub fn code(&self) -> &'static str {
        match self {
            Self::CollinearArcPoints => "TRUCK_DRAFTING_COLLINEAR_ARC_POINTS",
            Self::ParallelArcTangent => "TRUCK_DRAFTING_PARALLEL_ARC_TANGENT",
            Self::NonPositiveRadius => "TRUCK_DRAFTING_NON_POSITIVE_RADIUS",
            Self::DegenerateTangent => "TRUCK_DRAFTING_DEGENERATE_TANGENT",
            Self::ParallelLineDirections => "TRUCK_DRAFTING_PARALLEL_LINE_DIRECTIONS",
            Self::DegenerateConnectionCorner => "TRUCK_DRAFTING_DEGENERATE_CONNECTION_CORNER",
            Self::NoConnection => "TRUCK_DRAFTING_NO_CONNECTION",
            Self::DegenerateCorner => "TRUCK_DRAFTING_DEGENERATE_CORNER",
            Self::DegenerateFilletJacobian(..) => "TRUCK_DRAFTING_DEGENERATE_FILLET_JACOBIAN",
            Self::FilletNewtonNotConverged(..) => "TRUCK_DRAFTING_FILLET_NEWTON_NOT_CONVERGED",
            Self::NonPositiveChamferDistance => "TRUCK_DRAFTING_NON_POSITIVE_CHAMFER_DISTANCE",
            Self::CurveLengthOutOfRange => "TRUCK_DRAFTING_CURVE_LENGTH_OUT_OF_RANGE",
            Self::NonContinuousWire => "TRUCK_DRAFTING_NON_CONTINUOUS_WIRE",
            Self::GeometricError(error) => error.code(),
            Self::TopologicalError(error) => error.code(),
        }
    }
}

impl truck_base::diagnostics::CodedError for Error {
    fn code(&self) -> &'static str { self.code() }
}
