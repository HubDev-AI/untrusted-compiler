# 435 M16 Slice: sec4 run Runtime-Flag Contract and Naming-Lock CI Coverage

This chapter documents adding dedicated script-level contracts for `sec4 run` runtime-flag bridging and wiring them into naming-lock CI.

## 1) What it is

New scripts:

- `scripts/test-sec4-run-runtime-flag-contract.sh`
- `scripts/test-sec4-run-runtime-flag-contract-guard.sh`

Workflow integration:

- `.github/workflows/naming-lock.yml` now executes both scripts.

## 2) Why it exists

Rust integration tests already validate run-command behavior, but naming-lock CI is optimized around fast script contracts. Adding run-flag contract scripts provides an additional deterministic guardrail for CLI source drift in PR checks.

## 3) How it works internally

Contract script validates source-level patterns in `compiler/sec4-cli/src/main.rs`:

- required run fields: `port`, `oneshot`, `max_body_bytes`, `serve_timeout_ms`
- required dispatch forwarding to `cmd_run(&path, port, oneshot, max_body_bytes, serve_timeout_ms)`
- required env bridge wiring:
  - `SEC4_RT_HTTP_PORT`
  - `SEC4_RT_HTTP_SERVE_MODE`
  - `SEC4_RT_HTTP_MAX_BODY_BYTES`
  - `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`

Guard script creates synthetic fixtures and checks deterministic failures for:

- missing port forwarding in run dispatch
- timeout env bridge name drift

## 4) Inputs, outputs, constraints

Inputs:

- optional `--cli <path>` for contract script fixture/testing overrides

Outputs:

- pass/fail shell exit status with actionable diagnostics

Constraints:

- regex/source contracts are intentionally structural, not semantic/compiled checks
- behavior contracts remain additionally covered by Rust integration tests

## 5) Failure modes and diagnostics

- missing required source pattern -> explicit “missing run-flag contract pattern (...)”
- forbidden legacy pattern -> explicit “forbidden run-flag contract pattern (...)”
- guard script expectations fail fast when fixture no longer triggers intended contract failure

## 6) Example usage

```bash
scripts/test-sec4-run-runtime-flag-contract.sh
scripts/test-sec4-run-runtime-flag-contract-guard.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- source-pattern contracts can be brittle if formatting/structure changes significantly.
- keeping a guard fixture reduces accidental false confidence and captures expected failure modes.

Next steps:

- if CLI surface grows further, group run-flag contracts under a consolidated CLI-surface contract suite with shared fixture helpers.

## Verification

- `scripts/test-sec4-run-runtime-flag-contract.sh`
- `scripts/test-sec4-run-runtime-flag-contract-guard.sh`
- `scripts/check-milestone-closure.sh --format json --fail-on-pending`
- `scripts/test-check-milestone-closure.sh`
