# Current Status

**Last updated:** 2026-07-17

## Active milestone

M4 — Safe values and domain types. Its source, checking, lowering, diagnostics,
generated-helper mapping, and acceptance-fixture contracts are explicit in
`docs/specifications/safe-values-domain-types.md`.

## Completed

- M0 repository/compiler foundation, M1 lexical core, and M2 immutable parsed
  syntax
- Project loading from `zenith.toml`, deterministic recursive `.zen`
  discovery, one-class-per-file validation, and required Salesforce API
  version handling
- Cross-file, case-insensitive declaration/member/method resolution with
  stable diagnostics for collisions, visibility, modifiers, scopes, and
  unsupported parsed syntax
- Checked M3 primitive, class, boundary, and `List`/`Set`/`Map` types plus
  selected call/member/index targets in typed HIR
- Declaration-only handwritten Apex boundary summaries with unknown external
  effects and a real handwritten `Audit.cls` acceptance dependency
- A distinct validated Apex IR, deterministic readable Apex and metadata,
  per-class source maps, `build.json`, and an SFDX `sfdx-project.json`
- Public `check`, `build`, and `emit` CLI commands with status 1 compilation
  failures and status 2 usage failures
- Backend-neutral `passed`, `failed`, `unsupported`, and `internal-error`
  verification outcomes, including complete evidence recording
- Optional pinned Apex Exec M3 smoke verification at revision
  `1e4f1ca1938abfc996651ae447f227e0db680b6a` and capability profile
  `zenith-m3-apex-baseline`
- Byte-deterministic M3 acceptance output for two generated Zenith classes
  beside one handwritten Apex class; the pinned smoke reports
  `OK (3 classes, 3 source files)`
- 93 passing Rust tests: 58 library, 23 CLI, and 12 M3 end-to-end tests,
  including positive, negative, golden, determinism, visibility, filesystem,
  and verifier evidence
- Instrumented `cargo llvm-cov --all-targets` coverage of 88.92% lines overall,
  including 84.87% for the checker, 96.84% for lowering, 93.63% for emission,
  and 98.97% for verification

## Immediate target

Implement the complete M4 vertical slice and executable acceptance fixture
defined by `docs/specifications/safe-values-domain-types.md`.

## Known limitations

- M3 intentionally accepts a selected Apex-compatible service-class baseline,
  not arbitrary Apex source. Constructors, inheritance, interfaces, nullable
  suffixes, safe navigation, postfix expressions, `do while`, `throw`, and
  unlisted collection behavior are rejected explicitly.
- Strong non-null types, flow narrowing, immutable bindings, value records,
  typed SObject IDs, and exhaustive results begin in M4.
- Handwritten Apex interoperability uses a small developer-authored signature
  summary. Extracted semantic API indexes, inheritance, overload conversions,
  and external effect contracts are not implemented.
- No Salesforce schema, query-shape, security, governor-effect, DML, trigger,
  async, or runtime-helper support exists yet.
- The M3 Apex Exec adapter is an optional revision-pinned compile smoke using
  process exit status and preserved output. It is not a structured protocol,
  runtime test adapter, or Salesforce verification.
- Source maps cover generated class text. Backend diagnostic translation,
  stack-frame mapping, runtime coverage mapping, and generated-test provenance
  remain M10 work.
- Project caching and content fingerprints are not implemented.
- No repository license or crate-publication policy has been selected; future
  tasks must not infer one.

## Handoff checklist

After meaningful implementation work:

- Update the completed and limitation lists above.
- Keep exactly one immediate target aligned with the roadmap.
- Update `docs/COMPATIBILITY.md` for changed source or emitted behavior.
- Update the owning specification when observable behavior changes.
- Add executable positive and negative tests plus an emission golden when
  generated output changes.
- Add an ADR if an architectural boundary or expensive choice changed.
- Run the verification commands in `AGENTS.md`.
