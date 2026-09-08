//! Coded STEP diagnostics, including dependencies of omitted geometry.
use super::*;
use std::{error::Error, fmt};
use truck_base::diagnostics::{Code, CodedError, Diagnostic, Report};

/// A conversion failure whose condition and referenced entity are known.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConversionError {
    /// An unimplemented entity type.
    Unsupported { id: u64, entity: String },
    /// A record failed deserialization.
    Unreadable { id: u64, message: String },
    /// A numbered entity is absent.
    Missing { id: u64 },
    /// An entity depends on geometry already omitted.
    SkippedDependency { id: u64, entity: &'static str },
    /// A required reference or representation item cannot be resolved.
    Reference { message: String },
}
impl ConversionError {
    /// Stable failure condition.
    pub fn code(&self) -> Code {
        match self {
            Self::Unsupported { .. } => Code::StepUnsupportedEntity,
            Self::Unreadable { .. } => Code::StepUnreadableEntity,
            Self::Missing { .. } | Self::Reference { .. } => Code::StepMissingReference,
            Self::SkippedDependency { .. } => Code::StepSkippedDependency,
        }
    }
    /// Referenced entity, if this failure identifies one.
    pub fn entity_id(&self) -> Option<u64> {
        match self {
            Self::Unsupported { id, .. }
            | Self::Unreadable { id, .. }
            | Self::Missing { id }
            | Self::SkippedDependency { id, .. } => Some(*id),
            Self::Reference { .. } => None,
        }
    }
}
impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { id, entity } => {
                write!(f, "#{id} is {entity}, which is not implemented")
            }
            Self::Unreadable { id, message } => write!(f, "#{id} could not be read: {message}"),
            Self::Missing { id } => write!(f, "STEP entity #{id} could not be resolved"),
            Self::SkippedDependency { id, entity } => write!(f, "{entity} #{id} was skipped"),
            Self::Reference { message } => f.write_str(message),
        }
    }
}
impl Error for ConversionError {}
impl CodedError for ConversionError {
    fn code(&self) -> &'static str { self.code().as_str() }
}
impl CodedError for ParseError {
    fn code(&self) -> &'static str {
        match self {
            Self::Syntax(_) => Code::StepSyntax,
            Self::NoDataSection => Code::StepNoData,
        }
        .as_str()
    }
}

/// Converts a legacy boxed failure without inferring its type from display text.
pub fn conversion_diagnostic(error: StepConvertingError, operation: &'static str) -> Diagnostic {
    let known = error.downcast_ref::<ConversionError>();
    let code = known.map_or(Code::StepConversionFailed, ConversionError::code);
    let mut diagnostic = Diagnostic::new(code, operation, "convert_geometry");
    diagnostic.context.related_entity_id = known.and_then(ConversionError::entity_id);
    diagnostic.with_boxed_source(error)
}
impl convert::Skipped {
    /// Stable reason code. Unknown third-party errors use the conversion-failed fallback.
    pub fn code(&self) -> &'static str {
        self.reason
            .downcast_ref::<ConversionError>()
            .map_or(Code::StepConversionFailed, ConversionError::code)
            .as_str()
    }
    /// Reports both the omitted entity and any known failed dependency.
    pub fn into_diagnostic(self) -> Diagnostic {
        let mut diagnostic = conversion_diagnostic(self.reason, "step_convert");
        diagnostic.context.entity_id = Some(self.id);
        diagnostic.context.entity_type = Some(self.entity.into());
        diagnostic
    }
}
impl Table {
    /// Parses STEP and returns unreadable-record diagnostics with the usable table.
    /// Unsupported entity types can also be inspected through [`Self::unsupported`].
    pub fn from_step_with_diagnostics(text: &str) -> Result<Report<Self>, Diagnostic> {
        let table = Self::try_from_step(text).map_err(|e| {
            let code = match e {
                ParseError::Syntax(_) => Code::StepSyntax,
                ParseError::NoDataSection => Code::StepNoData,
            };
            Diagnostic::new(code, "step_parse", "parse").with_coded_source(e)
        })?;
        let diagnostics = table.record_diagnostics();
        Ok(Report {
            value: table,
            diagnostics,
        })
    }
    /// Unreadable records, in file order. Legacy `errors` remains available.
    pub fn record_diagnostics(&self) -> Vec<Diagnostic> {
        self.errors
            .iter()
            .map(|(id, message)| {
                let mut error = Diagnostic::new(
                    Code::StepUnreadableEntity,
                    "step_parse",
                    "deserialize_entity",
                );
                error.context.entity_id = Some(*id);
                error.message = message.clone();
                error
            })
            .collect()
    }
    /// Converts a shell while explicitly reporting every detected omission.
    pub fn to_compressed_shell_with_diagnostics(
        &self,
        shell: &impl convert::StepShell,
    ) -> Result<Report<CompressedShell<truck::Point3, Curve3D, Surface>>, Diagnostic> {
        let (value, skipped) = self
            .to_compressed_shell(shell)
            .map_err(|e| conversion_diagnostic(e, "step_convert_shell"))?;
        Ok(Report {
            value,
            diagnostics: skipped
                .into_iter()
                .map(convert::Skipped::into_diagnostic)
                .collect(),
        })
    }
    /// Converts a solid while explicitly reporting every detected omission.
    pub fn to_compressed_solid_with_diagnostics(
        &self,
        solid: &ManifoldSolidBrepHolder,
    ) -> Result<Report<CompressedSolid<truck::Point3, Curve3D, Surface>>, Diagnostic> {
        let (value, skipped) = self
            .to_compressed_solid(solid)
            .map_err(|e| conversion_diagnostic(e, "step_convert_solid"))?;
        Ok(Report {
            value,
            diagnostics: skipped
                .into_iter()
                .map(convert::Skipped::into_diagnostic)
                .collect(),
        })
    }
}
