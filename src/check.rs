use std::collections::BTreeMap;

use crate::apex_api::ApexBoundary;
use crate::ast;
use crate::diagnostic::{Diagnostic, Phase, SourceLabel};
use crate::hir;
use crate::project::SourceUnit;
use crate::source::Span;
use crate::types::Type;

#[derive(Clone, Debug)]
struct Symbols {
    classes: BTreeMap<String, ClassSymbol>,
    external: ApexBoundary,
}

#[derive(Clone, Debug)]
struct ClassSymbol {
    name: String,
    span: Span,
    fields: BTreeMap<String, ValueSymbol>,
    methods: Vec<MethodSymbol>,
}

#[derive(Clone, Debug)]
struct ValueSymbol {
    name: String,
    ty: Type,
    is_static: bool,
    is_property: bool,
    writable: bool,
    externally_accessible: bool,
    externally_writable: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct MethodSymbol {
    name: String,
    return_type: Type,
    parameters: Vec<Type>,
    is_static: bool,
    externally_accessible: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct Local {
    name: String,
    ty: Type,
    parameter: bool,
    span: Span,
}

pub fn check(
    units: &[SourceUnit],
    boundary: &ApexBoundary,
) -> Result<hir::Program, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut classes: BTreeMap<String, ClassSymbol> = BTreeMap::new();

    for external in boundary.classes.values() {
        if is_reserved_generated_name(&external.name) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.reserved-generated-name",
                    format!(
                        "boundary class name `{}` uses the reserved `ZenithGenerated_` prefix",
                        external.name
                    ),
                    Some(external.span),
                )
                .with_primary_label("reserved for compiler-generated declarations"),
            );
        }
        for method in &external.methods {
            if is_reserved_generated_name(&method.name) {
                diagnostics.push(
                    Diagnostic::coded_error(
                        Phase::Resolve,
                        "resolve.reserved-generated-name",
                        format!(
                            "boundary method name `{}` uses the reserved `ZenithGenerated_` prefix",
                            method.name
                        ),
                        Some(method.span),
                    )
                    .with_primary_label("reserved for compiler-generated declarations"),
                );
            }
        }
    }

    for unit in units {
        if unit.syntax.declarations().len() != 1 {
            diagnostics.push(Diagnostic::coded_error(
                Phase::Project,
                "project.one-class-per-file",
                format!(
                    "`{}` must contain exactly one top-level class",
                    unit.relative_path.display()
                ),
                Some(unit.syntax.span()),
            ));
            continue;
        }
        let declaration = &unit.syntax.declarations()[0];
        let canonical = declaration.name().canonical().to_owned();
        if is_reserved_generated_name(declaration.name().spelling()) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.reserved-generated-name",
                    format!(
                        "class name `{}` uses the reserved `ZenithGenerated_` prefix",
                        declaration.name().spelling()
                    ),
                    Some(declaration.name().span()),
                )
                .with_primary_label("reserved for compiler-generated declarations"),
            );
        }
        let stem = unit
            .relative_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !stem.eq_ignore_ascii_case(declaration.name().spelling()) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Project,
                    "project.class-file-mismatch",
                    format!(
                        "class `{}` must be declared in a matching `.zen` file",
                        declaration.name().spelling()
                    ),
                    Some(declaration.name().span()),
                )
                .with_note(format!(
                    "current file is `{}`",
                    unit.relative_path.display()
                )),
            );
        }
        if let Some(previous) = classes.get(&canonical) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.duplicate-class",
                    format!("duplicate class `{}`", declaration.name().spelling()),
                    Some(declaration.name().span()),
                )
                .with_secondary_label(SourceLabel::new(previous.span, "first declaration is here")),
            );
        } else if let Some(external) = boundary.classes.get(&canonical) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.duplicate-class",
                    format!(
                        "class `{}` conflicts with a handwritten Apex boundary class",
                        declaration.name().spelling()
                    ),
                    Some(declaration.name().span()),
                )
                .with_secondary_label(SourceLabel::new(
                    external.span,
                    "boundary declaration is here",
                )),
            );
        } else {
            classes.insert(
                canonical,
                ClassSymbol {
                    name: declaration.name().spelling().to_owned(),
                    span: declaration.name().span(),
                    fields: BTreeMap::new(),
                    methods: Vec::new(),
                },
            );
        }
    }

    let class_names: BTreeMap<_, _> = classes
        .iter()
        .map(|(canonical, symbol)| (canonical.clone(), symbol.name.clone()))
        .collect();
    for unit in units {
        let Some(declaration) = unit.syntax.declarations().first() else {
            continue;
        };
        let canonical = declaration.name().canonical();
        let Some(mut class) = classes.remove(canonical) else {
            continue;
        };
        collect_members(
            declaration,
            &class_names,
            boundary,
            &mut class,
            &mut diagnostics,
        );
        classes.insert(canonical.to_owned(), class);
    }

    let symbols = Symbols {
        classes,
        external: boundary.clone(),
    };
    let mut hir_classes = Vec::new();
    for unit in units {
        let Some(declaration) = unit.syntax.declarations().first() else {
            continue;
        };
        if symbols.classes.contains_key(declaration.name().canonical()) {
            hir_classes.push(check_class(unit, declaration, &symbols, &mut diagnostics));
        }
    }
    hir_classes.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });
    if diagnostics.is_empty() {
        Ok(hir::Program {
            classes: hir_classes,
        })
    } else {
        Err(diagnostics)
    }
}

fn collect_members(
    declaration: &ast::ClassDeclaration,
    class_names: &BTreeMap<String, String>,
    boundary: &ApexBoundary,
    class: &mut ClassSymbol,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_modifiers(
        declaration.modifiers(),
        &[
            "public",
            "global",
            "abstract",
            "virtual",
            "final",
            "with sharing",
            "without sharing",
            "inherited sharing",
        ],
        "class",
        diagnostics,
    );
    if declaration.extends().is_some() || !declaration.implements().is_empty() {
        diagnostics.push(unsupported(
            declaration.span(),
            "inheritance and interface implementation are not supported in M3",
        ));
    }
    for member in declaration.members() {
        match member {
            ast::ClassMember::Field(field) => {
                let ty = resolve_type(field.ty(), class_names, boundary, false, diagnostics);
                validate_member_value(
                    field.name(),
                    field.modifiers(),
                    ty,
                    MemberValueOptions {
                        is_property: false,
                        initialized_or_settable: field.initializer().is_some(),
                        setter_externally_accessible: true,
                    },
                    class,
                    diagnostics,
                );
            }
            ast::ClassMember::Property(property) => {
                let ty = resolve_type(property.ty(), class_names, boundary, false, diagnostics);
                let setter = property
                    .accessors()
                    .iter()
                    .find(|accessor| accessor.kind() == ast::AccessorKind::Set);
                validate_member_value(
                    property.name(),
                    property.modifiers(),
                    ty,
                    MemberValueOptions {
                        is_property: true,
                        initialized_or_settable: setter.is_some(),
                        setter_externally_accessible: setter.is_some_and(|accessor| {
                            !accessor.modifiers().iter().any(|modifier| {
                                matches!(modifier.canonical(), "private" | "protected")
                            })
                        }),
                    },
                    class,
                    diagnostics,
                );
                if property.accessors().is_empty() {
                    diagnostics.push(unsupported(
                        property.span(),
                        "properties require at least one automatic accessor",
                    ));
                } else if !property
                    .accessors()
                    .iter()
                    .any(|accessor| accessor.kind() == ast::AccessorKind::Get)
                {
                    diagnostics.push(unsupported(
                        property.span(),
                        "M3 automatic properties require a `get` accessor",
                    ));
                }
                let mut accessors = BTreeMap::new();
                for accessor in property.accessors() {
                    let allowed = if accessor.kind() == ast::AccessorKind::Set {
                        &["private", "protected"][..]
                    } else {
                        &[][..]
                    };
                    validate_modifiers(
                        accessor.modifiers(),
                        allowed,
                        "property accessor",
                        diagnostics,
                    );
                    let name = accessor.kind().as_str();
                    if accessors.insert(name, accessor.span()).is_some() {
                        diagnostics.push(Diagnostic::coded_error(
                            Phase::Resolve,
                            "resolve.duplicate-accessor",
                            format!("duplicate `{name}` accessor"),
                            Some(accessor.span()),
                        ));
                    }
                }
            }
            ast::ClassMember::Method(method) => {
                reject_reserved_name(method.name(), "method", diagnostics);
                validate_modifiers(
                    method.modifiers(),
                    &[
                        "public",
                        "private",
                        "protected",
                        "global",
                        "static",
                        "final",
                        "virtual",
                    ],
                    "method",
                    diagnostics,
                );
                if has_modifier(method.modifiers(), "virtual")
                    && !has_modifier(declaration.modifiers(), "virtual")
                    && !has_modifier(declaration.modifiers(), "abstract")
                {
                    diagnostics.push(Diagnostic::coded_error(
                        Phase::Resolve,
                        "resolve.invalid-modifier-context",
                        "virtual methods require a virtual or abstract class",
                        Some(method.name().span()),
                    ));
                }
                let return_type = resolve_type(
                    method.return_type(),
                    class_names,
                    boundary,
                    true,
                    diagnostics,
                );
                let parameters: Vec<_> = method
                    .parameters()
                    .iter()
                    .map(|parameter| {
                        resolve_type(parameter.ty(), class_names, boundary, false, diagnostics)
                    })
                    .collect();
                let is_static = has_modifier(method.modifiers(), "static");
                if let Some(previous) = class.methods.iter().find(|candidate| {
                    candidate
                        .name
                        .eq_ignore_ascii_case(method.name().spelling())
                        && candidate.parameters == parameters
                }) {
                    diagnostics.push(
                        Diagnostic::coded_error(
                            Phase::Resolve,
                            "resolve.duplicate-method",
                            format!("duplicate method `{}`", method.name().spelling()),
                            Some(method.name().span()),
                        )
                        .with_secondary_label(SourceLabel::new(
                            previous.span,
                            "first method is here",
                        )),
                    );
                } else {
                    class.methods.push(MethodSymbol {
                        name: method.name().spelling().to_owned(),
                        return_type,
                        parameters,
                        is_static,
                        externally_accessible: has_external_visibility(method.modifiers()),
                        span: method.name().span(),
                    });
                }
            }
            ast::ClassMember::Constructor(constructor) => diagnostics.push(unsupported(
                constructor.span(),
                "constructors are not supported in M3",
            )),
        }
    }
}

#[derive(Clone, Copy)]
struct MemberValueOptions {
    is_property: bool,
    initialized_or_settable: bool,
    setter_externally_accessible: bool,
}

fn validate_member_value(
    name: &crate::token::Identifier,
    modifiers: &[ast::Modifier],
    ty: Type,
    options: MemberValueOptions,
    class: &mut ClassSymbol,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let MemberValueOptions {
        is_property,
        initialized_or_settable,
        setter_externally_accessible,
    } = options;
    reject_reserved_name(
        name,
        if is_property { "property" } else { "field" },
        diagnostics,
    );
    let allowed = if is_property {
        &["public", "private", "protected", "global", "static"][..]
    } else {
        &[
            "public",
            "private",
            "protected",
            "global",
            "static",
            "final",
        ][..]
    };
    validate_modifiers(
        modifiers,
        allowed,
        if is_property { "property" } else { "field" },
        diagnostics,
    );
    if ty == Type::Void {
        diagnostics.push(type_error(
            "type.void-value",
            "`void` cannot be used as a field or property type",
            name.span(),
        ));
    }
    let is_final = has_modifier(modifiers, "final");
    if is_final && !is_property && !initialized_or_settable {
        diagnostics.push(type_error(
            "type.uninitialized-final-field",
            format!(
                "final field `{}` requires an initializer because M3 does not support constructors",
                name.spelling()
            ),
            name.span(),
        ));
    }
    let canonical = name.canonical().to_owned();
    if let Some(previous) = class.fields.get(&canonical) {
        diagnostics.push(
            Diagnostic::coded_error(
                Phase::Resolve,
                "resolve.duplicate-member",
                format!("duplicate member `{}`", name.spelling()),
                Some(name.span()),
            )
            .with_secondary_label(SourceLabel::new(previous.span, "first member is here")),
        );
    } else {
        class.fields.insert(
            canonical,
            ValueSymbol {
                name: name.spelling().to_owned(),
                ty,
                is_static: has_modifier(modifiers, "static"),
                is_property,
                writable: !is_final && (!is_property || initialized_or_settable),
                externally_accessible: has_external_visibility(modifiers),
                externally_writable: !is_final
                    && has_external_visibility(modifiers)
                    && (!is_property || setter_externally_accessible),
                span: name.span(),
            },
        );
    }
}

fn check_class(
    unit: &SourceUnit,
    declaration: &ast::ClassDeclaration,
    symbols: &Symbols,
    diagnostics: &mut Vec<Diagnostic>,
) -> hir::Class {
    let class = &symbols.classes[declaration.name().canonical()];
    let mut members = Vec::new();
    for member in declaration.members() {
        match member {
            ast::ClassMember::Field(field) => {
                let symbol = class.fields.get(field.name().canonical());
                let ty = symbol.map_or(Type::Error, |symbol| symbol.ty.clone());
                let initializer = field.initializer().map(|expression| {
                    let mut context = Context::new(
                        symbols,
                        class,
                        ty.clone(),
                        has_modifier(field.modifiers(), "static"),
                        diagnostics,
                    );
                    let checked = context.check_expression(expression);
                    require_assignable(&ty, &checked, expression.span(), context.diagnostics);
                    checked
                });
                members.push(hir::Member::Field(hir::Field {
                    name: field.name().spelling().to_owned(),
                    modifiers: modifier_spellings(field.modifiers()),
                    ty,
                    initializer,
                    span: field.span(),
                }));
            }
            ast::ClassMember::Property(property) => {
                let ty = class
                    .fields
                    .get(property.name().canonical())
                    .map_or(Type::Error, |symbol| symbol.ty.clone());
                members.push(hir::Member::Property(hir::Property {
                    name: property.name().spelling().to_owned(),
                    modifiers: modifier_spellings(property.modifiers()),
                    ty,
                    accessors: property
                        .accessors()
                        .iter()
                        .map(|accessor| hir::Accessor {
                            kind: accessor.kind().as_str().to_owned(),
                            modifiers: modifier_spellings(accessor.modifiers()),
                            span: accessor.span(),
                        })
                        .collect(),
                    span: property.span(),
                }));
            }
            ast::ClassMember::Method(method) => {
                let parameters: Vec<_> = method
                    .parameters()
                    .iter()
                    .map(|parameter| hir::Parameter {
                        name: parameter.name().spelling().to_owned(),
                        ty: resolve_type_quiet(parameter.ty(), symbols),
                        span: parameter.span(),
                    })
                    .collect();
                let return_type = resolve_type_quiet(method.return_type(), symbols);
                let mut context = Context::new(
                    symbols,
                    class,
                    return_type.clone(),
                    has_modifier(method.modifiers(), "static"),
                    diagnostics,
                );
                context.push_scope();
                for parameter in &parameters {
                    context.declare(&parameter.name, parameter.ty.clone(), true, parameter.span);
                }
                let body = context.check_block(method.body(), false);
                context.pop_scope();
                if return_type != Type::Void && !block_guarantees_return(&body) {
                    context.diagnostics.push(type_error(
                        "type.missing-return",
                        format!(
                            "method `{}` may complete without returning `{return_type}`",
                            method.name().spelling()
                        ),
                        method.body().span(),
                    ));
                }
                members.push(hir::Member::Method(hir::Method {
                    name: method.name().spelling().to_owned(),
                    modifiers: modifier_spellings(method.modifiers()),
                    return_type,
                    parameters,
                    body,
                    span: method.span(),
                }));
            }
            ast::ClassMember::Constructor(_) => {}
        }
    }
    hir::Class {
        name: declaration.name().spelling().to_owned(),
        modifiers: modifier_spellings(declaration.modifiers()),
        members,
        source_path: unit.relative_path.to_string_lossy().replace('\\', "/"),
        span: declaration.span(),
    }
}

struct Context<'a> {
    symbols: &'a Symbols,
    class: &'a ClassSymbol,
    return_type: Type,
    is_static: bool,
    scopes: Vec<BTreeMap<String, Local>>,
    diagnostics: &'a mut Vec<Diagnostic>,
    loop_depth: usize,
}

impl<'a> Context<'a> {
    fn new(
        symbols: &'a Symbols,
        class: &'a ClassSymbol,
        return_type: Type,
        is_static: bool,
        diagnostics: &'a mut Vec<Diagnostic>,
    ) -> Self {
        Self {
            symbols,
            class,
            return_type,
            is_static,
            scopes: Vec::new(),
            diagnostics,
            loop_depth: 0,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, ty: Type, parameter: bool, span: Span) {
        let canonical = name.to_ascii_lowercase();
        let scope = self
            .scopes
            .last_mut()
            .expect("declarations require a scope");
        if let Some(previous) = scope.get(&canonical) {
            self.diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.duplicate-local",
                    format!("duplicate local name `{name}`"),
                    Some(span),
                )
                .with_secondary_label(SourceLabel::new(previous.span, "first declaration is here")),
            );
        } else {
            scope.insert(
                canonical,
                Local {
                    name: name.to_owned(),
                    ty,
                    parameter,
                    span,
                },
            );
        }
    }

    fn find_local(&self, canonical: &str) -> Option<&Local> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(canonical))
    }

    fn check_block(&mut self, block: &ast::Block, create_scope: bool) -> hir::Block {
        if create_scope {
            self.push_scope();
        }
        let statements = block
            .statements()
            .iter()
            .map(|statement| self.check_statement(statement))
            .collect();
        if create_scope {
            self.pop_scope();
        }
        hir::Block {
            statements,
            span: block.span(),
        }
    }

    fn check_statement(&mut self, statement: &ast::Statement) -> hir::Statement {
        let kind = match statement.kind() {
            ast::StatementKind::Block(block) => {
                hir::StatementKind::Block(self.check_block(block, true))
            }
            ast::StatementKind::Variable(variable) => {
                let ty = resolve_type_quiet(variable.ty(), self.symbols);
                if ty == Type::Void {
                    self.diagnostics.push(type_error(
                        "type.void-value",
                        "`void` cannot be used as a local type",
                        variable.ty().span(),
                    ));
                }
                let initializer = variable.initializer().map(|expression| {
                    let checked = self.check_expression(expression);
                    require_assignable(&ty, &checked, expression.span(), self.diagnostics);
                    checked
                });
                self.declare(
                    variable.name().spelling(),
                    ty.clone(),
                    false,
                    variable.name().span(),
                );
                hir::StatementKind::Variable {
                    ty,
                    name: variable.name().spelling().to_owned(),
                    initializer,
                }
            }
            ast::StatementKind::Expression(expression) => {
                hir::StatementKind::Expression(self.check_expression(expression))
            }
            ast::StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.check_expression(condition);
                require_boolean(&condition, self.diagnostics);
                hir::StatementKind::If {
                    condition,
                    then_branch: Box::new(self.check_scoped_statement(then_branch)),
                    else_branch: else_branch
                        .as_deref()
                        .map(|branch| Box::new(self.check_scoped_statement(branch))),
                }
            }
            ast::StatementKind::While { condition, body } => {
                let condition = self.check_expression(condition);
                require_boolean(&condition, self.diagnostics);
                self.loop_depth += 1;
                let body = Box::new(self.check_scoped_statement(body));
                self.loop_depth -= 1;
                hir::StatementKind::While { condition, body }
            }
            ast::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.push_scope();
                let initializer = initializer.as_ref().map(|initializer| match initializer {
                    ast::ForInitializer::Variable(variable) => {
                        let ty = resolve_type_quiet(variable.ty(), self.symbols);
                        let initializer = variable.initializer().map(|expression| {
                            let checked = self.check_expression(expression);
                            require_assignable(&ty, &checked, expression.span(), self.diagnostics);
                            checked
                        });
                        self.declare(
                            variable.name().spelling(),
                            ty.clone(),
                            false,
                            variable.name().span(),
                        );
                        hir::ForInitializer::Variable {
                            ty,
                            name: variable.name().spelling().to_owned(),
                            initializer: initializer.map(Box::new),
                        }
                    }
                    ast::ForInitializer::Expressions(expressions) => {
                        hir::ForInitializer::Expressions(
                            expressions
                                .iter()
                                .map(|expression| self.check_expression(expression))
                                .collect(),
                        )
                    }
                });
                let condition = condition.as_ref().map(|expression| {
                    let checked = self.check_expression(expression);
                    require_boolean(&checked, self.diagnostics);
                    checked
                });
                let update = update
                    .iter()
                    .map(|expression| self.check_expression(expression))
                    .collect();
                self.loop_depth += 1;
                let body = Box::new(self.check_statement(body));
                self.loop_depth -= 1;
                self.pop_scope();
                hir::StatementKind::For {
                    initializer,
                    condition,
                    update,
                    body,
                }
            }
            ast::StatementKind::EnhancedFor {
                variable,
                iterable,
                body,
            } => {
                let iterable = self.check_expression(iterable);
                let element = match &iterable.ty {
                    Type::List(element) | Type::Set(element) => element.as_ref().clone(),
                    _ => {
                        self.diagnostics.push(type_error(
                            "type.not-iterable",
                            format!("type `{}` is not iterable in M3", iterable.ty),
                            iterable.span,
                        ));
                        Type::Error
                    }
                };
                let variable_type = resolve_type_quiet(variable.ty(), self.symbols);
                if !variable_type.accepts(&element) {
                    self.diagnostics.push(type_error(
                        "type.incompatible-value",
                        format!("cannot iterate `{element}` values as `{variable_type}`"),
                        variable.span(),
                    ));
                }
                self.push_scope();
                self.declare(
                    variable.name().spelling(),
                    variable_type.clone(),
                    false,
                    variable.name().span(),
                );
                self.loop_depth += 1;
                let body = Box::new(self.check_statement(body));
                self.loop_depth -= 1;
                self.pop_scope();
                hir::StatementKind::EnhancedFor {
                    ty: variable_type,
                    name: variable.name().spelling().to_owned(),
                    iterable,
                    body,
                }
            }
            ast::StatementKind::Return(expression) => {
                let expression = expression.as_ref().map(|expression| {
                    let checked = self.check_expression(expression);
                    require_assignable(
                        &self.return_type,
                        &checked,
                        expression.span(),
                        self.diagnostics,
                    );
                    checked
                });
                if expression.is_none() && self.return_type != Type::Void {
                    self.diagnostics.push(type_error(
                        "type.missing-return-value",
                        format!("method returning `{}` requires a value", self.return_type),
                        statement.span(),
                    ));
                } else if expression.is_some() && self.return_type == Type::Void {
                    self.diagnostics.push(type_error(
                        "type.unexpected-return-value",
                        "a `void` method cannot return a value",
                        statement.span(),
                    ));
                }
                hir::StatementKind::Return(expression)
            }
            ast::StatementKind::Break => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(type_error(
                        "type.break-outside-loop",
                        "`break` may only appear inside a loop",
                        statement.span(),
                    ));
                }
                hir::StatementKind::Break
            }
            ast::StatementKind::Continue => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(type_error(
                        "type.continue-outside-loop",
                        "`continue` may only appear inside a loop",
                        statement.span(),
                    ));
                }
                hir::StatementKind::Continue
            }
            ast::StatementKind::Empty => hir::StatementKind::Empty,
            ast::StatementKind::DoWhile { .. } => {
                self.diagnostics.push(unsupported(
                    statement.span(),
                    "`do while` is parsed but not supported in M3",
                ));
                hir::StatementKind::Empty
            }
            ast::StatementKind::Throw(_) => {
                self.diagnostics.push(unsupported(
                    statement.span(),
                    "`throw` is parsed but not supported in M3",
                ));
                hir::StatementKind::Empty
            }
        };
        hir::Statement {
            kind,
            span: statement.span(),
        }
    }

    fn check_scoped_statement(&mut self, statement: &ast::Statement) -> hir::Statement {
        self.push_scope();
        let checked = self.check_statement(statement);
        self.pop_scope();
        checked
    }

    fn check_expression(&mut self, expression: &ast::Expression) -> hir::Expression {
        match expression.kind() {
            ast::ExpressionKind::Name(name) => self.check_name(name, expression.span()),
            ast::ExpressionKind::Integer(value) => {
                if value.parse::<i32>().is_err() {
                    self.diagnostics.push(type_error(
                        "type.integer-out-of-range",
                        format!("integer literal `{value}` is outside the 32-bit range"),
                        expression.span(),
                    ));
                }
                self.expression(
                    hir::ExpressionKind::Integer(value.clone()),
                    Type::Integer,
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::String(value) => self.expression(
                hir::ExpressionKind::String(value.clone()),
                Type::String,
                expression.span(),
                false,
            ),
            ast::ExpressionKind::Boolean(value) => self.expression(
                hir::ExpressionKind::Boolean(*value),
                Type::Boolean,
                expression.span(),
                false,
            ),
            ast::ExpressionKind::Null => self.expression(
                hir::ExpressionKind::Null,
                Type::Null,
                expression.span(),
                false,
            ),
            ast::ExpressionKind::This => {
                if self.is_static {
                    self.diagnostics.push(type_error(
                        "type.this-in-static-context",
                        "`this` is unavailable in a static context",
                        expression.span(),
                    ));
                }
                self.expression(
                    hir::ExpressionKind::This,
                    Type::Class(self.class.name.clone()),
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::Parenthesized(inner) => {
                let inner = self.check_expression(inner);
                let ty = inner.ty.clone();
                let assignable = inner.assignable;
                self.expression(
                    hir::ExpressionKind::Parenthesized(Box::new(inner)),
                    ty,
                    expression.span(),
                    assignable,
                )
            }
            ast::ExpressionKind::Member {
                object,
                member,
                safe,
            } => {
                if *safe {
                    self.diagnostics.push(unsupported(
                        expression.span(),
                        "safe navigation is reserved for M4",
                    ));
                }
                let object = self.check_expression(object);
                self.check_member(object, member, expression.span())
            }
            ast::ExpressionKind::Call { callee, arguments } => {
                self.check_call(callee, arguments, expression.span())
            }
            ast::ExpressionKind::Index { object, index } => {
                let object = self.check_expression(object);
                let index = self.check_expression(index);
                let (ty, target) = match &object.ty {
                    Type::List(element) => {
                        require_type(&Type::Integer, &index, self.diagnostics);
                        (element.as_ref().clone(), hir::IndexTarget::List)
                    }
                    _ => {
                        self.diagnostics.push(type_error(
                            "type.not-indexable",
                            format!("type `{}` cannot be indexed", object.ty),
                            object.span,
                        ));
                        (Type::Error, hir::IndexTarget::List)
                    }
                };
                self.expression(
                    hir::ExpressionKind::Index {
                        object: Box::new(object),
                        index: Box::new(index),
                        target,
                    },
                    ty,
                    expression.span(),
                    true,
                )
            }
            ast::ExpressionKind::Unary { operator, operand } => {
                let operand = if operator.spelling() == "-" {
                    if let ast::ExpressionKind::Integer(value) = operand.kind() {
                        if value
                            .parse::<u64>()
                            .map_or(true, |value| value > 2_147_483_648)
                        {
                            self.diagnostics.push(type_error(
                                "type.integer-out-of-range",
                                format!(
                                    "integer literal `-{value}` is outside the signed 32-bit range"
                                ),
                                expression.span(),
                            ));
                        }
                        self.expression(
                            hir::ExpressionKind::Integer(value.clone()),
                            Type::Integer,
                            operand.span(),
                            false,
                        )
                    } else {
                        self.check_expression(operand)
                    }
                } else {
                    self.check_expression(operand)
                };
                let ty = match operator.spelling() {
                    "!" => {
                        require_type(&Type::Boolean, &operand, self.diagnostics);
                        Type::Boolean
                    }
                    "+" | "-" => {
                        if !operand.ty.is_numeric() {
                            self.diagnostics.push(type_error(
                                "type.invalid-operator",
                                format!(
                                    "operator `{}` requires a numeric operand",
                                    operator.spelling()
                                ),
                                expression.span(),
                            ));
                        }
                        operand.ty.clone()
                    }
                    _ => {
                        self.diagnostics.push(unsupported(
                            expression.span(),
                            format!(
                                "unary operator `{}` is not supported in M3",
                                operator.spelling()
                            ),
                        ));
                        Type::Error
                    }
                };
                self.expression(
                    hir::ExpressionKind::Unary {
                        operator: operator.spelling().to_owned(),
                        operand: Box::new(operand),
                    },
                    ty,
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.check_expression(left);
                let right = self.check_expression(right);
                let ty = check_binary(&left, operator.spelling(), &right, self.diagnostics);
                self.expression(
                    hir::ExpressionKind::Binary {
                        left: Box::new(left),
                        operator: operator.spelling().to_owned(),
                        right: Box::new(right),
                    },
                    ty,
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::Conditional {
                condition,
                then_expression,
                else_expression,
            } => {
                let condition = self.check_expression(condition);
                require_boolean(&condition, self.diagnostics);
                let then_expression = self.check_expression(then_expression);
                let else_expression = self.check_expression(else_expression);
                let ty = if then_expression.ty.accepts(&else_expression.ty) {
                    then_expression.ty.clone()
                } else if else_expression.ty.accepts(&then_expression.ty) {
                    else_expression.ty.clone()
                } else {
                    self.diagnostics.push(type_error(
                        "type.incompatible-branches",
                        format!(
                            "conditional branches have incompatible types `{}` and `{}`",
                            then_expression.ty, else_expression.ty
                        ),
                        expression.span(),
                    ));
                    Type::Error
                };
                self.expression(
                    hir::ExpressionKind::Conditional {
                        condition: Box::new(condition),
                        then_expression: Box::new(then_expression),
                        else_expression: Box::new(else_expression),
                    },
                    ty,
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::Assignment {
                target,
                operator,
                value,
            } => {
                let target = self.check_expression(target);
                let value = self.check_expression(value);
                if !target.assignable {
                    self.diagnostics.push(type_error(
                        "type.invalid-assignment-target",
                        "assignment target is not writable",
                        target.span,
                    ));
                }
                if operator.spelling() == "=" {
                    require_assignable(&target.ty, &value, value.span, self.diagnostics);
                } else if !matches!(operator.spelling(), "+=" | "-=" | "*=" | "/=")
                    || check_binary(&target, &operator.spelling()[..1], &value, self.diagnostics)
                        == Type::Error
                {
                    self.diagnostics.push(unsupported(
                        expression.span(),
                        format!(
                            "assignment operator `{}` is not supported for these operands",
                            operator.spelling()
                        ),
                    ));
                }
                let ty = target.ty.clone();
                self.expression(
                    hir::ExpressionKind::Assignment {
                        target: Box::new(target),
                        operator: operator.spelling().to_owned(),
                        value: Box::new(value),
                    },
                    ty,
                    expression.span(),
                    false,
                )
            }
            ast::ExpressionKind::New { .. }
            | ast::ExpressionKind::Super
            | ast::ExpressionKind::Postfix { .. } => {
                self.diagnostics.push(unsupported(
                    expression.span(),
                    "this expression form is parsed but not supported in M3",
                ));
                self.expression(
                    hir::ExpressionKind::Null,
                    Type::Error,
                    expression.span(),
                    false,
                )
            }
        }
    }

    fn check_name(&mut self, name: &crate::token::Identifier, span: Span) -> hir::Expression {
        if let Some(local) = self.find_local(name.canonical()) {
            return self.expression(
                hir::ExpressionKind::Name {
                    spelling: local.name.clone(),
                    target: if local.parameter {
                        hir::ValueTarget::Parameter
                    } else {
                        hir::ValueTarget::Local
                    },
                },
                local.ty.clone(),
                span,
                true,
            );
        }
        if let Some(field) = self.class.fields.get(name.canonical()) {
            if self.is_static && !field.is_static {
                self.diagnostics.push(type_error(
                    "type.instance-member-in-static-context",
                    format!(
                        "instance member `{}` is unavailable in a static context",
                        field.name
                    ),
                    span,
                ));
            }
            return self.expression(
                hir::ExpressionKind::Name {
                    spelling: field.name.clone(),
                    target: if field.is_property {
                        hir::ValueTarget::Property {
                            owner: self.class.name.clone(),
                            is_static: field.is_static,
                        }
                    } else {
                        hir::ValueTarget::Field {
                            owner: self.class.name.clone(),
                            is_static: field.is_static,
                        }
                    },
                },
                field.ty.clone(),
                span,
                field.writable,
            );
        }
        if let Some(class) = self.symbols.classes.get(name.canonical()) {
            return self.expression(
                hir::ExpressionKind::Name {
                    spelling: class.name.clone(),
                    target: hir::ValueTarget::Class { external: false },
                },
                Type::Class(class.name.clone()),
                span,
                false,
            );
        }
        if let Some(class) = self.symbols.external.classes.get(name.canonical()) {
            return self.expression(
                hir::ExpressionKind::Name {
                    spelling: class.name.clone(),
                    target: hir::ValueTarget::Class { external: true },
                },
                Type::ExternalClass(class.name.clone()),
                span,
                false,
            );
        }
        self.diagnostics.push(
            Diagnostic::coded_error(
                Phase::Resolve,
                "resolve.unknown-name",
                format!("cannot resolve name `{}`", name.spelling()),
                Some(span),
            )
            .with_primary_label("not found in local, member, project, or boundary scope"),
        );
        self.expression(
            hir::ExpressionKind::Name {
                spelling: name.spelling().to_owned(),
                target: hir::ValueTarget::Local,
            },
            Type::Error,
            span,
            false,
        )
    }

    fn check_member(
        &mut self,
        object: hir::Expression,
        member: &crate::token::Identifier,
        span: Span,
    ) -> hir::Expression {
        let class_name = object.ty.canonical_class_name().map(str::to_owned);
        if let Some(class_name) = class_name {
            let canonical = class_name.to_ascii_lowercase();
            if let Some(class) = self.symbols.classes.get(&canonical) {
                if let Some(field) = class.fields.get(member.canonical()) {
                    let receiver_is_class = matches!(
                        object.kind,
                        hir::ExpressionKind::Name {
                            target: hir::ValueTarget::Class { .. },
                            ..
                        }
                    );
                    if receiver_is_class != field.is_static {
                        self.diagnostics.push(type_error(
                            "type.invalid-member-receiver",
                            format!(
                                "member `{}` requires an {} receiver",
                                field.name,
                                if field.is_static { "class" } else { "instance" }
                            ),
                            span,
                        ));
                    }
                    let ty = field.ty.clone();
                    let same_class = class.name.eq_ignore_ascii_case(&self.class.name);
                    if !same_class && !field.externally_accessible {
                        self.diagnostics.push(
                            Diagnostic::coded_error(
                                Phase::Resolve,
                                "resolve.inaccessible-member",
                                format!(
                                    "member `{}` is not accessible from class `{}`",
                                    field.name, self.class.name
                                ),
                                Some(member.span()),
                            )
                            .with_primary_label("member is not public or global"),
                        );
                    }
                    let target = if field.is_property {
                        hir::ValueTarget::Property {
                            owner: class.name.clone(),
                            is_static: field.is_static,
                        }
                    } else {
                        hir::ValueTarget::Field {
                            owner: class.name.clone(),
                            is_static: field.is_static,
                        }
                    };
                    return self.expression(
                        hir::ExpressionKind::Member {
                            object: Box::new(object),
                            name: field.name.clone(),
                            target,
                        },
                        ty,
                        span,
                        field.writable && (same_class || field.externally_writable),
                    );
                }
            }
        }
        self.diagnostics.push(
            Diagnostic::coded_error(
                Phase::Resolve,
                "resolve.unknown-member",
                format!(
                    "type `{}` has no value member `{}`",
                    object.ty,
                    member.spelling()
                ),
                Some(member.span()),
            )
            .with_primary_label("unknown member"),
        );
        self.expression(
            hir::ExpressionKind::Member {
                object: Box::new(object),
                name: member.spelling().to_owned(),
                target: hir::ValueTarget::Local,
            },
            Type::Error,
            span,
            false,
        )
    }

    fn check_call(
        &mut self,
        callee: &ast::Expression,
        arguments: &[ast::Expression],
        span: Span,
    ) -> hir::Expression {
        let arguments: Vec<_> = arguments
            .iter()
            .map(|argument| self.check_expression(argument))
            .collect();
        let (receiver, name, candidates) = match callee.kind() {
            ast::ExpressionKind::Name(name) => (
                None,
                name.spelling().to_owned(),
                self.class
                    .methods
                    .iter()
                    .filter(|method| method.name.eq_ignore_ascii_case(name.spelling()))
                    .map(|method| Candidate::Project(self.class.name.clone(), method))
                    .collect(),
            ),
            ast::ExpressionKind::Member {
                object,
                member,
                safe,
            } => {
                if *safe {
                    self.diagnostics.push(unsupported(
                        callee.span(),
                        "safe navigation is reserved for M4",
                    ));
                }
                let receiver = self.check_expression(object);
                let candidates = self.call_candidates(&receiver, member.canonical());
                (Some(receiver), member.spelling().to_owned(), candidates)
            }
            _ => {
                self.diagnostics.push(type_error(
                    "type.invalid-call-target",
                    "call target must be a method name or member",
                    callee.span(),
                ));
                return self.expression(
                    hir::ExpressionKind::Call {
                        receiver: None,
                        name: "<invalid>".into(),
                        arguments,
                        target: hir::CallTarget::Collection {
                            collection: "<error>".into(),
                            parameter_types: Vec::new(),
                        },
                    },
                    Type::Error,
                    span,
                    false,
                );
            }
        };
        let matching: Vec<_> = candidates
            .into_iter()
            .filter(|candidate| {
                candidate.parameters().len() == arguments.len()
                    && candidate
                        .parameters()
                        .iter()
                        .zip(&arguments)
                        .all(|(parameter, argument)| parameter.accepts(&argument.ty))
            })
            .collect();
        if matching.len() != 1 {
            self.diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    if matching.is_empty() {
                        "resolve.no-matching-method"
                    } else {
                        "resolve.ambiguous-method"
                    },
                    format!(
                        "{} method `{name}` for argument types ({})",
                        if matching.is_empty() {
                            "no matching"
                        } else {
                            "ambiguous"
                        },
                        arguments
                            .iter()
                            .map(|argument| argument.ty.display_name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    Some(callee.span()),
                )
                .with_primary_label("method selection failed"),
            );
            return self.expression(
                hir::ExpressionKind::Call {
                    receiver: receiver.map(Box::new),
                    name,
                    arguments,
                    target: hir::CallTarget::Collection {
                        collection: "<error>".into(),
                        parameter_types: Vec::new(),
                    },
                },
                Type::Error,
                span,
                false,
            );
        }
        let candidate = &matching[0];
        let accessible = candidate.is_accessible_from(&self.class.name);
        let candidate_is_static = candidate.is_static();
        let return_type = candidate.return_type();
        let target = candidate.target();
        let selected_name = candidate.spelling().to_owned();
        if !accessible {
            self.diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.inaccessible-method",
                    format!(
                        "method `{}` is not accessible from class `{}`",
                        selected_name, self.class.name
                    ),
                    Some(callee.span()),
                )
                .with_primary_label("method is not public or global"),
            );
        }
        if receiver.is_none() && self.is_static && !candidate_is_static {
            self.diagnostics.push(type_error(
                "type.instance-member-in-static-context",
                format!("instance method `{name}` is unavailable in a static context"),
                callee.span(),
            ));
        }
        if let Some(receiver) = &receiver {
            let receiver_is_class = matches!(
                receiver.kind,
                hir::ExpressionKind::Name {
                    target: hir::ValueTarget::Class { .. },
                    ..
                }
            );
            let is_class_method = matches!(
                target,
                hir::CallTarget::Project { .. } | hir::CallTarget::External { .. }
            );
            if is_class_method && receiver_is_class != candidate_is_static {
                self.diagnostics.push(type_error(
                    "type.invalid-method-receiver",
                    format!(
                        "method `{selected_name}` requires an {} receiver",
                        if candidate_is_static {
                            "class"
                        } else {
                            "instance"
                        }
                    ),
                    callee.span(),
                ));
            }
        }
        self.expression(
            hir::ExpressionKind::Call {
                receiver: receiver.map(Box::new),
                name: selected_name,
                arguments,
                target,
            },
            return_type,
            span,
            false,
        )
    }

    fn call_candidates<'b>(&'b self, receiver: &hir::Expression, name: &str) -> Vec<Candidate<'b>> {
        match &receiver.ty {
            Type::Class(class_name) => self
                .symbols
                .classes
                .get(&class_name.to_ascii_lowercase())
                .into_iter()
                .flat_map(|class| {
                    class
                        .methods
                        .iter()
                        .filter(move |method| method.name.eq_ignore_ascii_case(name))
                        .map(move |method| Candidate::Project(class.name.clone(), method))
                })
                .collect(),
            Type::ExternalClass(class_name) => self
                .symbols
                .external
                .classes
                .get(&class_name.to_ascii_lowercase())
                .into_iter()
                .flat_map(|class| {
                    class
                        .methods
                        .iter()
                        .filter(move |method| method.name.eq_ignore_ascii_case(name))
                        .map(move |method| Candidate::External(class.name.clone(), method))
                })
                .collect(),
            Type::List(element) => collection_candidates(
                "List",
                name,
                &[
                    ("size", vec![], Type::Integer),
                    ("isempty", vec![], Type::Boolean),
                    ("add", vec![element.as_ref().clone()], Type::Void),
                    ("get", vec![Type::Integer], element.as_ref().clone()),
                ],
            ),
            Type::Set(element) => collection_candidates(
                "Set",
                name,
                &[
                    ("size", vec![], Type::Integer),
                    ("isempty", vec![], Type::Boolean),
                    ("add", vec![element.as_ref().clone()], Type::Boolean),
                    ("contains", vec![element.as_ref().clone()], Type::Boolean),
                ],
            ),
            Type::Map(key, value) => collection_candidates(
                "Map",
                name,
                &[
                    ("size", vec![], Type::Integer),
                    ("isempty", vec![], Type::Boolean),
                    ("get", vec![key.as_ref().clone()], value.as_ref().clone()),
                    (
                        "put",
                        vec![key.as_ref().clone(), value.as_ref().clone()],
                        value.as_ref().clone(),
                    ),
                    ("containskey", vec![key.as_ref().clone()], Type::Boolean),
                ],
            ),
            _ => Vec::new(),
        }
    }

    fn expression(
        &self,
        kind: hir::ExpressionKind,
        ty: Type,
        span: Span,
        assignable: bool,
    ) -> hir::Expression {
        hir::Expression {
            kind,
            ty,
            span,
            assignable,
        }
    }
}

enum Candidate<'a> {
    Project(String, &'a MethodSymbol),
    External(String, &'a crate::apex_api::ExternalMethod),
    Collection {
        collection: String,
        spelling: String,
        parameters: Vec<Type>,
        return_type: Type,
    },
}

impl Candidate<'_> {
    fn spelling(&self) -> &str {
        match self {
            Self::Project(_, method) => &method.name,
            Self::External(_, method) => &method.name,
            Self::Collection { spelling, .. } => spelling,
        }
    }

    fn parameters(&self) -> &[Type] {
        match self {
            Self::Project(_, method) => &method.parameters,
            Self::External(_, method) => &method.parameters,
            Self::Collection { parameters, .. } => parameters,
        }
    }

    fn return_type(&self) -> Type {
        match self {
            Self::Project(_, method) => method.return_type.clone(),
            Self::External(_, method) => method.return_type.clone(),
            Self::Collection { return_type, .. } => return_type.clone(),
        }
    }

    fn is_static(&self) -> bool {
        match self {
            Self::Project(_, method) => method.is_static,
            Self::External(_, method) => method.is_static,
            Self::Collection { .. } => false,
        }
    }

    fn is_accessible_from(&self, caller: &str) -> bool {
        match self {
            Self::Project(owner, method) => {
                owner.eq_ignore_ascii_case(caller) || method.externally_accessible
            }
            Self::External(..) | Self::Collection { .. } => true,
        }
    }

    fn target(&self) -> hir::CallTarget {
        match self {
            Self::Project(owner, method) => hir::CallTarget::Project {
                owner: owner.clone(),
                is_static: method.is_static,
                parameter_types: method.parameters.clone(),
            },
            Self::External(owner, method) => hir::CallTarget::External {
                owner: owner.clone(),
                is_static: method.is_static,
                parameter_types: method.parameters.clone(),
                effects_unknown: true,
            },
            Self::Collection {
                collection,
                parameters,
                ..
            } => hir::CallTarget::Collection {
                collection: collection.clone(),
                parameter_types: parameters.clone(),
            },
        }
    }
}

fn collection_candidates<'a>(
    collection: &str,
    name: &str,
    methods: &[(&str, Vec<Type>, Type)],
) -> Vec<Candidate<'a>> {
    methods
        .iter()
        .filter(|(method, _, _)| method.eq_ignore_ascii_case(name))
        .map(|(method, parameters, return_type)| Candidate::Collection {
            collection: collection.to_owned(),
            spelling: match *method {
                "isempty" => "isEmpty".into(),
                "containskey" => "containsKey".into(),
                other => other.into(),
            },
            parameters: parameters.clone(),
            return_type: return_type.clone(),
        })
        .collect()
}

fn resolve_type(
    syntax: &ast::Type,
    classes: &BTreeMap<String, String>,
    boundary: &ApexBoundary,
    allow_void: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> Type {
    if syntax.is_nullable() {
        diagnostics.push(unsupported(
            syntax.span(),
            "nullable types are reserved for M4",
        ));
    }
    let canonical = syntax.name().canonical();
    let arguments: Vec<_> = syntax
        .arguments()
        .iter()
        .map(|argument| resolve_type(argument, classes, boundary, false, diagnostics))
        .collect();
    match canonical {
        "void" if arguments.is_empty() && allow_void => Type::Void,
        "void" if arguments.is_empty() => {
            diagnostics.push(type_error(
                "type.void-value",
                "`void` is only valid as a method return type",
                syntax.span(),
            ));
            Type::Error
        }
        "boolean" if arguments.is_empty() => Type::Boolean,
        "integer" | "int" if arguments.is_empty() => Type::Integer,
        "long" if arguments.is_empty() => Type::Long,
        "decimal" if arguments.is_empty() => Type::Decimal,
        "double" if arguments.is_empty() => Type::Double,
        "string" if arguments.is_empty() => Type::String,
        "object" if arguments.is_empty() => Type::Object,
        "list" if arguments.len() == 1 => Type::List(Box::new(arguments[0].clone())),
        "set" if arguments.len() == 1 => Type::Set(Box::new(arguments[0].clone())),
        "map" if arguments.len() == 2 => Type::Map(
            Box::new(arguments[0].clone()),
            Box::new(arguments[1].clone()),
        ),
        "list" | "set" | "map" => {
            diagnostics.push(type_error(
                "type.wrong-generic-arity",
                format!(
                    "type `{}` has the wrong number of type arguments",
                    syntax.name().spelling()
                ),
                syntax.span(),
            ));
            Type::Error
        }
        _ if arguments.is_empty() => {
            if let Some(name) = classes.get(canonical) {
                Type::Class(name.clone())
            } else if let Some(class) = boundary.classes.get(canonical) {
                Type::ExternalClass(class.name.clone())
            } else {
                diagnostics.push(Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.unknown-type",
                    format!("cannot resolve type `{}`", syntax.name().spelling()),
                    Some(syntax.span()),
                ));
                Type::Error
            }
        }
        _ => {
            diagnostics.push(unsupported(
                syntax.span(),
                "user-defined generic types are not supported in M3",
            ));
            Type::Error
        }
    }
}

fn resolve_type_quiet(syntax: &ast::Type, symbols: &Symbols) -> Type {
    let classes: BTreeMap<_, _> = symbols
        .classes
        .iter()
        .map(|(canonical, class)| (canonical.clone(), class.name.clone()))
        .collect();
    resolve_type(
        syntax,
        &classes,
        &symbols.external,
        syntax.name().canonical() == "void",
        &mut Vec::new(),
    )
}

fn validate_modifiers(
    modifiers: &[ast::Modifier],
    allowed: &[&str],
    declaration: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = BTreeMap::new();
    let mut visibility = None;
    let mut sharing = None;
    for modifier in modifiers {
        let canonical = modifier.canonical();
        if !allowed.contains(&canonical) {
            diagnostics.push(unsupported(
                modifier.span(),
                format!(
                    "modifier `{}` is not supported on {declaration}s in M3",
                    modifier.spelling()
                ),
            ));
        }
        if let Some(previous) = seen.insert(canonical, modifier.span()) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.duplicate-modifier",
                    format!("duplicate modifier `{}`", modifier.spelling()),
                    Some(modifier.span()),
                )
                .with_secondary_label(SourceLabel::new(previous, "first modifier is here")),
            );
        }
        if matches!(canonical, "public" | "private" | "protected" | "global") {
            if let Some((previous_name, previous_span)) = visibility {
                diagnostics.push(
                    Diagnostic::coded_error(
                        Phase::Resolve,
                        "resolve.conflicting-modifiers",
                        format!(
                            "visibility modifiers `{previous_name}` and `{canonical}` conflict"
                        ),
                        Some(modifier.span()),
                    )
                    .with_secondary_label(SourceLabel::new(
                        previous_span,
                        "first visibility is here",
                    )),
                );
            } else {
                visibility = Some((canonical, modifier.span()));
            }
        }
        if canonical.ends_with(" sharing") {
            if let Some((previous_name, previous_span)) = sharing {
                diagnostics.push(
                    Diagnostic::coded_error(
                        Phase::Resolve,
                        "resolve.conflicting-modifiers",
                        format!("sharing modifiers `{previous_name}` and `{canonical}` conflict"),
                        Some(modifier.span()),
                    )
                    .with_secondary_label(SourceLabel::new(
                        previous_span,
                        "first sharing modifier is here",
                    )),
                );
            } else {
                sharing = Some((canonical, modifier.span()));
            }
        }
    }

    let conflicts: &[(&str, &str)] = match declaration {
        "class" => &[("abstract", "final"), ("virtual", "final")],
        "method" => &[("static", "virtual"), ("virtual", "final")],
        _ => &[],
    };
    for (left, right) in conflicts {
        if let (Some(left_span), Some(right_span)) = (seen.get(left), seen.get(right)) {
            diagnostics.push(
                Diagnostic::coded_error(
                    Phase::Resolve,
                    "resolve.conflicting-modifiers",
                    format!("modifiers `{left}` and `{right}` conflict on this {declaration}"),
                    Some(*right_span),
                )
                .with_secondary_label(SourceLabel::new(*left_span, "conflicting modifier is here")),
            );
        }
    }
}

fn has_modifier(modifiers: &[ast::Modifier], expected: &str) -> bool {
    modifiers
        .iter()
        .any(|modifier| modifier.canonical() == expected)
}

fn has_external_visibility(modifiers: &[ast::Modifier]) -> bool {
    modifiers
        .iter()
        .any(|modifier| matches!(modifier.canonical(), "public" | "global"))
}

fn is_reserved_generated_name(name: &str) -> bool {
    name.to_ascii_lowercase().starts_with("zenithgenerated_")
}

fn reject_reserved_name(
    name: &crate::token::Identifier,
    declaration: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_reserved_generated_name(name.spelling()) {
        diagnostics.push(
            Diagnostic::coded_error(
                Phase::Resolve,
                "resolve.reserved-generated-name",
                format!(
                    "{declaration} name `{}` uses the reserved `ZenithGenerated_` prefix",
                    name.spelling()
                ),
                Some(name.span()),
            )
            .with_primary_label("reserved for compiler-generated declarations"),
        );
    }
}

fn modifier_spellings(modifiers: &[ast::Modifier]) -> Vec<String> {
    modifiers
        .iter()
        .map(|modifier| modifier.spelling().to_owned())
        .collect()
}

fn check_binary(
    left: &hir::Expression,
    operator: &str,
    right: &hir::Expression,
    diagnostics: &mut Vec<Diagnostic>,
) -> Type {
    let valid_equal = left.ty == right.ty
        || left.ty.accepts(&right.ty)
        || right.ty.accepts(&left.ty)
        || left.ty == Type::Error
        || right.ty == Type::Error;
    match operator {
        "+" if left.ty == Type::String && right.ty == Type::String => Type::String,
        "+" | "-" | "*" | "/" | "%" if left.ty == right.ty && left.ty.is_numeric() => {
            left.ty.clone()
        }
        "<" | "<=" | ">" | ">=" if left.ty == right.ty && left.ty.is_numeric() => Type::Boolean,
        "==" | "!=" if valid_equal => Type::Boolean,
        "&&" | "||" if left.ty == Type::Boolean && right.ty == Type::Boolean => Type::Boolean,
        _ => {
            diagnostics.push(type_error(
                "type.invalid-operator",
                format!(
                    "operator `{operator}` is not defined for `{}` and `{}`",
                    left.ty, right.ty
                ),
                left.span,
            ));
            Type::Error
        }
    }
}

fn require_boolean(expression: &hir::Expression, diagnostics: &mut Vec<Diagnostic>) {
    require_type(&Type::Boolean, expression, diagnostics);
}

fn require_type(expected: &Type, expression: &hir::Expression, diagnostics: &mut Vec<Diagnostic>) {
    if is_class_reference(expression) {
        diagnostics.push(type_error(
            "type.class-name-as-value",
            "a class name cannot be used as a runtime value",
            expression.span,
        ));
    } else if !expected.accepts(&expression.ty) && expression.ty != Type::Error {
        diagnostics.push(type_error(
            "type.incompatible-value",
            format!("expected `{expected}`, found `{}`", expression.ty),
            expression.span,
        ));
    }
}

fn require_assignable(
    expected: &Type,
    expression: &hir::Expression,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_class_reference(expression) {
        diagnostics.push(type_error(
            "type.class-name-as-value",
            "a class name cannot be used as a runtime value",
            span,
        ));
    } else if !expected.accepts(&expression.ty)
        && *expected != Type::Error
        && expression.ty != Type::Error
    {
        diagnostics.push(type_error(
            "type.incompatible-value",
            format!("expected `{expected}`, found `{}`", expression.ty),
            span,
        ));
    }
}

fn is_class_reference(expression: &hir::Expression) -> bool {
    matches!(
        expression.kind,
        hir::ExpressionKind::Name {
            target: hir::ValueTarget::Class { .. },
            ..
        }
    )
}

fn block_guarantees_return(block: &hir::Block) -> bool {
    block
        .statements
        .last()
        .is_some_and(statement_guarantees_return)
}

fn statement_guarantees_return(statement: &hir::Statement) -> bool {
    match &statement.kind {
        hir::StatementKind::Return(_) => true,
        hir::StatementKind::Block(block) => block_guarantees_return(block),
        hir::StatementKind::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => statement_guarantees_return(then_branch) && statement_guarantees_return(else_branch),
        _ => false,
    }
}

fn type_error(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::coded_error(Phase::Type, code, message, Some(span))
}

fn unsupported(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::coded_error(Phase::Type, "type.unsupported-syntax", message, Some(span))
}

#[cfg(test)]
mod tests {
    use super::check_binary;
    use crate::hir::{Expression, ExpressionKind};
    use crate::source::{SourceId, Span};
    use crate::types::Type;

    fn expression(ty: Type) -> Expression {
        Expression {
            kind: ExpressionKind::Null,
            ty,
            span: Span::new(SourceId::from_raw(0), 0, 0).unwrap(),
            assignable: false,
        }
    }

    #[test]
    fn binary_rules_cover_strings_numbers_booleans_and_invalid_pairs() {
        let mut diagnostics = Vec::new();
        assert_eq!(
            check_binary(
                &expression(Type::String),
                "+",
                &expression(Type::String),
                &mut diagnostics
            ),
            Type::String
        );
        assert_eq!(
            check_binary(
                &expression(Type::Integer),
                "<",
                &expression(Type::Integer),
                &mut diagnostics
            ),
            Type::Boolean
        );
        assert_eq!(
            check_binary(
                &expression(Type::Boolean),
                "&&",
                &expression(Type::Boolean),
                &mut diagnostics
            ),
            Type::Boolean
        );
        assert_eq!(
            check_binary(
                &expression(Type::String),
                "-",
                &expression(Type::String),
                &mut diagnostics
            ),
            Type::Error
        );
        assert_eq!(diagnostics.len(), 1);
    }
}
