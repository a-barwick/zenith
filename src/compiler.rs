use std::path::Path;

use crate::Diagnostic;
use crate::check;
use crate::emit::{Artifact, emit};
use crate::hir;
use crate::lower;
use crate::project::{self, ProjectConfig, apex_package_path_from_output};
use crate::source::SourceMap;
use crate::verify::{APEX_EXEC_M3_PROFILE, APEX_EXEC_M4_PROFILE};

#[derive(Clone, Debug)]
pub struct Compilation {
    pub sources: SourceMap,
    pub config: Option<ProjectConfig>,
    pub hir: Option<hir::Program>,
    pub artifacts: Vec<Artifact>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Compilation {
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn class_count(&self) -> usize {
        self.hir.as_ref().map_or(0, |program| program.classes.len())
    }

    pub fn apex_exec_profile(&self) -> &'static str {
        if self
            .hir
            .as_ref()
            .is_some_and(program_requires_m4_apex_profile)
        {
            APEX_EXEC_M4_PROFILE
        } else {
            APEX_EXEC_M3_PROFILE
        }
    }
}

fn program_requires_m4_apex_profile(program: &hir::Program) -> bool {
    program.classes.iter().any(|class| {
        !matches!(class.kind, hir::ClassKind::Class)
            || class.members.iter().any(|member| match member {
                hir::Member::Field(field) => field
                    .initializer
                    .as_ref()
                    .is_some_and(expression_requires_m4_apex_profile),
                hir::Member::Property(_) => false,
                hir::Member::Method(method) => block_requires_m4_apex_profile(&method.body),
            })
    })
}

fn block_requires_m4_apex_profile(block: &hir::Block) -> bool {
    block
        .statements
        .iter()
        .any(statement_requires_m4_apex_profile)
}

fn statement_requires_m4_apex_profile(statement: &hir::Statement) -> bool {
    match &statement.kind {
        hir::StatementKind::Block(block) => block_requires_m4_apex_profile(block),
        hir::StatementKind::Variable { initializer, .. } => initializer
            .as_ref()
            .is_some_and(expression_requires_m4_apex_profile),
        hir::StatementKind::Expression(expression) => {
            expression_requires_m4_apex_profile(expression)
        }
        hir::StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expression_requires_m4_apex_profile(condition)
                || statement_requires_m4_apex_profile(then_branch)
                || else_branch
                    .as_deref()
                    .is_some_and(statement_requires_m4_apex_profile)
        }
        hir::StatementKind::While { condition, body } => {
            expression_requires_m4_apex_profile(condition)
                || statement_requires_m4_apex_profile(body)
        }
        hir::StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => {
            initializer
                .as_ref()
                .is_some_and(|initializer| match initializer {
                    hir::ForInitializer::Variable { initializer, .. } => initializer
                        .as_deref()
                        .is_some_and(expression_requires_m4_apex_profile),
                    hir::ForInitializer::Expressions(expressions) => {
                        expressions.iter().any(expression_requires_m4_apex_profile)
                    }
                })
                || condition
                    .as_ref()
                    .is_some_and(expression_requires_m4_apex_profile)
                || update.iter().any(expression_requires_m4_apex_profile)
                || statement_requires_m4_apex_profile(body)
        }
        hir::StatementKind::EnhancedFor { iterable, body, .. } => {
            expression_requires_m4_apex_profile(iterable)
                || statement_requires_m4_apex_profile(body)
        }
        hir::StatementKind::Match { .. } => true,
        hir::StatementKind::Return(expression) => expression
            .as_ref()
            .is_some_and(expression_requires_m4_apex_profile),
        hir::StatementKind::Break | hir::StatementKind::Continue | hir::StatementKind::Empty => {
            false
        }
    }
}

fn expression_requires_m4_apex_profile(expression: &hir::Expression) -> bool {
    match &expression.kind {
        hir::ExpressionKind::Name { .. }
        | hir::ExpressionKind::Integer(_)
        | hir::ExpressionKind::String(_)
        | hir::ExpressionKind::Boolean(_)
        | hir::ExpressionKind::Null
        | hir::ExpressionKind::This => false,
        hir::ExpressionKind::Parenthesized(inner) => expression_requires_m4_apex_profile(inner),
        hir::ExpressionKind::New { .. } => true,
        hir::ExpressionKind::Call {
            receiver,
            arguments,
            safe,
            ..
        } => {
            *safe
                || receiver
                    .as_deref()
                    .is_some_and(expression_requires_m4_apex_profile)
                || arguments.iter().any(expression_requires_m4_apex_profile)
        }
        hir::ExpressionKind::Member { object, safe, .. } => {
            *safe || expression_requires_m4_apex_profile(object)
        }
        hir::ExpressionKind::Index { object, index, .. } => {
            expression_requires_m4_apex_profile(object)
                || expression_requires_m4_apex_profile(index)
        }
        hir::ExpressionKind::Unary { operand, .. } => expression_requires_m4_apex_profile(operand),
        hir::ExpressionKind::Binary {
            left,
            operator,
            right,
        } => {
            operator == "??"
                || expression_requires_m4_apex_profile(left)
                || expression_requires_m4_apex_profile(right)
        }
        hir::ExpressionKind::Conditional {
            condition,
            then_expression,
            else_expression,
        } => {
            expression_requires_m4_apex_profile(condition)
                || expression_requires_m4_apex_profile(then_expression)
                || expression_requires_m4_apex_profile(else_expression)
        }
        hir::ExpressionKind::Assignment { target, value, .. } => {
            expression_requires_m4_apex_profile(target)
                || expression_requires_m4_apex_profile(value)
        }
    }
}

pub fn compile_project(root: &Path) -> Compilation {
    let project = match project::load_project(root) {
        Ok(project) => project,
        Err(failure) => {
            return Compilation {
                sources: failure.sources,
                config: None,
                hir: None,
                artifacts: Vec::new(),
                diagnostics: failure.diagnostics,
            };
        }
    };
    let program = match check::check(&project.units, &project.boundary) {
        Ok(program) => program,
        Err(diagnostics) => {
            return Compilation {
                sources: project.sources,
                config: Some(project.config),
                hir: None,
                artifacts: Vec::new(),
                diagnostics,
            };
        }
    };
    let apex = lower::lower(&program);
    let diagnostics = crate::apex_ir::validate(&apex);
    if !diagnostics.is_empty() {
        return Compilation {
            sources: project.sources,
            config: Some(project.config),
            hir: Some(program),
            artifacts: Vec::new(),
            diagnostics,
        };
    }
    let apex_package_path = apex_package_path_from_output(&project.config);
    let artifacts = emit(
        &apex,
        &project.config.salesforce_api_version,
        apex_package_path.as_deref(),
    );
    Compilation {
        sources: project.sources,
        config: Some(project.config),
        hir: Some(program),
        artifacts,
        diagnostics: Vec::new(),
    }
}
