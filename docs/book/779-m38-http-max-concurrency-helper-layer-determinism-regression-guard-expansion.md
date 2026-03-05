# M38-S152 HTTP Max-Concurrency Helper-Layer Determinism Regression Guard Expansion

## What it is

M38-S152 expands helper-layer coverage for deterministic max-concurrency contention guards.

## Why it exists

To prevent silent drift in contention test behavior when setup/assertion logic is refactored.

## How it works

- Canonical helper functions now own connect/retry, I/O, wait, and contract predicate paths used by contention tests.
- Queue, late-connection, burst-ingress, and security-header parity tests use the shared helper surface.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

## Next

- Normalize connector-thread spawn/join envelopes (`M38-S162`).
