# 232 M10 Slice: Orchestrator Readiness and Tooling Hardening

This chapter documents fixes for orchestrator hangs and environment-preflight issues discovered during end-to-end benchmark execution.

## What it is

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/run_profile.sh`
- `benchmark-suite/services/ailang/smoke.sh`
- `benchmark-suite/services/c/smoke.sh`
- `benchmark-suite/README.md`

Key hardening changes:
- fixed orchestrator startup hang caused by PID capture via command substitution,
- added configurable orchestrator port (`BENCH_PORT`, default `18085`),
- tightened readiness checks to require `/ping` response body equals `ok`,
- added per-service log files in `results/raw/*-service.log` and failure tail output,
- added explicit `wrk2` preflight in non-dry-run profile execution,
- hardened AILang/C smoke readiness checks and moved default smoke ports off `8080`.

## Why it exists

The previous runner could appear "stuck" when service startup used command substitution around long-running processes. Environment port collisions and missing `wrk2` also produced ambiguous behavior. This slice makes failures deterministic and debuggable.

## How it works internally

1. Service startup now sets `service_pid` directly (no `pid="$(...)"` command substitution on long-lived processes).
2. Service stdout/stderr is redirected to `results/raw/<impl>-service.log`.
3. Readiness loop validates both:
   - process is still alive,
   - `GET /ping` returns exact body `ok`.
4. On readiness failure, orchestrator prints log tail for the failing implementation.
5. `run_profile.sh` fails fast with a clear message if `wrk2` is missing.
6. AILang/C smoke scripts use dedicated default ports (`18084` and `18083`) and robust readiness state tracking.

## Inputs, outputs, and constraints

- Inputs:
  - benchmark orchestrator/profile args,
  - env overrides (`BENCH_PORT`, smoke port overrides).
- Outputs:
  - deterministic early errors for missing tooling/failed readiness,
  - per-service startup logs.
- Constraints:
  - real benchmark runs still require `wrk2` installed locally.

## Failure modes and diagnostics

- missing `wrk2` -> explicit profile error (`wrk2 is required but was not found in PATH`).
- service startup/readiness failure -> explicit orchestrator error with service log tail.
- incorrect `/ping` responder on target port -> readiness rejected (requires exact body `ok`).

## Example usage

Dry run:

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --dry-run
```

Run on custom port:

```bash
BENCH_PORT=19085 benchmark-suite/scripts/run_comparison_matrix.sh --impls ailang,node
```

## Tradeoffs and next steps

- Tradeoff:
  - orchestrator now writes extra startup logs, increasing artifact footprint.
- Next:
  - add explicit wrk2-install hint text per platform,
  - add optional startup timeout policy overrides,
  - surface readiness diagnostics in markdown report metadata.
