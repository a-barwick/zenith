use std::fmt;

use crate::source::{SourceFile, Span};

/// An identifier with source spelling preserved and a canonical lookup key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    spelling: String,
    canonical: String,
    span: Span,
}

impl Identifier {
    pub(crate) fn new(spelling: &str, span: Span) -> Self {
        debug_assert!(spelling.is_ascii());
        Self {
            spelling: spelling.to_owned(),
            canonical: spelling.to_ascii_lowercase(),
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
pub enum TokenKind {
    Keyword(String),
    Contextual(String),
    Identifier(Identifier),
    Integer,
    String(String),
    Punctuation(&'static str),
    Operator(&'static str),
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword(word) => write!(formatter, "keyword({word})"),
            Self::Contextual(word) => write!(formatter, "contextual({word})"),
            Self::Identifier(identifier) => {
                write!(formatter, "identifier({})", identifier.canonical())
            }
            Self::Integer => formatter.write_str("integer"),
            Self::String(_) => formatter.write_str("string"),
            Self::Punctuation(spelling) => write!(formatter, "punctuation({spelling})"),
            Self::Operator(spelling) => write!(formatter, "operator({spelling})"),
            Self::Eof => formatter.write_str("eof"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub(crate) const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

/// Formats the stable `tokens` inspection stream for one source file.
pub fn render_tokens(file: &SourceFile, tokens: &[Token]) -> String {
    let mut output = String::new();
    for token in tokens {
        let span = token.span();
        let start = file
            .location(span.start())
            .expect("lexer token starts at a valid source boundary");
        let end = file
            .location(span.end())
            .expect("lexer token ends at a valid source boundary");
        let spelling = file
            .slice(span)
            .expect("lexer token span belongs to the rendered source");
        output.push_str(&format!(
            "{}:{}..{}:{}\t{}\t{}\n",
            start.line,
            start.column,
            end.line,
            end.column,
            token.kind(),
            json_string(spelling)
        ));
    }
    output
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
                output.push_str(&format!("\\u{:04x}", u32::from(character)));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[cfg(test)]
mod tests {
    use super::render_tokens;
    use crate::lexer::lex;
    use crate::source::SourceMap;

    #[test]
    fn renders_locations_kinds_and_json_escaped_spelling() {
        let mut sources = SourceMap::new();
        let source = sources.add("test.zen", "Name\r\n'a\\n'");
        let file = sources.get(source).unwrap();
        let result = lex(file);

        assert_eq!(
            render_tokens(file, &result.tokens),
            "1:1..1:5\tidentifier(name)\t\"Name\"\n\
2:1..2:6\tstring\t\"'a\\\\n'\"\n\
2:6..2:6\teof\t\"\"\n"
        );
    }
}
