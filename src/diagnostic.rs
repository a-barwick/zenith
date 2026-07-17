use std::cmp::Ordering;
use std::fmt::{self, Write};

use crate::source::{SourceFile, SourceLocation, SourceMap, Span};

/// Compiler phase that owns a diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Source,
    Lex,
    Parse,
    Resolve,
    Type,
    Schema,
    Effect,
    Lower,
    Emit,
    Verify,
    Project,
}

impl fmt::Display for Phase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Source => "source",
            Self::Lex => "lex",
            Self::Parse => "parse",
            Self::Resolve => "resolve",
            Self::Type => "type",
            Self::Schema => "schema",
            Self::Effect => "effect",
            Self::Lower => "lower",
            Self::Emit => "emit",
            Self::Verify => "verify",
            Self::Project => "project",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLabel {
    pub span: Span,
    pub message: Option<String>,
}

impl SourceLabel {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: Some(message.into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub phase: Phase,
    pub message: String,
    pub span: Option<Span>,
    pub primary_label: Option<String>,
    pub secondary_labels: Vec<SourceLabel>,
    pub notes: Vec<String>,
    pub help: Vec<String>,
}

impl Diagnostic {
    pub fn error(phase: Phase, message: impl Into<String>, span: Option<Span>) -> Self {
        Self::coded_error(phase, format!("{phase}.error"), message, span)
    }

    pub fn warning(phase: Phase, message: impl Into<String>, span: Option<Span>) -> Self {
        Self::coded_warning(phase, format!("{phase}.warning"), message, span)
    }

    pub fn coded_error(
        phase: Phase,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self::new(Severity::Error, phase, code, message, span)
    }

    pub fn coded_warning(
        phase: Phase,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self::new(Severity::Warning, phase, code, message, span)
    }

    fn new(
        severity: Severity,
        phase: Phase,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            phase,
            message: message.into(),
            span,
            primary_label: None,
            secondary_labels: Vec::new(),
            notes: Vec::new(),
            help: Vec::new(),
        }
    }

    pub fn with_primary_label(mut self, message: impl Into<String>) -> Self {
        self.primary_label = Some(message.into());
        self
    }

    pub fn with_secondary_label(mut self, label: SourceLabel) -> Self {
        self.secondary_labels.push(label);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

/// Renders diagnostics in deterministic source order.
pub fn render_diagnostics(sources: &SourceMap, diagnostics: &[Diagnostic]) -> String {
    let mut ordered: Vec<_> = diagnostics.iter().collect();
    ordered.sort_by(|left, right| compare_diagnostics(left, right));

    let mut rendered = String::new();
    for (index, diagnostic) in ordered.into_iter().enumerate() {
        if index > 0 {
            rendered.push('\n');
        }
        render_diagnostic(&mut rendered, sources, diagnostic);
    }
    rendered
}

fn compare_diagnostics(left: &Diagnostic, right: &Diagnostic) -> Ordering {
    match (left.span, right.span) {
        (Some(left), Some(right)) => {
            (left.source(), left.start()).cmp(&(right.source(), right.start()))
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
    .then_with(|| severity_rank(left.severity).cmp(&severity_rank(right.severity)))
    .then_with(|| left.code.cmp(&right.code))
    .then_with(|| left.message.cmp(&right.message))
}

const fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
    }
}

fn render_diagnostic(output: &mut String, sources: &SourceMap, diagnostic: &Diagnostic) {
    writeln!(
        output,
        "{}[{}]: {}",
        diagnostic.severity, diagnostic.code, diagnostic.message
    )
    .expect("writing to String cannot fail");

    let mut generated_notes = Vec::new();
    if let Some(span) = diagnostic.span {
        if let Some(file) = sources.get(span.source()) {
            render_label(
                output,
                file,
                span,
                diagnostic.primary_label.as_deref(),
                " -->",
                &mut generated_notes,
            );
        } else {
            generated_notes.push("diagnostic source was unavailable for display".to_owned());
        }
    }

    for secondary in &diagnostic.secondary_labels {
        if let Some(file) = sources.get(secondary.span.source()) {
            render_label(
                output,
                file,
                secondary.span,
                secondary.message.as_deref(),
                " :::",
                &mut generated_notes,
            );
        } else {
            generated_notes.push("secondary diagnostic source was unavailable for display".into());
        }
    }

    for note in generated_notes.iter().chain(&diagnostic.notes) {
        writeln!(output, "  = note: {}", single_line(note)).expect("writing to String cannot fail");
    }
    for help in &diagnostic.help {
        writeln!(output, "  = help: {}", single_line(help)).expect("writing to String cannot fail");
    }
}

fn render_label(
    output: &mut String,
    file: &SourceFile,
    span: Span,
    label: Option<&str>,
    marker: &str,
    generated_notes: &mut Vec<String>,
) {
    let (span, was_clamped) = clamp_span(file, span);
    let start = file
        .location(span.start())
        .unwrap_or(SourceLocation { line: 1, column: 1 });
    let end = file.location(span.end()).unwrap_or(start);
    let path = file.path().to_string_lossy();

    writeln!(output, "{marker} {path}:{}:{}", start.line, start.column)
        .expect("writing to String cannot fail");

    let line_text = file.line_text(start.line).unwrap_or("");
    let displayed_line = line_text.replace('\t', " ");
    let gutter_width = start.line.to_string().len();
    writeln!(output, "{:width$} |", "", width = gutter_width)
        .expect("writing to String cannot fail");
    writeln!(output, "{} | {displayed_line}", start.line).expect("writing to String cannot fail");

    let underline_end_column = if end.line == start.line {
        end.column
    } else {
        line_text.chars().count() + 1
    };
    let underline_width = underline_end_column.saturating_sub(start.column).max(1);
    write!(
        output,
        "{:width$} | {}{}",
        "",
        " ".repeat(start.column.saturating_sub(1)),
        "^".repeat(underline_width),
        width = gutter_width
    )
    .expect("writing to String cannot fail");
    if let Some(label) = label {
        write!(output, " {}", single_line(label)).expect("writing to String cannot fail");
    }
    output.push('\n');
    writeln!(output, "{:width$} |", "", width = gutter_width)
        .expect("writing to String cannot fail");

    if end.line != start.line {
        generated_notes.push(format!(
            "primary span continues to {}:{}",
            end.line, end.column
        ));
    }
    if was_clamped {
        generated_notes.push("diagnostic span was invalid and was clamped for display".to_owned());
    }
}

fn clamp_span(file: &SourceFile, original: Span) -> (Span, bool) {
    let text = file.text();
    let mut start = original.start().min(text.len());
    let mut end = original.end().min(text.len());

    while !text.is_char_boundary(start) {
        start -= 1;
    }
    while end < text.len() && !text.is_char_boundary(end) {
        end += 1;
    }
    if end < start {
        end = start;
    }

    (
        Span::new(original.source(), start, end).expect("clamped span is ordered"),
        start != original.start() || end != original.end(),
    )
}

fn single_line(text: &str) -> String {
    text.replace('\r', "\\r").replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Phase, Severity, SourceLabel, render_diagnostics};
    use crate::source::{SourceMap, Span};

    #[test]
    fn renders_every_phase_name() {
        let phases = [
            (Phase::Source, "source"),
            (Phase::Lex, "lex"),
            (Phase::Parse, "parse"),
            (Phase::Resolve, "resolve"),
            (Phase::Type, "type"),
            (Phase::Schema, "schema"),
            (Phase::Effect, "effect"),
            (Phase::Lower, "lower"),
            (Phase::Emit, "emit"),
            (Phase::Verify, "verify"),
            (Phase::Project, "project"),
        ];

        for (phase, expected) in phases {
            assert_eq!(phase.to_string(), expected);
        }
    }

    #[test]
    fn diagnostics_retain_codes_phase_and_severity() {
        let error = Diagnostic::error(Phase::Type, "nullable value used here", None);
        let warning = Diagnostic::coded_warning(
            Phase::Verify,
            "verify.backend-unavailable",
            "backend unavailable",
            None,
        );

        assert_eq!(error.code, "type.error");
        assert_eq!(error.severity, Severity::Error);
        assert_eq!(error.phase, Phase::Type);
        assert_eq!(error.message, "nullable value used here");
        assert_eq!(warning.code, "verify.backend-unavailable");
        assert_eq!(warning.severity, Severity::Warning);
    }

    #[test]
    fn renders_labels_notes_help_tabs_and_unicode_columns() {
        let mut sources = SourceMap::new();
        let source = sources.add("unicode.zen", "let\té = $;\n");
        let primary = Span::new(source, 9, 10).unwrap();
        let secondary = Span::new(source, 4, 6).unwrap();
        let diagnostic = Diagnostic::coded_error(
            Phase::Lex,
            "lex.invalid-character",
            "unexpected character `$`",
            Some(primary),
        )
        .with_primary_label("unexpected character `$`")
        .with_secondary_label(SourceLabel::new(secondary, "related é"))
        .with_note("first\nsecond")
        .with_help("remove it");

        assert_eq!(
            render_diagnostics(&sources, &[diagnostic]),
            concat!(
                "error[lex.invalid-character]: unexpected character `$`\n",
                " --> unicode.zen:1:9\n",
                "  |\n",
                "1 | let é = $;\n",
                "  |         ^ unexpected character `$`\n",
                "  |\n",
                " ::: unicode.zen:1:5\n",
                "  |\n",
                "1 | let é = $;\n",
                "  |     ^ related é\n",
                "  |\n",
                "  = note: first\\nsecond\n",
                "  = help: remove it\n",
            )
        );
    }

    #[test]
    fn orders_diagnostics_and_clamps_invalid_spans_without_panicking() {
        let mut sources = SourceMap::new();
        let source = sources.add("main.zen", "é");
        let late = Diagnostic::coded_warning(Phase::Lex, "lex.z", "late", Span::new(source, 1, 99));
        let early = Diagnostic::coded_error(Phase::Lex, "lex.a", "early", Span::new(source, 0, 0));

        let rendered = render_diagnostics(&sources, &[late, early]);
        assert!(rendered.starts_with("error[lex.a]: early\n"));
        assert!(rendered.contains("\n\nwarning[lex.z]: late\n"));
        assert!(
            rendered
                .contains("  = note: diagnostic span was invalid and was clamped for display\n")
        );
    }

    #[test]
    fn renders_multiline_spans_with_the_exclusive_end_location() {
        let mut sources = SourceMap::new();
        let source = sources.add("main.zen", "/* first\r\nsecond */");
        let diagnostic = Diagnostic::coded_error(
            Phase::Lex,
            "lex.unterminated-comment",
            "comment",
            Span::new(source, 0, 12),
        );

        let rendered = render_diagnostics(&sources, &[diagnostic]);
        assert!(rendered.contains("  | ^^^^^^^^\n"));
        assert!(rendered.contains("  = note: primary span continues to 2:3\n"));
    }
}
