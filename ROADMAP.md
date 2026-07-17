# Roadmap

This roadmap works backward from `docs/VISION.md`. Milestones describe coherent,
demonstrable capabilities rather than dates. A milestone is complete only when
its exit criterion and required verification pass.

Status values: **Complete**, **Active**, **Planned**, and **Deferred**.

## M0 — Repository and compiler foundation

**Status:** Complete

### Scope

- Rust library and binary crate
- Pinned Rust toolchain and automated formatting, test, and lint verification
- Source file identities and byte spans
- Phase-owned diagnostics
- Bootstrap CLI help and version behavior
- Vision, roadmap, status, architecture, compatibility, development,
  specification, and decision records
- Agent-oriented branch, checkpoint, verification, and handoff conventions

### Exit criterion

The crate builds without third-party dependencies, its unit and CLI smoke tests
pass locally and in CI, and a new Codex task can identify the active milestone
and immediate work without reconstructing project intent.

## M1 — Lexical core and source diagnostics

**Status:** Active

### Scope

- Baseline Apex-compatible identifiers, keywords, literals, comments,
  delimiters, and operators
- Reserved punctuation required by near-term Zenith syntax
- Case-insensitive canonical names with source spelling preserved
- File-aware lexical diagnostics with line and column rendering
- `tokens <file.zen>` CLI inspection

### Exit criterion

`zenith tokens examples/hello.zen` prints a stable token stream, and executable
tests cover comments, escapes, keyword casing, operator boundaries, Unicode
source positions, invalid characters, and unterminated constructs.

## M2 — Parsed syntax and immutable AST

**Status:** Planned

### Scope

- Class, field, property, method, parameter, statement, and expression syntax
  needed by the baseline example
- Blocks, declarations, assignment, calls, member access, conditionals, and
  loops
- Type syntax including generic arguments and nullable suffix reservation
- Immutable parsed AST with shared visitors
- Syntax recovery at useful declaration and statement boundaries
- `ast <file.zen>` CLI inspection

### Exit criterion

The baseline example parses to a stable AST, invalid programs produce localized
syntax diagnostics, and the parser contains no name, type, effect, or emission
logic.

## M3 — Typed Apex baseline and deterministic emission

**Status:** Planned

### Scope

- Project discovery for `.zen` units and minimal `zenith.toml`
- Cross-file declaration collection and case-insensitive name resolution
- Primitive, class, method, field, property, and collection types required by
  ordinary service classes
- Checked call/member targets in typed HIR
- Apex IR distinct from source AST and typed HIR
- Deterministic, formatted `.cls` emission and source maps
- `check`, `build`, and `emit` CLI commands

### Exit criterion

A multi-file Apex-shaped Zenith service compiles into an SFDX-compatible set of
readable `.cls` files. Repeated builds are byte-identical, unsupported syntax
fails explicitly, and generated Apex passes the available Apex compiler check.

This is the first deployable-language checkpoint.

## M4 — Safe values and domain types

**Status:** Planned

### Scope

- Non-null types by default and explicit `T?`
- Flow-sensitive null narrowing and checked safe navigation
- Immutable `let` bindings
- Record/value declarations
- Typed `Id<T extends SObject>` values with zero-cost Apex lowering
- Sealed result types and exhaustive matching
- Source-mapped diagnostics through generated helper code

### Exit criterion

A service using nullable relationships, typed IDs, records, and exhaustive
results emits deployable Apex; null misuse, ID type confusion, mutation of
immutable bindings, and non-exhaustive matches fail during checking.

## M5 — Schema-aware queries and shaped records

**Status:** Planned

### Scope

- SFDX custom object and field metadata import
- Normalized case-insensitive Salesforce schema
- Dedicated static query syntax and bind expressions
- Query projection types such as `Account{Id, Name, Owner.Email}`
- Relationship and field nullability from schema plus query shape
- Static validation of filters, ordering, aggregates, and common relationships
- Deterministic SOQL emission

### Exit criterion

A repository method can query typed records and access only selected fields.
Unknown objects/fields, invalid binds, impossible relationship paths, and
unselected field reads fail before Apex emission.

This is the first distinctly Salesforce-aware checkpoint.

## M6 — Governor effects and bulk safety

**Status:** Planned

### Scope

- Inferred effects for SOQL, DML, callouts, enqueueing, and privilege changes
- Effect propagation through the checked call graph
- Loop, recursion, and trigger-batch amplification analysis
- Checked effect contracts and conservative unknown/unbounded effects
- Diagnostics that show the resource path from caller to operation
- Configuration profiles for relevant Salesforce transaction limits

### Exit criterion

The compiler rejects a query hidden beneath a record loop, verifies a
single-query bulk repository, and reports an actionable call path for every
failed resource contract. Unknown behavior is conservative rather than treated
as free.

This is Zenith's central safety checkpoint.

## M7 — Collection pipelines and custom generics

**Status:** Planned

### Scope

- First-class functions with explicit capture rules
- `map`, `filter`, `flatMap`, `groupBy`, `associateBy`, `partition`, and common
  reductions
- Loop-based lowering that avoids unnecessary Apex allocation
- Custom generic functions and classes
- Specialization/monomorphization for deployable Apex output
- Generic effect propagation

### Exit criterion

A bulk transformation and generic repository compile to readable loop-based
Apex with no runtime callback framework. Specialized output is deterministic,
collision-safe, and type-correct.

## M8 — Typed DML and security contexts

**Status:** Planned

### Scope

- `New<T>`, `Patch<T>`, and loaded query-shape record states
- Typed success and failure results for insert, update, upsert, and delete
- Explicit all-or-none and partial-success behavior
- Scoped `userMode` and `systemMode(reason: ...)` operations
- Security provenance at user-facing and invocable boundaries
- Auditable generated Apex for privilege transitions

### Exit criterion

A service performs partial bulk updates and handles every failure explicitly.
Invalid record states, ignored partial results, implicit privilege elevation,
and privileged-data escape fail during checking.

## M9 — Bulk-first triggers and transaction phases

**Status:** Planned

### Scope

- Event-specific `Change<T>` and change-set trigger constructs
- Checked before/after value and mutability availability
- Changed-field projections
- Bulk effects across trigger handlers
- Recursion policy and deterministic handler generation
- After-commit work with explicit transaction-boundary semantics

### Exit criterion

A multi-event trigger slice compiles into a thin Apex trigger plus generated
handler classes. Event-invalid access and per-record resource work are rejected,
and bulk/recursive behavior is covered by executable fixtures.

## M10 — Tests and local verification

**Status:** Planned

### Scope

- Zenith test discovery, assertions, parameterized cases, and generated Apex
  tests
- Source-mapped failures and coverage
- Apex Exec integration for supported local generated-Apex execution
- Deterministic schema/data fixtures
- Selective Salesforce validation and differential fixtures
- `test` and `verify` CLI workflows

### Exit criterion

A representative Zenith project runs its ordinary test loop locally, maps
failures and coverage to `.zen` source, and executes a targeted final validation
against Salesforce with recorded compatibility results.

## M11 — Durable asynchronous workflows

**Status:** Planned

### Scope

- Durable function syntax with explicit serializable state
- Queueable state-machine lowering
- Retry, backoff, idempotency, correlation, and terminal-failure policies
- Transaction-boundary and effect visibility across continuations
- Generated operational metadata and structured observability hooks

### Exit criterion

A multi-step integration workflow survives asynchronous boundaries and lowers
to ordinary Apex infrastructure with deterministic state classes, explicit
retry behavior, and executable recovery tests.

## M12 — Mature project toolchain

**Status:** Planned

### Scope

- Modules, imports, exports, and module-private visibility
- Safe compile-time derivations for JSON, builders, equality, and adapters
- Incremental compilation and dependency-scoped invalidation
- Content-addressed generated artifacts and build manifests
- API/version compatibility profiles
- IDE/LSP integration based on the compiler library

### Exit criterion

A multi-module enterprise project receives incremental diagnostics and
reproducible builds, uses derived adapters without hidden runtime reflection,
and exposes a stable compiler API for editor tooling.

## Deferred until evidence justifies them

- A custom production runtime or bytecode VM
- General unrestricted macros
- Implicit query hoisting or automatic bulkification that changes observable
  transaction semantics
- Features that require deploying an opaque managed runtime to every org
- Broad platform API coverage unrelated to a complete language slice
