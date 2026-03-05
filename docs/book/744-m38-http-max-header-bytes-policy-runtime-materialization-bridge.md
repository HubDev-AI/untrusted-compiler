# M38-S121 HTTP Max-Header-Bytes Policy/Runtime Materialization Bridge

## What it is

M38-S121 closes the HTTP ingress header-size bridge gap by persisting and materializing `http.max_header_bytes` from policy into runtime request handling.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`
- `runtime/c/sec4_runtime.c`

## Why it exists

`sec4.policy` already exposed `http.max_header_bytes`, but it was not persisted in the typed policy model, not bridged by `sec4 run`, and not enforced by runtime ingress logic. This made header-size policy values effectively inert.

M38-S121 makes that control real and test-locked.

## How it works

1. Extended `HttpPolicyConfig` with:
   - `max_header_bytes`
2. Added parser ingestion/validation:
   - `http.max_header_bytes >= 1`
3. `sec4 run` now exports:
   - `SEC4_RT_HTTP_MAX_HEADER_BYTES`
4. Runtime ingress now computes effective header cap from env (bounded by request-buffer size) and rejects oversized request headers deterministically with:
   - `HTTP/1.1 431 Request Header Fields Too Large`
   - body: `request headers too large`
5. Added command e2e to verify policy-driven header-size rejection in real oneshot flow.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_http_header_limit_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Runtime header-cap enforcement is bounded by fixed request buffer (`SEC4_RT_REQUEST_BUFFER_BYTES`), so very high policy values are clamped to that internal ceiling.
- Adds stricter ingress rejection behavior (`431`) which may surface previously tolerated oversized-header requests.

## Next

1. Bridge `http.max_multipart_bytes` into runtime materialization.
2. Add deterministic multipart-size rejection coverage in runtime/command e2e tests.
