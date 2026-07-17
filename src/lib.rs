//! Compiler front end for the Zenith language.
//!
//! The completed M2 surface loads UTF-8 source, tokenizes the selected lexical
//! baseline, and parses immutable source-spanned syntax without coupling the
//! AST to resolution, typing, lowering, or emission.

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
    TypeName, VariableDeclaration, Visitor, render_ast, walk_class_declaration, walk_class_member,
    walk_compilation_unit, walk_constructor_declaration, walk_expression, walk_field_declaration,
    walk_method_declaration, walk_parameter, walk_property_accessor, walk_property_declaration,
    walk_statement, walk_type, walk_variable_declaration,
};
pub use diagnostic::{Diagnostic, Phase, Severity, SourceLabel, render_diagnostics};
pub use lexer::{LexResult, lex};
pub use parser::{ParseResult, parse};
pub use source::{SourceFile, SourceId, SourceLocation, SourceMap, Span};
pub use token::{Identifier, Token, TokenKind, render_tokens};
