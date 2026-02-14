# 426 M16 Slice: sec4 run Hello-API Operator Smoke Script

This chapter documents an operator-focused end-to-end smoke script for live `sec4 run` HTTP serving.

## What it is

A new executable smoke script:

- `scripts/smoke-sec4-run-hello-api.sh`

It validates runtime behavior by issuing real HTTP requests against a oneshot `sec4 run` process.
It also supports optional artifact export via `--artifacts-dir`.

## Why it exists

Rust integration tests already covered runtime paths, but operators needed one simple command that proves end-to-end serving works from the CLI in a realistic flow.

## How it works

1. Copies `examples/hello-api` to a temporary workspace.
2. Patches `http.serve(...)` to a dynamically chosen free port.
3. Runs `sec4 run` in oneshot mode and validates:
   - `GET /health` with auth header (`200`, body `ok`),
   - `POST /users` with auth + CSRF headers (`201`, standard success envelope JSON).
4. Cleans temporary workspace automatically.

## Inputs and constraints

- Requires local tools: `cargo`, `curl`, `jq`, `python3`.
- Defaults to `examples/hello-api`, overrideable via `--project`.
- Optional: `--artifacts-dir <path>` exports request/response logs for CI/debug analysis.
- Uses runtime env vars:
  - `SEC4_RT_HTTP_SERVE_MODE=oneshot`
  - `SEC4_RT_HTTP_SERVE_TIMEOUT_MS=12000`

## Failure modes and diagnostics

- If `sec4 run` exits before request handling, script prints run log and exits non-zero.
- If response status/body drifts from expected contract, script prints response headers/body and exits non-zero.
- If patching serve port fails, script exits with explicit patch error.

## Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh
```

## Tradeoffs and next steps

- Script prioritizes determinism and fast operator feedback over full matrix coverage.
- Future expansion can add explicit negative-path checks (missing auth, missing CSRF) as optional flags.
