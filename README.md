# Zenith

**Salesforce code, held to a safer course.**

Zenith is a bulk-first language that compiles to readable, deployable Apex. Its
Rust compiler is designed to catch null, query-shape, ID, security, and
governor-limit mistakes before deployment. Salesforce remains the production
runtime. No separate VM.

## Current Position

The lexer and parser are complete. Zenith accepts the selected Apex-shaped
baseline and produces immutable, source-spanned syntax with deterministic
diagnostics.

The next checkpoint adds cross-file checking and deterministic Apex emission.
`check`, `build`, and `emit` remain under active development.

## First Run

Zenith requires Rust 1.88 or newer.

```bash
cargo build
cargo test
cargo run -- tokens examples/hello.zen
cargo run -- ast examples/hello.zen
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
- [Current status](docs/STATUS.md) — shipped work and the active checkpoint
- [Compatibility](docs/COMPATIBILITY.md) — supported language surface
- [Development](docs/DEVELOPMENT.md) — build, test, and contribution workflow
- [Architecture](docs/ARCHITECTURE.md) — compiler and runtime boundaries

Zenith is independent software. It is not affiliated with or endorsed by
Salesforce.
