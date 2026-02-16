# M38-S114 CORS Private-Network Policy Materialization Bridge

## What it is

M38-S114 promotes private-network CORS preflight control from ad-hoc runtime env override to first-class compiler policy flow.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`

## Why it exists

M38-S113 added deterministic runtime support for explicit private-network preflight opt-in (`SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK`). This slice ensures project policy (`sec4.policy`) is the source of truth so operators do not need manual per-run env wiring.

## How it works

1. Policy model adds `cors.allow_private_network` with default `false`.
2. Policy parser ingests `[cors] allow_private_network = <bool>`.
3. `sec4 run` maps policy value to runtime env:
   - `SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK=1|0`
4. Runtime preflight behavior (from M38-S113) now automatically follows project policy:
   - policy disabled/default: deny private-network preflight (`403`),
   - policy enabled: allow and emit `Access-Control-Allow-Private-Network: true`.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_from_policy`
- `cargo test -p sec4 --test commands run_command_oneshot_allows_private_network_preflight_when_cors_policy_enables_it`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Adds one policy field that must be understood by operators reviewing CORS behavior.
- Reduces configuration drift by keeping runtime behavior policy-driven instead of shell-env-driven.

## Next

1. Continue CORS/runtime policy materialization hardening for remaining security-sensitive toggles.
2. Keep runtime tests and `sec4 run` policy bridge coverage aligned per slice.
