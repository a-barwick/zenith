//! Compiler foundations for the Zenith language.
//!
//! The bootstrap milestone is complete. Source identity and diagnostic
//! foundations now support the active M1 lexer work without coupling source
//! handling to later compiler phases.

pub mod diagnostic;
pub mod source;

pub use diagnostic::{Diagnostic, Phase, Severity};
pub use source::{SourceFile, SourceId, SourceMap, Span};
