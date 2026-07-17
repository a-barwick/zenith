# Language Foundations

## Status

Source identity and diagnostic phase types are **Implemented**. All source
language rules below are **Planned** unless marked otherwise.

## Source files

Zenith source uses the `.zen` extension. A future `zenith.toml` will define
project roots, schema inputs, target API compatibility, and generated output.

Generated Apex lives under `.zenith/` by default and is not hand-authored
source.

## Names

Zenith identifiers are case-insensitive because the Apex target is
case-insensitive. Two declarations that differ only in case conflict.

Each identifier retains:

- exact source spelling for diagnostics and readable emission
- a canonical lookup key
- its file-aware source span

Canonicalization is for lookup only. It must not rewrite source text stored in
syntax.

## Baseline syntax

The early compiler accepts a deliberately selected Apex-shaped baseline:

- classes, fields, properties, methods, and constructors
- primitive and collection types
- expressions, calls, member access, assignment, blocks, and common control
  flow
- ordinary access and static/instance modifiers

The goal is not immediate full Apex parsing. Unsupported syntax is diagnosed
explicitly.

## Nullability

**Proposed for M4.** Values are non-null by default. `T?` admits null:

```zenith
Account account;
Account? maybeAccount;
```

Control flow narrows nullable values after checks. A nullable value cannot be
used where `T` is required without narrowing, safe navigation, coalescing, or
an explicit checked conversion.

Lowering may use ordinary nullable Apex references, but every compiler-created
path must preserve the Zenith static guarantee.

## Immutable bindings

**Proposed for M4.** `let` introduces a binding that cannot be reassigned:

```zenith
let total = calculateTotal(lines);
```

Immutability applies to the binding. Collection and object mutability require
separate type rules; `let` does not silently deep-freeze Apex objects.

## Typed Salesforce IDs

**Proposed for M4.** `Id<Account>` and `Id<Contact>` are distinct static types:

```zenith
Id<Account> accountId = account.Id;
Id<Contact> contactId = contact.Id;
```

Both lower to Apex `Id` without allocating runtime wrappers. Explicit dynamic
or erased forms remain available where schema-polymorphic APIs require them.

## Query-shaped records

**Proposed for M5.** A static query produces a record type containing exactly
its selected fields and relationships:

```zenith
let accounts = query Account {
    Id,
    Name,
    Owner.Email
    where Id in :ids
};
```

Access to an unselected field is a type error even though the underlying Apex
value is an `Account`. Query shape is a compile-time property and does not
require a runtime wrapper.

## Generated Apex contract

Every accepted construct declares a lowering class in
`docs/COMPATIBILITY.md`. Generated Apex must be:

- syntactically valid for the selected target profile
- deterministic for identical compiler inputs
- readable enough to inspect during debugging
- collision-safe
- mapped back to Zenith source

If these requirements cannot be met, the construct remains unsupported.
