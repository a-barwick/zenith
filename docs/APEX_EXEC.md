# Apex Exec Relationship

This document records how Zenith relates to the separate
[Apex Exec](https://github.com/a-barwick/apex-exec) project. It is comparison
and integration context, not a dependency declaration.

## Audited baseline

The repository was reviewed and the M3 generated-Apex smoke was run against
Apex Exec commit
[`1e4f1ca1938abfc996651ae447f227e0db680b6a`](https://github.com/a-barwick/apex-exec/tree/1e4f1ca1938abfc996651ae447f227e0db680b6a)
on 2026-07-17.

At that revision:

- formatting, 165 tests, and Clippy with warnings denied passed
- 14 real-world North Star lexer/parser goals remained intentionally ignored
- `apex-exec check examples/hello.zen` from Zenith's checkout reported `OK`
- Apex-shaped classes, project compilation, invocation, and a local test runner
  existed through M6
- M7 schema/storage boundaries were active, while SObjects, SOQL, SOSL, DML,
  triggers, sharing/security fidelity, and Salesforce differential verification
  were not implemented

These facts describe one revision. Re-run the comparison before relying on a
new Apex Exec capability.

## Recorded M3 smoke

The executable acceptance project is `examples/m3-service`. Zenith generates
`GreetingService.cls` and `MessageFormatter.cls` beside the configured
handwritten `Audit.cls`, then invokes the pinned backend against the emitted
SFDX project with capability profile `zenith-m3-apex-baseline`.

The reproducible command is:

```bash
scripts/verify-apex-exec-m3.sh /path/to/apex-exec-at-the-pinned-revision
```

The recorded result was:

```text
Apex verification: passed (apex-exec, revision 1e4f1ca1938abfc996651ae447f227e0db680b6a, profile zenith-m3-apex-baseline).
OK (3 classes, 3 source files)
Built 2 classes to examples/m3-service/.zenith.
```

This is local compiler-smoke evidence only. The pinned profile does not support
sharing modifiers, map bracket indexing, or the future Salesforce-specific
surface, and the result is not Salesforce verification.

## Different products

| Concern | Zenith | Apex Exec |
|---|---|---|
| Input | Selected Apex-compatible baseline plus Zenith constructs | Apex source |
| Production output | Readable deployable Apex | None; local interpreter/runtime |
| Core pipeline | Type/schema/effect checking, lowering, Apex IR, emission | Parsing, semantic HIR, tree-walking execution |
| Platform role | Compile-time Salesforce safety | Local platform/runtime emulation |
| Final oracle | Salesforce validation | Salesforce differential fixtures, eventually |

Zenith must not bend its source semantics to match Apex Exec's supported
surface. Apex Exec success is evidence that a generated artifact belongs to one
declared local capability profile, not proof of Salesforce compatibility.

## Lessons to reuse

The most valuable Apex Exec precedents are architectural and test-oriented:

- File-aware source identities must exist before project compilation. Apex Exec
  replaced a project-wide rebased span space with stable per-file `SourceId`
  values; Zenith starts with that model.
- Cached project paths should retain their source identity across reparses.
- Parsed syntax stays immutable while checked types and selected targets live in
  HIR-owned data.
- Runtime or lowering stages consume selected calls and members rather than
  repeating overload resolution.
- Normalized Salesforce schema stays independent from raw metadata, runtime
  values, and storage backends.
- Real-world corpora need immutable provenance, license texts, fingerprints,
  and phase-specific progress indicators that do not overstate compatibility.

Zenith should use a smaller lexical subset corpus and generated-Apex fixtures
rather than treating Apex Exec's full Apex North Star corpus as a Zenith source
acceptance requirement.

## Hard boundary

ADR 0004 establishes a process boundary. In particular, Zenith does not:

- add a path dependency on a sibling `../apex-exec` checkout
- import Apex Exec AST, HIR, type, span, diagnostic, schema, or runtime types
- parse human-readable Apex Exec stderr as a stable protocol
- let backend availability determine whether Zenith source typechecks
- copy Apex Exec code or fixtures without an explicit compatible license and
  preserved provenance

The two repositories may align concepts or collaborate on a structured
protocol, but each compiler remains independently buildable and testable.

## Integration stages

### M1-M2: reference only

Lexing and parsing are specified and tested independently. Apex Exec is not an
installed tool, crate dependency, or acceptance oracle.

### M3: implemented optional compile smoke evidence

After Zenith has checked, lowered, validated Apex IR, and emitted an isolated
SFDX project, `zenith build --verify-apex-exec <executable>` may invoke the
pinned Apex Exec revision for the supported baseline. The smoke adapter uses
exit status and preserves complete human output, but it does not parse or
source-map that output.

The result categories are `passed`, `failed`, `unsupported`, and
`internal-error`. Missing capability is `unsupported`, never success. The
backend revision and capability profile are recorded with the verification
result. `zenith check` and ordinary `zenith build` remain independent of Apex
Exec.

### M4-M9: capability-gated evidence

Only generated constructs declared by the configured backend profile are sent
to Apex Exec. Query, DML, security, trigger, or helper output outside that
profile is explicitly skipped as unsupported while golden output and Zenith's
own validation continue to run.

### M10: supported test/verification adapter

Formal source-mapped checking, testing, stacks, output, and coverage require a
versioned structured protocol. The protocol must report at least:

- protocol, tool, commit, and capability-profile versions
- operation and result category
- structured diagnostics with generated paths and byte spans
- test names, outcomes, stack frames, and output events
- coverage ranges where available

Zenith owns translation from generated spans to `.zen` spans through its build
manifest and source map. Until Apex Exec exposes such a protocol, M3 smoke
evidence stays deliberately narrow.

## Maintenance

When the integration contract or audited Apex Exec baseline changes:

1. pin the reviewed revision or release
2. run Apex Exec's required verification
3. update the capability statement above
4. update `docs/COMPATIBILITY.md` without promoting unsupported behavior
5. add or update a differential fixture rather than relying on prose alone
