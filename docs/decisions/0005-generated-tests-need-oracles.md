# ADR 0005: Keep generated coverage distinct from trustworthy tests

**Status:** Accepted

**Date:** 2026-07-16

## Context

Creating enough Apex tests and setup data to exercise branches is a major
development cost. Zenith's typed HIR, control-flow graph, schema knowledge,
effect summaries, and local coverage feedback can automate much of that work.

Code coverage alone does not establish correct behavior. A generated test that
only invokes code, or snapshots an accidental result, can raise coverage while
preserving a bug. Automatically deploying such tests as if they were authored
specifications would create false confidence.

## Decision

Zenith will support test generation, but every generated case carries
provenance, a coverage goal, and an oracle classification.

Generated output is divided into three categories:

1. **Compiler-owned semantic tests** have an oracle derived from language
   semantics, an explicit contract, an invariant, or an authored example. They
   may be regenerated deterministically and included in normal local and
   Salesforce verification.
2. **Editable test drafts** contain generated fixtures, inputs, mocks, and
   branch targets, but require the developer to complete or approve the
   behavioral assertion before they become trusted project tests.
3. **Characterization tests** record observed behavior. They are opt-in,
   review-required, and labeled so they cannot silently become normative.

Assertion-free coverage probes may guide local synthesis, but they are not
deployable tests by default and are never reported as correctness evidence.
Coverage and test-trust results remain separate in reports and build manifests.

Coverage-guided synthesis executes generated Apex through a compatible backend,
then maps branch feedback to Zenith control-flow identities. It does not execute
Zenith HIR as a substitute runtime. Candidate inputs must respect schema,
nullability, ID, security-context, and effect constraints, and generated tests
must not hide governor-risk paths behind unrealistic fixtures.

## Consequences

- Zenith can eliminate substantial test boilerplate without claiming that
  coverage equals correctness.
- Contracts, invariants, examples, and deterministic fixtures become valuable
  test-generation inputs as well as static-checking inputs.
- Generated tests and drafts need stable identities, source maps, manifests,
  and stale-output detection.
- Developers must review cases whose expected behavior cannot be derived
  independently.
- Local coverage feedback from Apex Exec can drive input generation, while
  Salesforce remains the final oracle for platform-sensitive cases.
- Test generation is a planned M10 product capability, not an implemented
  bootstrap feature.
