use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::apex_ir;
use crate::diagnostic::{Diagnostic, Phase};
use crate::source::Span;
use crate::verify::VerificationResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl Artifact {
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Segment {
    generated_start: usize,
    generated_end: usize,
    source_start: usize,
    source_end: usize,
}

pub fn emit(
    program: &apex_ir::Program,
    api_version: &str,
    apex_package_path: Option<&str>,
) -> Vec<Artifact> {
    let mut artifacts = Vec::new();
    let mut sources = BTreeSet::new();
    let mut class_records = Vec::new();
    for class in &program.classes {
        sources.insert(class.source_path.clone());
        let mut writer = ApexWriter::new();
        writer.class(class);
        let class_path = format!("generated/main/default/classes/{}.cls", class.name);
        let metadata_path = format!("generated/main/default/classes/{}.cls-meta.xml", class.name);
        let map_path = format!("maps/{}.cls.map.json", class.name);
        let map = render_source_map(&class_path, &class.source_path, &writer.segments);
        artifacts.push(Artifact {
            path: class_path.clone(),
            bytes: writer.output.into_bytes(),
        });
        artifacts.push(Artifact {
            path: metadata_path.clone(),
            bytes: render_metadata(api_version).into_bytes(),
        });
        artifacts.push(Artifact {
            path: map_path.clone(),
            bytes: map.into_bytes(),
        });
        class_records.push((class.name.clone(), class_path, metadata_path, map_path));
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = render_manifest(api_version, &sources, &class_records);
    artifacts.push(Artifact {
        path: "build.json".into(),
        bytes: manifest.into_bytes(),
    });
    artifacts.push(Artifact {
        path: "sfdx-project.json".into(),
        bytes: render_sfdx_project(api_version, apex_package_path).into_bytes(),
    });
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    artifacts
}

pub fn write_artifacts(output_root: &Path, artifacts: &[Artifact]) -> Result<(), Box<Diagnostic>> {
    for directory in ["generated", "maps"] {
        let path = output_root.join(directory);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|error| Box::new(write_error(&path, error)))?;
        }
    }
    let build_manifest = output_root.join("build.json");
    if build_manifest.exists() {
        fs::remove_file(&build_manifest)
            .map_err(|error| Box::new(write_error(&build_manifest, error)))?;
    }
    for artifact in artifacts {
        let path = output_root.join(&artifact.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| Box::new(write_error(parent, error)))?;
        }
        fs::write(&path, &artifact.bytes).map_err(|error| Box::new(write_error(&path, error)))?;
    }
    Ok(())
}

pub fn render_artifacts(artifacts: &[Artifact]) -> String {
    let mut output = String::new();
    for artifact in artifacts {
        writeln!(output, "== {} ==", artifact.path).expect("writing to String cannot fail");
        output.push_str(artifact.text().expect("M3 artifacts are UTF-8"));
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }
    output
}

pub fn record_verification(artifacts: &mut [Artifact], result: &VerificationResult) {
    let manifest = artifacts
        .iter_mut()
        .find(|artifact| artifact.path == "build.json")
        .expect("emitted artifact set contains build.json");
    let text = std::str::from_utf8(&manifest.bytes).expect("build manifest is UTF-8");
    let exit_status = result
        .exit_status
        .map_or_else(|| "null".into(), |status| status.to_string());
    let evidence = format!(
        "[{{\"backend\": {}, \"revision\": {}, \"capabilityProfile\": {}, \"outcome\": {}, \"exitStatus\": {}, \"stdout\": {}, \"stderr\": {}}}]",
        json_string(&result.backend),
        json_string(&result.revision),
        json_string(&result.capability_profile),
        json_string(result.outcome.as_str()),
        exit_status,
        json_string(&result.stdout),
        json_string(&result.stderr),
    );
    manifest.bytes = text
        .replace(
            "\"verification\": []",
            &format!("\"verification\": {evidence}"),
        )
        .into_bytes();
}

fn write_error(path: &Path, error: std::io::Error) -> Diagnostic {
    Diagnostic::coded_error(
        Phase::Emit,
        "emit.write-failed",
        format!("failed to write `{}`", path.to_string_lossy()),
        None,
    )
    .with_note(error.to_string())
}

struct ApexWriter {
    output: String,
    segments: Vec<Segment>,
    indent: usize,
}

impl ApexWriter {
    fn new() -> Self {
        Self {
            output: String::new(),
            segments: Vec::new(),
            indent: 0,
        }
    }

    fn class(&mut self, class: &apex_ir::Class) {
        self.indentation();
        self.modifiers(&class.modifiers);
        self.output.push_str("class ");
        self.mapped(&class.name, class.origin);
        self.output.push_str(" {\n");
        self.indent += 1;
        for (index, member) in class.members.iter().enumerate() {
            if index > 0 {
                self.output.push('\n');
            }
            self.member(member);
        }
        self.indent -= 1;
        self.output.push_str("}\n");
    }

    fn member(&mut self, member: &apex_ir::Member) {
        match member {
            apex_ir::Member::Field(field) => {
                self.indentation();
                self.modifiers(&field.modifiers);
                write!(self.output, "{} ", field.ty).expect("writing to String cannot fail");
                self.mapped(&field.name, field.origin);
                if let Some(initializer) = &field.initializer {
                    self.output.push_str(" = ");
                    self.expression(initializer);
                }
                self.output.push_str(";\n");
            }
            apex_ir::Member::Property(property) => {
                self.indentation();
                self.modifiers(&property.modifiers);
                write!(self.output, "{} ", property.ty).expect("writing to String cannot fail");
                self.mapped(&property.name, property.origin);
                self.output.push_str(" {\n");
                self.indent += 1;
                for accessor in &property.accessors {
                    self.indentation();
                    self.modifiers(&accessor.modifiers);
                    self.mapped(&accessor.kind, accessor.origin);
                    self.output.push_str(";\n");
                }
                self.indent -= 1;
                self.indentation();
                self.output.push_str("}\n");
            }
            apex_ir::Member::Method(method) => {
                self.indentation();
                self.modifiers(&method.modifiers);
                write!(self.output, "{} ", method.return_type)
                    .expect("writing to String cannot fail");
                self.mapped(&method.name, method.origin);
                self.output.push('(');
                for (index, parameter) in method.parameters.iter().enumerate() {
                    if index > 0 {
                        self.output.push_str(", ");
                    }
                    write!(self.output, "{} ", parameter.ty)
                        .expect("writing to String cannot fail");
                    self.mapped(&parameter.name, parameter.origin);
                }
                self.output.push_str(") ");
                self.block(&method.body);
                self.output.push('\n');
            }
        }
    }

    fn block(&mut self, block: &apex_ir::Block) {
        self.output.push_str("{\n");
        self.indent += 1;
        for statement in &block.statements {
            self.statement(statement);
        }
        self.indent -= 1;
        self.indentation();
        self.output.push('}');
    }

    fn statement(&mut self, statement: &apex_ir::Statement) {
        use apex_ir::StatementKind as Kind;
        match &statement.kind {
            Kind::Block(block) => {
                self.indentation();
                self.block(block);
                self.output.push('\n');
            }
            Kind::Variable {
                ty,
                name,
                initializer,
            } => {
                self.indentation();
                write!(self.output, "{ty} ").expect("writing to String cannot fail");
                self.mapped(name, statement.origin);
                if let Some(initializer) = initializer {
                    self.output.push_str(" = ");
                    self.expression(initializer);
                }
                self.output.push_str(";\n");
            }
            Kind::Expression(expression) => {
                self.indentation();
                self.expression(expression);
                self.output.push_str(";\n");
            }
            Kind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.indentation();
                self.output.push_str("if (");
                self.expression(condition);
                self.output.push_str(") ");
                self.statement_body(then_branch);
                if let Some(branch) = else_branch {
                    self.indentation();
                    self.output.push_str("else ");
                    self.statement_body(branch);
                }
            }
            Kind::While { condition, body } => {
                self.indentation();
                self.output.push_str("while (");
                self.expression(condition);
                self.output.push_str(") ");
                self.statement_body(body);
            }
            Kind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.indentation();
                self.output.push_str("for (");
                if let Some(initializer) = initializer {
                    self.for_initializer(initializer, statement.origin);
                }
                self.output.push_str("; ");
                if let Some(condition) = condition {
                    self.expression(condition);
                }
                self.output.push_str("; ");
                self.expression_list(update);
                self.output.push_str(") ");
                self.statement_body(body);
            }
            Kind::EnhancedFor {
                ty,
                name,
                iterable,
                body,
            } => {
                self.indentation();
                write!(self.output, "for ({ty} ").expect("writing to String cannot fail");
                self.mapped(name, statement.origin);
                self.output.push_str(" : ");
                self.expression(iterable);
                self.output.push_str(") ");
                self.statement_body(body);
            }
            Kind::Return(expression) => {
                self.indentation();
                self.output.push_str("return");
                if let Some(expression) = expression {
                    self.output.push(' ');
                    self.expression(expression);
                }
                self.output.push_str(";\n");
            }
            Kind::Break => {
                self.indentation();
                self.mapped("break", statement.origin);
                self.output.push_str(";\n");
            }
            Kind::Continue => {
                self.indentation();
                self.mapped("continue", statement.origin);
                self.output.push_str(";\n");
            }
            Kind::Empty => {
                self.indentation();
                self.output.push_str(";\n");
            }
        }
    }

    fn statement_body(&mut self, statement: &apex_ir::Statement) {
        if let apex_ir::StatementKind::Block(block) = &statement.kind {
            self.block(block);
            self.output.push('\n');
        } else {
            self.output.push_str("{\n");
            self.indent += 1;
            self.statement(statement);
            self.indent -= 1;
            self.indentation();
            self.output.push_str("}\n");
        }
    }

    fn for_initializer(&mut self, initializer: &apex_ir::ForInitializer, origin: Span) {
        match initializer {
            apex_ir::ForInitializer::Variable {
                ty,
                name,
                initializer,
            } => {
                write!(self.output, "{ty} ").expect("writing to String cannot fail");
                self.mapped(name, origin);
                if let Some(initializer) = initializer {
                    self.output.push_str(" = ");
                    self.expression(initializer);
                }
            }
            apex_ir::ForInitializer::Expressions(expressions) => self.expression_list(expressions),
        }
    }

    fn expression_list(&mut self, expressions: &[apex_ir::Expression]) {
        for (index, expression) in expressions.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.expression(expression);
        }
    }

    fn expression(&mut self, expression: &apex_ir::Expression) {
        use apex_ir::ExpressionKind as Kind;
        match &expression.kind {
            Kind::Name(name) | Kind::Integer(name) => self.mapped(name, expression.origin),
            Kind::String(value) => {
                let escaped = apex_string(value);
                self.mapped(&escaped, expression.origin);
            }
            Kind::Boolean(value) => {
                self.mapped(if *value { "true" } else { "false" }, expression.origin)
            }
            Kind::Null => self.mapped("null", expression.origin),
            Kind::This => self.mapped("this", expression.origin),
            Kind::Parenthesized(inner) => {
                self.output.push('(');
                self.expression(inner);
                self.output.push(')');
            }
            Kind::Call {
                receiver,
                name,
                arguments,
            } => {
                if let Some(receiver) = receiver {
                    self.expression(receiver);
                    self.output.push('.');
                }
                self.mapped(name, expression.origin);
                self.output.push('(');
                self.expression_list(arguments);
                self.output.push(')');
            }
            Kind::Member { object, name } => {
                self.expression(object);
                self.output.push('.');
                self.mapped(name, expression.origin);
            }
            Kind::Index { object, index } => {
                self.expression(object);
                self.output.push('[');
                self.expression(index);
                self.output.push(']');
            }
            Kind::Unary { operator, operand } => {
                self.output.push_str(operator);
                self.expression(operand);
            }
            Kind::Binary {
                left,
                operator,
                right,
            } => {
                self.expression(left);
                write!(self.output, " {operator} ").expect("writing to String cannot fail");
                self.expression(right);
            }
            Kind::Conditional {
                condition,
                then_expression,
                else_expression,
            } => {
                self.expression(condition);
                self.output.push_str(" ? ");
                self.expression(then_expression);
                self.output.push_str(" : ");
                self.expression(else_expression);
            }
            Kind::Assignment {
                target,
                operator,
                value,
            } => {
                self.expression(target);
                write!(self.output, " {operator} ").expect("writing to String cannot fail");
                self.expression(value);
            }
        }
    }

    fn modifiers(&mut self, modifiers: &[String]) {
        for modifier in modifiers {
            self.output.push_str(modifier);
            self.output.push(' ');
        }
    }

    fn indentation(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }

    fn mapped(&mut self, text: &str, origin: Span) {
        let start = self.output.len();
        self.output.push_str(text);
        let end = self.output.len();
        if start != end {
            self.segments.push(Segment {
                generated_start: start,
                generated_end: end,
                source_start: origin.start(),
                source_end: origin.end(),
            });
        }
    }
}

fn apex_string(value: &str) -> String {
    let mut output = String::from("'");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\'' => output.push_str("\\'"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            character => output.push(character),
        }
    }
    output.push('\'');
    output
}

fn render_metadata(api_version: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ApexClass xmlns=\"http://soap.sforce.com/2006/04/metadata\">\n    <apiVersion>{api_version}</apiVersion>\n    <status>Active</status>\n</ApexClass>\n"
    )
}

fn render_sfdx_project(api_version: &str, apex_package_path: Option<&str>) -> String {
    let mut output = String::new();
    output.push_str("{\n  \"packageDirectories\": [\n");
    output.push_str("    {\"path\": \"generated\", \"default\": true}");
    if let Some(path) = apex_package_path {
        writeln!(output, ",").unwrap();
        write!(output, "    {{\"path\": {}}}", json_string(path)).unwrap();
    }
    output.push_str("\n  ],\n");
    writeln!(
        output,
        "  \"sourceApiVersion\": {}\n}}",
        json_string(api_version)
    )
    .unwrap();
    output
}

fn render_source_map(generated_path: &str, source_path: &str, segments: &[Segment]) -> String {
    let mut output = String::new();
    writeln!(output, "{{").unwrap();
    writeln!(output, "  \"version\": 1,").unwrap();
    writeln!(output, "  \"generated\": {},", json_string(generated_path)).unwrap();
    writeln!(output, "  \"source\": {},", json_string(source_path)).unwrap();
    writeln!(output, "  \"segments\": [").unwrap();
    for (index, segment) in segments.iter().enumerate() {
        writeln!(
            output,
            "    {{\"generated\": [{}, {}], \"source\": [{}, {}]}}{}",
            segment.generated_start,
            segment.generated_end,
            segment.source_start,
            segment.source_end,
            if index + 1 == segments.len() { "" } else { "," }
        )
        .unwrap();
    }
    output.push_str("  ]\n}\n");
    output
}

fn render_manifest(
    api_version: &str,
    sources: &BTreeSet<String>,
    classes: &[(String, String, String, String)],
) -> String {
    let mut output = String::new();
    output.push_str("{\n  \"formatVersion\": 1,\n");
    writeln!(
        output,
        "  \"salesforceApiVersion\": {},",
        json_string(api_version)
    )
    .unwrap();
    output.push_str("  \"sfdxProject\": \"sfdx-project.json\",\n");
    output.push_str("  \"sources\": [");
    for (index, source) in sources.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&json_string(source));
    }
    output.push_str("],\n  \"classes\": [\n");
    for (index, (name, class_path, metadata_path, map_path)) in classes.iter().enumerate() {
        writeln!(
            output,
            "    {{\"name\": {}, \"apex\": {}, \"metadata\": {}, \"sourceMap\": {}}}{}",
            json_string(name),
            json_string(class_path),
            json_string(metadata_path),
            json_string(map_path),
            if index + 1 == classes.len() { "" } else { "," }
        )
        .unwrap();
    }
    output.push_str("  ],\n  \"verification\": []\n}\n");
    output
}

fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{001f}' => {
                write!(output, "\\u{:04x}", u32::from(character)).unwrap();
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

pub fn artifact_path(output_root: &Path, artifact: &Artifact) -> PathBuf {
    output_root.join(&artifact.path)
}

#[cfg(test)]
mod tests {
    use super::{apex_string, render_metadata};

    #[test]
    fn escapes_apex_strings_and_emits_exact_metadata() {
        assert_eq!(apex_string("a'b\\c\n"), "'a\\'b\\\\c\\n'");
        assert!(render_metadata("65.0").contains("<apiVersion>65.0</apiVersion>"));
    }
}
