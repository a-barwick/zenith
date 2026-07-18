use crate::source::Span;
use crate::types::Type;

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
    pub span: Span,
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
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    pub name: String,
    pub modifiers: Vec<String>,
    pub ty: Type,
    pub accessors: Vec<Accessor>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Accessor {
    pub kind: String,
    pub modifiers: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Method {
    pub name: String,
    pub modifiers: Vec<String>,
    pub return_type: Type,
    pub parameters: Vec<Parameter>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
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
    pub ty: Type,
    pub span: Span,
    pub assignable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Name {
        spelling: String,
        target: ValueTarget,
    },
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
        target: CallTarget,
    },
    Member {
        object: Box<Expression>,
        name: String,
        target: ValueTarget,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        target: IndexTarget,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueTarget {
    Local,
    Parameter,
    Field { owner: String, is_static: bool },
    Property { owner: String, is_static: bool },
    Class { external: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallTarget {
    Project {
        owner: String,
        is_static: bool,
        parameter_types: Vec<Type>,
    },
    External {
        owner: String,
        is_static: bool,
        parameter_types: Vec<Type>,
        effects_unknown: bool,
    },
    Collection {
        collection: String,
        parameter_types: Vec<Type>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexTarget {
    List,
}
