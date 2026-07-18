use std::path::Path;

use crate::Diagnostic;
use crate::check;
use crate::emit::{Artifact, emit};
use crate::hir;
use crate::lower;
use crate::project::{self, ProjectConfig, apex_package_path_from_output};
use crate::source::SourceMap;

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
