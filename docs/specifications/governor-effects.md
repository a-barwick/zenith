# Governor Effects

## Status

**Proposed for M6.** No effect syntax or analysis is implemented.

This document captures the design constraints that make governor effects more
than a linter rule. Exact syntax and summary algebra remain open until the
typed call graph exists.

## Goals

The effect system should:

- Identify SOQL, DML, callout, enqueue, and privilege-transition operations.
- Propagate effects through ordinary, virtual, generic, and generated calls.
- Make loop and recursion amplification visible.
- Check claimed resource contracts rather than trusting annotations.
- Explain the source call path that caused a contract failure.
- Remain conservative when external Apex behavior is unknown.

## Candidate source form

```zenith
fn loadAccounts(Set<Id<Account>> ids)
    -> List<Account{Id, Name}>
    effects { soql <= 1 }
{
    return query Account { Id, Name where Id in :ids };
}
```

The annotation is a checked upper bound, not a promise that suppresses
analysis.

## Initial effect domains

| Effect | Meaning |
|---|---|
| `soql` | Number of SOQL statements on a path |
| `queryRows` | Conservative row bound where it can be expressed |
| `dml` | Number of DML statements on a path |
| `dmlRows` | Conservative record count where it can be expressed |
| `callout` | External callout operations |
| `enqueue` | Asynchronous jobs scheduled |
| `systemAccess` | Entry into elevated object/field or sharing context |

CPU and heap may begin as coarse unknown/relative effects until useful static
bounds can be justified.

## Composition requirements

- Sequential effects add where both execute.
- Conditional effects use a conservative path maximum.
- A statically bounded loop multiplies its body summary.
- A data-dependent loop makes resource-counting body effects unbounded unless a
  stronger bulk rule applies.
- Recursion is unbounded unless a checked decreasing or explicit bounded model
  exists.
- Unknown external Apex effects remain unknown, not zero.

## Diagnostics

A failed contract should show:

1. the declared contract
2. the call or loop that amplified the effect
3. the nested source path to the resource operation
4. whether the result is known to exceed the limit or cannot be proven bounded

The diagnostic must originate from effect HIR, not from scanning emitted Apex.

## Non-goals

- Automatically hoist queries when doing so could change transaction behavior.
- Pretend every runtime-dependent row count has a precise static bound.
- Replace Salesforce runtime enforcement.
- Treat warnings as proof that a program is bulk-safe.
- Treat generated test coverage of a path as proof that its effect bound is
  safe for every runtime input.
