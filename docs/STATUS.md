# Current Status

**Last updated:** 2026-07-17

## Active milestone

M2 — Parsed syntax and immutable AST

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

## Immediate target

Specify M2's parsed baseline and recovery contract, then implement immutable
AST and parser modules plus `zenith ast <file.zen>` for
`examples/lexical-baseline.zen`. Keep resolution, typing, effects, lowering,
and emission out of the parser.

## Known limitations

- Zenith source is tokenized but not parsed, resolved, checked, lowered, or
  emitted. Token recognition does not imply parser acceptance.
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
