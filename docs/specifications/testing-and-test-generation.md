# Testing and Test Generation

## Status

**Proposed for M10.** No Zenith test syntax, local execution integration,
coverage mapping, or test generation is implemented.

## Goals

Zenith testing should:

- run the generated Apex artifact used for deployment
- map failures, stack frames, and coverage back to `.zen` source
- generate deterministic schema and data fixtures where constraints are known
- synthesize inputs that target uncovered statements and branches
- derive assertions from explicit contracts, invariants, examples, and
  compiler-defined semantics
- make platform-sensitive cases easy to select for Salesforce verification
- report coverage separately from the strength and provenance of test oracles

## Execution model

`zenith test` first checks and lowers the complete project. A compatible local
backend receives the generated SFDX Apex, not Zenith AST or HIR. Backend
diagnostics, runtime frames, and coverage locations refer to generated files;
Zenith maps them through `source-map.json` before rendering results.

Backend capability is explicit. If a generated construct is outside the local
backend's declared profile, the result is **unsupported locally**, not a Zenith
compile failure and not a passing test. Salesforce validation remains required
for compatibility-sensitive behavior.

## Test-generation inputs

The generator may consume:

- checked signatures and nullability
- sealed variants and exhaustive matches
- branch conditions and control-flow identities
- Salesforce schema, required fields, and relationship constraints
- checked governor and security effects
- authored examples and parameterized cases
- deterministic fixtures, clocks, IDs, callouts, and async policies
- future preconditions, postconditions, and type invariants
- source-mapped coverage from earlier generated-Apex executions

Unknown external Apex effects, dynamic schema behavior, and unsupported
platform APIs remain conservative constraints rather than guessed behavior.

## Oracle classes

Every generated case declares one oracle class:

| Class | Meaning | Default disposition |
|---|---|---|
| Semantic | Expected behavior follows from Zenith language semantics or compiler-owned lowering rules | Managed generated test |
| Contract | Assertions follow from an explicit developer-authored contract, invariant, or example | Managed generated test |
| Reviewable | Inputs and setup are generated, but the behavioral assertion needs developer review | Editable `.zen` draft |
| Characterization | Expected values were observed from an execution backend | Opt-in, review-required draft |
| Probe | Invokes a path only to collect coverage or runtime information | Local synthesis input only |

Coverage reached by any class can be displayed, but only semantic and contract
cases count as trusted generated tests without review.

## Synthesis loop

A planned coverage-guided loop is:

1. Build a control-flow and branch-goal plan from checked Zenith HIR.
2. Generate schema-valid fixtures and candidate inputs.
3. Lower candidates with the project and execute the resulting Apex locally.
4. Map coverage and failures to stable Zenith branch identities.
5. Retain and minimize candidates that add useful coverage.
6. Emit managed tests only when a valid oracle exists; otherwise emit an
   editable draft or keep the case as a local probe.
7. Select platform-sensitive cases for differential Salesforce verification.

The same seed, source, schema, configuration, and tool versions should produce
the same candidate order and managed output.

## Generated artifacts

The build manifest should distinguish:

- authored Zenith tests
- compiler-owned generated tests
- editable generated drafts
- characterization tests
- non-deployable coverage probes

Managed files are disposable and regenerated atomically. Editable drafts live
outside the managed build directory, are never overwritten after adoption, and
must receive stable test names and branch-goal comments.

## Safety requirements

- Generated tests must obey the same type, query-shape, security, and effect
  checks as authored Zenith code.
- A test must not gain trusted status merely because it increases coverage.
- Observed output is not silently promoted into an expected result.
- Generated setup must respect required fields, relationships, uniqueness, and
  bulk behavior.
- Local backend success is not labeled Salesforce verification.
- Test generation must not mutate developer-owned source files without an
  explicit generate or adopt action.

## Non-goals

- Promise complete path coverage for arbitrary programs.
- Infer business intent from implementation alone.
- Replace developer review for ambiguous behavior.
- Generate assertion-free tests solely to satisfy a deployment percentage.
- Treat one local runtime profile as proof of all Salesforce behavior.
