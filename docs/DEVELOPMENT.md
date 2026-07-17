# Development

## Requirements

- Rust 1.88 or newer
- Cargo
- Git

`rust-toolchain.toml` pins Rust 1.88.0 with `rustfmt` and Clippy so local and CI
quality gates use the declared minimum toolchain. The bootstrap crate has no
third-party dependencies. Salesforce CLI and Apex Exec become optional
verification tools only when their roadmap integrations arrive.

## Build and inspect

```bash
cargo build
cargo run -- --help
cargo run -- --version
```

The baseline source fixture is `examples/hello.zen`. The active M1 milestone
will introduce:

```bash
cargo run -- tokens examples/hello.zen
```

Do not document a CLI command as current behavior before an executable test
passes.

## Required verification

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Run the relevant CLI example after changing command behavior. Once emission
exists, inspect the generated Apex and run its available compiler verification.
GitHub Actions runs the required gates with Rust 1.88.0 and also executes the
test suite on the current stable Rust release.

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
6. the directly relevant specification and ADRs

`STATUS.md` identifies the single immediate target. The roadmap defines its
boundary and exit criterion.

### 2. Confirm repository state

Inspect the current branch, working tree, recent commits, and relevant tests.
Preserve unrelated changes. Create a scoped `codex/` branch before editing.

### 3. Select one vertical slice

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
- Differential fixtures for Apex Exec and Salesforce compatibility

Full-program `.zen` scenarios should combine several supported features and run
through the public compiler API and built CLI. Keep narrow edge cases in the
owning module.

## Generated-output review

Generated Apex is reviewable product output:

- Format it deterministically.
- Prefer straightforward Apex over clever compact output.
- Keep generated names stable and collision-safe.
- Preserve source maps through helpers and desugared statements.
- Test behavior, not only text, when a compatible execution backend exists.
- Never hand-edit build output to make a fixture pass.

## Documentation ownership

- Product intent belongs in `docs/VISION.md`.
- Milestone sequencing belongs in `ROADMAP.md`.
- Immediate continuation state belongs in `docs/STATUS.md`.
- Shipped claims belong in `docs/COMPATIBILITY.md`.
- Phase boundaries belong in `docs/ARCHITECTURE.md`.
- Intended language rules belong in `docs/specifications/`.
- Expensive rationale belongs in an ADR.
- Recurring agent instructions belong in `AGENTS.md`.
