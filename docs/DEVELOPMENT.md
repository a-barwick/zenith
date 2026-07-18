# Development

## Requirements

- Rust 1.88 or newer
- Cargo
- Git

`rust-toolchain.toml` pins Rust 1.88.0 with `rustfmt` and Clippy so local and CI
quality gates use the declared minimum toolchain. The compiler crate has no
third-party dependencies. Apex Exec is optional for the revision-pinned M3
generated-Apex smoke; ordinary checking and building require neither Apex Exec
nor Salesforce CLI.

## Build and inspect

```bash
cargo build
cargo run -- --help
cargo run -- --version
```

The smallest source fixture is `examples/hello.zen`; the broader M1/M2 baseline
is `examples/lexical-baseline.zen`. Inspect either through the shipped token and
AST commands:

```bash
cargo run -- tokens examples/hello.zen
cargo run -- tokens examples/lexical-baseline.zen
cargo run -- ast examples/hello.zen
cargo run -- ast examples/lexical-baseline.zen
cargo run -- ast examples/parsed-baseline.zen
```

`examples/parsed-baseline.zen` is the broad executable parser/renderer fixture
covering the complete M2 surface beyond the required lexical baseline.

The complete M3 project fixture is `examples/m3-service`:

```bash
cargo run -- check examples/m3-service
cargo run -- emit examples/m3-service
cargo run -- build examples/m3-service
```

`check` writes nothing, `emit` prints the complete ordered artifact set, and
`build` writes the SFDX-compatible tree beneath the configured `.zenith`
directory.

## Required verification

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Run the relevant CLI example after changing command behavior. Inspect generated
Apex after emission changes. GitHub Actions runs the required gates with Rust
1.88.0 and also executes the test suite on the current stable Rust release.

Coverage is measured separately from correctness:

```bash
rustup run stable cargo llvm-cov --all-targets --summary-only
```

To repeat the optional M3 Apex Exec smoke, use a checkout at the exact revision
recorded in `docs/APEX_EXEC.md`:

```bash
scripts/verify-apex-exec-m3.sh /path/to/pinned/apex-exec
```

The script refuses any other revision, builds the backend with its lockfile,
builds the M3 fixture, and requires a `passed` verification outcome.

## Git workflow

- Create an appropriately scoped branch before changing files. Agent-created
  branches use the `codex/` prefix, such as `codex/m1-lexer`.
- Do not implement directly on `main`.
- Commit coherent checkpoints after their focused tests pass.
- Keep every checkpoint buildable and independently reviewable.
- Keep unrelated or user-owned working-tree changes out of commits.
- Before completing a milestone, satisfy its exit criterion, run the full
  verification suite, update the project documents, and merge the milestone
  branch into `main`.

## Agentic development loop

The repository is designed so a fresh Codex task can continue without relying
on conversational memory.

### 1. Reload intent

Read, in order:

1. `docs/VISION.md`
2. `ROADMAP.md`
3. `docs/STATUS.md`
4. `docs/ARCHITECTURE.md`
5. `docs/COMPATIBILITY.md`
6. `docs/specifications/README.md`
7. `docs/decisions/README.md`
8. the directly relevant specification and ADRs

Read `docs/APEX_EXEC.md` before work on Apex compatibility, generated-Apex
verification, or a local execution adapter.

`STATUS.md` identifies the single immediate target. The roadmap defines its
boundary and exit criterion.

### 2. Confirm repository state

Inspect the current branch, working tree, recent commits, and relevant tests.
Preserve unrelated changes. Create a scoped `codex/` branch before editing.

### 3. Select one vertical slice

Before a milestone becomes Active, make its acceptance fixture and the
specifications required for implementation explicit. If those are missing,
specification is the first slice rather than agent-local design-by-code.

Choose the smallest behavior that crosses every required phase honestly. For
example, one operator may require a token, parser rule, typed HIR fact, lowering
rule, emitted fixture, and diagnostic. Avoid adding several syntax forms that
stop at parsing.

Write down the observable success and failure cases before implementation.

### 4. Establish executable evidence

Add tests at the narrowest owning layer:

- a positive case proving the intended behavior
- a negative case proving invalid or unsupported behavior is rejected
- a golden emitted-Apex fixture when lowering changes
- a CLI scenario when file handling or rendering changes

Tests are the compatibility evidence; documentation states what that evidence
means.

### 5. Implement through owned phases

Keep compiler decisions in their owning module. Do not let the emitter resolve
types, the type checker count emitted text, or verification compensate for a
missing diagnostic.

When an intended construct reaches an unimplemented phase, reject it explicitly
until the complete slice is ready.

### 6. Verify and checkpoint

Run focused tests during development, then the full required suite before a
checkpoint. Commit a coherent, buildable unit with a descriptive message.

Long tasks should create several verified checkpoints rather than one large
end-state commit.

### 7. Reconcile project state

After meaningful behavior:

- update shipped claims in `docs/COMPATIBILITY.md`
- update completed work, limitations, and immediate handoff in
  `docs/STATUS.md`
- update a specification when the intended rule changed
- add an ADR for a consequential, expensive-to-reverse choice

Do not duplicate detailed requirements. Link to the authoritative document.

### 8. Complete or hand off

A milestone is complete only when its exit criterion passes. If work stops
mid-milestone, leave the branch buildable and make `STATUS.md` precise enough
for the next task to resume at the next unchecked behavior.

## Testing strategy

Behavior should be exercised at the narrowest useful layer:

- Source tests for identities, spans, Unicode boundaries, and line mapping
- Lexer tests for token boundaries, trivia, spelling, and lexical errors
- Parser tests for syntax shape, precedence, and recovery
- Resolver tests for scopes, imports, visibility, and case-insensitive clashes
- Type tests for nullability, conversions, generics, and selected targets
- Schema/query tests for projections, binds, and relationship shapes
- Effect tests for call paths, loops, recursion, and contracts
- Lowering tests for semantic transformations and generated helpers
- Emitter golden tests for deterministic, readable Apex
- Project tests for discovery, dependencies, caching, and manifests
- CLI tests for filesystem behavior and rendered diagnostics
- Verification-adapter contract tests for protocol versions, declared
  capabilities, and passed/unsupported/failed outcomes
- Differential fixtures for Apex Exec and Salesforce compatibility
- Generated-test planner tests for stable branch goals, schema-valid fixtures,
  oracle provenance, candidate minimization, and stale-output detection

Full-program `.zen` scenarios should combine several supported features and run
through the public compiler API and built CLI. Keep narrow edge cases in the
owning module.

Real-world vendored fixtures must record their immutable upstream revision,
license, content fingerprint, and why they exercise a distinct supported or
aspirational boundary. An ignored progress indicator never counts as shipped
compatibility evidence.

### Verification backend rules

- Execute emitted Apex for user-facing local tests; do not add a separate
  Zenith HIR runtime as a shortcut.
- Keep `check` and `build` independent of Apex Exec availability.
- Pin the backend revision/profile and record complete evidence. A structured
  backend protocol remains M10 work.
- Treat unsupported backend surface as a distinct verification outcome, not a
  Zenith compile failure or a pass.
- Map backend diagnostics, stack frames, and coverage through generated Apex
  spans before rendering Zenith locations.
- Keep Salesforce differential evidence distinct from local backend evidence.

### Generated-test rules

- Give every generated case a stable identity, branch goal, provenance, and
  oracle class.
- Emit a managed trusted test only when its assertion follows from language
  semantics or an explicit contract, invariant, or example.
- Emit ambiguous cases as editable drafts, or retain them as non-deployable
  local coverage probes.
- Never silently convert observed behavior into a normative assertion.
- Never report code coverage as equivalent to behavioral correctness.
- Do not overwrite an adopted developer-owned test during regeneration.

## Generated-output review

Generated Apex is reviewable product output:

- Format it deterministically.
- Prefer straightforward Apex over clever compact output.
- Keep generated names stable and collision-safe.
- Preserve source maps through helpers and desugared statements.
- Test behavior, not only text, when a compatible execution backend exists.
- Never hand-edit build output to make a fixture pass.
- Record verification backend versions, capabilities, and result fingerprints.
- Keep managed generated tests and temporary coverage probes distinguishable in
  the manifest and generated tree.

## Documentation ownership

- Product intent belongs in `docs/VISION.md`.
- Milestone sequencing belongs in `ROADMAP.md`.
- Immediate continuation state belongs in `docs/STATUS.md`.
- Shipped claims belong in `docs/COMPATIBILITY.md`.
- Phase boundaries belong in `docs/ARCHITECTURE.md`.
- Intended language rules belong in `docs/specifications/`.
- Expensive rationale belongs in an ADR.
- Recurring agent instructions belong in `AGENTS.md`.
