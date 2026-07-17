# Vision

## Mission

Zenith is a safe, bulk-first language for Salesforce development. It extends
the useful shape of Apex with stronger types and platform-aware static analysis,
then compiles deterministically to readable, deployable Apex.

Salesforce remains the production runtime and final compatibility oracle.
Zenith should make the ordinary edit-check-test loop local and make classes of
Salesforce-specific mistakes fail before deployment.

## Why a language

Many serious Apex failures are not ordinary syntax errors:

- A field is accessed even though the SOQL projection did not select it.
- An SObject ID is passed to code expecting another object type.
- A nullable relationship is dereferenced.
- A query or DML operation is hidden beneath a loop or helper call.
- Elevated data escapes into a user-facing result.
- Partial DML failures are ignored.
- Trigger logic works for one record and fails for a bulk transaction.

Linters can identify some local patterns, and frameworks can standardize some
conventions. Neither can make these properties part of the program's types and
checked call graph. Zenith exists to do that.

## Dream-state experience

A developer creates ordinary `.zen` source beside Salesforce metadata:

```bash
zenith check
```

The compiler imports the project schema, validates types, query projections,
security contexts, and governor effects, and reports source-native diagnostics
without deploying.

```bash
zenith build
```

The compiler emits deterministic SFDX-compatible Apex, a build manifest, and
source maps under `.zenith/generated/`.

```bash
zenith test
```

Tests compile and run locally through a compatible execution backend where
possible. Generated Apex can also be checked through Apex Exec without making
it a production dependency.

```bash
zenith verify --target-org staging
```

The final gate deploys or validates generated Apex against Salesforce and maps
target diagnostics and test failures back to Zenith source.

## Product principles

1. **Ordinary Apex is the deployment artifact.** Zenith does not require a
   separate production VM.
2. **Platform constraints belong in the compiler.** SObject schema, bulk
   behavior, security context, and governor effects are core language concerns.
3. **Safety must survive lowering.** A feature is incomplete until its emitted
   Apex has defined, tested semantics.
4. **Generated output is inspectable.** Apex should be deterministic, readable,
   and traceable to its Zenith source.
5. **Bulk is the default unit of thought.** APIs and trigger constructs should
   make multi-record execution natural and per-record work conspicuous.
6. **Incremental adoption matters.** Projects should be able to begin with a
   small Zenith module and interoperate with existing Apex.
7. **Unsupported behavior is explicit.** The compiler never accepts syntax it
   cannot lower honestly.
8. **Local feedback is deterministic.** Identical source, schema, configuration,
   and compiler versions produce identical artifacts and diagnostics.
9. **Compatibility is measured.** Golden output, Apex Exec checks, and eventual
   Salesforce differential fixtures support every compatibility claim.

## Initial language pillars

### Safer values

- Non-null types by default with explicit nullable types
- Flow-sensitive null narrowing
- Immutable `let` bindings
- Records and sealed result types
- `Id<Account>`-style typed Salesforce IDs
- Typed `New<T>` and `Patch<T>` record states

### Safer data access

- Query-shaped SObject types that track selected fields
- Schema-aware relationship nullability
- Typed DML results and explicit partial-failure handling
- Scoped and auditable user/system access

### Safer resource use

- Inferred SOQL, DML, callout, enqueue, and privilege effects
- Call-graph propagation and loop amplification diagnostics
- Checked resource contracts such as `effects { soql <= 1 }`
- Bulk-first trigger change sets

### Better expression

- First-class functions and allocation-conscious collection pipelines
- Custom generics lowered by specialization where practical
- Exhaustive pattern matching
- Modules and safe compile-time derivations

## Non-goals

- Replace the Salesforce runtime in production.
- Reimplement every Salesforce API or metadata type.
- Hide governor limits behind implicit runtime work.
- Accept all Apex before delivering useful Zenith slices.
- Become a general-purpose programming language.
- Require generated Apex to be hand-edited.
- Preserve source-level compatibility when a Zenith feature has no honest Apex
  lowering.
- Claim Salesforce compatibility without executable evidence.

## Success measures

- A baseline Apex-shaped class compiles to deployable Apex with stable source
  maps.
- A selected-but-unqueried field access fails during Zenith checking.
- A query hidden under a loop or helper call produces a useful effect
  diagnostic.
- Typed IDs prevent cross-object mistakes without runtime wrappers.
- Generated Apex is stable enough for code review and reproducible CI builds.
- Existing Apex and generated Apex interoperate in an ordinary SFDX project.
- Most development checks run locally; org verification is a targeted final
  gate.
