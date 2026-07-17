# ADR 0002: Use case-insensitive semantic names

**Status:** Accepted

**Date:** 2026-07-16

## Context

Apex identifiers are case-insensitive. If Zenith allowed case-sensitive
declarations, two distinct Zenith symbols could collapse to one Apex symbol
during emission. Name mangling could preserve some distinctions but would make
ordinary Apex interoperability surprising and complicate every public API.

Diagnostics and generated code still need the original spelling chosen by the
developer.

## Decision

Zenith names are case-insensitive at every semantic layer.

Each source identifier retains its exact spelling and span while exposing a
canonical lookup key. Duplicate and overload rules compare canonical names.
Emission preserves a stable source spelling where possible.

## Consequences

- Zenith cannot declare two symbols that differ only in letter case.
- Lookup behavior matches the target platform and existing Apex expectations.
- Every name-bearing compiler representation must distinguish source spelling
  from its canonical key.
- Canonicalization must be centralized and tested rather than repeated ad hoc.
- Diagnostics remain source-faithful even though semantic lookup ignores case.
