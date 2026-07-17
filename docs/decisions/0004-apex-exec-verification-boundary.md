# ADR 0004: Integrate Apex Exec at the generated-Apex boundary

**Status:** Accepted

**Date:** 2026-07-16

## Context

Zenith needs fast local validation and execution, while Apex Exec is building a
local Apex compiler and runtime. Directly sharing parser, AST, HIR, or runtime
internals would couple two languages and release schedules. Executing Zenith HIR
directly would also create a local semantics path that bypasses the Apex
lowering deployed to Salesforce.

Zenith must remain useful when Apex Exec is absent or does not yet support an
emitted Apex feature. At the same time, handwritten Apex in an incrementally
adopted project must be visible to Zenith resolution without requiring Zenith
to become a second complete Apex compiler.

## Decision

Zenith owns `.zen` parsing, semantic analysis, lowering, Apex IR, emission, and
source maps. Apex Exec is an optional verification backend that consumes the
generated SFDX-compatible Apex project. User-facing local execution always runs
the generated Apex, never Zenith AST or HIR.

The initial integration uses a versioned, machine-readable process protocol.
It exchanges generated project locations, target profiles, capability
information, structured diagnostics, test events, stack frames, and coverage.
A small shared protocol schema or data-only crate may follow, but Zenith does
not depend on Apex Exec compiler or runtime internals.

The adapter reports three distinct outcomes:

1. **Passed:** the backend claims the required capability and validation or
   execution succeeded.
2. **Unsupported:** the backend cannot validate part of the emitted surface;
   the Zenith build remains valid but requires another verification backend.
3. **Failed:** the backend claims support and reports a compile, test, or
   runtime failure.

Salesforce validation remains the final compatibility oracle.

Apex Exec may additionally export a versioned semantic API index for
handwritten Apex declarations. Zenith can consume that index for names, types,
members, overloads, and visibility. Zenith remains responsible for assigning
conservative effects and security assumptions to external calls.

## Consequences

- Local tests exercise the same Apex lowering that is deployed.
- `zenith check` and `zenith build` do not require Apex Exec.
- An Apex Exec compatibility gap cannot make otherwise valid Zenith source
  illegal.
- Generated-file diagnostics, stack frames, and coverage must be mapped back
  through Zenith source maps.
- Both teams can evolve independently behind a pinned protocol and declared
  capability profile.
- Cross-project conformance fixtures are required for every claimed integration
  capability.
- Sharing a neutral metadata catalog remains possible, but sharing compiler
  semantic representations requires a separate decision.
