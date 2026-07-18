#!/usr/bin/env bash
set -euo pipefail

expected_revision="1e4f1ca1938abfc996651ae447f227e0db680b6a"
checkout="${1:?usage: scripts/verify-apex-exec-m3.sh <pinned-apex-exec-checkout>}"
actual_revision="$(git -C "$checkout" rev-parse HEAD)"

if [[ "$actual_revision" != "$expected_revision" ]]; then
    echo "error: Apex Exec checkout is at $actual_revision, expected $expected_revision" >&2
    exit 2
fi

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --locked --manifest-path "$checkout/Cargo.toml"
output="$(
    cargo run --quiet --manifest-path "$repository_root/Cargo.toml" -- \
        build "$repository_root/examples/m3-service" \
        --verify-apex-exec "$checkout/target/debug/apex-exec" 2>&1
)"
printf '%s\n' "$output"

if [[ "$output" != *"Apex verification: passed"* ]]; then
    echo "error: pinned Apex Exec did not accept the generated M3 fixture" >&2
    exit 1
fi
