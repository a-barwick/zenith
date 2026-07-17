//! Compiler foundations for the Zenith language.
//!
//! The repository is intentionally at its bootstrap milestone. Source identity
//! and diagnostics exist so the lexer can be implemented without coupling
//! source handling to later compiler phases.

pub mod diagnostic;
pub mod source;

pub use diagnostic::{Diagnostic, Phase, Severity};
pub use source::{SourceFile, SourceId, SourceMap, Span};
