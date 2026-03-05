# M38-S126 HTTP Max-Concurrency Runtime Fallback/Clamp Hardening

## What it is

M38-S126 adds deterministic runtime test coverage for fallback/clamp behavior around `SEC4_RT_HTTP_MAX_CONCURRENCY`, and extends CLI help contract coverage to include the new max-concurrency bridge flag.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

## Why it exists

After max-concurrency policy and CLI override wiring landed, the remaining risk was env-parse drift in runtime edge cases:

- invalid env token (non-numeric)
- empty env token
- over-cap numeric value

This slice locks those paths with deterministic e2e runtime coverage.

## How it works

1. Added shared runtime e2e helper:
   - Builds a minimal HTTP health route fixture (`c-bin`).
   - Runs the compiled binary in oneshot mode.
   - Injects optional `SEC4_RT_HTTP_MAX_CONCURRENCY` env values.
   - Asserts successful route response envelope.

2. Added three focused tests:
   - Invalid token fallback (`"invalid"`) -> route still serves `200`.
   - Empty token fallback (`""`) -> route still serves `200`.
   - Over-cap token clamp (`"999999999"`) -> route still serves `200` under bounded runtime behavior.

3. Updated CLI help contract check:
   - `run --help` assertions now include `--max-concurrency <MAX_CONCURRENCY>`.

## Validation

- `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- `cargo test -p sec4 --test json_output run_command_help_lists_runtime_bridge_flags`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- This slice hardens behavior through runtime e2e contracts; it does not alter queueing algorithm design.
- Over-cap clamp verification is behavior-based (successful runtime operation under oversized env), not direct internal-introspection of clamp value.

## Next

1. Add queue-boundary throttle-ordering coverage for over-cap accepted-connection pressure.
2. Lock deterministic `503` throttle response invariants (status/body/trace/header composition) under concurrency contention.
