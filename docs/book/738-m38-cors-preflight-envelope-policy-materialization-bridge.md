# M38-S115 CORS Preflight Envelope Policy Materialization Bridge

## What it is

M38-S115 closes the remaining CORS policy bridge gap by making preflight envelope fields first-class in compiler policy and forwarding them through `sec4 run` runtime env wiring.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`

## Why it exists

Before this slice, `sec4.policy` accepted CORS preflight envelope keys in TOML shape but the core policy model did not persist them, and `sec4 run` did not materialize them to runtime env. This created configuration drift between intended policy and effective runtime behavior.

## How it works

1. `CorsPolicyConfig` now persists:
   - `allowed_methods`
   - `allowed_headers`
   - `exposed_headers`
   - `max_age_seconds`
2. Parser ingestion now maps those values from `[cors]`.
3. `cors.max_age_seconds` is validated as `>= 1` (`P6003` on invalid input).
4. `sec4 run` exports deterministic env bridge keys:
   - `SEC4_RT_CORS_ALLOWED_METHODS`
   - `SEC4_RT_CORS_ALLOWED_HEADERS`
   - `SEC4_RT_CORS_EXPOSED_HEADERS`
   - `SEC4_RT_CORS_MAX_AGE_SECONDS`
5. Run-command e2e tests validate that runtime responses reflect policy values on both success and preflight paths.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_from_policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_preflight_methods_headers_and_max_age_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds more explicit CORS policy surface area that must stay synchronized with runtime env keys.
- Improves determinism and operator trust by removing silent policy-to-runtime gaps.

## Next

1. Continue materialization hardening for remaining runtime security knobs so policy is always the canonical source.
2. Keep run-command integration tests as the contract layer for policy bridge behavior.
