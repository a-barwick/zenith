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

## Tooling foundation

| Feature | Status | Lowering | Notes |
|---|---|---|---|
| `zenith --help` | Local | None | Bootstrap CLI only |
| `zenith --version` | Local | None | Prints the crate version |
| Local and CI verification | Local | None | Rust 1.88.0 fmt/test/Clippy gates plus current-stable tests |
| Source identities | Local | None | Stable within one compiler session |
| File-aware byte spans | Local | None | Ordered construction and UTF-8-safe slicing; line/column rendering planned in M1 |
| Phase-owned diagnostics | Local | None | Source, language, schema, verification, and project phases represented |
| `.zen` compilation | Planned | Native/varies | Begins with M1 lexing; no source is accepted today |

## Language surface

| Feature | Parse | Check | Emit | Status | Target |
|---|---:|---:|---:|---|---|
| Apex-shaped classes and methods | No | No | No | Planned | M2–M3 |
| Primitive expressions/control flow | No | No | No | Planned | M2–M3 |
| Collections and ordinary calls | No | No | No | Planned | M2–M3 |
| Case-insensitive resolution | No | No | No | Planned | M1–M3 |
| Non-null and nullable types | No | No | No | Planned | M4 |
| Immutable `let` | No | No | No | Planned | M4 |
| Records/value types | No | No | No | Planned | M4 |
| Typed `Id<T>` | No | No | No | Planned | M4 |
| Sealed results/pattern matching | No | No | No | Planned | M4 |
| Query projection shapes | No | No | No | Planned | M5 |
| Schema-aware relationship types | No | No | No | Planned | M5 |
| Governor effects/contracts | No | No | No | Planned | M6 |
| Lambdas/collection pipelines | No | No | No | Planned | M7 |
| Custom generics | No | No | No | Planned | M7 |
| Typed record states and DML | No | No | No | Planned | M8 |
| Scoped security contexts | No | No | No | Planned | M8 |
| Bulk-first trigger changes | No | No | No | Planned | M9 |
| Durable async workflows | No | No | No | Planned | M11 |
| Modules and derivations | No | No | No | Planned | M12 |

Proposed examples in the vision and specifications do not change these rows.

## Apex target support

| Capability | Status | Target |
|---|---|---|
| Deterministic Apex text emission | Planned | M3 |
| SFDX class layout | Planned | M3 |
| Generated-name collision checks | Planned | M3 |
| Zenith-to-Apex source maps | Planned | M3 |
| Apex Exec local checking | Planned | M3/M10 |
| Salesforce validation | Planned | M10 |
| API-version compatibility profiles | Planned | M12 |
| Runtime helper library | Deferred | Only when a complete feature requires it |

## Compatibility policy

- A language row becomes implemented only when parsing, checking, lowering, and
  emission are complete for its documented cases.
- Compile-only restrictions may be listed as **Local**, but they cannot imply
  target-runtime behavior.
- Generated output changes require golden fixture updates and a compatibility
  note when users could observe the change.
- Promote behavior to **Verified** only after recording the Salesforce version,
  fixture, and result.
- If a feature cannot preserve its documented semantics in Apex, it remains
  unsupported or its contract must change explicitly.

## Updating this document

Every change to accepted syntax, static guarantees, generated Apex, helper
requirements, or validation behavior must update the relevant row in the same
checkpoint.
