# Current Status

**Last updated:** 2026-07-16

## Active milestone

M1 — Lexical core and source diagnostics

## Completed

- Rust binary and library crate with no third-party dependencies
- Repository-level product, roadmap, architecture, compatibility, development,
  decision, and specification documentation
- Tested `zenith --help` and `zenith --version` bootstrap CLI
- Session-local source identities, file-aware byte spans, and source slicing
- Compiler diagnostic categories for source, lexing, parsing, resolution,
  typing, effects, lowering, emission, and project orchestration
- Initial decisions covering compile-to-Apex deployment, case-insensitive
  names, and generated Apex as a product surface
- A baseline `.zen` example reserved for the first compiler slice

## Immediate target

Implement M1 as a complete source-to-token slice:

1. Define token kinds for the baseline Apex-compatible grammar and the reserved
   Zenith punctuation needed by near-term milestones.
2. Add an identifier representation with exact spelling, a canonical
   case-insensitive lookup key, and a file-aware span.
3. Tokenize whitespace, comments, identifiers, keywords, strings, integers,
   delimiters, and operators.
4. Produce explicit lexical diagnostics for invalid characters and unterminated
   strings/comments.
5. Add `zenith tokens <file.zen>` as a thin CLI adapter over the library.
6. Execute `examples/hello.zen` through the token pipeline.

Do not begin parsing until the M1 exit criterion passes.

## Known limitations

- No Zenith or Apex syntax is tokenized, parsed, checked, lowered, or emitted.
- `check`, `build`, `emit`, `test`, and `verify` are documented targets but are
  intentionally unavailable in the bootstrap CLI.
- Source IDs are stable only within one compiler session; project caching and
  content identities are later work.
- Source spans are byte offsets; line/column rendering arrives with M1
  diagnostics.
- The proposed language examples document direction, not shipped syntax.
- No Salesforce schema, Apex Exec, Salesforce CLI, or org integration exists.

## Handoff checklist

After meaningful implementation work:

- Update the completed and limitation lists above.
- Keep exactly one immediate target aligned with the active milestone.
- Update `docs/COMPATIBILITY.md` for changed source or emitted behavior.
- Add or update executable positive, negative, and golden fixtures.
- Add an ADR if an architectural boundary or expensive choice changed.
- Run the verification commands in `AGENTS.md`.
