//! Compiler for the checked M3 Zenith-to-Apex baseline.

pub mod apex_api;
pub mod apex_ir;
pub mod ast;
pub mod check;
pub mod compiler;
pub mod diagnostic;
pub mod emit;
pub mod hir;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod project;
pub mod source;
pub mod token;
pub mod types;
pub mod verify;

pub use ast::{
    AccessorKind, Block, ClassDeclaration, ClassMember, CompilationUnit, ConstructorDeclaration,
    Expression, ExpressionKind, FieldDeclaration, ForInitializer, MethodDeclaration, Modifier,
    Operator, Parameter, PropertyAccessor, PropertyDeclaration, Statement, StatementKind, Type,
    TypeName, VariableDeclaration, Visitor, render_ast, walk_class_declaration, walk_class_member,
    walk_compilation_unit, walk_constructor_declaration, walk_expression, walk_field_declaration,
    walk_method_declaration, walk_parameter, walk_property_accessor, walk_property_declaration,
    walk_statement, walk_type, walk_variable_declaration,
};
pub use compiler::{Compilation, compile_project};
pub use diagnostic::{Diagnostic, Phase, Severity, SourceLabel, render_diagnostics};
pub use emit::{Artifact, record_verification, render_artifacts, write_artifacts};
pub use lexer::{LexResult, lex};
pub use parser::{ParseResult, parse};
pub use source::{SourceFile, SourceId, SourceLocation, SourceMap, Span};
pub use token::{Identifier, Token, TokenKind, render_tokens};
pub use types::Type as CheckedType;
pub use verify::{
    APEX_EXEC_M3_PROFILE, APEX_EXEC_M3_REVISION, ProcessVerifier, VerificationOutcome,
    VerificationResult,
};
