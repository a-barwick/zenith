# ADR 0004: Keep Apex Exec behind a verification process boundary

**Status:** Accepted

**Date:** 2026-07-16

## Context

Zenith will emit Apex that benefits from fast local checking and, later, local
test execution. Apex Exec is a related local Apex compiler/runtime, but its
supported language and platform surface is intentionally partial and evolves
independently. Its internal Rust types and human CLI output are not a stable
cross-project protocol.

Making Zenith depend on Apex Exec internals would couple source semantics,
release cadence, diagnostics, schema models, and build reproducibility to a
sibling project. Treating backend success as Salesforce compatibility would
also overstate the available evidence.

## Decision

Apex Exec is an optional, capability-gated verification backend behind a
process boundary.

- Zenith performs source checking, lowering, Apex IR validation, and emission
  independently.
- M3 may pin a narrow generated-Apex compile smoke check without making Apex
  Exec a required user installation.
- M10 requires a versioned structured protocol before source-mapped checking,
  tests, stacks, output, or coverage become a supported integration.
- Backend results distinguish success, failure, unsupported capability, and
  internal error.
- Zenith records the backend version and capability profile and owns mapping
  generated spans back to Zenith source.
- Salesforce remains the final compatibility oracle.

Zenith does not add sibling path dependencies or import Apex Exec compiler or
runtime representations.

## Consequences

- Both repositories remain independently buildable, testable, and releasable.
- Apex Exec can improve local feedback without defining Zenith semantics.
- Early M3 evidence is intentionally narrower than the eventual M10 adapter.
- A structured protocol must be designed or added upstream before rich
  integration is reliable.
- Unsupported backend coverage remains visible rather than weakening Zenith's
  checks or being reported as success.
