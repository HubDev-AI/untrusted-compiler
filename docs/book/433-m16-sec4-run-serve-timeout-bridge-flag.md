# 433 M16 Slice: sec4 run Serve-Timeout Bridge Flag

This chapter documents adding `--serve-timeout-ms` to `sec4 run` and migrating operator smoke flows to use the flag instead of direct timeout environment wiring.

## 1) What it is

`sec4 run` now accepts:

- `--serve-timeout-ms <N>`

It bridges to runtime env contract:

- `SEC4_RT_HTTP_SERVE_TIMEOUT_MS=<N>`

## 2) Why it exists

After `--oneshot` and `--max-body-bytes`, timeout remained the last operator-facing runtime knob that required direct env setup. Promoting it to a CLI flag keeps runtime control discoverable and consistent with the rest of run-command UX.

## 3) How it works internally

In `compiler/sec4-cli/src/main.rs`:

1. `Commands::Run` gained `serve_timeout_ms: Option<u64>`.
2. Command dispatch forwards timeout value into `cmd_run(...)`.
3. `cmd_run(...)` sets `SEC4_RT_HTTP_SERVE_TIMEOUT_MS` on the child process when provided.

Then integration and smoke layers were migrated:

- run-command e2e tests now pass timeout through `--serve-timeout-ms`.
- `scripts/smoke-sec4-run-hello-api.sh` now launches with:
  - `sec4 run --oneshot --serve-timeout-ms 12000`

## 4) Inputs, outputs, constraints

Inputs:

- `sec4 run --serve-timeout-ms <N>`

Outputs:

- same runtime execution behavior as before, with timeout sourced from CLI.

Constraints:

- timeout is optional; default runtime behavior remains unchanged when omitted.
- env-based timeout is still compatible for backward compatibility.

## 5) Failure modes and diagnostics

- non-numeric timeout input is rejected by CLI parsing.
- runtime early-exit and HTTP response failure paths remain guarded by existing smoke/e2e diagnostics.
- smoke script still fails fast with request/response context when runtime exits or contracts drift.

## 6) Example usage

```bash
sec4 run --path examples/hello-api --oneshot --serve-timeout-ms 12000
```

```bash
sec4 run --path examples/hello-api --oneshot --max-body-bytes 16384 --serve-timeout-ms 12000
```

## 7) Trade-offs and next steps

Trade-offs:

- CLI now exposes one more run-time bridge flag, increasing surface area slightly.
- preserving env compatibility means dual configuration paths remain available.

Next steps:

- if run-command bridge flags stabilize, consider documenting a preferred “flags-first” operator profile and limiting env use to advanced/legacy paths.

## Verification

- `cargo test -p sec4 --test json_output run_command_help_lists_runtime_bridge_flags`
- `cargo test -p sec4 --test json_output run_command_executes_compiled_binary_when_clang_available`
- `cargo test -p sec4 --test json_output run_command_executes_hello_api_example_when_clang_available`
- `cargo test -p sec4 --test json_output run_command_serves_http_route_in_oneshot_mode_when_clang_available`
- `cargo test -p sec4 --test json_output run_command_enforces_max_body_bytes_flag_when_clang_available`
- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
