# Zenith

**Salesforce code, held to a safer course.**

Zenith is a bulk-first language that compiles to readable, deployable Apex. Its
Rust compiler is designed to catch null, query-shape, ID, security, and
governor-limit mistakes before deployment. Salesforce remains the production
runtime. No separate VM.

## Current Position

The first deployable-language checkpoint is complete. Zenith now discovers
multi-file projects, resolves and type-checks a selected Apex-shaped service
baseline, and emits deterministic SFDX class source, metadata, manifests, and
source maps.

`check`, `build`, and `emit` are available now. The next checkpoint brings
stronger nullability and domain types; M3 remains deliberately smaller than
full Apex.

## First Run

Zenith requires Rust 1.88 or newer.

```bash
cargo build
cargo test
cargo run -- tokens examples/hello.zen
cargo run -- ast examples/hello.zen
cargo run -- check examples/m3-service
cargo run -- emit examples/m3-service
cargo run -- build examples/m3-service
```

## System Map

```text
Zenith source + Salesforce metadata + Apex API summaries
  → lex and parse
  → resolve and type-check
  → check schema, security, and governor effects
  → lower to Apex IR
  → emit SFDX source and source maps
  → verify generated Apex
```

Each phase owns one boundary. Unsupported behavior must fail explicitly.
Generated Apex is inspectable product output, not a hidden runtime detail.

## Bearings

- [Vision](docs/VISION.md) — purpose and operating principles
- [Roadmap](ROADMAP.md) — milestones and exit criteria
- [Current status](docs/STATUS.md) — shipped work and the next checkpoint
- [Compatibility](docs/COMPATIBILITY.md) — supported language surface
- [Development](docs/DEVELOPMENT.md) — build, test, and contribution workflow
- [Architecture](docs/ARCHITECTURE.md) — compiler and runtime boundaries

Zenith is independent software. It is not affiliated with or endorsed by
Salesforce.
