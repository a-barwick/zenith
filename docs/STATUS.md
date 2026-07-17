# Current Status

**Last updated:** 2026-07-17

## Active milestone

None. M2 — Parsed syntax and immutable AST is complete. M3 remains Planned
until its acceptance fixture and checking/emission specifications are explicit.

## Completed

- Rust binary and library crate with no third-party dependencies
- Rust 1.88.0 toolchain pin plus GitHub Actions gates for formatting, tests,
  Clippy, and current-stable test compatibility
- Repository-level product, roadmap, architecture, compatibility, development,
  decision, and specification documentation
- Tested bootstrap help/version aliases, no-argument help, unavailable-command
  behavior, and non-UTF-8 argument handling
- Session-local source identities, ordered file-aware byte spans, and safe
  source slicing at UTF-8 boundaries
- Compiler diagnostic categories for source, lexing, parsing, resolution,
  typing, schema checking, effects, lowering, emission, verification, and
  project orchestration
- Initial decisions covering compile-to-Apex deployment, case-insensitive
  names, generated Apex as a product surface, the Apex Exec process boundary,
  and generated-test oracle requirements
- M1 lexical and diagnostic contracts that settle identifiers, keyword and
  operator inventory, strings/comments, recovery, positions, codes, ordering,
  CLI output, and exit behavior
- Small and broad `.zen` fixtures reserved for the first token slice
- A revision-pinned Apex Exec comparison and staged integration contract
- Documented target architecture for optional generated-Apex verification,
  handwritten Apex API summaries, and coverage-guided test generation
- Complete M1 token model for Apex-shaped and reserved Zenith words, integers,
  decoded single-quoted strings, punctuation, maximal-munch operators, and EOF
- Exact identifier spelling, ASCII-lowercase canonical keys, and file-aware
  spans exposed through the public compiler API
- UTF-8 source loading and Unicode-scalar line/column mapping across LF, CRLF,
  and CR line endings
- Recoverable lexical diagnostics for invalid identifiers, characters, escapes,
  double-quoted strings, and unterminated strings/comments
- Stable diagnostic codes, primary/secondary labels, notes, help, deterministic
  ordering, and panic-safe rendering of malformed spans
- `zenith tokens <file.zen>` with golden output for both M1 acceptance fixtures,
  explicit status 1 source/compiler failures, and status 2 usage failures
- Executable coverage of the complete keyword, contextual-word, punctuation,
  operator, escape, comment, recovery, source-position, and CLI contracts
- M2 parsed-syntax specification covering declarations, types, statements,
  expressions, precedence, immutable syntax, recovery, diagnostics, and CLI
- Immutable source-spanned AST for classes, fields, properties, accessors,
  methods, constructors, parameters, nested/nullable type syntax, statements,
  and expressions
- Shared read-only AST visitor with deterministic source-order traversal and no
  semantic facts stored in parsed syntax
- Parser support for blocks, variables, assignments, calls, member/index
  access, conditionals, traditional/enhanced loops, returns, throws, and the
  specified unary/binary/postfix expression precedence
- Generic-close handling for nested types despite maximal-munch `>>` and `>>>`
  lexer tokens
- Localized declaration, member, and statement recovery with stable parse
  diagnostics and recoverable partial trees
- `zenith ast <file.zen>` with deterministic goldens for both M2 acceptance
  fixtures, lexical/parse gating, and explicit status 1/2 behavior
- 54 passing Rust tests spanning source, diagnostics, lexing, parser shape and
  purity, visitor traversal, recovery, CLI diagnostics, and five inspection
  goldens, including the complete `examples/parsed-baseline.zen` surface
- Instrumented `cargo llvm-cov --all-targets` coverage of 88.70% lines overall,
  including 87.03% for the parser, 82.09% for the AST/renderer, 98.92% for the
  CLI, and 98.88% for the lexer

## Immediate target

Specify M3's multi-file acceptance fixture, project/configuration contract,
checked baseline, Apex IR and emission format, source maps, handwritten-Apex
boundary summaries, and backend-neutral verification outcomes before
activating M3.

## Known limitations

- Zenith source is tokenized and the selected M2 grammar is parsed, but it is
  not resolved, checked, lowered, or emitted. Parser acceptance is Local syntax
  support and does not imply Apex compatibility.
- `check`, `build`, `emit`, `test`, and `verify` are documented targets but are
  intentionally unavailable.
- `SourceMap::add` assigns a new session-local ID per loaded entry. Stable
  path identity across project reparses, caching, and content identities are M3
  work.
- Source spans remain byte offsets; console locations are derived as 1-based
  Unicode-scalar line and column pairs.
- The proposed language examples document direction, not shipped syntax.
- No Salesforce schema, Apex Exec executable adapter, Salesforce CLI, or org
  integration exists. `docs/APEX_EXEC.md` is a boundary contract, not an
  implemented integration.
- No test execution, coverage, or generated-test integration exists. The
  documented generated-test workflow is a planned interface.
- No repository license or crate-publication policy has been selected; future
  tasks must not infer one.

## Handoff checklist

After meaningful implementation work:

- Update the completed and limitation lists above.
- Keep exactly one immediate target aligned with the active milestone.
- Update `docs/COMPATIBILITY.md` for changed source or emitted behavior.
- Update the owning specification when observable behavior changes.
- Add or update executable positive, negative, and golden fixtures.
- Add an ADR if an architectural boundary or expensive choice changed.
- Run the verification commands in `AGENTS.md`.
