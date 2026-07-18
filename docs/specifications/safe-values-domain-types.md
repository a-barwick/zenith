# Safe Values and Domain Types

## Status

**Implemented in M4.** This document defines the complete M4 source, checking,
and lowering contract. Anything not listed here remains unsupported.

## Top-level declarations

M4 keeps the one-declaration-per-file rule. A declaration name must match its
`.zen` file stem case-insensitively. In addition to M3 classes, a file may
contain one record, sealed result, or SObject domain declaration:

```text
record-declaration :=
    visibility? "record" identifier "(" record-component-list? ")" ";"

result-declaration :=
    visibility? "sealed" "result" identifier
    "{" result-variant* "}"

result-variant :=
    "case" identifier ("(" record-component-list? ")")? ";"

sobject-declaration :=
    visibility? "sobject" identifier ";"

record-component-list :=
    type identifier ("," type identifier)*

visibility := "public" | "global"
```

Names remain case-insensitive. Record component names and result variant names
must be unique within their declaration. SObject domain declarations are
compile-time nominal tags, not schema definitions: they declare no fields and
emit no Apex class. M5 will replace or validate these explicit tags using
normalized Salesforce schema input.

## Nullability

Every value type is non-null by default. `T?` explicitly admits `null`.
`void?`, nested nullable types such as `T??`, and nullable SObject domain tags
are invalid.

`null` is assignable only to a nullable type or erased `Object`. A nullable
value is not assignable to its non-null underlying type. The conditional
operator joins `T` and `T?` as `T?`. `left ?? right` requires a nullable left
operand and a right operand assignable to the non-null underlying type; its
result is non-null.

Because Apex default-initializes fields, automatic properties, and uninitialized
locals to null, M4 requires an initializer for every non-null class field and
ordinary local. M4 automatic properties must be nullable; later constructor
and accessor-body slices may establish non-null initialization through checked
control flow. Record components and result payload slots are initialized by
their compiler-generated constructors or factories.

Ordinary member access or method calls through `T?` produce
`type.nullable-dereference`. Safe navigation `value?.member` and
`value?.method(...)` requires a nullable receiver and yields a nullable result.
Safe navigation on a statically non-null receiver is rejected as unnecessary.
Assignment through safe navigation is not supported.

Nullable types lower to their ordinary Apex representation. Safe navigation
and null coalescing lower to Apex `?.` and `??` respectively.

Unchecked handwritten-Apex boundary methods conservatively return `T?` for
reference-like `T` because the boundary summary carries no trusted nullability.
`Map<K, V>.get` and `put` likewise return `V?`; callers must narrow or
coalesce the possibly absent value. Erased `Object` retains Apex boundary
behavior and may contain null.

## Flow-sensitive narrowing

The checker narrows local and parameter bindings for direct comparisons with
`null`:

```zenith
if (owner != null) {
    return owner.email;
}
```

The true and false branches receive the corresponding facts for `!=` and `==`.
The inverse fact remains in scope after a guard branch that is known to return.
Fields and properties are not narrowed because another call or alias may
mutate them. Assigning a mutable local invalidates its previous refinement and
then refines it to the assigned value when that value is known non-null.

## Immutable bindings

`let` introduces a local binding whose type is inferred from its required
initializer:

```text
let-declaration := "let" identifier "=" expression ";"
```

The inferred type cannot be `void`, `null` alone, a class-name reference, or an
SObject domain tag. A `let` binding cannot be assigned, incremented, or used as
an assignment target; violations produce `type.immutable-assignment`.
Immutability applies to the binding, not recursively to the referenced object.
It lowers to an ordinary Apex local because the restriction is enforced
statically.

## Records

A record is a nominal immutable value type. Its components are ordered and
readable but not assignable. Construction uses the existing `new` expression
and requires exact arity and assignable argument types:

```zenith
public record OwnerContact(Id<Contact> contactId, String email);

let owner = new OwnerContact(contactId, 'owner@example.com');
```

A record lowers to a final-field Apex class with a constructor in component
order. The generated fields and constructor preserve source spelling and use
`global` accessibility when the record is global, otherwise `public`.
Structural equality, hashing, inheritance, methods, default component values,
and record destructuring are not part of M4.

## Typed Salesforce IDs

`Id<T>` is valid only when `T` names an explicit SObject domain declaration.
Two typed IDs are assignable only when their domain names are equal
case-insensitively:

```zenith
public sobject Account;
public sobject Contact;

Id<Account> accountId = requested;
Id<Contact> contactId = requested; // type error
```

Erased `Id` remains available at explicit handwritten-Apex boundaries and is
assignable from any `Id<T>`. An erased `Id` is not implicitly assignable to a
typed ID. Both forms lower directly to Apex `Id`; no wrapper or runtime check
is generated.

## Sealed results and matching

A sealed result declares a closed set of named variants with zero or more
ordered payloads:

```zenith
public sealed result LookupResult {
    case Found(AccountSummary summary);
    case Missing(String reason);
}
```

Each variant is constructed through a generated static factory on the result
type, such as `LookupResult.Found(summary)`. A `match` is a statement:

```text
match-statement :=
    "match" "(" expression ")" "{"
    match-arm*
    "}"

match-arm :=
    "when" identifier ("(" identifier-list? ")")? block
```

The subject must be a non-null sealed-result value. Every declared variant must
appear exactly once, no unknown variant may appear, and the arm binding count
must equal the variant payload count. Arm bindings infer their payload types
and are immutable. A match is return-complete when every arm is
return-complete.

Sealed results lower to a generated Apex class containing an integer
discriminant, payload slots, a private constructor, and one static factory per
variant. Generated result members use `global` accessibility when the result
is global, otherwise `public`. A match evaluates its subject once into a
collision-safe generated local and lowers to an exhaustive `if`/`else if`
chain with typed payload bindings. No default branch is emitted because
checking proves exhaustiveness.

## Generated names and source maps

M4 helpers continue to use the case-insensitively reserved
`ZenithGenerated_` prefix. Match temporaries derive from stable source
identities and byte offsets. The prefix is rejected on user declarations,
members, parameters, locals, record components, result variants, payloads, and
match bindings. Result discriminants, payload slots, factories, record
constructors, and desugared match operations carry source-map segments pointing
to the record component, variant, match subject, or arm that owns their
semantics. Generated scaffolding never receives a fabricated Zenith span.

## Diagnostics

M4 adds these stable diagnostic codes:

- `resolve.duplicate-record-component`
- `resolve.duplicate-result-variant`
- `resolve.unknown-result-variant`
- `type.invalid-nullable-type`
- `type.uninitialized-non-null`
- `type.nullable-dereference`
- `type.invalid-safe-navigation`
- `type.invalid-null-coalescing`
- `type.immutable-assignment`
- `type.cannot-infer-let`
- `type.invalid-id-domain`
- `type.not-constructible`
- `type.non-exhaustive-match`
- `type.duplicate-match-arm`
- `type.invalid-match-subject`
- `type.match-binding-count`

As in M3, any diagnostic prevents lowering and emission.

## Acceptance fixture

`examples/m4-safe-values` is the executable M4 fixture. It declares Account and
Contact SObject domains, nullable relationship records, a sealed lookup result,
and a service that constructs records and results, compares typed IDs, narrows
nullable values, uses safe navigation and coalescing, binds immutable locals,
and exhaustively matches every result variant.

Positive tests cover the fixture through the public compiler and CLI, complete
golden output, source maps through generated declarations and match lowering,
and repeated-build determinism. Negative tests cover null misuse, invalid safe
navigation/coalescing, branch narrowing, immutable reassignment, record
construction and component mutation, invalid ID domains and cross-domain
assignment, unknown/duplicate/non-exhaustive result arms, payload binding
counts, nullable match subjects, declaration collisions, and generated-name
collisions.
