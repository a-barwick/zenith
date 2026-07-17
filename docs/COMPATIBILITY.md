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
| `zenith --help` | Local | None | Bootstrap CLI only |
| `zenith --version` | Local | None | Prints the crate version |
| Local and CI verification | Local | None | Rust 1.88.0 fmt/test/Clippy gates plus current-stable tests |
| Source identities | Local | None | Stable within one compiler session |
| File-aware byte spans | Local | None | Ordered construction and UTF-8-safe slicing; line/column rendering planned in M1 |
| Phase-owned diagnostics | Local | None | Source, language, schema, verification, and project phases represented |
| `.zen` compilation | Planned | TBD | Begins with M1 lexing; no source is accepted today |

## Lexical surface

| Feature | Status | Notes |
|---|---|---|
| UTF-8 source loading | Planned | M1 source diagnostics reject invalid UTF-8 without panicking |
| Apex-shaped words and reserved words | Planned | M1 contract is in `docs/specifications/lexical-structure.md` |
| Exact and canonical identifier spelling | Planned | ASCII identifier baseline with case-insensitive lookup key |
| Integer and single-quoted string tokens | Planned | Numeric type/range belongs to later typing |
| Comments, punctuation, and operators | Planned | Maximal-munch M1 inventory; tokenization is not parse acceptance |
| `tokens <file.zen>` inspection | Planned | Stable golden output and explicit diagnostic exit behavior |
| Rendered lexical diagnostics | Planned | Stable codes and Unicode-scalar line/column rendering |

## Language surface

| Feature | Parse | Check | Emit | Status | Lowering | Target |
|---|---:|---:|---:|---|---|---|
| Apex-shaped classes and methods | No | No | No | Planned | TBD | M2–M3 |
| Primitive expressions/control flow | No | No | No | Planned | TBD | M2–M3 |
| Collections and ordinary calls | No | No | No | Planned | TBD | M2–M3 |
| Case-insensitive resolution | No | No | No | Planned | None | M1–M3 |
| Non-null and nullable types | No | No | No | Planned | TBD | M4 |
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
| SFDX `.trigger` and `.trigger-meta.xml` layout | Planned | M9 |
| Structured Apex Exec checking/testing adapter | Planned | M10 |
| Salesforce validation | Planned | M10 |
| Multiple API-version compatibility profiles | Planned | M12 |
| Runtime helper library | Deferred | Only when a complete feature requires it |

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

## Updating this document

Every change to accepted syntax, static guarantees, generated Apex, helper
requirements, or validation behavior must update the relevant row in the same
checkpoint.
