//! Compiler foundations for the Zenith language.
//!
//! The bootstrap milestone is complete. Source identity and diagnostic
//! foundations now support the active M1 lexer work without coupling source
//! handling to later compiler phases.

pub mod diagnostic;
pub mod lexer;
pub mod source;
pub mod token;

pub use diagnostic::{Diagnostic, Phase, Severity, SourceLabel, render_diagnostics};
pub use lexer::{LexResult, lex};
pub use source::{SourceFile, SourceId, SourceLocation, SourceMap, Span};
pub use token::{Identifier, Token, TokenKind, render_tokens};
