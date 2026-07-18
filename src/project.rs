use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::apex_api::{ApexBoundary, parse_boundary};
use crate::ast::CompilationUnit;
use crate::diagnostic::{Diagnostic, Phase};
use crate::lexer::lex;
use crate::parser::parse;
use crate::source::{SourceId, SourceMap};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectConfig {
    pub salesforce_api_version: String,
    pub source_root: PathBuf,
    pub output_root: PathBuf,
    pub apex_boundary: Option<PathBuf>,
    pub apex_source_root: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct SourceUnit {
    pub relative_path: PathBuf,
    pub source: SourceId,
    pub syntax: CompilationUnit,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub root: PathBuf,
    pub config: ProjectConfig,
    pub sources: SourceMap,
    pub units: Vec<SourceUnit>,
    pub boundary: ApexBoundary,
}

#[derive(Clone, Debug)]
pub struct ProjectFailure {
    pub sources: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn load_project(root: &Path) -> Result<Project, ProjectFailure> {
    let root = match fs::canonicalize(root) {
        Ok(root) => root,
        Err(error) => {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics: vec![
                    Diagnostic::coded_error(
                        Phase::Project,
                        "project.root-unavailable",
                        format!("cannot open Zenith project `{}`", root.to_string_lossy()),
                        None,
                    )
                    .with_note(error.to_string()),
                ],
            });
        }
    };
    let config_path = root.join("zenith.toml");
    let config_text = match read_utf8(&config_path, "project configuration") {
        Ok(text) => text,
        Err(diagnostic) => {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics: vec![*diagnostic],
            });
        }
    };
    let config = match parse_config(&config_text) {
        Ok(config) => config,
        Err(diagnostics) => {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics,
            });
        }
    };
    if let Some(apex_source_root) = &config.apex_source_root {
        let path = root.join(apex_source_root);
        if !path.is_dir() {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics: vec![Diagnostic::coded_error(
                    Phase::Project,
                    "project.apex-source-root-unavailable",
                    format!(
                        "handwritten Apex source root `{}` is not a directory",
                        apex_source_root.display()
                    ),
                    None,
                )],
            });
        }
    }
    let source_root = root.join(&config.source_root);
    let paths = match discover_sources(&source_root) {
        Ok(paths) if !paths.is_empty() => paths,
        Ok(_) => {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics: vec![Diagnostic::coded_error(
                    Phase::Project,
                    "project.no-sources",
                    format!(
                        "no `.zen` files were found beneath `{}`",
                        config.source_root.display()
                    ),
                    None,
                )],
            });
        }
        Err(diagnostic) => {
            return Err(ProjectFailure {
                sources: SourceMap::new(),
                diagnostics: vec![*diagnostic],
            });
        }
    };

    let mut sources = SourceMap::new();
    let mut units = Vec::new();
    let mut diagnostics = Vec::new();
    for path in paths {
        let relative_path = path
            .strip_prefix(&root)
            .expect("discovered source is beneath project root")
            .to_path_buf();
        let text = match read_utf8(&path, "Zenith source") {
            Ok(text) => text,
            Err(diagnostic) => {
                diagnostics.push(*diagnostic);
                continue;
            }
        };
        let source = sources.add(&relative_path, text);
        let file = sources.get(source).expect("source was inserted");
        let lexical = lex(file);
        if lexical.has_errors() {
            diagnostics.extend(lexical.diagnostics);
            continue;
        }
        let parsed = parse(file, &lexical.tokens);
        if parsed.has_errors() {
            diagnostics.extend(parsed.diagnostics);
            continue;
        }
        units.push(SourceUnit {
            relative_path,
            source,
            syntax: parsed
                .unit
                .expect("successful parsing produces a compilation unit"),
        });
    }

    let boundary = if let Some(relative) = &config.apex_boundary {
        let path = root.join(relative);
        match read_utf8(&path, "Apex boundary summary") {
            Ok(text) => match parse_boundary(relative, text, &mut sources) {
                Ok(boundary) => boundary,
                Err(mut boundary_diagnostics) => {
                    diagnostics.append(&mut boundary_diagnostics);
                    ApexBoundary {
                        classes: Default::default(),
                    }
                }
            },
            Err(diagnostic) => {
                diagnostics.push(*diagnostic);
                ApexBoundary {
                    classes: Default::default(),
                }
            }
        }
    } else {
        ApexBoundary {
            classes: Default::default(),
        }
    };

    if diagnostics.is_empty() {
        Ok(Project {
            root,
            config,
            sources,
            units,
            boundary,
        })
    } else {
        Err(ProjectFailure {
            sources,
            diagnostics,
        })
    }
}

pub fn apex_package_path_from_output(config: &ProjectConfig) -> Option<String> {
    config
        .apex_source_root
        .as_ref()
        .map(|target| relative_path(&config.output_root, target))
}

fn relative_path(from: &Path, to: &Path) -> String {
    let from: Vec<_> = from
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::CurDir => None,
            _ => None,
        })
        .collect();
    let to: Vec<_> = to
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::CurDir => None,
            _ => None,
        })
        .collect();
    let common = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = vec!["..".to_owned(); from.len() - common];
    parts.extend(to.into_iter().skip(common));
    if parts.is_empty() {
        ".".into()
    } else {
        parts.join("/")
    }
}

fn parse_config(text: &str) -> Result<ProjectConfig, Vec<Diagnostic>> {
    let mut api_version = None;
    let mut source_root = None;
    let mut output_root = None;
    let mut apex_boundary = None;
    let mut apex_source_root = None;
    let mut diagnostics = Vec::new();

    for (line_index, raw_line) in text.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            diagnostics.push(config_error(
                "project.invalid-config",
                format!("expected `key = \"value\"` on line {}", line_index + 1),
            ));
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        let Some(value) = value
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
        else {
            diagnostics.push(config_error(
                "project.invalid-config",
                format!("configuration value for `{key}` must be a quoted string"),
            ));
            continue;
        };
        let slot = match key {
            "salesforce-api-version" => &mut api_version,
            "source-root" => &mut source_root,
            "output-root" => &mut output_root,
            "apex-boundary" => &mut apex_boundary,
            "apex-source-root" => &mut apex_source_root,
            _ => {
                diagnostics.push(config_error(
                    "project.unknown-config-key",
                    format!("unknown configuration key `{key}`"),
                ));
                continue;
            }
        };
        if slot.replace(value.to_owned()).is_some() {
            diagnostics.push(config_error(
                "project.duplicate-config-key",
                format!("configuration key `{key}` is declared more than once"),
            ));
        }
    }

    let Some(api_version) = api_version else {
        diagnostics.push(config_error(
            "project.missing-api-version",
            "`salesforce-api-version` is required",
        ));
        return Err(diagnostics);
    };
    if !valid_api_version(&api_version) {
        diagnostics.push(config_error(
            "project.invalid-api-version",
            format!("Salesforce API version `{api_version}` must use the form `<integer>.0`"),
        ));
    }

    let source_root = source_root.unwrap_or_else(|| "src".into());
    let output_root = output_root.unwrap_or_else(|| ".zenith".into());
    for (key, value) in [
        ("source-root", source_root.as_str()),
        ("output-root", output_root.as_str()),
    ]
    .into_iter()
    .chain(
        apex_boundary
            .as_deref()
            .map(|value| ("apex-boundary", value)),
    )
    .chain(
        apex_source_root
            .as_deref()
            .map(|value| ("apex-source-root", value)),
    ) {
        if !valid_relative_path(value) {
            diagnostics.push(config_error(
                "project.invalid-config-path",
                format!("`{key}` must be a non-empty relative path without `..`"),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(ProjectConfig {
            salesforce_api_version: api_version,
            source_root: PathBuf::from(source_root),
            output_root: PathBuf::from(output_root),
            apex_boundary: apex_boundary.map(PathBuf::from),
            apex_source_root: apex_source_root.map(PathBuf::from),
        })
    } else {
        Err(diagnostics)
    }
}

fn valid_api_version(version: &str) -> bool {
    version
        .strip_suffix(".0")
        .is_some_and(|major| !major.is_empty() && major.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| !matches!(component, Component::ParentDir | Component::RootDir))
}

fn config_error(code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::coded_error(Phase::Project, code, message, None)
}

fn discover_sources(root: &Path) -> Result<Vec<PathBuf>, Box<Diagnostic>> {
    let mut paths = Vec::new();
    discover_into(root, &mut paths)?;
    paths.sort_by(|left, right| {
        left.to_string_lossy()
            .replace('\\', "/")
            .cmp(&right.to_string_lossy().replace('\\', "/"))
    });
    Ok(paths)
}

fn discover_into(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Box<Diagnostic>> {
    let entries = fs::read_dir(directory).map_err(|error| {
        Box::new(
            Diagnostic::coded_error(
                Phase::Project,
                "project.source-root-unavailable",
                format!("cannot read source root `{}`", directory.to_string_lossy()),
                None,
            )
            .with_note(error.to_string()),
        )
    })?;
    let mut entries = entries.collect::<Result<Vec<_>, _>>().map_err(|error| {
        Box::new(
            Diagnostic::coded_error(
                Phase::Project,
                "project.discovery-failed",
                format!("cannot enumerate `{}`", directory.to_string_lossy()),
                None,
            )
            .with_note(error.to_string()),
        )
    })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| {
            Box::new(
                Diagnostic::coded_error(
                    Phase::Project,
                    "project.discovery-failed",
                    format!("cannot inspect `{}`", entry.path().to_string_lossy()),
                    None,
                )
                .with_note(error.to_string()),
            )
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            discover_into(&entry.path(), paths)?;
        } else if file_type.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "zen")
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn read_utf8(path: &Path, description: &str) -> Result<String, Box<Diagnostic>> {
    let bytes = fs::read(path).map_err(|error| {
        Box::new(
            Diagnostic::coded_error(
                Phase::Project,
                "project.read-failed",
                format!("failed to read {description} `{}`", path.to_string_lossy()),
                None,
            )
            .with_note(error.to_string()),
        )
    })?;
    String::from_utf8(bytes).map_err(|error| {
        Box::new(
            Diagnostic::coded_error(
                Phase::Source,
                "source.invalid-utf8",
                format!("{} is not valid UTF-8", path.to_string_lossy()),
                None,
            )
            .with_note(format!(
                "invalid UTF-8 begins at byte {}",
                error.utf8_error().valid_up_to()
            ))
            .with_help("save project files as UTF-8"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{ProjectConfig, apex_package_path_from_output, parse_config};
    use std::path::Path;

    #[test]
    fn config_requires_a_valid_api_version_and_safe_known_paths() {
        let diagnostics = parse_config(
            "source-root = \"../src\"\nunknown = \"x\"\nsalesforce-api-version = \"65\"\n",
        )
        .unwrap_err();
        let codes: Vec<_> = diagnostics.iter().map(|item| item.code.as_str()).collect();
        assert!(codes.contains(&"project.invalid-config-path"));
        assert!(codes.contains(&"project.unknown-config-key"));
        assert!(codes.contains(&"project.invalid-api-version"));
    }

    #[test]
    fn config_applies_documented_defaults() {
        let config = parse_config("salesforce-api-version = \"65.0\"\n").unwrap();
        assert_eq!(config.source_root, Path::new("src"));
        assert_eq!(config.output_root, Path::new(".zenith"));
        assert_eq!(config.apex_boundary, None);
        assert_eq!(config.apex_source_root, None);
    }

    #[test]
    fn computes_sfdx_package_paths_relative_to_the_output_root() {
        let config = ProjectConfig {
            salesforce_api_version: "65.0".into(),
            source_root: "src".into(),
            output_root: ".zenith".into(),
            apex_boundary: None,
            apex_source_root: Some("handwritten".into()),
        };
        assert_eq!(
            apex_package_path_from_output(&config).as_deref(),
            Some("../handwritten")
        );
    }
}
