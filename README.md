# Zenith

A safe, bulk-first language that compiles to readable Salesforce Apex,
implemented in Rust.

Zenith is designed around the mistakes Apex developers most need help
preventing: nullable values, unqueried fields, mixed SObject IDs, unsafe
privilege transitions, partial DML handling, and code paths that can exceed
governor limits. It keeps the Salesforce deployment model rather than
introducing a separate production runtime.

The repository has completed its lexical milestone. It now tokenizes the
selected Apex-shaped baseline with case-insensitive names, stable source spans,
recoverable diagnostics, and deterministic CLI output. Parsing and language
compilation are not implemented yet.

```console
$ cargo run -- --version
zenith 0.1.0

$ cargo run -- tokens examples/hello.zen
1:1..1:7	keyword(public)	"public"
1:8..1:13	keyword(class)	"class"
1:14..1:25	identifier(hellozenith)	"HelloZenith"
```

The intended workflow is:

```bash
zenith check
zenith build
zenith emit
zenith test
zenith verify --target-org staging
```

Those compiler commands are roadmap targets, not current functionality.

## Language direction

A future Zenith method may look like:

```zenith
record AccountSummary(Id<Account> id, String name);

fn loadAccounts(Set<Id<Account>> ids)
    -> List<AccountSummary>
    effects { soql <= 1 }
{
    return query Account { Id, Name where Id in :ids }
        .map(account => AccountSummary(account.Id, account.Name));
}
```

The compiler should understand which fields a query selected, which values can
be null, which SObject type an ID belongs to, and how SOQL/DML effects compose
through the call graph. It should lower those guarantees into deterministic,
human-readable Apex.

## Testing direction

Zenith is intended to automate a large part of Apex test creation. Typed
control flow, schema constraints, explicit contracts, examples, and local
coverage can drive deterministic fixture and input generation. Cases with a
trustworthy oracle can become managed generated tests; ambiguous cases become
editable drafts rather than assertion-free coverage theater.

Local execution runs the generated Apex through a compatible backend such as
Apex Exec and maps failures and coverage back to Zenith. Apex Exec is optional,
does not define Zenith semantics, and is not a production dependency.

## Architecture at a glance

```text
Zenith source + Salesforce metadata + handwritten-Apex API summaries
  → tokens
  → immutable syntax
  → name resolution and typed HIR
  → schema and governor-effect analysis
  → lowered Apex IR
  → generated SFDX source + source maps
  → optional Apex Exec verification of generated Apex
  → final Salesforce verification
```

The compiler phases stay independently testable. Generated Apex is a product
surface, not a disposable implementation detail. User-facing local tests do
not bypass lowering by executing Zenith HIR directly.

## Project documentation

- [Vision](docs/VISION.md) — product north star and non-goals
- [Roadmap](ROADMAP.md) — milestones and executable exit criteria
- [Current status](docs/STATUS.md) — immediate implementation handoff
- [Architecture](docs/ARCHITECTURE.md) — current and target compiler design
- [Compatibility](docs/COMPATIBILITY.md) — shipped language and lowering claims
- [Development](docs/DEVELOPMENT.md) — human and agentic working loop
- [Apex Exec relationship](docs/APEX_EXEC.md) — comparison and verification
  boundary
- [Decisions](docs/decisions/README.md) — durable architectural rationale
- [Specifications](docs/specifications/README.md) — intended language behavior

Zenith is an independent developer tool and is not affiliated with or endorsed
by Salesforce.
