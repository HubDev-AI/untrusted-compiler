# M38-S117 CSRF Cookie/Header-Name Policy Materialization Bridge

## What it is

M38-S117 closes CSRF naming drift by making policy-configured CSRF token names first-class in compiler policy and forwarding them through `sec4 run` runtime env materialization.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`

## Why it exists

Runtime CSRF middleware already supports configurable token names (`SEC4_RT_CSRF_COOKIE_NAME`, `SEC4_RT_CSRF_HEADER_NAME`), but policy flow did not persist or materialize those values. This slice removes that gap so CSRF naming is policy-driven and deterministic.

## How it works

1. `CsrfPolicyConfig` now persists:
   - `cookie_name`
   - `header_name`
2. Policy parser ingests `csrf.cookie_name` / `csrf.header_name` and enforces non-empty values.
3. `sec4 run` now exports:
   - `SEC4_RT_CSRF_COOKIE_NAME`
   - `SEC4_RT_CSRF_HEADER_NAME`
4. Runtime CSRF middleware behavior in run-command flows now respects policy token names for double-submit validation.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_csrf_cookie_and_header_names_from_policy`
- `cargo test -p sec4 --test commands run_command_oneshot_disables_csrf_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Expands CSRF policy surface that must stay synchronized with runtime env bridge keys.
- Improves determinism by making CSRF token-name behavior explicitly policy-controlled.

## Next

1. Continue closing remaining policy-to-runtime env gaps in auth/transport surfaces.
2. Keep run-command integration tests as the contract for CSRF middleware policy materialization.
