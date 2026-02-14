# 431 M16 Slice: sec4 run Runtime Bridge Flags

This chapter documents adding explicit runtime bridge flags to `sec4 run` so operators can trigger deterministic oneshot serving and request-size limits without hand-setting environment variables.

## 1) What it is

`sec4 run` now accepts two new optional flags:

- `--oneshot`
- `--max-body-bytes <N>`

These map to existing runtime env contracts:

- `--oneshot` -> `SEC4_RT_HTTP_SERVE_MODE=oneshot`
- `--max-body-bytes <N>` -> `SEC4_RT_HTTP_MAX_BODY_BYTES=<N>`

## 2) Why it exists

Before this slice, runtime smoke/e2e workflows depended on direct env setup. That worked, but it made operator usage less discoverable and more error-prone than the rest of CLI UX.

Adding explicit flags keeps runtime behavior deterministic while moving the common knobs into the command surface where `--help` can document them.

## 3) How it works internally

In `compiler/sec4-cli/src/main.rs`:

1. `Commands::Run` gained `oneshot: bool` and `max_body_bytes: Option<u64>` fields.
2. Command dispatch now forwards those values into `cmd_run(...)`.
3. `cmd_run(...)` creates the runtime process command and conditionally injects:
   - `SEC4_RT_HTTP_SERVE_MODE=oneshot` when `oneshot=true`
   - `SEC4_RT_HTTP_MAX_BODY_BYTES=<value>` when `max_body_bytes` is set
4. Binary execution flow remains unchanged otherwise.

## 4) Inputs, outputs, constraints

Inputs:

- CLI flags: `--oneshot`, `--max-body-bytes <N>`
- Project path via `--path` (existing behavior)

Outputs:

- Same runtime binary execution behavior as before, with optional env bridge values applied.

Constraints:

- Flags are additive and optional.
- Existing env-driven workflows remain valid and unchanged.

## 5) Failure modes and diagnostics

- Invalid numeric value for `--max-body-bytes` is rejected by CLI parsing.
- Runtime exits or connection failures still surface through existing run-command integration assertions.
- If body size exceeds configured cap, runtime remains deterministic (`413` + `LIMIT.BODY_BYTES`).

## 6) Example usage

```bash
# One request and exit (deterministic smoke/e2e flow)
sec4 run --path examples/hello-api --oneshot

# One request and stricter runtime body cap
sec4 run --path examples/hello-api --oneshot --max-body-bytes 16384
```

## 7) Trade-offs and next steps

Trade-offs:

- This is a thin bridge over runtime env settings, not a new runtime behavior.
- Keeping env compatibility means two configuration entrypoints remain available.

Next steps:

- optionally migrate operator smoke scripts to prefer CLI flags over direct env for serve mode,
- consider exposing additional runtime knobs as explicit flags only when they are stable contracts.

## Verification

- `cargo test -p sec4 --test json_output run_command_help_lists_runtime_bridge_flags`
- `cargo test -p sec4 --test json_output run_command_serves_http_route_in_oneshot_mode_when_clang_available`
- `cargo test -p sec4 --test json_output run_command_enforces_max_body_bytes_flag_when_clang_available`
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
