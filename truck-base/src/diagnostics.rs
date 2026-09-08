//! Stable operation diagnostics. Codes are the consumer contract; messages may change.
use serde::Serialize;
use std::{error::Error, fmt};

/// Machine-readable failure conditions. Handle future variants with a fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Code {
    /// The tolerance must be finite and positive.
    #[serde(rename = "TRUCK_INVALID_TOLERANCE")]
    InvalidTolerance,
    /// The parameter is invalid.
    #[serde(rename = "TRUCK_INVALID_PARAMETER")]
    InvalidParameter,
    /// The selected edge does not belong to the input.
    #[serde(rename = "TRUCK_SELECTION_UNKNOWN_EDGE")]
    UnknownEdge,
    /// The selected face does not belong to the input.
    #[serde(rename = "TRUCK_SELECTION_UNKNOWN_FACE")]
    UnknownFace,
    /// This operation requires a nonempty selection.
    #[serde(rename = "TRUCK_SELECTION_EMPTY")]
    EmptySelection,
    /// The selection contains duplicate entities.
    #[serde(rename = "TRUCK_SELECTION_DUPLICATE")]
    DuplicateSelection,
    /// The selected edges must belong to one shell.
    #[serde(rename = "TRUCK_SELECTION_MULTIPLE_SHELLS")]
    MultipleShells,
    /// The input topology is invalid.
    #[serde(rename = "TRUCK_INVALID_INPUT_TOPOLOGY")]
    InvalidInputTopology,
    /// The constructed topology is invalid.
    #[serde(rename = "TRUCK_INVALID_OUTPUT_TOPOLOGY")]
    InvalidOutputTopology,
    /// A face could not be tessellated.
    #[serde(rename = "TRUCK_TESSELLATION_FAILED")]
    TessellationFailed,
    /// The tessellation has zero or nonfinite signed volume.
    #[serde(rename = "TRUCK_DEGENERATE_MESH")]
    DegenerateMesh,
    /// The sampled shell boundaries intersect or touch.
    #[serde(rename = "TRUCK_SHELLS_INTERSECT")]
    ShellsIntersect,
    /// The sampled shell containment is inconsistent.
    #[serde(rename = "TRUCK_INCONSISTENT_SHELL_NESTING")]
    InconsistentNesting,
    /// Shell orientation does not alternate with nesting.
    #[serde(rename = "TRUCK_INCONSISTENT_SHELL_ORIENTATION")]
    InconsistentOrientation,
    /// The sampled geometry could not be classified.
    #[serde(rename = "TRUCK_CLASSIFICATION_FAILED")]
    ClassificationFailed,
    /// The requested result has no supported bounded representation.
    #[serde(rename = "TRUCK_UNBOUNDED_RESULT")]
    UnboundedResult,
    /// Intersection construction failed.
    #[serde(rename = "TRUCK_INTERSECTION_FAILED")]
    IntersectionFailed,
    /// A face could not be divided along its intersection loops.
    #[serde(rename = "TRUCK_FACE_DIVISION_FAILED")]
    FaceDivisionFailed,
    /// An intersection curve could not be fitted.
    #[serde(rename = "TRUCK_CURVE_FITTING_FAILED")]
    CurveFittingFailed,
    /// A point could not be projected onto the required geometry.
    #[serde(rename = "TRUCK_PROJECTION_FAILED")]
    ProjectionFailed,
    /// The constructed curve has too few distinct points.
    #[serde(rename = "TRUCK_DEGENERATE_CURVE")]
    DegenerateCurve,
    /// This fillet operation requires planar faces.
    #[serde(rename = "TRUCK_FILLET_NON_PLANAR_FACE")]
    NonPlanarFace,
    /// The geometry is outside the supported scope of this operation.
    #[serde(rename = "TRUCK_UNSUPPORTED_GEOMETRY")]
    UnsupportedGeometry,
    /// The topology is outside the supported scope of this operation.
    #[serde(rename = "TRUCK_UNSUPPORTED_TOPOLOGY")]
    UnsupportedTopology,
    /// Blend construction failed; the numerical or geometric constraint could not be resolved.
    #[serde(rename = "TRUCK_BLEND_CONSTRUCTION_FAILED")]
    BlendConstructionFailed,
    /// No usable intersection was found in the sampled domains.
    #[serde(rename = "TRUCK_NO_INTERSECTION")]
    NoIntersection,
    /// The moved boundary leaves its neighbouring face.
    #[serde(rename = "TRUCK_OUTSIDE_NEIGHBOUR")]
    OutsideNeighbour,
    /// The requested surface offset cannot be constructed.
    #[serde(rename = "TRUCK_NO_OFFSET")]
    NoOffset,
    /// This operation does not support crossing a concave edge.
    #[serde(rename = "TRUCK_CONCAVE_EDGE")]
    ConcaveEdge,
    /// The STEP exchange structure could not be parsed.
    #[serde(rename = "TRUCK_STEP_SYNTAX")]
    StepSyntax,
    /// The STEP file has no DATA section.
    #[serde(rename = "TRUCK_STEP_NO_DATA")]
    StepNoData,
    /// The STEP entity type is not implemented.
    #[serde(rename = "TRUCK_STEP_UNSUPPORTED_ENTITY")]
    StepUnsupportedEntity,
    /// The STEP entity could not be deserialized.
    #[serde(rename = "TRUCK_STEP_UNREADABLE_ENTITY")]
    StepUnreadableEntity,
    /// A referenced STEP entity could not be resolved.
    #[serde(rename = "TRUCK_STEP_MISSING_REFERENCE")]
    StepMissingReference,
    /// A required STEP entity was skipped.
    #[serde(rename = "TRUCK_STEP_SKIPPED_DEPENDENCY")]
    StepSkippedDependency,
    /// STEP geometry conversion failed.
    #[serde(rename = "TRUCK_STEP_CONVERSION_FAILED")]
    StepConversionFailed,
    /// The operation omitted geometry; inspect the reported diagnostics.
    #[serde(rename = "TRUCK_INCOMPLETE_RESULT")]
    IncompleteResult,
    /// The shape JSON could not be read.
    #[serde(rename = "TRUCK_INVALID_JSON")]
    InvalidJson,
    /// Mesh input or output failed.
    #[serde(rename = "TRUCK_MESH_IO")]
    MeshIo,
}

impl Code {
    /// Stable external identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidTolerance => "TRUCK_INVALID_TOLERANCE",
            Self::InvalidParameter => "TRUCK_INVALID_PARAMETER",
            Self::UnknownEdge => "TRUCK_SELECTION_UNKNOWN_EDGE",
            Self::UnknownFace => "TRUCK_SELECTION_UNKNOWN_FACE",
            Self::EmptySelection => "TRUCK_SELECTION_EMPTY",
            Self::DuplicateSelection => "TRUCK_SELECTION_DUPLICATE",
            Self::MultipleShells => "TRUCK_SELECTION_MULTIPLE_SHELLS",
            Self::InvalidInputTopology => "TRUCK_INVALID_INPUT_TOPOLOGY",
            Self::InvalidOutputTopology => "TRUCK_INVALID_OUTPUT_TOPOLOGY",
            Self::TessellationFailed => "TRUCK_TESSELLATION_FAILED",
            Self::DegenerateMesh => "TRUCK_DEGENERATE_MESH",
            Self::ShellsIntersect => "TRUCK_SHELLS_INTERSECT",
            Self::InconsistentNesting => "TRUCK_INCONSISTENT_SHELL_NESTING",
            Self::InconsistentOrientation => "TRUCK_INCONSISTENT_SHELL_ORIENTATION",
            Self::ClassificationFailed => "TRUCK_CLASSIFICATION_FAILED",
            Self::UnboundedResult => "TRUCK_UNBOUNDED_RESULT",
            Self::IntersectionFailed => "TRUCK_INTERSECTION_FAILED",
            Self::FaceDivisionFailed => "TRUCK_FACE_DIVISION_FAILED",
            Self::CurveFittingFailed => "TRUCK_CURVE_FITTING_FAILED",
            Self::ProjectionFailed => "TRUCK_PROJECTION_FAILED",
            Self::DegenerateCurve => "TRUCK_DEGENERATE_CURVE",
            Self::NonPlanarFace => "TRUCK_FILLET_NON_PLANAR_FACE",
            Self::UnsupportedGeometry => "TRUCK_UNSUPPORTED_GEOMETRY",
            Self::UnsupportedTopology => "TRUCK_UNSUPPORTED_TOPOLOGY",
            Self::BlendConstructionFailed => "TRUCK_BLEND_CONSTRUCTION_FAILED",
            Self::NoIntersection => "TRUCK_NO_INTERSECTION",
            Self::OutsideNeighbour => "TRUCK_OUTSIDE_NEIGHBOUR",
            Self::NoOffset => "TRUCK_NO_OFFSET",
            Self::ConcaveEdge => "TRUCK_CONCAVE_EDGE",
            Self::StepSyntax => "TRUCK_STEP_SYNTAX",
            Self::StepNoData => "TRUCK_STEP_NO_DATA",
            Self::StepUnsupportedEntity => "TRUCK_STEP_UNSUPPORTED_ENTITY",
            Self::StepUnreadableEntity => "TRUCK_STEP_UNREADABLE_ENTITY",
            Self::StepMissingReference => "TRUCK_STEP_MISSING_REFERENCE",
            Self::StepSkippedDependency => "TRUCK_STEP_SKIPPED_DEPENDENCY",
            Self::StepConversionFailed => "TRUCK_STEP_CONVERSION_FAILED",
            Self::IncompleteResult => "TRUCK_INCOMPLETE_RESULT",
            Self::InvalidJson => "TRUCK_INVALID_JSON",
            Self::MeshIo => "TRUCK_MESH_IO",
        }
    }
    /// Default explanation of the observed condition.
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidTolerance => "The tolerance must be finite and positive.",
            Self::InvalidParameter => "The parameter is invalid.",
            Self::UnknownEdge => "The selected edge does not belong to the input.",
            Self::UnknownFace => "The selected face does not belong to the input.",
            Self::EmptySelection => "This operation requires a nonempty selection.",
            Self::DuplicateSelection => "The selection contains duplicate entities.",
            Self::MultipleShells => "The selected edges must belong to one shell.",
            Self::InvalidInputTopology => "The input topology is invalid.",
            Self::InvalidOutputTopology => "The constructed topology is invalid.",
            Self::TessellationFailed => "A face could not be tessellated.",
            Self::DegenerateMesh => "The tessellation has zero or nonfinite signed volume.",
            Self::ShellsIntersect => "The sampled shell boundaries intersect or touch.",
            Self::InconsistentNesting => "The sampled shell containment is inconsistent.",
            Self::InconsistentOrientation => "Shell orientation does not alternate with nesting.",
            Self::ClassificationFailed => "The sampled geometry could not be classified.",
            Self::UnboundedResult => "The requested result has no supported bounded representation.",
            Self::IntersectionFailed => "Intersection construction failed.",
            Self::FaceDivisionFailed => "A face could not be divided along its intersection loops.",
            Self::CurveFittingFailed => "An intersection curve could not be fitted.",
            Self::ProjectionFailed => "A point could not be projected onto the required geometry.",
            Self::DegenerateCurve => "The constructed curve has too few distinct points.",
            Self::NonPlanarFace => "This fillet operation requires planar faces.",
            Self::UnsupportedGeometry => "The geometry is outside the supported scope of this operation.",
            Self::UnsupportedTopology => "The topology is outside the supported scope of this operation.",
            Self::BlendConstructionFailed => "Blend construction failed; the numerical or geometric constraint could not be resolved.",
            Self::NoIntersection => "No usable intersection was found in the sampled domains.",
            Self::OutsideNeighbour => "The moved boundary leaves its neighbouring face.",
            Self::NoOffset => "The requested surface offset cannot be constructed.",
            Self::ConcaveEdge => "This operation does not support crossing a concave edge.",
            Self::StepSyntax => "The STEP exchange structure could not be parsed.",
            Self::StepNoData => "The STEP file has no DATA section.",
            Self::StepUnsupportedEntity => "The STEP entity type is not implemented.",
            Self::StepUnreadableEntity => "The STEP entity could not be deserialized.",
            Self::StepMissingReference => "A referenced STEP entity could not be resolved.",
            Self::StepSkippedDependency => "A required STEP entity was skipped.",
            Self::StepConversionFailed => "STEP geometry conversion failed.",
            Self::IncompleteResult => "The operation omitted geometry; inspect the reported diagnostics.",
            Self::InvalidJson => "The shape JSON could not be read.",
            Self::MeshIo => "Mesh input or output failed.",
        }
    }
}

/// An error with a stable code, independent of its display text.
pub trait CodedError: Error {
    /// Stable external identifier.
    fn code(&self) -> &'static str;
}

/// Locations refer to the input of this invocation, unless `stage` is `validate_output`.
#[derive(Debug, Default, Serialize)]
pub struct Context {
    /// Zero-based operand index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<usize>,
    /// Zero-based shell index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_index: Option<usize>,
    /// Zero-based face index within the shell, or across all input faces when shell_index is absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_index: Option<usize>,
    /// Zero-based edge index in the current traversal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_index: Option<usize>,
    /// Zero-based index in the supplied selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_index: Option<usize>,
    /// Other operand for a face-pair diagnostic, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_operand: Option<usize>,
    /// Related entity index of the same kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_index: Option<usize>,
    /// STEP entity identifier; zero denotes an inline entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_entity_id")]
    pub entity_id: Option<u64>,
    /// STEP identifier of a required entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_entity_id")]
    pub related_entity_id: Option<u64>,
    /// STEP entity type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    /// Name of the invalid parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<&'static str>,
    /// Parameter value; text preserves NaN and infinities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Parameter station at which a numerical check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<f64>,
}

/// A serializable immediate cause with a stable code.
#[derive(Debug, Serialize)]
pub struct Cause {
    /// Stable code of the underlying error.
    pub code: &'static str,
    /// Human-readable underlying explanation.
    pub message: String,
}

/// An operation failure with its original Rust source preserved.
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    /// Stable failure condition.
    pub code: Code,
    /// Public operation requested by the caller.
    pub operation: &'static str,
    /// Algorithm stage where the failure was observed.
    pub stage: &'static str,
    /// Human-readable explanation; do not parse this field.
    pub message: String,
    /// Relevant input locations and parameters.
    pub context: Box<Context>,
    /// Coded immediate cause, when the underlying error implements [`CodedError`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<Cause>>,
    /// Original error. Serialization includes its display text in `message`.
    #[serde(skip)]
    source: Option<Box<dyn Error>>,
}

impl Diagnostic {
    /// Creates a diagnostic at the point a condition is observed.
    pub fn new(code: Code, operation: &'static str, stage: &'static str) -> Self {
        Self {
            code,
            operation,
            stage,
            message: code.message().into(),
            context: Box::default(),
            source: None,
            cause: None,
        }
    }
    /// Retains an underlying error and includes its explanation.
    pub fn with_source(mut self, source: impl Error + 'static) -> Self {
        self.message = format!("{} {}", self.message, source);
        self.source = Some(Box::new(source));
        self
    }
    /// Retains the underlying error and exposes its stable code to serialized consumers.
    pub fn with_coded_source(mut self, source: impl CodedError + 'static) -> Self {
        self.cause = Some(Box::new(Cause {
            code: source.code(),
            message: source.to_string(),
        }));
        self.with_source(source)
    }
    /// Retains an already boxed error without losing its dynamic type.
    pub fn with_boxed_source(mut self, source: Box<dyn Error>) -> Self {
        self.message = format!("{} {}", self.message, source);
        self.source = Some(source);
        self
    }
    /// Records the operand index.
    pub fn operand(mut self, index: usize) -> Self {
        self.context.operand = Some(index);
        self
    }
    /// Records the shell index.
    pub fn shell(mut self, index: usize) -> Self {
        self.context.shell_index = Some(index);
        self
    }
    /// Records the face index.
    pub fn face(mut self, index: usize) -> Self {
        self.context.face_index = Some(index);
        self
    }
    /// Records the index in the supplied selection.
    pub fn selection(mut self, index: usize) -> Self {
        self.context.selection_index = Some(index);
        self
    }
    /// Records a related entity index.
    pub fn related(mut self, index: usize) -> Self {
        self.context.related_index = Some(index);
        self
    }
    /// Records an invalid parameter, including nonfinite values.
    pub fn parameter(mut self, name: &'static str, value: impl fmt::Display) -> Self {
        self.context.parameter = Some(name);
        self.context.value = Some(value.to_string());
        self
    }
    /// Records the algorithm stage at which a helper failed.
    pub fn stage(mut self, stage: &'static str) -> Self {
        self.stage = stage;
        self
    }
    /// Keeps the failure stage and cause while recording the caller's public operation.
    pub fn operation(mut self, operation: &'static str) -> Self {
        self.operation = operation;
        self
    }
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({} / {})",
            self.code.as_str(),
            self.message,
            self.operation,
            self.stage
        )
    }
}
impl Error for Diagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> { self.source.as_deref() }
}
impl CodedError for Diagnostic {
    fn code(&self) -> &'static str { self.code.as_str() }
}

/// Geometry returned together with every omission detected during conversion or meshing.
#[derive(Debug, Serialize)]
pub struct Report<T> {
    /// Complete or partial output.
    pub value: T,
    /// Detected omissions. Empty means no omissions were reported, not geometric certification.
    pub diagnostics: Vec<Diagnostic>,
}
impl<T> Report<T> {
    /// Whether no omissions were reported.
    pub fn is_complete(&self) -> bool { self.diagnostics.is_empty() }
    /// Rejects partial output, returning all omission diagnostics.
    pub fn into_complete(self) -> Result<T, Vec<Diagnostic>> {
        if self.is_complete() {
            Ok(self.value)
        } else {
            Err(self.diagnostics)
        }
    }
}

/// Validates a finite, positive tolerance before entering numerical routines.
pub fn validate_tolerance(value: f64, operation: &'static str) -> Result<(), Diagnostic> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(
            Diagnostic::new(Code::InvalidTolerance, operation, "validate_input")
                .parameter("tol", value),
        )
    }
}

fn serialize_entity_id<S: serde::Serializer>(
    id: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match id {
        Some(id) => serializer.serialize_str(&id.to_string()),
        None => serializer.serialize_none(),
    }
}
