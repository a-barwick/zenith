# ADR 0003: Treat generated Apex as a product surface

**Status:** Accepted

**Date:** 2026-07-16

## Context

Generated Apex participates in Salesforce compilation, packaging, logs,
stack traces, code-size limits, security review, and production debugging. If
the output is unstable or unreadable, developers cannot audit privilege
transitions, understand failures, or review the practical cost of a Zenith
feature.

Treating text emission as an incidental backend detail would also make
regressions difficult to detect.

## Decision

Generated Apex is deterministic, formatted, readable, collision-safe, and
covered by golden tests. It is produced from a distinct valid Apex IR after
Zenith typing, effect analysis, and lowering are complete.

Builds also emit source maps and a manifest that identify generated helpers and
their semantic owners. The emitter does not resolve names, infer types, or
repair invalid lowering state.

## Consequences

- Output changes are reviewable and intentional.
- Identical inputs must produce byte-identical generated artifacts.
- Generated-name schemes and formatting become compatibility concerns.
- Golden tests supplement, but do not replace, behavioral verification.
- Some compiler refactors may require explicit output migrations even when
  Zenith source semantics do not change.
- The separate Apex IR adds implementation work but keeps target validity and
  formatting out of the source AST and type checker.
