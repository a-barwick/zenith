use crate::diagnostic::{Diagnostic, Phase};
use crate::source::{SourceFile, Span};
use crate::token::{Identifier, Token, TokenKind};

const RESERVED_WORDS: &[&str] = &[
    "abstract",
    "activate",
    "and",
    "any",
    "array",
    "as",
    "asc",
    "autonomous",
    "begin",
    "bigdecimal",
    "blob",
    "boolean",
    "break",
    "bulk",
    "by",
    "byte",
    "case",
    "cast",
    "catch",
    "char",
    "class",
    "collect",
    "commit",
    "const",
    "continue",
    "currency",
    "date",
    "datetime",
    "decimal",
    "default",
    "delete",
    "desc",
    "do",
    "double",
    "else",
    "end",
    "enum",
    "exception",
    "exit",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "from",
    "global",
    "goto",
    "group",
    "having",
    "hint",
    "if",
    "implements",
    "import",
    "in",
    "inner",
    "insert",
    "int",
    "integer",
    "interface",
    "into",
    "instanceof",
    "join",
    "like",
    "limit",
    "list",
    "long",
    "loop",
    "map",
    "merge",
    "new",
    "not",
    "null",
    "nulls",
    "number",
    "object",
    "of",
    "on",
    "or",
    "outer",
    "override",
    "package",
    "parallel",
    "pragma",
    "private",
    "protected",
    "public",
    "retrieve",
    "return",
    "rollback",
    "select",
    "set",
    "short",
    "sobject",
    "sort",
    "static",
    "string",
    "super",
    "switch",
    "synchronized",
    "system",
    "testmethod",
    "then",
    "this",
    "throw",
    "time",
    "transaction",
    "transient",
    "trigger",
    "true",
    "try",
    "undelete",
    "update",
    "upsert",
    "using",
    "virtual",
    "void",
    "webservice",
    "when",
    "where",
    "while",
];

const CONTEXTUAL_WORDS: &[&str] = &[
    "after",
    "before",
    "count",
    "excludes",
    "first",
    "get",
    "includes",
    "inherited",
    "last",
    "order",
    "sharing",
    "with",
    "without",
];

const ZENITH_WORDS: &[&str] = &["effects", "fn", "let", "query", "record"];

const OPERATORS: &[&str] = &[
    ">>>=", "===", "!==", "<<=", ">>=", ">>>", "++", "+=", "--", "-=", "*=", "/=", "&&", "||",
    "&=", "|=", "^=", "<<", ">>", "?.", "=>", "->", "??", "==", "!=", "<=", ">=", "=", "<", ">",
    "+", "-", "*", "/", "%", "!", "&", "|", "^", "~", "?",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

impl LexResult {
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Tokenizes one source file without invoking parsing or later compiler phases.
pub fn lex(file: &SourceFile) -> LexResult {
    Lexer::new(file).run()
}

struct Lexer<'source> {
    file: &'source SourceFile,
    cursor: usize,
    result: LexResult,
}

struct ErrorDetails {
    label: Option<String>,
    note: Option<String>,
    help: Option<String>,
}

impl<'source> Lexer<'source> {
    fn new(file: &'source SourceFile) -> Self {
        Self {
            file,
            cursor: 0,
            result: LexResult::default(),
        }
    }

    fn run(mut self) -> LexResult {
        while self.cursor < self.text().len() {
            let start = self.cursor;
            let character = self.current();

            if matches!(character, ' ' | '\t' | '\u{000c}' | '\r' | '\n') {
                self.consume_whitespace();
            } else if self.rest().starts_with("//") {
                self.consume_line_comment();
            } else if self.rest().starts_with("/*") {
                self.consume_block_comment();
            } else if character.is_ascii_alphabetic() {
                self.consume_word();
            } else if character.is_ascii_digit() {
                self.consume_integer();
            } else if character == '_' || character.is_alphanumeric() {
                self.consume_invalid_identifier();
            } else if character == '\'' {
                self.consume_string();
            } else if character == '"' {
                self.consume_double_quoted_string();
            } else if let Some(operator) = self.match_operator() {
                self.cursor += operator.len();
                self.push(TokenKind::Operator(operator), start, self.cursor);
            } else if let Some(punctuation) = punctuation(character) {
                self.cursor += character.len_utf8();
                self.push(TokenKind::Punctuation(punctuation), start, self.cursor);
            } else {
                self.cursor += character.len_utf8();
                let shown = display_character(character);
                self.error(
                    "lex.invalid-character",
                    format!("unexpected character `{shown}`"),
                    start,
                    self.cursor,
                    ErrorDetails {
                        label: Some(format!("unexpected character `{shown}`")),
                        note: Some(format!(
                            "`{shown}` is not in the M1 punctuation or operator inventory"
                        )),
                        help: Some("remove the character or use a supported operator".to_owned()),
                    },
                );
            }
        }

        let end = self.text().len();
        self.push(TokenKind::Eof, end, end);
        self.result
    }

    fn text(&self) -> &str {
        self.file.text()
    }

    fn rest(&self) -> &str {
        &self.text()[self.cursor..]
    }

    fn current(&self) -> char {
        self.rest().chars().next().expect("cursor is before EOF")
    }

    fn consume_whitespace(&mut self) {
        while self.cursor < self.text().len() {
            let character = self.current();
            if !matches!(character, ' ' | '\t' | '\u{000c}' | '\r' | '\n') {
                break;
            }
            self.cursor += character.len_utf8();
        }
    }

    fn consume_line_comment(&mut self) {
        self.cursor += 2;
        while self.cursor < self.text().len() {
            if matches!(self.current(), '\r' | '\n') {
                break;
            }
            self.cursor += self.current().len_utf8();
        }
    }

    fn consume_block_comment(&mut self) {
        let start = self.cursor;
        self.cursor += 2;
        if let Some(relative_end) = self.rest().find("*/") {
            self.cursor += relative_end + 2;
        } else {
            self.cursor = self.text().len();
            self.error(
                "lex.unterminated-comment",
                "unterminated block comment",
                start,
                self.cursor,
                ErrorDetails {
                    label: Some("block comment starts here but has no closing `*/`".to_owned()),
                    note: None,
                    help: Some("add `*/` before the end of the file".to_owned()),
                },
            );
        }
    }

    fn consume_word(&mut self) {
        let start = self.cursor;
        self.consume_word_run();
        let spelling = &self.text()[start..self.cursor];
        let span = self.span(start, self.cursor);

        if !spelling
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            self.invalid_identifier(start, self.cursor);
            return;
        }

        let canonical = spelling.to_ascii_lowercase();
        let kind = if RESERVED_WORDS.contains(&canonical.as_str())
            || ZENITH_WORDS.contains(&canonical.as_str())
        {
            TokenKind::Keyword(canonical)
        } else if CONTEXTUAL_WORDS.contains(&canonical.as_str()) {
            TokenKind::Contextual(canonical)
        } else {
            TokenKind::Identifier(Identifier::new(spelling, span))
        };
        self.result.tokens.push(Token::new(kind, span));
    }

    fn consume_invalid_identifier(&mut self) {
        let start = self.cursor;
        self.consume_word_run();
        self.invalid_identifier(start, self.cursor);
    }

    fn consume_word_run(&mut self) {
        while self.cursor < self.text().len() {
            let character = self.current();
            if !(character.is_alphanumeric() || character == '_') {
                break;
            }
            self.cursor += character.len_utf8();
        }
    }

    fn invalid_identifier(&mut self, start: usize, end: usize) {
        let spelling = self.text()[start..end].to_owned();
        self.error(
            "lex.invalid-identifier",
            format!("invalid M1 identifier `{spelling}`"),
            start,
            end,
            ErrorDetails {
                label: Some("identifiers use ASCII letters, digits, and underscores".to_owned()),
                note: Some("M1 identifiers must begin with an ASCII letter".to_owned()),
                help: Some(
                    "rename this identifier to use the M1 ASCII identifier grammar".to_owned(),
                ),
            },
        );
    }

    fn consume_integer(&mut self) {
        let start = self.cursor;
        while self.cursor < self.text().len() && self.current().is_ascii_digit() {
            self.cursor += 1;
        }
        self.push(TokenKind::Integer, start, self.cursor);
    }

    fn consume_string(&mut self) {
        let start = self.cursor;
        self.cursor += 1;
        let mut decoded = String::new();
        let mut valid = true;

        while self.cursor < self.text().len() {
            let character = self.current();
            match character {
                '\'' => {
                    self.cursor += 1;
                    if valid {
                        self.push(TokenKind::String(decoded), start, self.cursor);
                    }
                    return;
                }
                '\r' | '\n' => {
                    self.unterminated_string(start);
                    return;
                }
                '\\' => {
                    let escape_start = self.cursor;
                    self.cursor += 1;
                    if self.cursor == self.text().len() || matches!(self.current(), '\r' | '\n') {
                        self.unterminated_string(start);
                        return;
                    }

                    let escaped = self.current();
                    self.cursor += escaped.len_utf8();
                    if let Some(value) = decode_escape(escaped) {
                        decoded.push(value);
                    } else {
                        valid = false;
                        let spelling = self.text()[escape_start..self.cursor].to_owned();
                        self.error(
                            "lex.invalid-escape",
                            format!("unknown escape sequence `{spelling}`"),
                            escape_start,
                            self.cursor,
                            ErrorDetails {
                                label: Some(format!(
                                    "`{spelling}` is not a supported Apex escape"
                                )),
                                note: Some(
                                    "supported escapes are \\b, \\t, \\n, \\f, \\r, \\\", \\', and \\\\"
                                        .to_owned(),
                                ),
                                help: Some("use a supported escape sequence".to_owned()),
                            },
                        );
                    }
                }
                _ => {
                    decoded.push(character);
                    self.cursor += character.len_utf8();
                }
            }
        }

        self.unterminated_string(start);
    }

    fn unterminated_string(&mut self, start: usize) {
        self.error(
            "lex.unterminated-string",
            "unterminated string literal",
            start,
            self.cursor,
            ErrorDetails {
                label: Some("string literal has no closing single quote".to_owned()),
                note: None,
                help: Some("add a closing single quote before the end of the line".to_owned()),
            },
        );
    }

    fn consume_double_quoted_string(&mut self) {
        let start = self.cursor;
        self.cursor += 1;
        while self.cursor < self.text().len() {
            match self.current() {
                '"' => {
                    self.cursor += 1;
                    break;
                }
                '\r' | '\n' => break,
                '\\' => {
                    self.cursor += 1;
                    if self.cursor < self.text().len() && !matches!(self.current(), '\r' | '\n') {
                        self.cursor += self.current().len_utf8();
                    }
                }
                character => self.cursor += character.len_utf8(),
            }
        }
        self.error(
            "lex.double-quoted-string",
            "double-quoted strings are not supported",
            start,
            self.cursor,
            ErrorDetails {
                label: Some("use Apex-compatible single quotes for string literals".to_owned()),
                note: None,
                help: Some("replace the double quotes with single quotes".to_owned()),
            },
        );
    }

    fn match_operator(&self) -> Option<&'static str> {
        OPERATORS
            .iter()
            .copied()
            .find(|operator| self.rest().starts_with(operator))
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.result
            .tokens
            .push(Token::new(kind, self.span(start, end)));
    }

    fn error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        start: usize,
        end: usize,
        details: ErrorDetails,
    ) {
        let mut diagnostic =
            Diagnostic::coded_error(Phase::Lex, code, message, Some(self.span(start, end)));
        if let Some(label) = details.label {
            diagnostic = diagnostic.with_primary_label(label);
        }
        if let Some(note) = details.note {
            diagnostic = diagnostic.with_note(note);
        }
        if let Some(help) = details.help {
            diagnostic = diagnostic.with_help(help);
        }
        self.result.diagnostics.push(diagnostic);
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.file.id(), start, end).expect("lexer spans are ordered")
    }
}

const fn punctuation(character: char) -> Option<&'static str> {
    match character {
        '(' => Some("("),
        ')' => Some(")"),
        '{' => Some("{"),
        '}' => Some("}"),
        '[' => Some("["),
        ']' => Some("]"),
        ';' => Some(";"),
        ',' => Some(","),
        '.' => Some("."),
        ':' => Some(":"),
        '@' => Some("@"),
        _ => None,
    }
}

const fn decode_escape(character: char) -> Option<char> {
    match character {
        'b' => Some('\u{0008}'),
        't' => Some('\t'),
        'n' => Some('\n'),
        'f' => Some('\u{000c}'),
        'r' => Some('\r'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '\\' => Some('\\'),
        _ => None,
    }
}

fn display_character(character: char) -> String {
    character.escape_default().collect()
}

#[cfg(test)]
mod tests {
    use super::{CONTEXTUAL_WORDS, OPERATORS, RESERVED_WORDS, ZENITH_WORDS, lex};
    use crate::source::SourceMap;
    use crate::token::TokenKind;

    fn lex_text(text: &str) -> super::LexResult {
        let mut sources = SourceMap::new();
        let source = sources.add("test.zen", text);
        lex(sources.get(source).unwrap())
    }

    fn kinds(text: &str) -> Vec<String> {
        lex_text(text)
            .tokens
            .iter()
            .map(|token| token.kind().to_string())
            .collect()
    }

    #[test]
    fn classifies_every_reserved_and_zenith_word_case_insensitively() {
        for word in RESERVED_WORDS.iter().chain(ZENITH_WORDS) {
            let result = lex_text(&word.to_ascii_uppercase());
            assert!(result.diagnostics.is_empty(), "{word}");
            assert_eq!(
                result.tokens[0].kind().to_string(),
                format!("keyword({word})")
            );
        }
    }

    #[test]
    fn classifies_contextual_words_separately() {
        for word in CONTEXTUAL_WORDS {
            assert_eq!(
                kinds(&word.to_ascii_uppercase()),
                [format!("contextual({word})"), "eof".to_owned()]
            );
        }
    }

    #[test]
    fn preserves_identifier_spelling_span_and_canonical_key() {
        let result = lex_text("My_Field__c");
        let TokenKind::Identifier(identifier) = result.tokens[0].kind() else {
            panic!("expected identifier");
        };

        assert_eq!(identifier.spelling(), "My_Field__c");
        assert_eq!(identifier.canonical(), "my_field__c");
        assert_eq!(identifier.span(), result.tokens[0].span());
    }

    #[test]
    fn omits_whitespace_and_both_comment_forms() {
        let result = lex_text("alpha // β\n /* not /* nested */ omega");
        assert!(result.diagnostics.is_empty());
        assert_eq!(
            kinds("alpha // β\n /* not /* nested */ omega"),
            ["identifier(alpha)", "identifier(omega)", "eof",]
        );
    }

    #[test]
    fn tokenizes_integers_separately_from_following_words() {
        assert_eq!(kinds("007abc"), ["integer", "identifier(abc)", "eof"]);
    }

    #[test]
    fn decodes_every_supported_string_escape() {
        let result = lex_text(r#"'a\b\t\n\f\r\"\'\\z'"#);
        assert!(result.diagnostics.is_empty());
        assert_eq!(
            result.tokens[0].kind(),
            &TokenKind::String("a\u{8}\t\n\u{c}\r\"'\\z".to_owned())
        );
    }

    #[test]
    fn uses_maximal_munch_and_does_not_invent_percent_assignment() {
        assert_eq!(
            kinds(">>>= >>> >> %= === !== ?. ?? -> =>"),
            [
                "operator(>>>=)",
                "operator(>>>)",
                "operator(>>)",
                "operator(%)",
                "operator(=)",
                "operator(===)",
                "operator(!==)",
                "operator(?.)",
                "operator(??)",
                "operator(->)",
                "operator(=>)",
                "eof",
            ]
        );
    }

    #[test]
    fn recognizes_the_complete_operator_inventory() {
        let source = OPERATORS.join(" ");
        let result = lex_text(&source);
        assert!(result.diagnostics.is_empty());
        assert_eq!(
            result
                .tokens
                .iter()
                .take(OPERATORS.len())
                .map(|token| token.kind().to_string())
                .collect::<Vec<_>>(),
            OPERATORS
                .iter()
                .map(|operator| format!("operator({operator})"))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn recognizes_every_punctuation_spelling() {
        assert_eq!(
            kinds("(){}[];,.:@"),
            [
                "punctuation(()",
                "punctuation())",
                "punctuation({)",
                "punctuation(})",
                "punctuation([)",
                "punctuation(])",
                "punctuation(;)",
                "punctuation(,)",
                "punctuation(.)",
                "punctuation(:)",
                "punctuation(@)",
                "eof",
            ]
        );
    }

    #[test]
    fn recovers_from_invalid_identifiers_as_one_run() {
        let result = lex_text("_foo fooébar éclair ok");
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            [
                "lex.invalid-identifier",
                "lex.invalid-identifier",
                "lex.invalid-identifier",
            ]
        );
        assert_eq!(
            result
                .tokens
                .iter()
                .map(|token| token.kind().to_string())
                .collect::<Vec<_>>(),
            ["identifier(ok)", "eof"]
        );
    }

    #[test]
    fn reports_invalid_escape_but_recovers_after_the_string() {
        let result = lex_text(r#"'bad\q' next"#);
        assert_eq!(result.diagnostics[0].code, "lex.invalid-escape");
        assert_eq!(
            result
                .tokens
                .iter()
                .map(|token| token.kind().to_string())
                .collect::<Vec<_>>(),
            ["identifier(next)", "eof"]
        );
    }

    #[test]
    fn recovers_unterminated_strings_at_each_line_break_style() {
        for separator in ["\n", "\r", "\r\n"] {
            let result = lex_text(&format!("'bad{separator}next"));
            assert_eq!(result.diagnostics[0].code, "lex.unterminated-string");
            assert_eq!(result.tokens[0].kind().to_string(), "identifier(next)");
        }
    }

    #[test]
    fn reports_double_quotes_once_and_continues() {
        let result = lex_text("\"bad \\\" $\" next");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "lex.double-quoted-string");
        assert_eq!(result.tokens[0].kind().to_string(), "identifier(next)");
    }

    #[test]
    fn unterminated_comment_consumes_to_eof_and_still_emits_eof() {
        let result = lex_text("before /* never closed");
        assert_eq!(result.diagnostics[0].code, "lex.unterminated-comment");
        assert_eq!(result.tokens.last().unwrap().kind(), &TokenKind::Eof);
    }

    #[test]
    fn invalid_characters_recover_one_unicode_scalar_at_a_time() {
        let result = lex_text("$💥ok");
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            ["lex.invalid-character", "lex.invalid-character"]
        );
        assert_eq!(result.tokens[0].kind().to_string(), "identifier(ok)");
    }
}
