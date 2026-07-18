use crate::source::Span;
use crate::types::Type;
use crate::{Diagnostic, Phase};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Class {
    pub name: String,
    pub modifiers: Vec<String>,
    pub members: Vec<Member>,
    pub source_path: String,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Member {
    Field(Field),
    Property(Property),
    Method(Method),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub modifiers: Vec<String>,
    pub ty: Type,
    pub initializer: Option<Expression>,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    pub name: String,
    pub modifiers: Vec<String>,
    pub ty: Type,
    pub accessors: Vec<Accessor>,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Accessor {
    pub kind: String,
    pub modifiers: Vec<String>,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Method {
    pub name: String,
    pub modifiers: Vec<String>,
    pub return_type: Type,
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatementKind {
    Block(Block),
    Variable {
        ty: Type,
        name: String,
        initializer: Option<Expression>,
    },
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        initializer: Option<ForInitializer>,
        condition: Option<Expression>,
        update: Vec<Expression>,
        body: Box<Statement>,
    },
    EnhancedFor {
        ty: Type,
        name: String,
        iterable: Expression,
        body: Box<Statement>,
    },
    Return(Option<Expression>),
    Break,
    Continue,
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForInitializer {
    Variable {
        ty: Type,
        name: String,
        initializer: Option<Box<Expression>>,
    },
    Expressions(Vec<Expression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub origin: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Name(String),
    Integer(String),
    String(String),
    Boolean(bool),
    Null,
    This,
    Parenthesized(Box<Expression>),
    Call {
        receiver: Option<Box<Expression>>,
        name: String,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        name: String,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Unary {
        operator: String,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expression: Box<Expression>,
        else_expression: Box<Expression>,
    },
    Assignment {
        target: Box<Expression>,
        operator: String,
        value: Box<Expression>,
    },
}

pub fn validate(program: &Program) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for class in &program.classes {
        if class.name.is_empty()
            || class
                .name
                .to_ascii_lowercase()
                .starts_with("zenithgenerated_")
        {
            diagnostics.push(invalid_ir(
                class.origin,
                "Apex IR contains an invalid or reserved class name",
            ));
        }
        for member in &class.members {
            match member {
                Member::Field(field) if matches!(field.ty, Type::Void | Type::Error) => {
                    diagnostics.push(invalid_ir(
                        field.origin,
                        "Apex IR field has a non-emittable type",
                    ));
                }
                Member::Property(property) if matches!(property.ty, Type::Void | Type::Error) => {
                    diagnostics.push(invalid_ir(
                        property.origin,
                        "Apex IR property has a non-emittable type",
                    ));
                }
                Member::Method(method) => {
                    if method.return_type == Type::Error
                        || method
                            .parameters
                            .iter()
                            .any(|parameter| matches!(parameter.ty, Type::Void | Type::Error))
                    {
                        diagnostics.push(invalid_ir(
                            method.origin,
                            "Apex IR method has a non-emittable signature",
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    diagnostics
}

fn invalid_ir(span: Span, message: &str) -> Diagnostic {
    Diagnostic::coded_error(Phase::Lower, "lower.invalid-apex-ir", message, Some(span))
}

#[cfg(test)]
mod tests {
    use super::{Class, Program, validate};
    use crate::source::{SourceId, Span};

    #[test]
    fn validation_rejects_reserved_generated_names() {
        let span = Span::new(SourceId::from_raw(0), 0, 1).unwrap();
        let diagnostics = validate(&Program {
            classes: vec![Class {
                name: "ZenithGenerated_Collision".into(),
                modifiers: vec![],
                members: vec![],
                source_path: "src/Bad.zen".into(),
                origin: span,
            }],
        });
        assert_eq!(diagnostics[0].code, "lower.invalid-apex-ir");
    }
}
