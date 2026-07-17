use std::fmt;

use crate::source::Span;

/// Compiler phase that owns a diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Source,
    Lex,
    Parse,
    Resolve,
    Type,
    Effect,
    Lower,
    Emit,
    Project,
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Source => "source",
            Self::Lex => "lex",
            Self::Parse => "parse",
            Self::Resolve => "resolve",
            Self::Type => "type",
            Self::Effect => "effect",
            Self::Lower => "lower",
            Self::Emit => "emit",
            Self::Project => "project",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub phase: Phase,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn error(phase: Phase, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Error,
            phase,
            message: message.into(),
            span,
        }
    }

    pub fn warning(phase: Phase, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: Severity::Warning,
            phase,
            message: message.into(),
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Phase, Severity};

    #[test]
    fn diagnostics_retain_phase_and_severity() {
        let diagnostic = Diagnostic::error(Phase::Type, "nullable value used here", None);

        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.phase, Phase::Type);
        assert_eq!(diagnostic.message, "nullable value used here");
        assert_eq!(diagnostic.phase.to_string(), "type");
    }
}
