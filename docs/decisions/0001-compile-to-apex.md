# ADR 0001: Compile Zenith to ordinary Apex

**Status:** Accepted

**Date:** 2026-07-16

## Context

Zenith adds guarantees and expression that Apex does not provide directly, but
Salesforce remains the required production platform. A custom VM or managed
runtime would add deployment, security review, packaging, observability, and
operational costs before the language delivers value.

Existing Salesforce projects also need incremental adoption and direct
interoperation with hand-written Apex.

## Decision

Zenith's production artifact is ordinary, SFDX-compatible Apex plus source maps
and a deterministic build manifest.

The compiler may use native syntax, local desugaring, or generated helper
declarations. A runtime helper is permitted only when it is explicit, versioned,
and justified by a complete feature.

Apex Exec may later validate and execute supported generated code locally, but
it is neither the production runtime nor a semantic fallback for incomplete
lowering. Salesforce is the final compatibility oracle.

## Consequences

- Zenith can be introduced into existing SFDX projects one module at a time.
- Generated declarations must obey Apex visibility, naming, size, and governor
  constraints.
- Every language feature requires a semantics-preserving Apex lowering before
  it is considered implemented.
- Some otherwise attractive features will remain unsupported because their
  production cost or semantics cannot be made honest.
- Source maps and generated-code diagnostics are core compiler responsibilities.
- Production debugging can inspect familiar Apex rather than opaque bytecode.
