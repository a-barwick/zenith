use std::path::{Path, PathBuf};

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
    use super::{SourceMap, Span};

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
}
