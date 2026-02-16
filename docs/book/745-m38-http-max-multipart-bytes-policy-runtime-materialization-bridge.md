# M38-S122 HTTP Max-Multipart-Bytes Policy/Runtime Materialization Bridge

## What it is

M38-S122 closes multipart ingress-size policy drift by persisting and materializing `http.max_multipart_bytes` and enforcing it in runtime ingress handling.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `runtime/c/sec4_runtime.c`

## Why it exists

`sec4.policy` already declared `http.max_multipart_bytes`, but runtime behavior did not consume or enforce it. As a result, multipart-size policy values were not effective in `sec4 run` execution.

## How it works

1. Extended `HttpPolicyConfig` with:
   - `max_multipart_bytes`
2. Added parser ingestion/validation:
   - `http.max_multipart_bytes >= 1`
3. `sec4 run` now exports:
   - `SEC4_RT_HTTP_MAX_MULTIPART_BYTES`
4. Runtime ingress now detects `multipart/form-data` requests and applies configured multipart cap before handler execution.
5. Oversized multipart payloads are rejected deterministically with:
   - `HTTP/1.1 413 Payload Too Large`
   - body: `multipart payload exceeds runtime limit`

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_http_multipart_limit_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Multipart-size rejection is currently ingress-level and content-type token based; it does not implement full multipart parser semantics.
- Behavior remains bounded by existing request buffer and body handling architecture.

## Next

1. Enforce generic ingress body-cap rejection for non-JSON paths as a first-class runtime contract.
2. Expand runtime/command coverage to lock that behavior across content-type variants.
