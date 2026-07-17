//! Compiler foundations for the Zenith language.
//!
//! The bootstrap milestone is complete. Source identity and diagnostic
//! foundations now support the active M1 lexer work without coupling source
//! handling to later compiler phases.

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod token;

pub use ast::{
    AccessorKind, Block, ClassDeclaration, ClassMember, CompilationUnit, ConstructorDeclaration,
    Expression, ExpressionKind, FieldDeclaration, ForInitializer, MethodDeclaration, Modifier,
    Operator, Parameter, PropertyAccessor, PropertyDeclaration, Statement, StatementKind, Type,
    TypeName, VariableDeclaration, Visitor, render_ast,
};
pub use diagnostic::{Diagnostic, Phase, Severity, SourceLabel, render_diagnostics};
pub use lexer::{LexResult, lex};
pub use parser::{ParseResult, parse};
pub use source::{SourceFile, SourceId, SourceLocation, SourceMap, Span};
pub use token::{Identifier, Token, TokenKind, render_tokens};
