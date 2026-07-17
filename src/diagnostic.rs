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
    Schema,
    Effect,
    Lower,
    Emit,
    Verify,
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
            Self::Schema => "schema",
            Self::Effect => "effect",
            Self::Lower => "lower",
            Self::Emit => "emit",
            Self::Verify => "verify",
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

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
        })
    }
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
    fn renders_every_phase_name() {
        let phases = [
            (Phase::Source, "source"),
            (Phase::Lex, "lex"),
            (Phase::Parse, "parse"),
            (Phase::Resolve, "resolve"),
            (Phase::Type, "type"),
            (Phase::Schema, "schema"),
            (Phase::Effect, "effect"),
            (Phase::Lower, "lower"),
            (Phase::Emit, "emit"),
            (Phase::Verify, "verify"),
            (Phase::Project, "project"),
        ];

        for (phase, expected) in phases {
            assert_eq!(phase.to_string(), expected);
        }
    }

    #[test]
    fn diagnostics_retain_phase_and_severity() {
        let error = Diagnostic::error(Phase::Type, "nullable value used here", None);
        let warning = Diagnostic::warning(Phase::Verify, "backend unavailable", None);

        assert_eq!(error.severity, Severity::Error);
        assert_eq!(error.phase, Phase::Type);
        assert_eq!(error.message, "nullable value used here");
        assert_eq!(error.severity.to_string(), "error");
        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(warning.phase, Phase::Verify);
        assert_eq!(warning.severity.to_string(), "warning");
    }
}
