# M38-S118 HTTP Body/Timeout Policy Materialization Bridge

## What it is

M38-S118 closes the HTTP ingress policy bridge gap by persisting policy-driven HTTP limits in compiler policy and materializing them into runtime env keys through `sec4 run`.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`

## Why it exists

`sec4.policy` already exposed `[http]` controls (`max_body_bytes`, `default_timeout_ms`), but those values were not represented in the typed `Policy` model and were not forwarded to runtime by `sec4 run`. That created real policy drift: configured HTTP limits looked accepted but were not enforced.

## How it works

1. Added `HttpPolicyConfig` to the typed `Policy` model:
   - `max_body_bytes`
   - `default_timeout_ms`
2. Added parser ingestion/validation:
   - `http.max_body_bytes >= 1`
   - `http.default_timeout_ms >= 1`
3. Added policy-hash inclusion for `http` fields to keep contract hashes aligned with effective behavior.
4. `sec4 run` now exports HTTP bridge keys from policy by default:
   - `SEC4_RT_HTTP_MAX_BODY_BYTES`
   - `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`
5. Existing CLI flags still override policy values when explicitly provided:
   - `--max-body-bytes`
   - `--serve-timeout-ms`

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_http_body_limit_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Extends policy surface and hash coupling with runtime env bridge keys.
- Improves determinism and safety by making `[http]` values effective at runtime without requiring ad-hoc CLI flags.

## Next

1. Add explicit command e2e for policy-vs-CLI precedence on both HTTP knobs.
2. Add targeted timeout-materialization coverage to reduce regressions in oneshot serving behavior.
