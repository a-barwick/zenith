# Diagnostics

## Status

Severity, phase ownership, stable codes, labels, notes/help, rendering,
ordering, and multi-diagnostic results are **Implemented in M1**.

## Diagnostic model

Every user-facing diagnostic eventually contains:

- a stable symbolic code, such as `lex.invalid-character`
- `error` or `warning` severity
- one owning compiler phase
- a concise primary message
- an optional primary source label
- zero or more secondary source labels
- ordered notes or help text

Codes use `<phase>.<kebab-case-name>`. Renaming or reusing a shipped code is a
compatibility change. Initial M1 codes include:

- `source.read-failed`
- `source.invalid-utf8`
- `lex.invalid-character`
- `lex.invalid-identifier`
- `lex.invalid-escape`
- `lex.double-quoted-string`
- `lex.unterminated-string`
- `lex.unterminated-comment`

Unsupported constructs use a dedicated code owned by the phase that knows the
construct is unsupported. They are not reported as internal errors and are
never silently approximated.

## Phase ownership

The diagnostic phase set is:

```text
source, lex, parse, resolve, type, schema, effect, lower, emit, verify, project
```

Schema/query diagnostics do not masquerade as ordinary type errors.
Verification diagnostics describe generated-artifact or backend evidence and
do not change whether Zenith source passed its own compiler phases. Project
diagnostics own configuration, discovery, dependency, and orchestration errors.

## Spans and rendering

Byte spans are the source of truth. The M1 console renderer uses:

- 1-based line numbers
- 1-based Unicode-scalar columns
- a source line and caret underline for the primary label
- file paths rendered lossily only when the operating system path is not UTF-8

Tabs count as one reported column in M1. LSP UTF-16 positions and display-cell
width are future conversions and never replace byte spans.

The complete single-label layout is:

```text
error[lex.invalid-character]: unexpected character `$`
 --> path/to/file.zen:3:9
  |
3 | let x = $;
  |         ^ unexpected character `$`
  |
  = note: `$` is not in the M1 operator inventory
  = help: remove the character or use a supported operator
```

These formatting rules are part of the M1 CLI contract:

- The heading is `<severity>[<code>]: <message>` followed by exactly one
  newline. The location line begins with one space, `-->`, one space, and
  `<path>:<line>:<column>`.
- The line-number gutter is as wide as the rendered decimal line number. The
  source line excludes its line terminator. Each tab is rendered as one ASCII
  space so the displayed source and scalar-column underline agree.
- The underline uses `^`, begins at the primary start column, and has at least
  one character. A zero-length span therefore marks its insertion point with
  one `^`. The optional primary-label message follows one space after the
  underline.
- For a span contained on one line, the underline covers every Unicode scalar
  touched by the half-open span. For a multiline span, the first line is shown,
  the underline continues through that line's final scalar, and an ordered note
  reads `primary span continues to <line>:<column>` using the exclusive end
  location. An empty first line still receives one caret.
- Secondary labels, when present, follow the primary block in insertion order.
  Each uses ` ::: <path>:<line>:<column>`, then the same gutter/source/underline
  layout. M1 lexical diagnostics have at most one primary label, but this fixes
  the representation before later phases emit relationships.
- Notes and help entries retain model order. Each appears as
  `  = note: <text>` or `  = help: <text>`; embedded line breaks are replaced
  with `\n` so one entry remains one rendered line.
- Consecutive diagnostics are separated by one empty line. The complete stderr
  stream ends with exactly one newline and has no leading or trailing blank
  line.

Rendering malformed, out-of-bounds, or non-character-boundary spans must not
panic. For display, offsets are clamped to the source byte length, the start is
moved to the preceding UTF-8 boundary, the end is moved to the following UTF-8
boundary, and an end before the adjusted start becomes an empty span. The
renderer appends the note `diagnostic span was invalid and was clamped for
display`. Compiler-created invalid spans remain internal defects even though
the user-facing renderer degrades safely.

## Ordering and recovery

Diagnostics are deterministic. Project-wide ordering is:

1. source-file load order
2. primary span start, with span-less diagnostics after spanned diagnostics
3. errors before warnings at the same location
4. stable code
5. message as the final tie-breaker

Lexing and parsing may recover and return multiple diagnostics. Later semantic
phases do not run for a source unit whose prerequisite phase produced errors,
unless a specification explicitly defines a safe partial-analysis mode.

## CLI behavior

- Status 0: requested operation completed without errors.
- Status 1: source or compiler diagnostics were emitted, or execution/verification
  completed with a user-program failure.
- Status 2: invalid CLI usage or an unavailable command.
- A panic, backtrace, or status 101 is never a user diagnostic.

Diagnostics go to stderr. Normal command artifacts and inspection output go to
stdout. Tests assert complete output for stable command surfaces.

## External verification diagnostics

Apex Exec and Salesforce diagnostics first refer to generated files. The
verification adapter records the backend name, version, capability profile,
generated path, and generated byte span, then maps the result through Zenith's
manifest and source map. An unsupported backend capability remains
`unsupported`; it is not rewritten as Zenith success or failure.
