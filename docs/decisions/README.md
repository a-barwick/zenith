# Architecture Decision Records

Architecture Decision Records preserve the reason behind consequential or
expensive-to-reverse choices.

Use the next numeric filename and this structure:

```markdown
# ADR NNNN: Decision title

**Status:** Proposed | Accepted | Superseded
**Date:** YYYY-MM-DD

## Context

What forces or constraints require a decision?

## Decision

What was selected?

## Consequences

What becomes easier, harder, required, or deliberately deferred?
```

Do not create ADRs for routine implementation choices. If an accepted decision
changes, add a new ADR and mark the prior record superseded rather than
rewriting history.

## Index

- [0001 — Compile Zenith to ordinary Apex](0001-compile-to-apex.md)
- [0002 — Use case-insensitive semantic names](0002-case-insensitive-names.md)
- [0003 — Treat generated Apex as a product surface](0003-generated-apex-product-surface.md)
- [0004 — Integrate Apex Exec at the generated-Apex boundary](0004-apex-exec-verification-boundary.md)
- [0005 — Keep generated coverage distinct from trustworthy tests](0005-generated-tests-need-oracles.md)
