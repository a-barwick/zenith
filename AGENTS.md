# Zenith Project Guidance

## Project intent

Zenith is an Apex-targeting, bulk-first language with a deliberately selected
Apex-compatible baseline and additional checked constructs. It compiles to
readable, deployable Salesforce Apex and should catch ordinary type,
query-shape, security-context, and governor-limit mistakes locally without
claiming to accept all Apex source.

Before changing behavior, read:

1. `docs/VISION.md`
2. `ROADMAP.md`
3. `docs/STATUS.md`
4. `docs/ARCHITECTURE.md`
5. `docs/COMPATIBILITY.md`
6. `docs/DEVELOPMENT.md`
7. `docs/specifications/README.md`
8. `docs/decisions/README.md`

Then read the directly relevant specifications and ADRs. Work touching
generated-Apex verification or Apex compatibility must also read
`docs/APEX_EXEC.md`.

## Working rules

- Create an appropriately named `codex/` branch before making changes; never
  implement directly on `main`.
- Commit coherent, verified checkpoints while work is in progress. Keep each
  checkpoint buildable and give it a descriptive message.
- Work within the active roadmap milestone unless the user explicitly changes
  scope.
- Do not activate a milestone until its observable acceptance fixture and the
  specifications needed to implement it without guesswork are explicit.
- Keep source management, lexing, parsing, resolution, typing, schema/query
  checking, effect analysis, lowering, Apex emission, project orchestration,
  and verification separate.
- Apex and therefore Zenith names are case-insensitive. Preserve source
  spelling and spans for diagnostics; canonicalize only for lookup.
- Every accepted Zenith construct must either lower with defined semantics or
  produce an explicit unsupported diagnostic. Never silently approximate it.
- Treat generated Apex and source maps as observable product output. Add golden
  tests when output changes intentionally.
- Add executable positive and negative tests for every observable behavior and
  every bug fix.
- Record expensive or consequential design choices in `docs/decisions/`.
- Update `docs/STATUS.md` and `docs/COMPATIBILITY.md` after meaningful feature
  work.
- Prefer a complete vertical language slice over several disconnected syntax
  additions.
- Do not add speculative empty modules. Add a module when its owning behavior
  arrives.
- Never depend on a sibling `apex-exec` checkout or import its internal compiler
  types. It is an optional, capability-gated verification backend.

## Verification

Run all of the following before declaring implementation work complete:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

For CLI or emitted-Apex behavior, also execute the relevant example through
`cargo run` and inspect or test the generated output.
