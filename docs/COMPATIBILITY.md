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

## Tooling foundation

| Feature | Status | Lowering | Notes |
|---|---|---|---|
| `zenith --help` | Local | None | Bootstrap CLI only |
| `zenith --version` | Local | None | Prints the crate version |
| Source identities | Local | None | Stable within one compiler session |
| File-aware byte spans | Local | None | Line/column rendering planned in M1 |
| Phase-owned diagnostics | Local | None | Source through project phases represented |
| `.zen` compilation | Planned | Native/varies | Begins with M1 lexing; no source is accepted today |

## Language surface

| Feature | Parse | Check | Emit | Status | Target |
|---|---:|---:|---:|---|---|
| Apex-shaped classes and methods | No | No | No | Planned | M2–M3 |
| Primitive expressions/control flow | No | No | No | Planned | M2–M3 |
| Collections and ordinary calls | No | No | No | Planned | M2–M3 |
| Case-insensitive resolution | No | No | No | Planned | M1–M3 |
| Handwritten Apex boundary declarations | No | No | No | Planned | M3 |
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
| Imported Apex semantic API indexes | No | No | No | Planned | M12 |

Proposed examples in the vision and specifications do not change these rows.

## Apex target support

| Capability | Status | Target |
|---|---|---|
| Deterministic Apex text emission | Planned | M3 |
| SFDX class layout | Planned | M3 |
| Generated-name collision checks | Planned | M3 |
| Zenith-to-Apex source maps | Planned | M3 |
| Backend-neutral verification outcomes | Planned | M3 |
| Apex Exec generated-Apex checking | Planned | M3/M10 |
| Versioned Apex Exec capability protocol | Planned | M10 |
| Generated-Apex local test execution | Planned | M10 |
| Source-mapped stack frames and coverage | Planned | M10 |
| Salesforce validation | Planned | M10 |
| API-version compatibility profiles | Planned | M12 |
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
