use std::collections::BTreeMap;
use std::path::Path;

use crate::diagnostic::{Diagnostic, Phase};
use crate::source::{SourceId, SourceMap, Span};
use crate::types::Type;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApexBoundary {
    pub classes: BTreeMap<String, ExternalClass>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalClass {
    pub name: String,
    pub methods: Vec<ExternalMethod>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalMethod {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<Type>,
    pub is_static: bool,
    pub span: Span,
    pub effects: ExternalEffects,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalEffects {
    Unknown,
}

#[derive(Clone, Debug)]
enum Kind {
    Word(String),
    Symbol(char),
    Eof,
}

#[derive(Clone, Debug)]
struct Lexeme {
    kind: Kind,
    span: Span,
}

pub fn parse_boundary(
    path: &Path,
    text: String,
    sources: &mut SourceMap,
) -> Result<ApexBoundary, Vec<Diagnostic>> {
    let source = sources.add(path, text);
    let file = sources.get(source).expect("boundary source was inserted");
    let (tokens, mut diagnostics) = tokenize(source, file.text());
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut parser = Parser {
        tokens,
        cursor: 0,
        diagnostics: Vec::new(),
    };
    let boundary = parser.parse();
    diagnostics.append(&mut parser.diagnostics);
    if diagnostics.is_empty() {
        Ok(boundary)
    } else {
        Err(diagnostics)
    }
}

fn tokenize(source: SourceId, text: &str) -> (Vec<Lexeme>, Vec<Diagnostic>) {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        if bytes[cursor].is_ascii_alphabetic() || bytes[cursor] == b'_' {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
            tokens.push(Lexeme {
                kind: Kind::Word(text[start..cursor].to_owned()),
                span: Span::new(source, start, cursor).expect("ordered boundary token"),
            });
            continue;
        }
        let character = char::from(bytes[cursor]);
        if "{}();,<>".contains(character) {
            tokens.push(Lexeme {
                kind: Kind::Symbol(character),
                span: Span::new(source, cursor, cursor + 1).expect("ordered boundary symbol"),
            });
            cursor += 1;
            continue;
        }
        let span = Span::new(source, cursor, cursor + 1).expect("ordered boundary error span");
        diagnostics.push(
            Diagnostic::coded_error(
                Phase::Project,
                "project.invalid-apex-boundary",
                format!("unexpected character `{character}` in Apex boundary summary"),
                Some(span),
            )
            .with_primary_label("not part of the boundary summary grammar"),
        );
        cursor += 1;
    }
    let eof = Span::new(source, text.len(), text.len()).expect("ordered EOF span");
    tokens.push(Lexeme {
        kind: Kind::Eof,
        span: eof,
    });
    (tokens, diagnostics)
}

struct Parser {
    tokens: Vec<Lexeme>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn parse(&mut self) -> ApexBoundary {
        let mut classes: BTreeMap<String, ExternalClass> = BTreeMap::new();
        while !matches!(self.current().kind, Kind::Eof) {
            let Some(class) = self.parse_class() else {
                self.synchronize_class();
                continue;
            };
            let canonical = class.name.to_ascii_lowercase();
            if let Some(previous) = classes.get(&canonical) {
                self.diagnostics.push(
                    Diagnostic::coded_error(
                        Phase::Resolve,
                        "resolve.duplicate-class",
                        format!("duplicate boundary class `{}`", class.name),
                        Some(class.span),
                    )
                    .with_primary_label("duplicate declaration")
                    .with_note(format!(
                        "the first declaration starts at byte {}",
                        previous.span.start()
                    )),
                );
            } else {
                classes.insert(canonical, class);
            }
        }
        ApexBoundary { classes }
    }

    fn parse_class(&mut self) -> Option<ExternalClass> {
        let start = self.expect_word("class")?;
        let (name, _) = self.take_word("class name")?;
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.at_symbol('}') && !matches!(self.current().kind, Kind::Eof) {
            if let Some(method) = self.parse_method() {
                if methods.iter().any(|existing: &ExternalMethod| {
                    existing.name.eq_ignore_ascii_case(&method.name)
                        && existing.parameters == method.parameters
                }) {
                    self.error(
                        "resolve.duplicate-method",
                        format!("duplicate boundary method `{}`", method.name),
                        method.span,
                    );
                } else {
                    methods.push(method);
                }
            } else {
                self.synchronize_member();
            }
        }
        let end = self.expect_symbol('}')?;
        Some(ExternalClass {
            name,
            methods,
            span: Span::new(start.source(), start.start(), end.end()).expect("ordered class span"),
        })
    }

    fn parse_method(&mut self) -> Option<ExternalMethod> {
        let start = self.current().span;
        let is_static = self.take_if_word("static");
        let return_type = self.parse_type()?;
        let (name, _) = self.take_word("method name")?;
        self.expect_symbol('(')?;
        let mut parameters = Vec::new();
        if !self.at_symbol(')') {
            loop {
                parameters.push(self.parse_type()?);
                self.take_word("parameter name")?;
                if !self.take_if_symbol(',') {
                    break;
                }
            }
        }
        self.expect_symbol(')')?;
        let end = self.expect_symbol(';')?;
        Some(ExternalMethod {
            name,
            return_type,
            parameters,
            is_static,
            span: Span::new(start.source(), start.start(), end.end()).expect("ordered method span"),
            effects: ExternalEffects::Unknown,
        })
    }

    fn parse_type(&mut self) -> Option<Type> {
        let (name, span) = self.take_word("type")?;
        let canonical = name.to_ascii_lowercase();
        let mut arguments = Vec::new();
        if self.take_if_symbol('<') {
            loop {
                arguments.push(self.parse_type()?);
                if !self.take_if_symbol(',') {
                    break;
                }
            }
            self.expect_symbol('>')?;
        }
        match type_from_parts(&name, &canonical, arguments) {
            Ok(ty) => Some(ty),
            Err(message) => {
                self.error("project.invalid-apex-boundary", message, span);
                None
            }
        }
    }

    fn current(&self) -> &Lexeme {
        &self.tokens[self.cursor.min(self.tokens.len() - 1)]
    }

    fn at_symbol(&self, expected: char) -> bool {
        matches!(self.current().kind, Kind::Symbol(found) if found == expected)
    }

    fn take_if_symbol(&mut self, expected: char) -> bool {
        if self.at_symbol(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, expected: char) -> Option<Span> {
        if self.take_if_symbol(expected) {
            Some(self.tokens[self.cursor - 1].span)
        } else {
            let span = self.current().span;
            self.error(
                "project.invalid-apex-boundary",
                format!("expected `{expected}` in Apex boundary summary"),
                span,
            );
            None
        }
    }

    fn take_if_word(&mut self, expected: &str) -> bool {
        if matches!(&self.current().kind, Kind::Word(found) if found.eq_ignore_ascii_case(expected))
        {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect_word(&mut self, expected: &str) -> Option<Span> {
        if self.take_if_word(expected) {
            Some(self.tokens[self.cursor - 1].span)
        } else {
            let span = self.current().span;
            self.error(
                "project.invalid-apex-boundary",
                format!("expected `{expected}` in Apex boundary summary"),
                span,
            );
            None
        }
    }

    fn take_word(&mut self, description: &str) -> Option<(String, Span)> {
        let token = self.current().clone();
        if let Kind::Word(word) = token.kind {
            self.cursor += 1;
            Some((word, token.span))
        } else {
            self.error(
                "project.invalid-apex-boundary",
                format!("expected {description} in Apex boundary summary"),
                token.span,
            );
            None
        }
    }

    fn error(&mut self, code: &str, message: String, span: Span) {
        let phase = if code.starts_with("resolve.") {
            Phase::Resolve
        } else {
            Phase::Project
        };
        self.diagnostics.push(
            Diagnostic::coded_error(phase, code, message, Some(span))
                .with_primary_label("invalid boundary declaration"),
        );
    }

    fn synchronize_member(&mut self) {
        while !matches!(self.current().kind, Kind::Eof) && !self.at_symbol('}') {
            self.cursor += 1;
            if matches!(self.tokens[self.cursor - 1].kind, Kind::Symbol(';')) {
                break;
            }
        }
    }

    fn synchronize_class(&mut self) {
        while !matches!(self.current().kind, Kind::Eof) {
            if matches!(&self.current().kind, Kind::Word(word) if word.eq_ignore_ascii_case("class"))
            {
                break;
            }
            self.cursor += 1;
        }
    }
}

fn type_from_parts(name: &str, canonical: &str, arguments: Vec<Type>) -> Result<Type, String> {
    let arity = arguments.len();
    let ty = match canonical {
        "void" if arity == 0 => Type::Void,
        "boolean" if arity == 0 => Type::Boolean,
        "integer" | "int" if arity == 0 => Type::Integer,
        "long" if arity == 0 => Type::Long,
        "decimal" if arity == 0 => Type::Decimal,
        "double" if arity == 0 => Type::Double,
        "string" if arity == 0 => Type::String,
        "object" if arity == 0 => Type::Object,
        "id" if arity == 0 => Type::Id(None),
        "list" if arity == 1 => Type::List(Box::new(arguments[0].clone())),
        "set" if arity == 1 => Type::Set(Box::new(arguments[0].clone())),
        "map" if arity == 2 => Type::Map(
            Box::new(arguments[0].clone()),
            Box::new(arguments[1].clone()),
        ),
        "list" | "set" | "map" => {
            return Err(format!("wrong number of type arguments for `{name}`"));
        }
        _ if arity == 0 => Type::ExternalClass(name.to_owned()),
        _ => return Err(format!("generic boundary class `{name}` is not supported")),
    };
    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::{ExternalEffects, parse_boundary};
    use crate::source::SourceMap;
    use crate::types::Type;
    use std::path::Path;

    #[test]
    fn parses_case_insensitive_external_summaries_with_unknown_effects() {
        let mut sources = SourceMap::new();
        let boundary = parse_boundary(
            Path::new("apex.api"),
            "CLASS Audit { STATIC void Record(List<String> values); }".into(),
            &mut sources,
        )
        .unwrap();
        let method = &boundary.classes["audit"].methods[0];
        assert_eq!(method.name, "Record");
        assert_eq!(method.parameters, vec![Type::List(Box::new(Type::String))]);
        assert_eq!(method.effects, ExternalEffects::Unknown);
    }

    #[test]
    fn rejects_invalid_boundary_grammar() {
        let mut sources = SourceMap::new();
        let diagnostics = parse_boundary(
            Path::new("apex.api"),
            "class Bad { static void run(String x); @ }".into(),
            &mut sources,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|item| item.code == "project.invalid-apex-boundary")
        );
    }

    #[test]
    fn rejects_case_insensitive_duplicate_classes_and_methods() {
        let mut sources = SourceMap::new();
        let diagnostics = parse_boundary(
            Path::new("apex.api"),
            "class Bad { static void run(String x); static void RUN(String y); }\n\
             class BAD { void other(); }"
                .into(),
            &mut sources,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|item| item.code == "resolve.duplicate-method")
        );
        assert!(
            diagnostics
                .iter()
                .any(|item| item.code == "resolve.duplicate-class")
        );
        assert!(
            diagnostics
                .iter()
                .filter(|item| item.code.starts_with("resolve."))
                .all(|item| item.phase == crate::diagnostic::Phase::Resolve)
        );
    }
}
