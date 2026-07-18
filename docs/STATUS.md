# Current Status

**Last updated:** 2026-07-17

## Active milestone

No implementation milestone is active. M4 — Safe values and domain types is
complete. M5 remains Planned until its standard-schema artifact,
fingerprint/version policy, metadata merge precedence, specification, and
observable acceptance fixture are explicit.

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
- M4 non-null defaults, explicit nullable types, local/parameter flow
  narrowing, checked safe navigation/coalescing, immutable inferred `let`
  bindings, nominal records, typed zero-cost SObject IDs, sealed results, and
  exhaustive statement matching
- Deterministic record/result helper classes, match desugaring with
  single-evaluation temporaries, generated-helper source maps, non-emitting
  SObject domain manifest entries, and global/public accessibility preservation
- Capability preflight that records M4-only output as unsupported by the pinned
  M3 Apex Exec profile without launching the backend
- Byte-deterministic M4 acceptance output for four generated classes and two
  compile-time SObject domain tags
- 111 passing Rust tests: 59 library, 23 CLI, 12 M3 end-to-end, and 17 M4
  end-to-end tests, including positive/negative typing, branch refinement,
  collisions, golden output, source maps, determinism, filesystem builds, and
  verifier capability gating
- Instrumented `cargo llvm-cov --all-targets` coverage of 88.68% lines overall,
  including 88.36% for checking, 91.27% for parsing, 98.39% for lowering,
  95.06% for emission, and 99.12% for verification

## Immediate target

Specify M5's deterministic standard-schema input, version/fingerprint policy,
metadata merge precedence, complete query-shape contract, and executable
acceptance fixture before activating implementation.

## Known limitations

- Zenith intentionally accepts a selected Apex-compatible service-class
  baseline plus the completed M4 surface, not arbitrary Apex source.
  Constructors, inheritance, interfaces, mutable postfix expressions,
  `do while`, `throw`, and unlisted collection behavior are rejected
  explicitly.
- Null refinement is limited to direct local and parameter comparisons, Boolean
  short-circuit branches, conditionals, and return guards. Fields/properties are
  deliberately not narrowed across possible aliasing or calls.
- M4 automatic properties must be nullable; constructors and accessor bodies
  are not yet available to prove non-null initialization.
- Records do not yet provide structural equality, hashing, inheritance,
  methods, defaults, or destructuring. Sealed matching is statement-only.
- Explicit SObject declarations are nominal compile-time tags, not schema
  definitions. M5 will replace or validate them using normalized metadata.
- Handwritten Apex interoperability uses a small developer-authored signature
  summary. Extracted semantic API indexes, inheritance, overload conversions,
  and external effect contracts are not implemented.
- No Salesforce schema, query-shape, security, governor-effect, DML, trigger,
  async, or runtime-helper support exists yet.
- The Apex Exec adapter remains an optional revision-pinned M3 compile smoke
  using process exit status and preserved output. M4-only target output is
  reported as unsupported; there is no M4 backend compile or Salesforce
  verification evidence.
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
