# Compatibility

This document states the currently supported Zenith surface and how shipped
features lower to Apex. It is a product contract, not a statement of intended
syntax.

## Feature status

| Status | Meaning |
|---|---|
| Verified | Generated behavior has executable Salesforce differential evidence |
| Compatible | Intended to preserve the documented Apex-observable behavior |
| Local | Implemented compiler/tool behavior with no Apex runtime claim |
| Experimental | Implemented but intentionally unstable |
| Unsupported | Recognized or requested but rejected explicitly |
| Planned | Not implemented |

No feature is currently **Verified** or **Compatible** because the compiler
does not yet emit Apex.

## Verification outcomes

Feature status and the result of one verification run are different concepts.
Every backend operation reports one of these outcomes:

| Outcome | Meaning |
|---|---|
| Passed | The backend declares the required capability and the requested check or execution succeeded |
| Unsupported | The backend cannot cover part of the emitted surface; the Zenith build remains valid but requires another backend |
| Failed | The backend declares support and reports a compile, test, or runtime failure |

An Apex Exec pass is local evidence for its declared profile, not Salesforce
verification. An unsupported result is neither a pass nor a Zenith source
error.

## Lowering classes

| Class | Meaning |
|---|---|
| Native | Emits the equivalent Apex construct directly |
| Desugared | Emits ordinary local Apex statements or declarations |
| Generated | Emits deterministic helper types or methods |
| Runtime-assisted | Requires an explicit versioned helper library |
| None | Compiler/tooling behavior or no lowering exists |
| TBD | Allowed only for a Planned row whose lowering design is not yet active |

## Tooling foundation

| Feature | Status | Lowering | Notes |
|---|---|---|---|
| `zenith --help` | Local | None | Documents current inspection commands |
| `zenith --version` | Local | None | Prints the crate version |
| Local and CI verification | Local | None | Rust 1.88.0 fmt/test/Clippy gates plus current-stable tests |
| Source identities | Local | None | Stable within one compiler session |
| File-aware byte spans | Local | None | UTF-8-safe slicing and Unicode-scalar line/column views |
| Phase-owned diagnostics | Local | None | Source, language, schema, verification, and project phases represented |
| `.zen` parsing | Local | None | Selected M2 grammar produces immutable source-spanned syntax |
| `.zen` compilation | Planned | TBD | Parsing is implemented; checking and emission begin in M3 |

## Lexical surface

| Feature | Status | Notes |
|---|---|---|
| UTF-8 source loading | Local | Invalid UTF-8 produces `source.invalid-utf8` without panicking |
| Apex-shaped words and reserved words | Local | Case-insensitive reserved, contextual, and Zenith word tables |
| Exact and canonical identifier spelling | Local | ASCII identifier baseline with case-insensitive lookup key |
| Integer and single-quoted string tokens | Local | Strings decode supported escapes; numeric range belongs to typing |
| Comments, punctuation, and operators | Local | Trivia omission and complete maximal-munch M1 inventory |
| `tokens <file.zen>` inspection | Local | Stable goldens and explicit diagnostic/usage exit behavior |
| Rendered lexical diagnostics | Local | Stable codes, labels, recovery, ordering, and Unicode-scalar locations |

## Parsed syntax surface

| Feature | Status | Notes |
|---|---|---|
| Apex-shaped class declarations | Local | Classes, inheritance clauses, fields, properties, methods, constructors, parameters, and selected modifiers |
| Parsed type syntax | Local | Source-faithful type names, nested generic arguments, and reserved nullable suffix |
| Statements and expressions | Local | Blocks, declarations, assignment, calls, member/index access, control flow, loops, precedence, conditional/nullable-type disambiguation, and the complete selected M2 operator surface |
| Immutable AST and visitor | Local | Read-only accessors, complete file-aware spans, and deterministic source-order traversal |
| Syntax recovery | Local | Partial trees plus stable declaration/member/statement diagnostics, bounded generic-closer recovery, and panic-safe rejection of incomplete token streams; semantic phases remain gated |
| `ast <file.zen>` inspection | Local | Stable goldens for both M2 acceptance fixtures and status 1/2 CLI behavior |

## Language surface

| Feature | Parse | Check | Emit | Status | Lowering | Target |
|---|---:|---:|---:|---|---|---|
| Apex-shaped classes and methods | Yes | No | No | Local | None | M2–M3 |
| Primitive expressions/control flow | Yes | No | No | Local | None | M2–M3 |
| Collections and ordinary calls | Yes | No | No | Local | None | M2–M3 |
| Case-insensitive resolution | No | No | No | Planned | None | M1–M3 |
| Handwritten Apex boundary declarations | No | No | No | Planned | None | M3 |
| Non-null and nullable types | Yes | No | No | Local | None | M2/M4 |
| Immutable `let` | No | No | No | Planned | TBD | M4 |
| Records/value types | No | No | No | Planned | TBD | M4 |
| Typed `Id<T>` | No | No | No | Planned | TBD | M4 |
| Sealed results/pattern matching | No | No | No | Planned | TBD | M4 |
| Query projection shapes | No | No | No | Planned | TBD | M5 |
| Schema-aware relationship types | No | No | No | Planned | TBD | M5 |
| Governor effects/contracts | No | No | No | Planned | None | M6 |
| Lambdas/collection pipelines | No | No | No | Planned | TBD | M7 |
| Custom generics | No | No | No | Planned | TBD | M7 |
| Typed record states and DML | No | No | No | Planned | TBD | M8 |
| Scoped security contexts | No | No | No | Planned | TBD | M8 |
| Bulk-first trigger changes | No | No | No | Planned | TBD | M9 |
| Durable async workflows | No | No | No | Planned | TBD | M11 |
| Modules and derivations | No | No | No | Planned | TBD | M12 |
| Imported Apex semantic API indexes | No | No | No | Planned | None | M12 |

Proposed examples in the vision and specifications do not change these rows.

## Apex target support

| Capability | Status | Target |
|---|---|---|
| Deterministic Apex text emission | Planned | M3 |
| SFDX `.cls` and `.cls-meta.xml` layout | Planned | M3 |
| Required target Salesforce API version | Planned | M3 |
| Generated-name collision checks | Planned | M3 |
| Zenith-to-Apex source maps | Planned | M3 |
| Pinned Apex Exec compile smoke evidence | Planned | M3 |
| Backend-neutral verification outcomes | Planned | M3 |
| Apex Exec generated-Apex checking | Planned | M3/M10 |
| SFDX `.trigger` and `.trigger-meta.xml` layout | Planned | M9 |
| Structured Apex Exec checking/testing adapter | Planned | M10 |
| Versioned Apex Exec capability protocol | Planned | M10 |
| Generated-Apex local test execution | Planned | M10 |
| Source-mapped stack frames and coverage | Planned | M10 |
| Salesforce validation | Planned | M10 |
| Multiple API-version compatibility profiles | Planned | M12 |
| Runtime helper library | Deferred | Only when a complete feature requires it |

## Test-generation support

| Capability | Status | Target |
|---|---|---|
| Authored Zenith tests | Planned | M10 |
| Deterministic schema/data fixtures | Planned | M10 |
| Stable branch-goal identities | Planned | M10 |
| Coverage-guided input generation | Planned | M10 |
| Managed tests from semantic or contract oracles | Planned | M10 |
| Editable test drafts for review | Planned | M10 |
| Opt-in characterization tests | Planned | M10 |
| Non-deployable local coverage probes | Planned | M10 |
| Assertion-free tests generated only to raise deployment coverage | Unsupported | Product policy |

No generated test or coverage capability is implemented. The oracle classes
and trust rules are defined in
[`docs/specifications/testing-and-test-generation.md`](specifications/testing-and-test-generation.md).

## Compatibility policy

- Parse, check, and emit columns may advance independently as milestones land.
  The overall row remains **Local** or **Experimental** until its documented
  accepted cases have complete checking and lowering.
- Recognized syntax without complete semantics is recorded as **Unsupported**
  and produces a dedicated diagnostic; parser acceptance alone is not a
  language compatibility claim.
- Compile-only restrictions may be listed as **Local**, but they cannot imply
  target-runtime behavior.
- Generated output changes require golden fixture updates and a compatibility
  note when users could observe the change.
- Promote behavior to **Verified** only after recording the Salesforce version,
  fixture, and result.
- Record every external verifier's version, capability profile, fixture, and
  result. Apex Exec evidence follows `docs/APEX_EXEC.md` and is never labeled
  Salesforce verification.
- If a feature cannot preserve its documented semantics in Apex, it remains
  unsupported or its contract must change explicitly.
- Local user tests exercise generated Apex. Direct Zenith HIR execution cannot
  establish target compatibility.
- Backend capability gaps produce **Unsupported** verification outcomes rather
  than Zenith compilation failures.
- Coverage and oracle provenance are reported separately. A generated case does
  not become trusted merely because it reaches code.

## Updating this document

Every change to accepted syntax, static guarantees, generated Apex, helper
requirements, or validation behavior must update the relevant row in the same
checkpoint.
