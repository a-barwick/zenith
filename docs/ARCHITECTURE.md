# Architecture

## Current compiler front end

M2 implements the complete source-to-immutable-syntax boundary:

```text
CLI help/version/tokens/ast

SourceMap
  ├── SourceFile
  ├── SourceId
  ├── file-aware byte Span
  └── Unicode-scalar source locations
          │
          ▼
Lexer ───► tokens with exact spans, canonical names, and decoded strings
          │
          ▼
Parser ──► immutable untyped syntax with source-spanned nodes
          │
          ├──► shared read-only visitor
          └──► stable AST inspection renderer

Diagnostic
  ├── stable code, severity, and owning phase
  ├── primary/secondary labels, notes, and help
  └── deterministic source rendering
```

There is intentionally no placeholder resolver, semantic checker, or emitter.
The parser contains no name, type, schema, effect, lowering, or emission logic.
Each later module arrives with the milestone that owns executable behavior.

## Product and runtime boundary

Zenith is a source-to-source compiler with a TypeScript-like deployment model:
developers author a richer language, while ordinary target-language artifacts
remain the unit of deployment and execution. It is not a token-rewriting
transpiler. Query shapes, nullability, typed IDs, security provenance, governor
effects, and selected calls require a complete semantic pipeline before Apex
can be emitted.

Salesforce is the production runtime. Apex Exec is an optional local
verification and execution backend, not a Zenith runtime and not the owner of
Zenith semantics. `check` and `build` must remain usable without it.

## Target compiler pipeline

```text
Zenith source + project configuration + Salesforce metadata
              + explicit or extracted handwritten-Apex API summaries
    │
    ▼
Source loader ─► stable file identities, line indexes, and source text
    │
    ▼
Lexer ─────────► tokens with spelling, canonical names, and spans
    │
    ▼
Parser ────────► immutable untyped syntax
    │
    ▼
Resolver ──────► declarations, imports, scopes, and selected symbols
    │
    ▼
Type checker ─► typed HIR and selected calls/members/conversions
    │
    ▼
Schema checker ► query shapes, SObject states, and security provenance
    │
    ▼
Effect checker ► resource summaries and checked effect contracts
    │
    ▼
Lowering ──────► explicit Apex-oriented IR and collision-checked helpers
    │
    ▼
Apex emitter ─► formatted SFDX source, metadata, and source-map segments
    │
    ▼
Project output ► deterministic layout, manifest, and compatibility evidence
    │
    ├──► versioned verification adapter ─► Apex Exec checks/tests
    │                                      where supported
    └──► Salesforce adapter ─────────────► final validation oracle
```

The CLI is a thin adapter over the compiler library. It owns arguments,
filesystem interaction, and rendering, not language semantics.

## Phase contracts

### Source management

Source management owns file identities, byte ranges, line/column mapping, and
source text. Later project caching may add content identities, but compiler
phases must never infer file ownership from concatenated offsets.

The current `SourceMap::add` API assigns an identity to each loaded entry. M3
project caching must retain one session-local identity for a canonical project
path across reparses rather than allocating a new identity for every edit.

### Lexing

The lexer recognizes spelling and token boundaries. It canonicalizes names for
case-insensitive lookup while retaining exact source spelling. It does not
classify an identifier by scope or type.

Its observable contract is `docs/specifications/lexical-structure.md`.

### Parsing

The parser produces immutable syntax. It owns precedence, grammar recovery, and
syntax diagnostics. It does not resolve names, infer nullability, count effects,
or make emission choices.

The M2 AST owns source-faithful names and complete file-aware node spans behind
read-only accessors. A shared visitor walks declaration, type, statement, and
expression children in source order. The parser may return a recovered partial
tree alongside diagnostics for tooling, but later semantic phases do not run
when parse errors exist. The observable grammar, recovery boundaries, stable
diagnostic codes, and inspection format are defined in
`docs/specifications/syntax.md`.

### Resolution and typing

Resolution selects declarations independent of runtime values. Typing records
the selected call/member, conversions, nullability, generic substitutions, and
value category in HIR. Lowering must not repeat overload resolution.

Handwritten Apex declarations enter through explicit boundary declarations or
a versioned semantic API index produced by a compatible Apex compiler. The
index may supply names, signatures, members, inheritance, and visibility; it
does not supply Zenith governor or security guarantees. External behavior
without a checked Zenith contract remains conservative.

### Schema checking

Schema checking consumes a normalized Salesforce schema rather than raw SFDX
XML. Query projection shapes and record states are compiler types, not emitter
guesses. The schema model remains independent from any local database backend.

### Effect checking

Effect checking summarizes platform operations through the call graph.
Unknown calls remain unknown or unbounded; they are never treated as zero-cost.
Loop and recursion analysis are conservative and produce source-traceable
diagnostics.

### Lowering

Lowering makes every Zenith-only construct explicit in an Apex-oriented IR.
It may generate helper declarations, specialize generics, and convert
expression-level constructs into statements. Stable semantic helper identities
and collision checks are complete before emission. Lowering does not format
text.

### Apex emission

The emitter serializes valid Apex IR deterministically. It owns formatting,
target file names, companion Salesforce metadata, and source-map segments. It
does not allocate semantic helper identities, repair invalid typed state, or
invent lowering semantics.

### Project orchestration

Project orchestration owns configuration, required target API version,
discovery, dependency order, caches, output layout, and `build.json`. It invokes
phases without absorbing their semantic logic and records verification evidence
without rewriting compiler results.

### Verification

Verification checks generated artifacts without weakening the compiler's own
guarantees. Apex Exec can provide fast local feedback for its compatible
surface through the boundary in `docs/APEX_EXEC.md`; Salesforce remains the
final oracle for deployment behavior.

Local user tests execute generated Apex rather than Zenith AST or HIR. This
ensures the normal test loop exercises lowering, generated helpers, and target
emission. Verification translates generated-file diagnostics, stack frames,
and coverage through the build source map before reporting them to a Zenith
developer.

The Apex Exec adapter uses a versioned machine-readable protocol rather than
depending on compiler or runtime internals. Requests identify the generated
project and target profile. Responses include backend and protocol versions,
declared capabilities, structured diagnostics, test events, runtime frames,
and coverage.

Every operation produces one of three outcomes:

| Outcome | Meaning |
|---|---|
| Passed | The backend declares the required capability and the operation succeeded |
| Unsupported | The backend cannot cover part of the emitted surface; the Zenith build remains valid and another backend is required |
| Failed | The backend declares support and reports a compile, test, or runtime failure |

Unsupported local verification must never be rendered as either a Zenith
compile error or a passing test. The complete boundary is recorded in
[ADR 0004](decisions/0004-apex-exec-process-boundary.md).

### Test planning and generation

Test generation consumes checked HIR, control-flow identities, schema
constraints, effect summaries, authored examples or contracts, deterministic
fixtures, and source-mapped coverage. It produces an explicit test plan before
any test source or Apex IR is emitted.

```text
Typed/effect HIR + schema + contracts/examples
                     │
                     ▼
Test planner ─────► branch goals, inputs, fixtures, oracle classes
                     │
                     ▼
Candidate lowering ► generated Apex tests or local coverage probes
                     │
                     ▼
Apex Exec ─────────► coverage and runtime results
                     │
                     └──► source-map feedback to stable Zenith branch goals
```

Compiler-owned tests require an oracle derived from language semantics or an
explicit developer-authored contract, invariant, or example. When no trustworthy
oracle exists, Zenith may generate an editable draft or use a non-deployable
coverage probe during synthesis. It does not silently promote invocation-only
code or observed behavior into a trusted test.

Coverage and oracle strength are independent report dimensions. Generated cases
carry provenance and stable identities so managed output can be regenerated,
editable drafts can be adopted without later overwrite, and stale cases can be
identified. The proposed behavior is specified in
[Testing and test generation](specifications/testing-and-test-generation.md)
and its trust boundary is recorded in
[ADR 0005](decisions/0005-generated-tests-need-oracles.md).

## Current modules

| Module | Responsibility |
|---|---|
| `source` | Session-local file identities, byte spans, source slicing, and line/column views |
| `diagnostic` | Stable diagnostics, labels, ordering, and console rendering |
| `token` | Token, identifier, canonical-name, and stable inspection representation |
| `lexer` | Recoverable source-to-token conversion |
| `ast` | Immutable parsed declarations, types, statements, expressions, shared visitor, and stable rendering |
| `parser` | Precedence, parsed-baseline grammar, localized recovery, and syntax diagnostics |
| `lib` | Public compiler front-end façade |
| `main` | CLI argument, source loading, and inspection behavior |

## Planned module boundaries

| Module | Responsibility |
|---|---|
| `resolve` | Cross-file names, scopes, imports, and symbol identities |
| `hir` / `types` | Checked expressions, declarations, conversions, and types |
| `schema` / `query` | Salesforce metadata normalization and query shapes |
| `effects` | Resource inference, contracts, and call-path diagnostics |
| `lower` | Zenith-to-Apex semantic desugaring |
| `apex_ir` / `emit` | Valid target representation, Apex text, companion metadata, and source-map segments |
| `project` | Configuration, dependency graph, caching, output layout, and build manifests |
| `apex_api` | Explicit or extracted declarations for handwritten Apex interoperability |
| `test_plan` | Branch goals, generated fixtures, oracle provenance, and candidate minimization |
| `verify` | Versioned Apex Exec and Salesforce validation adapters |

Module names can evolve through implementation, but the ownership boundaries
require an ADR to collapse.

## Core invariants

### Names

Zenith names are case-insensitive because distinct source names cannot remain
distinct after Apex emission. Every identifier eventually carries:

- exact source spelling
- a canonical lookup key
- its source span

Diagnostics and emitted user-facing names preserve source spelling whenever
collision rules allow it.

### No phase backfilling

- The parser never compensates for missing lexical structure.
- Typing never depends on runtime values.
- Effect checking never relies on emitted-text inspection.
- Lowering never performs unresolved name lookup.
- The emitter never accepts malformed target IR.
- Verification never turns an unsupported Zenith construct into accepted code.
- Local execution never bypasses lowering by interpreting Zenith HIR.
- Test generation never equates reached coverage with a valid behavioral
  oracle.

### Explicit unsupported behavior

Syntax can be recognized before its semantics are supported, but checking must
then produce a dedicated unsupported diagnostic. Emitting a plausible-looking
approximation is forbidden.

### Deterministic generated names

Generated names must derive from stable semantic identities, not hash-map
iteration order or process randomness. Collisions with user declarations are
detected before emission.

### Source mapping

Every generated user-semantic operation should map to its originating Zenith
span. Generated scaffolding without a direct source construct maps to the
enclosing declaration and is labeled generated in the manifest.

Backend diagnostics, runtime frames, and coverage first resolve in generated
Apex span space, then map to Zenith spans. Managed generated tests and
assertions additionally retain their test-plan and oracle identities.

## Generated artifact layout

The target layout is:

```text
.zenith/
  generated/
    force-app/main/default/classes/
      Example.cls
      Example.cls-meta.xml
  test-candidates/
  build.json
  source-map.json
  verification.json
```

Generated files are disposable build artifacts. Golden fixtures under `tests/`
are the exception and exist specifically to review emitter behavior.
`test-candidates/` contains non-deployable synthesis probes. An explicit adopt
operation may create editable `.zen` drafts outside `.zenith/`; adopted files
become developer-owned and are not overwritten.

`build.json` should record at least:

- compiler version
- target Salesforce API version
- configuration and schema fingerprints
- input content identities
- generated files and semantic owners
- runtime-helper requirements, if any
- compatibility profile
- external verifier version, capability profile, and result when one ran
- generated-test provenance and oracle class
- required verification capabilities

`verification.json` records the selected backend, protocol and backend
versions, declared capability profile, result state, and any Salesforce
differential evidence. A prior result is stale when its generated artifact,
configuration, schema, or backend profile fingerprints no longer match.

## Compatibility boundaries

Zenith features fall into four lowering classes:

1. **Native:** maps directly to equivalent Apex syntax.
2. **Desugared:** expands locally into ordinary Apex statements or declarations.
3. **Generated:** requires deterministic helper types or methods.
4. **Runtime-assisted:** requires an explicit, versioned helper library.

Runtime-assisted features carry a higher adoption cost and should be rare.
Every shipped feature declares its class in `docs/COMPATIBILITY.md`.

## Apex Exec boundary

Apex Exec is a separate local Apex runtime and an optional verification backend,
not a Zenith compiler phase or source-language dependency. M3 may use a pinned
compile smoke check after emission. Rich source-mapped test integration waits
for the versioned protocol required by M10 and ADR 0004. Backend unsupported
results remain visible and never become compiler success.

Verification backends are not a fifth lowering class. They consume emitted
Apex and cannot make an otherwise unsupported lowering acceptable.

## Performance direction

Correctness and transparent output come before optimization. Project-scale
performance should later use:

- Interned canonical names and types
- Stable declaration identities
- Dependency-scoped incremental parsing and checking
- Cached typed/effect HIR
- Content-addressed emission
- Parallel checking only where diagnostic order remains deterministic

Optimizations cannot weaken diagnostics, output stability, or source maps.
