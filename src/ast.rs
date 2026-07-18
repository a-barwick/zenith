use std::fmt::Write;

use crate::source::{SourceFile, Span};
use crate::token::Identifier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationUnit {
    declarations: Vec<ClassDeclaration>,
    span: Span,
}

impl CompilationUnit {
    pub(crate) fn new(declarations: Vec<ClassDeclaration>, span: Span) -> Self {
        Self { declarations, span }
    }

    pub fn declarations(&self) -> &[ClassDeclaration] {
        &self.declarations
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn visit<V: Visitor + ?Sized>(&self, visitor: &mut V) {
        visitor.visit_compilation_unit(self);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Modifier {
    spelling: String,
    canonical: String,
    span: Span,
}

impl Modifier {
    pub(crate) fn new(spelling: String, canonical: String, span: Span) -> Self {
        Self {
            spelling,
            canonical,
            span,
        }
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassDeclaration {
    modifiers: Vec<Modifier>,
    name: Identifier,
    kind: DeclarationKind,
    extends: Option<Type>,
    implements: Vec<Type>,
    members: Vec<ClassMember>,
    span: Span,
}

impl ClassDeclaration {
    pub(crate) fn new(
        modifiers: Vec<Modifier>,
        name: Identifier,
        extends: Option<Type>,
        implements: Vec<Type>,
        members: Vec<ClassMember>,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            name,
            kind: DeclarationKind::Class,
            extends,
            implements,
            members,
            span,
        }
    }

    pub(crate) fn new_record(
        modifiers: Vec<Modifier>,
        name: Identifier,
        components: Vec<RecordComponent>,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            name,
            kind: DeclarationKind::Record { components },
            extends: None,
            implements: Vec::new(),
            members: Vec::new(),
            span,
        }
    }

    pub(crate) fn new_result(
        modifiers: Vec<Modifier>,
        name: Identifier,
        variants: Vec<ResultVariant>,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            name,
            kind: DeclarationKind::SealedResult { variants },
            extends: None,
            implements: Vec::new(),
            members: Vec::new(),
            span,
        }
    }

    pub(crate) fn new_sobject(modifiers: Vec<Modifier>, name: Identifier, span: Span) -> Self {
        Self {
            modifiers,
            name,
            kind: DeclarationKind::SObject,
            extends: None,
            implements: Vec::new(),
            members: Vec::new(),
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn kind(&self) -> &DeclarationKind {
        &self.kind
    }

    pub const fn extends(&self) -> Option<&Type> {
        self.extends.as_ref()
    }

    pub fn implements(&self) -> &[Type] {
        &self.implements
    }

    pub fn members(&self) -> &[ClassMember] {
        &self.members
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationKind {
    Class,
    Record { components: Vec<RecordComponent> },
    SealedResult { variants: Vec<ResultVariant> },
    SObject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordComponent {
    ty: Type,
    name: Identifier,
    span: Span,
}

impl RecordComponent {
    pub(crate) fn new(ty: Type, name: Identifier, span: Span) -> Self {
        Self { ty, name, span }
    }

    pub const fn ty(&self) -> &Type {
        &self.ty
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultVariant {
    name: Identifier,
    payloads: Vec<RecordComponent>,
    span: Span,
}

impl ResultVariant {
    pub(crate) fn new(name: Identifier, payloads: Vec<RecordComponent>, span: Span) -> Self {
        Self {
            name,
            payloads,
            span,
        }
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn payloads(&self) -> &[RecordComponent] {
        &self.payloads
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassMember {
    Field(FieldDeclaration),
    Property(PropertyDeclaration),
    Method(MethodDeclaration),
    Constructor(ConstructorDeclaration),
}

impl ClassMember {
    pub const fn span(&self) -> Span {
        match self {
            Self::Field(declaration) => declaration.span(),
            Self::Property(declaration) => declaration.span(),
            Self::Method(declaration) => declaration.span(),
            Self::Constructor(declaration) => declaration.span(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDeclaration {
    modifiers: Vec<Modifier>,
    ty: Type,
    name: Identifier,
    initializer: Option<Expression>,
    span: Span,
}

impl FieldDeclaration {
    pub(crate) fn new(
        modifiers: Vec<Modifier>,
        ty: Type,
        name: Identifier,
        initializer: Option<Expression>,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            ty,
            name,
            initializer,
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn ty(&self) -> &Type {
        &self.ty
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyDeclaration {
    modifiers: Vec<Modifier>,
    ty: Type,
    name: Identifier,
    accessors: Vec<PropertyAccessor>,
    span: Span,
}

impl PropertyDeclaration {
    pub(crate) fn new(
        modifiers: Vec<Modifier>,
        ty: Type,
        name: Identifier,
        accessors: Vec<PropertyAccessor>,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            ty,
            name,
            accessors,
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn ty(&self) -> &Type {
        &self.ty
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn accessors(&self) -> &[PropertyAccessor] {
        &self.accessors
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyAccessor {
    modifiers: Vec<Modifier>,
    kind: AccessorKind,
    span: Span,
}

impl PropertyAccessor {
    pub(crate) fn new(modifiers: Vec<Modifier>, kind: AccessorKind, span: Span) -> Self {
        Self {
            modifiers,
            kind,
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn kind(&self) -> AccessorKind {
        self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessorKind {
    Get,
    Set,
}

impl AccessorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Set => "set",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodDeclaration {
    modifiers: Vec<Modifier>,
    return_type: Type,
    name: Identifier,
    parameters: Vec<Parameter>,
    body: Block,
    span: Span,
}

impl MethodDeclaration {
    pub(crate) fn new(
        modifiers: Vec<Modifier>,
        return_type: Type,
        name: Identifier,
        parameters: Vec<Parameter>,
        body: Block,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            return_type,
            name,
            parameters,
            body,
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn return_type(&self) -> &Type {
        &self.return_type
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub const fn body(&self) -> &Block {
        &self.body
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstructorDeclaration {
    modifiers: Vec<Modifier>,
    name: Identifier,
    parameters: Vec<Parameter>,
    body: Block,
    span: Span,
}

impl ConstructorDeclaration {
    pub(crate) fn new(
        modifiers: Vec<Modifier>,
        name: Identifier,
        parameters: Vec<Parameter>,
        body: Block,
        span: Span,
    ) -> Self {
        Self {
            modifiers,
            name,
            parameters,
            body,
            span,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub const fn body(&self) -> &Block {
        &self.body
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    ty: Type,
    name: Identifier,
    span: Span,
}

impl Parameter {
    pub(crate) fn new(ty: Type, name: Identifier, span: Span) -> Self {
        Self { ty, name, span }
    }

    pub const fn ty(&self) -> &Type {
        &self.ty
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    name: TypeName,
    arguments: Vec<Type>,
    nullable_suffixes: usize,
    span: Span,
}

impl Type {
    pub(crate) fn new(
        name: TypeName,
        arguments: Vec<Type>,
        nullable_suffixes: usize,
        span: Span,
    ) -> Self {
        Self {
            name,
            arguments,
            nullable_suffixes,
            span,
        }
    }

    pub const fn name(&self) -> &TypeName {
        &self.name
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub const fn is_nullable(&self) -> bool {
        self.nullable_suffixes != 0
    }

    pub const fn nullable_suffixes(&self) -> usize {
        self.nullable_suffixes
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn display_name(&self) -> String {
        let mut rendered = self.name.spelling().to_owned();
        if !self.arguments.is_empty() {
            rendered.push('<');
            for (index, argument) in self.arguments.iter().enumerate() {
                if index > 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&argument.display_name());
            }
            rendered.push('>');
        }
        for _ in 0..self.nullable_suffixes {
            rendered.push('?');
        }
        rendered
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeName {
    Identifier(Identifier),
    Keyword {
        spelling: String,
        canonical: String,
        span: Span,
    },
}

impl TypeName {
    pub fn spelling(&self) -> &str {
        match self {
            Self::Identifier(identifier) => identifier.spelling(),
            Self::Keyword { spelling, .. } => spelling,
        }
    }

    pub fn canonical(&self) -> &str {
        match self {
            Self::Identifier(identifier) => identifier.canonical(),
            Self::Keyword { canonical, .. } => canonical,
        }
    }

    pub const fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span(),
            Self::Keyword { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    span: Span,
}

impl Block {
    pub(crate) fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statement {
    kind: StatementKind,
    span: Span,
}

impl Statement {
    pub(crate) fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &StatementKind {
        &self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatementKind {
    Block(Block),
    Variable(VariableDeclaration),
    Let(LetDeclaration),
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
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    For {
        initializer: Option<ForInitializer>,
        condition: Option<Expression>,
        update: Vec<Expression>,
        body: Box<Statement>,
    },
    EnhancedFor {
        variable: VariableDeclaration,
        iterable: Expression,
        body: Box<Statement>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
    },
    Return(Option<Expression>),
    Throw(Expression),
    Break,
    Continue,
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LetDeclaration {
    name: Identifier,
    initializer: Expression,
    span: Span,
}

impl LetDeclaration {
    pub(crate) fn new(name: Identifier, initializer: Expression, span: Span) -> Self {
        Self {
            name,
            initializer,
            span,
        }
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn initializer(&self) -> &Expression {
        &self.initializer
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchArm {
    variant: Identifier,
    bindings: Vec<Identifier>,
    body: Block,
    span: Span,
}

impl MatchArm {
    pub(crate) fn new(
        variant: Identifier,
        bindings: Vec<Identifier>,
        body: Block,
        span: Span,
    ) -> Self {
        Self {
            variant,
            bindings,
            body,
            span,
        }
    }

    pub const fn variant(&self) -> &Identifier {
        &self.variant
    }

    pub fn bindings(&self) -> &[Identifier] {
        &self.bindings
    }

    pub const fn body(&self) -> &Block {
        &self.body
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForInitializer {
    Variable(Box<VariableDeclaration>),
    Expressions(Vec<Expression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableDeclaration {
    ty: Type,
    name: Identifier,
    initializer: Option<Expression>,
    span: Span,
}

impl VariableDeclaration {
    pub(crate) fn new(
        ty: Type,
        name: Identifier,
        initializer: Option<Expression>,
        span: Span,
    ) -> Self {
        Self {
            ty,
            name,
            initializer,
            span,
        }
    }

    pub const fn ty(&self) -> &Type {
        &self.ty
    }

    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    kind: ExpressionKind,
    span: Span,
}

impl Expression {
    pub(crate) fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &ExpressionKind {
        &self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Name(Identifier),
    Integer(String),
    String(String),
    Boolean(bool),
    Null,
    This,
    Super,
    Parenthesized(Box<Expression>),
    New {
        ty: Type,
        arguments: Vec<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        member: Identifier,
        safe: bool,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Postfix {
        operand: Box<Expression>,
        operator: Operator,
    },
    Unary {
        operator: Operator,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expression: Box<Expression>,
        else_expression: Box<Expression>,
    },
    Assignment {
        target: Box<Expression>,
        operator: Operator,
        value: Box<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operator {
    spelling: String,
    span: Span,
}

impl Operator {
    pub(crate) fn new(spelling: String, span: Span) -> Self {
        Self { spelling, span }
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

pub trait Visitor {
    fn visit_compilation_unit(&mut self, node: &CompilationUnit) {
        walk_compilation_unit(self, node);
    }

    fn visit_class_declaration(&mut self, node: &ClassDeclaration) {
        walk_class_declaration(self, node);
    }

    fn visit_modifier(&mut self, _node: &Modifier) {}

    fn visit_class_member(&mut self, node: &ClassMember) {
        walk_class_member(self, node);
    }

    fn visit_field_declaration(&mut self, node: &FieldDeclaration) {
        walk_field_declaration(self, node);
    }

    fn visit_property_declaration(&mut self, node: &PropertyDeclaration) {
        walk_property_declaration(self, node);
    }

    fn visit_property_accessor(&mut self, node: &PropertyAccessor) {
        walk_property_accessor(self, node);
    }

    fn visit_method_declaration(&mut self, node: &MethodDeclaration) {
        walk_method_declaration(self, node);
    }

    fn visit_constructor_declaration(&mut self, node: &ConstructorDeclaration) {
        walk_constructor_declaration(self, node);
    }

    fn visit_type(&mut self, node: &Type) {
        walk_type(self, node);
    }

    fn visit_parameter(&mut self, node: &Parameter) {
        walk_parameter(self, node);
    }

    fn visit_block(&mut self, node: &Block) {
        walk_block(self, node);
    }

    fn visit_statement(&mut self, node: &Statement) {
        walk_statement(self, node);
    }

    fn visit_variable_declaration(&mut self, node: &VariableDeclaration) {
        walk_variable_declaration(self, node);
    }

    fn visit_let_declaration(&mut self, node: &LetDeclaration) {
        walk_let_declaration(self, node);
    }

    fn visit_match_arm(&mut self, node: &MatchArm) {
        walk_match_arm(self, node);
    }

    fn visit_expression(&mut self, node: &Expression) {
        walk_expression(self, node);
    }
}

pub fn walk_compilation_unit<V: Visitor + ?Sized>(visitor: &mut V, node: &CompilationUnit) {
    for declaration in node.declarations() {
        visitor.visit_class_declaration(declaration);
    }
}

pub fn walk_class_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &ClassDeclaration) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
    match node.kind() {
        DeclarationKind::Class => {
            if let Some(extends) = node.extends() {
                visitor.visit_type(extends);
            }
            for implemented in node.implements() {
                visitor.visit_type(implemented);
            }
            for member in node.members() {
                visitor.visit_class_member(member);
            }
        }
        DeclarationKind::Record { components } => {
            for component in components {
                visitor.visit_type(component.ty());
            }
        }
        DeclarationKind::SealedResult { variants } => {
            for variant in variants {
                for payload in variant.payloads() {
                    visitor.visit_type(payload.ty());
                }
            }
        }
        DeclarationKind::SObject => {}
    }
}

pub fn walk_class_member<V: Visitor + ?Sized>(visitor: &mut V, node: &ClassMember) {
    match node {
        ClassMember::Field(field) => visitor.visit_field_declaration(field),
        ClassMember::Property(property) => visitor.visit_property_declaration(property),
        ClassMember::Method(method) => visitor.visit_method_declaration(method),
        ClassMember::Constructor(constructor) => visitor.visit_constructor_declaration(constructor),
    }
}

pub fn walk_field_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &FieldDeclaration) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
    visitor.visit_type(node.ty());
    if let Some(initializer) = node.initializer() {
        visitor.visit_expression(initializer);
    }
}

pub fn walk_property_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &PropertyDeclaration) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
    visitor.visit_type(node.ty());
    for accessor in node.accessors() {
        visitor.visit_property_accessor(accessor);
    }
}

pub fn walk_property_accessor<V: Visitor + ?Sized>(visitor: &mut V, node: &PropertyAccessor) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
}

pub fn walk_method_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &MethodDeclaration) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
    visitor.visit_type(node.return_type());
    for parameter in node.parameters() {
        visitor.visit_parameter(parameter);
    }
    visitor.visit_block(node.body());
}

pub fn walk_constructor_declaration<V: Visitor + ?Sized>(
    visitor: &mut V,
    node: &ConstructorDeclaration,
) {
    for modifier in node.modifiers() {
        visitor.visit_modifier(modifier);
    }
    for parameter in node.parameters() {
        visitor.visit_parameter(parameter);
    }
    visitor.visit_block(node.body());
}

pub fn walk_type<V: Visitor + ?Sized>(visitor: &mut V, node: &Type) {
    for argument in node.arguments() {
        visitor.visit_type(argument);
    }
}

pub fn walk_parameter<V: Visitor + ?Sized>(visitor: &mut V, node: &Parameter) {
    visitor.visit_type(node.ty());
}

pub fn walk_block<V: Visitor + ?Sized>(visitor: &mut V, node: &Block) {
    for statement in node.statements() {
        visitor.visit_statement(statement);
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, node: &Statement) {
    match node.kind() {
        StatementKind::Block(block) => visitor.visit_block(block),
        StatementKind::Variable(variable) => visitor.visit_variable_declaration(variable),
        StatementKind::Let(declaration) => visitor.visit_let_declaration(declaration),
        StatementKind::Expression(expression) | StatementKind::Throw(expression) => {
            visitor.visit_expression(expression);
        }
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visitor.visit_expression(condition);
            visitor.visit_statement(then_branch);
            if let Some(else_branch) = else_branch {
                visitor.visit_statement(else_branch);
            }
        }
        StatementKind::While { condition, body } => {
            visitor.visit_expression(condition);
            visitor.visit_statement(body);
        }
        StatementKind::DoWhile { body, condition } => {
            visitor.visit_statement(body);
            visitor.visit_expression(condition);
        }
        StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => {
            if let Some(initializer) = initializer {
                match initializer {
                    ForInitializer::Variable(variable) => {
                        visitor.visit_variable_declaration(variable);
                    }
                    ForInitializer::Expressions(expressions) => {
                        for expression in expressions {
                            visitor.visit_expression(expression);
                        }
                    }
                }
            }
            if let Some(condition) = condition {
                visitor.visit_expression(condition);
            }
            for expression in update {
                visitor.visit_expression(expression);
            }
            visitor.visit_statement(body);
        }
        StatementKind::EnhancedFor {
            variable,
            iterable,
            body,
        } => {
            visitor.visit_variable_declaration(variable);
            visitor.visit_expression(iterable);
            visitor.visit_statement(body);
        }
        StatementKind::Match { subject, arms } => {
            visitor.visit_expression(subject);
            for arm in arms {
                visitor.visit_match_arm(arm);
            }
        }
        StatementKind::Return(expression) => {
            if let Some(expression) = expression {
                visitor.visit_expression(expression);
            }
        }
        StatementKind::Break | StatementKind::Continue | StatementKind::Empty => {}
    }
}

pub fn walk_let_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &LetDeclaration) {
    visitor.visit_expression(node.initializer());
}

pub fn walk_match_arm<V: Visitor + ?Sized>(visitor: &mut V, node: &MatchArm) {
    visitor.visit_block(node.body());
}

pub fn walk_variable_declaration<V: Visitor + ?Sized>(visitor: &mut V, node: &VariableDeclaration) {
    visitor.visit_type(node.ty());
    if let Some(initializer) = node.initializer() {
        visitor.visit_expression(initializer);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(visitor: &mut V, node: &Expression) {
    match node.kind() {
        ExpressionKind::Parenthesized(expression) => visitor.visit_expression(expression),
        ExpressionKind::New { ty, arguments } => {
            visitor.visit_type(ty);
            for argument in arguments {
                visitor.visit_expression(argument);
            }
        }
        ExpressionKind::Call { callee, arguments } => {
            visitor.visit_expression(callee);
            for argument in arguments {
                visitor.visit_expression(argument);
            }
        }
        ExpressionKind::Member { object, .. }
        | ExpressionKind::Index { object, .. }
        | ExpressionKind::Postfix {
            operand: object, ..
        }
        | ExpressionKind::Unary {
            operand: object, ..
        } => visitor.visit_expression(object),
        ExpressionKind::Binary { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_expression(right);
        }
        ExpressionKind::Conditional {
            condition,
            then_expression,
            else_expression,
        } => {
            visitor.visit_expression(condition);
            visitor.visit_expression(then_expression);
            visitor.visit_expression(else_expression);
        }
        ExpressionKind::Assignment { target, value, .. } => {
            visitor.visit_expression(target);
            visitor.visit_expression(value);
        }
        ExpressionKind::Name(_)
        | ExpressionKind::Integer(_)
        | ExpressionKind::String(_)
        | ExpressionKind::Boolean(_)
        | ExpressionKind::Null
        | ExpressionKind::This
        | ExpressionKind::Super => {}
    }
}

pub fn render_ast(file: &SourceFile, unit: &CompilationUnit) -> String {
    let mut renderer = AstRenderer {
        file,
        output: String::new(),
    };
    renderer.node(0, "compilation-unit", "", unit.span());
    for declaration in unit.declarations() {
        renderer.class_declaration(1, declaration);
    }
    renderer.output
}

struct AstRenderer<'a> {
    file: &'a SourceFile,
    output: String,
}

impl AstRenderer<'_> {
    fn node(&mut self, depth: usize, kind: &str, details: &str, span: Span) {
        let start = self
            .file
            .location(span.start())
            .expect("AST starts at a source boundary");
        let end = self
            .file
            .location(span.end())
            .expect("AST ends at a source boundary");
        let _ = write!(self.output, "{:width$}{kind}", "", width = depth * 2);
        if !details.is_empty() {
            let _ = write!(self.output, " {details}");
        }
        let _ = writeln!(
            self.output,
            " @{}:{}..{}:{}",
            start.line, start.column, end.line, end.column
        );
    }

    fn modifiers(modifiers: &[Modifier]) -> String {
        if modifiers.is_empty() {
            String::new()
        } else {
            format!(
                " modifiers=[{}]",
                modifiers
                    .iter()
                    .map(Modifier::spelling)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    fn class_declaration(&mut self, depth: usize, declaration: &ClassDeclaration) {
        let kind = match declaration.kind() {
            DeclarationKind::Class => "class",
            DeclarationKind::Record { .. } => "record",
            DeclarationKind::SealedResult { .. } => "sealed-result",
            DeclarationKind::SObject => "sobject",
        };
        self.node(
            depth,
            kind,
            &format!(
                "{}{}",
                declaration.name().spelling(),
                Self::modifiers(declaration.modifiers())
            ),
            declaration.span(),
        );
        match declaration.kind() {
            DeclarationKind::Class => {
                if let Some(extends) = declaration.extends() {
                    self.ty(depth + 1, "extends-type", extends);
                }
                for implemented in declaration.implements() {
                    self.ty(depth + 1, "implements-type", implemented);
                }
                for member in declaration.members() {
                    self.member(depth + 1, member);
                }
            }
            DeclarationKind::Record { components } => {
                for component in components {
                    self.node(
                        depth + 1,
                        "component",
                        component.name().spelling(),
                        component.span(),
                    );
                    self.ty(depth + 2, "type", component.ty());
                }
            }
            DeclarationKind::SealedResult { variants } => {
                for variant in variants {
                    self.node(
                        depth + 1,
                        "variant",
                        variant.name().spelling(),
                        variant.span(),
                    );
                    for payload in variant.payloads() {
                        self.node(
                            depth + 2,
                            "payload",
                            payload.name().spelling(),
                            payload.span(),
                        );
                        self.ty(depth + 3, "type", payload.ty());
                    }
                }
            }
            DeclarationKind::SObject => {}
        }
    }

    fn member(&mut self, depth: usize, member: &ClassMember) {
        match member {
            ClassMember::Field(field) => {
                self.node(
                    depth,
                    "field",
                    &format!(
                        "{}{}",
                        field.name().spelling(),
                        Self::modifiers(field.modifiers())
                    ),
                    field.span(),
                );
                self.ty(depth + 1, "type", field.ty());
                if let Some(initializer) = field.initializer() {
                    self.expression(depth + 1, initializer);
                }
            }
            ClassMember::Property(property) => {
                self.node(
                    depth,
                    "property",
                    &format!(
                        "{}{}",
                        property.name().spelling(),
                        Self::modifiers(property.modifiers())
                    ),
                    property.span(),
                );
                self.ty(depth + 1, "type", property.ty());
                for accessor in property.accessors() {
                    self.node(
                        depth + 1,
                        "accessor",
                        &format!(
                            "{}{}",
                            accessor.kind().as_str(),
                            Self::modifiers(accessor.modifiers())
                        ),
                        accessor.span(),
                    );
                }
            }
            ClassMember::Method(method) => {
                self.node(
                    depth,
                    "method",
                    &format!(
                        "{}{}",
                        method.name().spelling(),
                        Self::modifiers(method.modifiers())
                    ),
                    method.span(),
                );
                self.ty(depth + 1, "return-type", method.return_type());
                for parameter in method.parameters() {
                    self.parameter(depth + 1, parameter);
                }
                self.block(depth + 1, method.body());
            }
            ClassMember::Constructor(constructor) => {
                self.node(
                    depth,
                    "constructor",
                    &format!(
                        "{}{}",
                        constructor.name().spelling(),
                        Self::modifiers(constructor.modifiers())
                    ),
                    constructor.span(),
                );
                for parameter in constructor.parameters() {
                    self.parameter(depth + 1, parameter);
                }
                self.block(depth + 1, constructor.body());
            }
        }
    }

    fn ty(&mut self, depth: usize, kind: &str, ty: &Type) {
        self.node(depth, kind, &ty.display_name(), ty.span());
    }

    fn parameter(&mut self, depth: usize, parameter: &Parameter) {
        self.node(
            depth,
            "parameter",
            parameter.name().spelling(),
            parameter.span(),
        );
        self.ty(depth + 1, "type", parameter.ty());
    }

    fn block(&mut self, depth: usize, block: &Block) {
        self.node(depth, "block", "", block.span());
        for statement in block.statements() {
            self.statement(depth + 1, statement);
        }
    }

    fn statement(&mut self, depth: usize, statement: &Statement) {
        match statement.kind() {
            StatementKind::Block(block) => self.block(depth, block),
            StatementKind::Variable(variable) => {
                self.node(
                    depth,
                    "variable",
                    variable.name().spelling(),
                    statement.span(),
                );
                self.ty(depth + 1, "type", variable.ty());
                if let Some(initializer) = variable.initializer() {
                    self.expression(depth + 1, initializer);
                }
            }
            StatementKind::Let(declaration) => {
                self.node(
                    depth,
                    "let",
                    declaration.name().spelling(),
                    statement.span(),
                );
                self.expression(depth + 1, declaration.initializer());
            }
            StatementKind::Expression(expression) => {
                self.node(depth, "expression-statement", "", statement.span());
                self.expression(depth + 1, expression);
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.node(depth, "if", "", statement.span());
                self.expression(depth + 1, condition);
                self.statement(depth + 1, then_branch);
                if let Some(else_branch) = else_branch {
                    self.node(depth + 1, "else", "", else_branch.span());
                    self.statement(depth + 2, else_branch);
                }
            }
            StatementKind::While { condition, body } => {
                self.node(depth, "while", "", statement.span());
                self.expression(depth + 1, condition);
                self.statement(depth + 1, body);
            }
            StatementKind::DoWhile { body, condition } => {
                self.node(depth, "do-while", "", statement.span());
                self.statement(depth + 1, body);
                self.expression(depth + 1, condition);
            }
            StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.node(depth, "for", "", statement.span());
                if let Some(initializer) = initializer {
                    match initializer {
                        ForInitializer::Variable(variable) => {
                            self.node(
                                depth + 1,
                                "for-variable",
                                variable.name().spelling(),
                                variable.span(),
                            );
                            self.ty(depth + 2, "type", variable.ty());
                            if let Some(initializer) = variable.initializer() {
                                self.expression(depth + 2, initializer);
                            }
                        }
                        ForInitializer::Expressions(expressions) => {
                            for expression in expressions {
                                self.expression(depth + 1, expression);
                            }
                        }
                    }
                }
                if let Some(condition) = condition {
                    self.expression(depth + 1, condition);
                }
                for expression in update {
                    self.expression(depth + 1, expression);
                }
                self.statement(depth + 1, body);
            }
            StatementKind::EnhancedFor {
                variable,
                iterable,
                body,
            } => {
                self.node(
                    depth,
                    "enhanced-for",
                    variable.name().spelling(),
                    statement.span(),
                );
                self.ty(depth + 1, "type", variable.ty());
                self.expression(depth + 1, iterable);
                self.statement(depth + 1, body);
            }
            StatementKind::Match { subject, arms } => {
                self.node(depth, "match", "", statement.span());
                self.expression(depth + 1, subject);
                for arm in arms {
                    self.node(depth + 1, "match-arm", arm.variant().spelling(), arm.span());
                    for binding in arm.bindings() {
                        self.node(depth + 2, "binding", binding.spelling(), binding.span());
                    }
                    self.block(depth + 2, arm.body());
                }
            }
            StatementKind::Return(expression) => {
                self.node(depth, "return", "", statement.span());
                if let Some(expression) = expression {
                    self.expression(depth + 1, expression);
                }
            }
            StatementKind::Throw(expression) => {
                self.node(depth, "throw", "", statement.span());
                self.expression(depth + 1, expression);
            }
            StatementKind::Break => self.node(depth, "break", "", statement.span()),
            StatementKind::Continue => self.node(depth, "continue", "", statement.span()),
            StatementKind::Empty => self.node(depth, "empty", "", statement.span()),
        }
    }

    fn expression(&mut self, depth: usize, expression: &Expression) {
        match expression.kind() {
            ExpressionKind::Name(name) => {
                self.node(depth, "name", name.spelling(), expression.span())
            }
            ExpressionKind::Integer(value) => self.node(depth, "integer", value, expression.span()),
            ExpressionKind::String(value) => {
                self.node(depth, "string", &json_string(value), expression.span())
            }
            ExpressionKind::Boolean(value) => {
                self.node(depth, "boolean", &value.to_string(), expression.span())
            }
            ExpressionKind::Null => self.node(depth, "null", "", expression.span()),
            ExpressionKind::This => self.node(depth, "this", "", expression.span()),
            ExpressionKind::Super => self.node(depth, "super", "", expression.span()),
            ExpressionKind::Parenthesized(inner) => {
                self.node(depth, "parenthesized", "", expression.span());
                self.expression(depth + 1, inner);
            }
            ExpressionKind::New { ty, arguments } => {
                self.node(depth, "new", "", expression.span());
                self.ty(depth + 1, "type", ty);
                for argument in arguments {
                    self.expression(depth + 1, argument);
                }
            }
            ExpressionKind::Call { callee, arguments } => {
                self.node(depth, "call", "", expression.span());
                self.expression(depth + 1, callee);
                for argument in arguments {
                    self.expression(depth + 1, argument);
                }
            }
            ExpressionKind::Member {
                object,
                member,
                safe,
            } => {
                self.node(
                    depth,
                    if *safe { "safe-member" } else { "member" },
                    member.spelling(),
                    expression.span(),
                );
                self.expression(depth + 1, object);
            }
            ExpressionKind::Index { object, index } => {
                self.node(depth, "index", "", expression.span());
                self.expression(depth + 1, object);
                self.expression(depth + 1, index);
            }
            ExpressionKind::Postfix { operand, operator } => {
                self.node(depth, "postfix", operator.spelling(), expression.span());
                self.expression(depth + 1, operand);
            }
            ExpressionKind::Unary { operator, operand } => {
                self.node(depth, "unary", operator.spelling(), expression.span());
                self.expression(depth + 1, operand);
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                self.node(depth, "binary", operator.spelling(), expression.span());
                self.expression(depth + 1, left);
                self.expression(depth + 1, right);
            }
            ExpressionKind::Conditional {
                condition,
                then_expression,
                else_expression,
            } => {
                self.node(depth, "conditional", "", expression.span());
                self.expression(depth + 1, condition);
                self.expression(depth + 1, then_expression);
                self.expression(depth + 1, else_expression);
            }
            ExpressionKind::Assignment {
                target,
                operator,
                value,
            } => {
                self.node(depth, "assignment", operator.spelling(), expression.span());
                self.expression(depth + 1, target);
                self.expression(depth + 1, value);
            }
        }
    }
}

fn json_string(text: &str) -> String {
    let mut output = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{001f}' => {
                let _ = write!(output, "\\u{:04x}", u32::from(character));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}
