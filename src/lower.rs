use crate::{apex_ir, hir};

pub fn lower(program: &hir::Program) -> apex_ir::Program {
    apex_ir::Program {
        classes: program.classes.iter().map(lower_class).collect(),
    }
}

fn lower_class(class: &hir::Class) -> apex_ir::Class {
    apex_ir::Class {
        name: class.name.clone(),
        modifiers: class.modifiers.clone(),
        members: class.members.iter().map(lower_member).collect(),
        source_path: class.source_path.clone(),
        origin: class.span,
    }
}

fn lower_member(member: &hir::Member) -> apex_ir::Member {
    match member {
        hir::Member::Field(field) => apex_ir::Member::Field(apex_ir::Field {
            name: field.name.clone(),
            modifiers: field.modifiers.clone(),
            ty: field.ty.clone(),
            initializer: field.initializer.as_ref().map(lower_expression),
            origin: field.span,
        }),
        hir::Member::Property(property) => apex_ir::Member::Property(apex_ir::Property {
            name: property.name.clone(),
            modifiers: property.modifiers.clone(),
            ty: property.ty.clone(),
            accessors: property
                .accessors
                .iter()
                .map(|accessor| apex_ir::Accessor {
                    kind: accessor.kind.clone(),
                    modifiers: accessor.modifiers.clone(),
                    origin: accessor.span,
                })
                .collect(),
            origin: property.span,
        }),
        hir::Member::Method(method) => apex_ir::Member::Method(apex_ir::Method {
            name: method.name.clone(),
            modifiers: method.modifiers.clone(),
            return_type: method.return_type.clone(),
            parameters: method
                .parameters
                .iter()
                .map(|parameter| apex_ir::Parameter {
                    name: parameter.name.clone(),
                    ty: parameter.ty.clone(),
                    origin: parameter.span,
                })
                .collect(),
            body: lower_block(&method.body),
            origin: method.span,
        }),
    }
}

fn lower_block(block: &hir::Block) -> apex_ir::Block {
    apex_ir::Block {
        statements: block.statements.iter().map(lower_statement).collect(),
        origin: block.span,
    }
}

fn lower_statement(statement: &hir::Statement) -> apex_ir::Statement {
    let kind = match &statement.kind {
        hir::StatementKind::Block(block) => apex_ir::StatementKind::Block(lower_block(block)),
        hir::StatementKind::Variable {
            ty,
            name,
            initializer,
        } => apex_ir::StatementKind::Variable {
            ty: ty.clone(),
            name: name.clone(),
            initializer: initializer.as_ref().map(lower_expression),
        },
        hir::StatementKind::Expression(expression) => {
            apex_ir::StatementKind::Expression(lower_expression(expression))
        }
        hir::StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => apex_ir::StatementKind::If {
            condition: lower_expression(condition),
            then_branch: Box::new(lower_statement(then_branch)),
            else_branch: else_branch.as_deref().map(lower_statement).map(Box::new),
        },
        hir::StatementKind::While { condition, body } => apex_ir::StatementKind::While {
            condition: lower_expression(condition),
            body: Box::new(lower_statement(body)),
        },
        hir::StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => apex_ir::StatementKind::For {
            initializer: initializer.as_ref().map(lower_for_initializer),
            condition: condition.as_ref().map(lower_expression),
            update: update.iter().map(lower_expression).collect(),
            body: Box::new(lower_statement(body)),
        },
        hir::StatementKind::EnhancedFor {
            ty,
            name,
            iterable,
            body,
        } => apex_ir::StatementKind::EnhancedFor {
            ty: ty.clone(),
            name: name.clone(),
            iterable: lower_expression(iterable),
            body: Box::new(lower_statement(body)),
        },
        hir::StatementKind::Return(expression) => {
            apex_ir::StatementKind::Return(expression.as_ref().map(lower_expression))
        }
        hir::StatementKind::Break => apex_ir::StatementKind::Break,
        hir::StatementKind::Continue => apex_ir::StatementKind::Continue,
        hir::StatementKind::Empty => apex_ir::StatementKind::Empty,
    };
    apex_ir::Statement {
        kind,
        origin: statement.span,
    }
}

fn lower_for_initializer(initializer: &hir::ForInitializer) -> apex_ir::ForInitializer {
    match initializer {
        hir::ForInitializer::Variable {
            ty,
            name,
            initializer,
        } => apex_ir::ForInitializer::Variable {
            ty: ty.clone(),
            name: name.clone(),
            initializer: initializer.as_deref().map(lower_expression).map(Box::new),
        },
        hir::ForInitializer::Expressions(expressions) => {
            apex_ir::ForInitializer::Expressions(expressions.iter().map(lower_expression).collect())
        }
    }
}

fn lower_expression(expression: &hir::Expression) -> apex_ir::Expression {
    use hir::ExpressionKind as Hir;
    let kind = match &expression.kind {
        Hir::Name { spelling, .. } => apex_ir::ExpressionKind::Name(spelling.clone()),
        Hir::Integer(value) => apex_ir::ExpressionKind::Integer(value.clone()),
        Hir::String(value) => apex_ir::ExpressionKind::String(value.clone()),
        Hir::Boolean(value) => apex_ir::ExpressionKind::Boolean(*value),
        Hir::Null => apex_ir::ExpressionKind::Null,
        Hir::This => apex_ir::ExpressionKind::This,
        Hir::Parenthesized(inner) => {
            apex_ir::ExpressionKind::Parenthesized(Box::new(lower_expression(inner)))
        }
        Hir::Call {
            receiver,
            name,
            arguments,
            ..
        } => apex_ir::ExpressionKind::Call {
            receiver: receiver.as_deref().map(lower_expression).map(Box::new),
            name: name.clone(),
            arguments: arguments.iter().map(lower_expression).collect(),
        },
        Hir::Member { object, name, .. } => apex_ir::ExpressionKind::Member {
            object: Box::new(lower_expression(object)),
            name: name.clone(),
        },
        Hir::Index { object, index, .. } => apex_ir::ExpressionKind::Index {
            object: Box::new(lower_expression(object)),
            index: Box::new(lower_expression(index)),
        },
        Hir::Unary { operator, operand } => apex_ir::ExpressionKind::Unary {
            operator: operator.clone(),
            operand: Box::new(lower_expression(operand)),
        },
        Hir::Binary {
            left,
            operator,
            right,
        } => apex_ir::ExpressionKind::Binary {
            left: Box::new(lower_expression(left)),
            operator: operator.clone(),
            right: Box::new(lower_expression(right)),
        },
        Hir::Conditional {
            condition,
            then_expression,
            else_expression,
        } => apex_ir::ExpressionKind::Conditional {
            condition: Box::new(lower_expression(condition)),
            then_expression: Box::new(lower_expression(then_expression)),
            else_expression: Box::new(lower_expression(else_expression)),
        },
        Hir::Assignment {
            target,
            operator,
            value,
        } => apex_ir::ExpressionKind::Assignment {
            target: Box::new(lower_expression(target)),
            operator: operator.clone(),
            value: Box::new(lower_expression(value)),
        },
    };
    apex_ir::Expression {
        kind,
        origin: expression.span,
    }
}

#[cfg(test)]
mod tests {
    use super::lower;
    use crate::hir;

    #[test]
    fn lowers_empty_hir_into_a_distinct_empty_apex_program() {
        let lowered = lower(&hir::Program { classes: vec![] });
        assert!(lowered.classes.is_empty());
    }
}
