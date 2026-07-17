use std::path::{Path, PathBuf};

/// A 1-based line and Unicode-scalar column in a source file.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

/// Stable identity for a source file within one compiler session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(u32);

impl SourceId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A half-open byte range within one source file.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    source: SourceId,
    start: usize,
    end: usize,
}

impl Span {
    pub const fn new(source: SourceId, start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { source, start, end })
        } else {
            None
        }
    }

    pub const fn source(self) -> SourceId {
        self.source
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    id: SourceId,
    path: PathBuf,
    text: String,
}

impl SourceFile {
    pub fn id(&self) -> SourceId {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn slice(&self, span: Span) -> Option<&str> {
        if span.source != self.id {
            return None;
        }
        self.text.get(span.start..span.end)
    }

    /// Converts a byte offset to a 1-based line and Unicode-scalar column.
    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }

        let mut line = 1;
        let mut column = 1;
        let mut cursor = 0;
        let bytes = self.text.as_bytes();

        while cursor < offset {
            match bytes[cursor] {
                b'\r' => {
                    cursor += 1;
                    if cursor < bytes.len() && bytes[cursor] == b'\n' {
                        cursor += 1;
                    }
                    line += 1;
                    column = 1;
                }
                b'\n' => {
                    cursor += 1;
                    line += 1;
                    column = 1;
                }
                _ => {
                    let character = self.text[cursor..].chars().next().expect("valid UTF-8");
                    cursor += character.len_utf8();
                    column += 1;
                }
            }
        }

        Some(SourceLocation { line, column })
    }

    /// Returns a line without its line terminator, using a 1-based line number.
    pub fn line_text(&self, requested_line: usize) -> Option<&str> {
        if requested_line == 0 {
            return None;
        }

        let mut line = 1;
        let mut start = 0;
        let mut cursor = 0;
        let bytes = self.text.as_bytes();

        while cursor < bytes.len() {
            if line == requested_line && matches!(bytes[cursor], b'\r' | b'\n') {
                return Some(&self.text[start..cursor]);
            }

            match bytes[cursor] {
                b'\r' => {
                    cursor += 1;
                    if cursor < bytes.len() && bytes[cursor] == b'\n' {
                        cursor += 1;
                    }
                    line += 1;
                    start = cursor;
                }
                b'\n' => {
                    cursor += 1;
                    line += 1;
                    start = cursor;
                }
                _ => {
                    cursor += self.text[cursor..]
                        .chars()
                        .next()
                        .expect("valid UTF-8")
                        .len_utf8();
                }
            }
        }

        (line == requested_line).then_some(&self.text[start..])
    }
}

/// Session-local collection of source files.
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: impl Into<PathBuf>, text: impl Into<String>) -> SourceId {
        let raw = u32::try_from(self.files.len()).expect("source map exhausted u32 identities");
        let id = SourceId::from_raw(raw);
        self.files.push(SourceFile {
            id,
            path: path.into(),
            text: text.into(),
        });
        id
    }

    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.raw() as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceLocation, SourceMap, Span};

    #[test]
    fn assigns_stable_session_local_source_ids() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.zen", "first");
        let second = sources.add("second.zen", "second");

        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        assert_eq!(
            sources.get(first).unwrap().path().to_str(),
            Some("first.zen")
        );
    }

    #[test]
    fn slices_only_the_matching_source() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.zen", "record Money");
        let second = sources.add("second.zen", "other");
        let span = Span::new(first, 0, 6).unwrap();

        assert_eq!(sources.get(first).unwrap().slice(span), Some("record"));
        assert_eq!(sources.get(second).unwrap().slice(span), None);
    }

    #[test]
    fn rejects_reversed_spans() {
        let mut sources = SourceMap::new();
        let source = sources.add("main.zen", "");

        assert!(Span::new(source, 3, 2).is_none());
    }

    #[test]
    fn exposes_only_ordered_span_coordinates() {
        let mut sources = SourceMap::new();
        let source = sources.add("main.zen", "value");
        let span = Span::new(source, 2, 2).unwrap();

        assert_eq!(span.source(), source);
        assert_eq!(span.start(), 2);
        assert_eq!(span.end(), 2);
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn slices_unicode_only_at_valid_byte_boundaries() {
        let mut sources = SourceMap::new();
        let source = sources.add("unicode.zen", "aéz");
        let file = sources.get(source).unwrap();

        assert_eq!(file.slice(Span::new(source, 1, 3).unwrap()), Some("é"));
        assert_eq!(file.slice(Span::new(source, 2, 3).unwrap()), None);
        assert_eq!(file.slice(Span::new(source, 1, 8).unwrap()), None);
    }

    #[test]
    fn maps_mixed_line_endings_and_unicode_to_scalar_locations() {
        let mut sources = SourceMap::new();
        let source = sources.add("positions.zen", "aé\r\nβ\rc\n");
        let file = sources.get(source).unwrap();

        assert_eq!(
            file.location(0),
            Some(SourceLocation { line: 1, column: 1 })
        );
        assert_eq!(
            file.location("a".len()),
            Some(SourceLocation { line: 1, column: 2 })
        );
        assert_eq!(
            file.location("aé\r\n".len()),
            Some(SourceLocation { line: 2, column: 1 })
        );
        assert_eq!(
            file.location("aé\r\nβ\r".len()),
            Some(SourceLocation { line: 3, column: 1 })
        );
        assert_eq!(
            file.location(file.text().len()),
            Some(SourceLocation { line: 4, column: 1 })
        );
        assert_eq!(file.location(2), None);
        assert_eq!(file.location(file.text().len() + 1), None);
    }

    #[test]
    fn returns_lines_without_any_supported_terminator() {
        let mut sources = SourceMap::new();
        let source = sources.add("lines.zen", "first\r\nsecond\rthird\n");
        let file = sources.get(source).unwrap();

        assert_eq!(file.line_text(1), Some("first"));
        assert_eq!(file.line_text(2), Some("second"));
        assert_eq!(file.line_text(3), Some("third"));
        assert_eq!(file.line_text(4), Some(""));
        assert_eq!(file.line_text(0), None);
        assert_eq!(file.line_text(5), None);
    }
}
