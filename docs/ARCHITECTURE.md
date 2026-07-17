# Architecture

## Current foundation

The bootstrap repository contains only the boundaries needed by the active
lexer milestone:

```text
CLI help/version

SourceMap
  ├── SourceFile
  ├── SourceId
  └── file-aware byte Span

Diagnostic
  ├── Severity
  └── owning compiler Phase
```

There is intentionally no placeholder lexer, parser, semantic checker, or
emitter. Each module arrives with the milestone that owns executable behavior.

## Target compiler pipeline

```text
Zenith source + project configuration + Salesforce metadata
    │
    ▼
Source loader ─► stable file identities and source text
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
Lowering ──────► explicit Apex-oriented IR and generated helpers
    │
    ▼
Apex emitter ─► formatted SFDX source, source maps, and build manifest
    │
    ├──► Apex Exec local checks/tests where supported
    └──► Salesforce validation as the final oracle
```

The CLI is a thin adapter over the compiler library. It owns arguments,
filesystem interaction, and rendering, not language semantics.

## Phase contracts

### Source management

Source management owns file identities, byte ranges, line/column mapping, and
source text. Later project caching may add content identities, but compiler
phases must never infer file ownership from concatenated offsets.

### Lexing

The lexer recognizes spelling and token boundaries. It canonicalizes names for
case-insensitive lookup while retaining exact source spelling. It does not
classify an identifier by scope or type.

### Parsing

The parser produces immutable syntax. It owns precedence, grammar recovery, and
syntax diagnostics. It does not resolve names, infer nullability, count effects,
or make emission choices.

### Resolution and typing

Resolution selects declarations independent of runtime values. Typing records
the selected call/member, conversions, nullability, generic substitutions, and
value category in HIR. Lowering must not repeat overload resolution.

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
expression-level constructs into statements. It does not format text.

### Apex emission

The emitter serializes valid Apex IR deterministically. It owns formatting,
generated names, manifests, and source-map segments. It does not repair invalid
typed state or invent lowering semantics.

### Verification

Verification checks generated artifacts without weakening the compiler's own
guarantees. Apex Exec can provide fast local feedback for its compatible
surface; Salesforce remains the final oracle for deployment behavior.

## Current modules

| Module | Responsibility |
|---|---|
| `source` | Session-local file identities, byte spans, and source slicing |
| `diagnostic` | Severity and explicit compiler-phase ownership |
| `lib` | Public compiler-foundation façade |
| `main` | Bootstrap CLI argument and process behavior |

## Planned module boundaries

| Module | Responsibility |
|---|---|
| `token` / `lexer` | Token representation and source-to-token conversion |
| `ast` / `parser` | Immutable parsed syntax and grammar |
| `resolve` | Cross-file names, scopes, imports, and symbol identities |
| `hir` / `types` | Checked expressions, declarations, conversions, and types |
| `schema` / `query` | Salesforce metadata normalization and query shapes |
| `effects` | Resource inference, contracts, and call-path diagnostics |
| `lower` | Zenith-to-Apex semantic desugaring |
| `apex_ir` / `emit` | Valid target representation and deterministic Apex text |
| `project` | Configuration, dependency graph, caching, and build manifests |
| `verify` | Apex Exec and Salesforce validation adapters |

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

## Generated artifact layout

The target layout is:

```text
.zenith/
  generated/
    force-app/main/default/classes/
  build.json
  source-map.json
```

Generated files are disposable build artifacts. Golden fixtures under `tests/`
are the exception and exist specifically to review emitter behavior.

`build.json` should record at least:

- compiler version
- configuration and schema fingerprints
- input content identities
- generated files and semantic owners
- runtime-helper requirements, if any
- compatibility profile

## Compatibility boundaries

Zenith features fall into four lowering classes:

1. **Native:** maps directly to equivalent Apex syntax.
2. **Desugared:** expands locally into ordinary Apex statements or declarations.
3. **Generated:** requires deterministic helper types or methods.
4. **Runtime-assisted:** requires an explicit, versioned helper library.

Runtime-assisted features carry a higher adoption cost and should be rare.
Every shipped feature declares its class in `docs/COMPATIBILITY.md`.

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
