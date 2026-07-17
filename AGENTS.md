# Zenith Project Guidance

## Project intent

Zenith is a safe, bulk-first superset of Apex that compiles to readable,
deployable Salesforce Apex. It should catch ordinary type, query-shape,
security-context, and governor-limit mistakes locally while preserving a
practical path to standard Salesforce deployment.

Before changing behavior, read:

1. `docs/VISION.md`
2. `ROADMAP.md`
3. `docs/STATUS.md`
4. `docs/ARCHITECTURE.md`
5. `docs/COMPATIBILITY.md`
6. `docs/specifications/README.md`

## Working rules

- Create an appropriately named `codex/` branch before making changes; never
  implement directly on `main`.
- Commit coherent, verified checkpoints while work is in progress. Keep each
  checkpoint buildable and give it a descriptive message.
- Work within the active roadmap milestone unless the user explicitly changes
  scope.
- Keep source management, lexing, parsing, resolution, typing, effect analysis,
  lowering, Apex emission, and verification separate.
- Apex and therefore Zenith names are case-insensitive. Preserve source
  spelling and spans for diagnostics; canonicalize only for lookup.
- Every accepted Zenith construct must either lower with defined semantics or
  produce an explicit unsupported diagnostic. Never silently approximate it.
- Treat generated Apex and source maps as observable product output. Add golden
  tests when output changes intentionally.
- Run user-facing local verification against generated Apex, never directly
  against Zenith AST or HIR. Backend capability gaps are unsupported local
  verification, not Zenith compilation failures.
- Keep generated-test coverage separate from correctness. A managed generated
  test needs a semantic or developer-authored oracle; otherwise emit a
  reviewable draft or a non-deployable probe.
- Add executable positive and negative tests for every observable behavior and
  every bug fix.
- Record expensive or consequential design choices in `docs/decisions/`.
- Update `docs/STATUS.md` and `docs/COMPATIBILITY.md` after meaningful feature
  work.
- Prefer a complete vertical language slice over several disconnected syntax
  additions.
- Do not add speculative empty modules. Add a module when its owning behavior
  arrives.

## Verification

Run all of the following before declaring implementation work complete:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

For CLI or emitted-Apex behavior, also execute the relevant example through
`cargo run` and inspect or test the generated output.
