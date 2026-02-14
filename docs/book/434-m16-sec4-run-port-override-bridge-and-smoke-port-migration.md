# 434 M16 Slice: sec4 run Port Override Bridge and Smoke Port Migration

This chapter documents adding `--port` to `sec4 run`, wiring runtime port override behavior, and migrating the operator smoke script away from source-level port patching.

## 1) What it is

New run-command option:

- `--port <N>`

Bridge behavior:

- `sec4 run --port <N>` sets `SEC4_RT_HTTP_PORT=<N>` for the runtime process.
- runtime `sec4_rt_http_serve(...)` now honors `SEC4_RT_HTTP_PORT` when present.

## 2) Why it exists

Before this slice, smoke automation picked a free port by copying and patching `src/main.ut` (`http.serve(8080, router)` replacement). That worked, but it coupled operator tooling to source rewriting.

Port override in `sec4 run` keeps smoke deterministic while removing source mutation and patch tooling complexity.

## 3) How it works internally

CLI (`compiler/sec4-cli/src/main.rs`):

1. `Commands::Run` gained `port: Option<u16>`.
2. Dispatch forwards `port` into `cmd_run(...)`.
3. `cmd_run(...)` sets child env `SEC4_RT_HTTP_PORT` when provided.

Runtime (`runtime/c/sec4_runtime.c`):

1. `sec4_rt_http_serve` now parses `SEC4_RT_HTTP_PORT` with fallback to compiled `http.serve(...)` argument.
2. Effective port is validated (`1..65535`) before socket bind.

Smoke script (`scripts/smoke-sec4-run-hello-api.sh`):

1. Keeps free-port selection via Python socket probe.
2. Stops patching source files.
3. Launches `sec4 run --port "${port}" --oneshot --serve-timeout-ms 12000`.

## 4) Inputs, outputs, constraints

Inputs:

- `sec4 run --port <N>`

Outputs:

- runtime bind target changes to override port while preserving route behavior.

Constraints:

- override uses runtime env bridge; invalid ports fail bind/start path deterministically.
- if `--port` is omitted, source-declared `http.serve(...)` port remains active.

## 5) Failure modes and diagnostics

- invalid `--port` value is rejected by CLI parsing.
- runtime bind failures still surface as run-command process failure.
- smoke script keeps request/response/log dumps for port or startup drift.

## 6) Example usage

```bash
sec4 run --path examples/hello-api --port 18080
```

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- runtime now has one more env-based override path (`SEC4_RT_HTTP_PORT`) via run-command bridge.
- direct binary execution still supports env override; this is intentional for backward compatibility.

Next steps:

- if run-command bridge stabilizes further, consider centralizing all recommended operator knobs as CLI flags and documenting env paths as advanced/legacy controls.

## Verification

- `cargo test -p sec4 --test json_output run_command_help_lists_runtime_bridge_flags`
- `cargo test -p sec4 --test json_output run_command_port_flag_overrides_http_serve_port_when_clang_available`
- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
