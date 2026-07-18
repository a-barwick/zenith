# Typed Apex Baseline and Emission

## Status

**Implemented in M3.** This document defines the first complete
project-to-deployable-Apex slice. Syntax accepted by the M2 parser but not
listed here is rejected by checking with an explicit unsupported diagnostic.

## Project contract

A Zenith project is a directory containing `zenith.toml`. M3 accepts these
top-level string keys:

```toml
salesforce-api-version = "65.0"
source-root = "src"
output-root = ".zenith"
apex-boundary = "apex-boundary.api"
```

`salesforce-api-version` is required and must be an integer API version followed
by `.0`. The other keys are optional and default to the values shown, except
`apex-boundary`, which has no default. Unknown, duplicate, malformed, absolute,
or parent-traversing paths are project errors.

The compiler recursively discovers `.zen` files beneath `source-root`, sorts
them by normalized project-relative path, and requires at least one source
file. Symlinks are not followed. A class name must be unique across the whole
project under ASCII case-insensitive comparison. M3 requires exactly one
top-level class per source file and requires its spelling to match the file
stem case-insensitively.

`check`, `build`, and `emit` accept an optional project directory and otherwise
use the current directory:

```text
zenith check [project]
zenith build [project]
zenith emit [project]
```

`check` performs discovery, parsing, declaration collection, type checking,
lowering, and Apex IR validation without writing output. `emit` performs the
same work and prints every generated artifact in path order without writing.
`build` writes the artifact set beneath `output-root`.

## Handwritten Apex boundary summaries

When `apex-boundary` is configured, the referenced UTF-8 file contains
declaration-only summaries:

```text
class Audit {
  static void record(String message);
}
```

The M3 grammar is intentionally small:

```text
file      := class*
class     := "class" identifier "{" member* "}"
member    := ("static")? type identifier "(" parameters? ")" ";"
parameters := type identifier ("," type identifier)*
type      := identifier ("<" type ("," type)* ">")?
```

Boundary class and method names participate in the same case-insensitive
collision and lookup rules as Zenith declarations. Boundary methods may be
called but are never emitted. Their effects are recorded as unknown; M3 does
not infer that an external call is resource-free or security-safe.

## Checked type baseline

M3 supports these types:

- `void` for method returns only
- `Boolean`, `Integer`/`Int`, `Long`, `Decimal`, `Double`, `String`, and
  `Object`
- project and boundary class types
- `List<T>`, `Set<T>`, and `Map<K, V>`

Primitive aliases are canonicalized for type equality and emitted using
`Boolean`, `Integer`, `Long`, `Decimal`, `Double`, `String`, `Object`, and
`void`. Generic arity is checked. Nullable suffixes, inheritance,
interfaces, safe navigation, constructors, and user-defined generic types are
reserved for later milestones and rejected explicitly.

Each class may contain fields, automatic properties, and methods. M3 accepts
`public`, `private`, `protected`, `global`, `static`, `final`, `virtual`,
`override`, and one sharing modifier where Apex permits their direct emission.
Illegal, duplicate, or contradictory modifiers are rejected.

Names resolve case-insensitively. Local variables and parameters shadow fields;
duplicate names in the same scope are rejected. A bare field reference and
`this.field` select the same declared field. A class name may be used only as a
static member receiver.

Method calls are selected during checking, not emission. M3 supports exact
arity and exact parameter-type matching, with duplicate signatures rejected.
It supports project methods, summarized boundary methods, and these collection
members:

- `List<T>.size() -> Integer`, `isEmpty() -> Boolean`, `add(T) -> Boolean`,
  and `get(Integer) -> T`
- `Set<T>.size() -> Integer`, `isEmpty() -> Boolean`, `add(T) -> Boolean`,
  and `contains(T) -> Boolean`
- `Map<K, V>.size() -> Integer`, `isEmpty() -> Boolean`, `get(K) -> V`,
  `put(K, V) -> V`, and `containsKey(K) -> Boolean`

Indexing is supported for `List<T>[Integer] -> T` and `Map<K, V>[K] -> V`.

## Expressions and statements

M3 checks and emits integer, string, Boolean, and null literals; names; `this`;
parentheses; member access; calls; collection indexing; assignment; arithmetic,
comparison, equality, Boolean, and conditional operators; local declarations;
blocks; `if`/`else`; `while`; traditional and enhanced `for`; `return`;
`break`; `continue`; and empty statements.

Integer literals are range-checked as signed 32-bit values. `null` is accepted
for reference-like values in this Apex-compatible baseline; Zenith's stronger
nullability rules begin in M4. Conditions must be Boolean. Assignments and
returns require equal types, except `null` may flow to reference-like types.
Arithmetic requires equal numeric operands, relational operators require equal
numeric operands, `&&`/`||` require Boolean operands, and `+` additionally
supports two Strings.

Parsed constructs not covered above produce `type.unsupported-syntax`.
Unresolved names and members use `resolve.*` diagnostics; incompatible values
use `type.*` diagnostics. Semantic phases do not emit partial Apex after any
error.

## Typed HIR and Apex IR

Successful checking produces immutable typed HIR. Every expression records its
checked type and every name, field, index, and call records a selected target.
Lowering consumes those selections and never repeats name or overload
resolution.

Lowering produces a distinct Apex IR containing only validated target
constructs. The Apex emitter accepts Apex IR, not source AST or HIR. Generated
names use a reserved `Zenith$` prefix; M3 rejects user or boundary declarations
with that prefix case-insensitively even though the baseline does not yet need
helpers.

## Output contract

For each class `Name`, M3 emits:

```text
<output-root>/generated/main/default/classes/Name.cls
<output-root>/generated/main/default/classes/Name.cls-meta.xml
<output-root>/maps/Name.cls.map.json
```

It also emits `<output-root>/build.json`. Class text uses four-space
indentation, LF line endings, one declaration or statement per line, a final
newline, and stable source spelling. Metadata is:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ApexClass xmlns="http://soap.sforce.com/2006/04/metadata">
    <apiVersion>65.0</apiVersion>
    <status>Active</status>
</ApexClass>
```

Source maps use version `1` and contain sorted, non-overlapping generated byte
ranges paired with the originating project-relative Zenith path and source byte
range. Metadata has no Zenith source segments. `build.json` records format
version `1`, the target API version, source paths, generated artifact paths,
map paths, and any verification evidence.

Artifact paths and JSON keys have deterministic ordering. Repeated builds from
identical input are byte-identical.

## Verification outcomes

The M3 process adapter runs an explicitly selected generated-Apex compiler
executable against the generated classes directory. It records backend name,
revision, capability profile, exit status, and complete stdout/stderr.

The backend-neutral result is one of:

- `passed`: the backend declares the requested profile and exits successfully
- `failed`: it declares support and rejects the generated artifact
- `unsupported`: the executable is absent or the requested profile is not
  declared
- `internal-error`: the adapter cannot launch or observe the backend reliably

This narrow smoke result is optional evidence. It cannot change whether Zenith
checking or building succeeded, is never described as Salesforce verification,
and does not parse human-readable backend diagnostics.

## Acceptance fixture

`examples/m3-service` is the executable milestone fixture. It contains two
Zenith classes and one handwritten Apex boundary summary. It exercises
cross-file calls with different source casing, fields, an automatic property,
locals, `List<String>`, collection calls and indexing, conditional control
flow, static boundary calls, deterministic SFDX metadata, build manifest, and
source maps.

Positive tests compile and build the fixture through the public library and
CLI. Negative tests cover configuration, discovery, duplicate declarations,
file/class mismatch, modifiers, unsupported syntax, names, members, calls,
operators, assignment, conditions, returns, generated-name collisions, and
boundary summaries. Golden tests cover every emitted artifact, and repeated
build tests compare complete artifact bytes.
