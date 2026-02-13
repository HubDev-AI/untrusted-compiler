# 216 M10 Slice: Benchmark Profile Runner

This chapter documents the next M10 harness step: a single-command profile runner for raw+summary output flow.

## What it is

Added:
- `benchmark-suite/scripts/run_profile.sh`
- `benchmark-suite/scripts/test_run_profile.sh`

And wired Make targets:
- `make -C benchmark-suite bench-ping IMPL=<impl>`
- `make -C benchmark-suite bench-decode IMPL=<impl>`
- `make -C benchmark-suite bench-users IMPL=<impl>`
- `make -C benchmark-suite bench-users-get IMPL=<impl>`
- `make -C benchmark-suite bench-profile IMPL=<impl> ENDPOINT=<ping|decode|users-post|users-get>`

## Why it exists

Running load tests manually per endpoint is error-prone and inconsistent. The profile runner centralizes target rates, script paths, and output naming so benchmark runs stay reproducible.

## How it works internally

1. `run_profile.sh` maps endpoint -> fixed target profile:
   - `ping` -> `R10000`
   - `decode` -> `R2000`
   - `users-post` -> `R500`
2. Script selects load generator:
   - prefer `wrk2` (keeps constant-rate `-R` behavior),
   - fallback to `wrk` when `wrk2` is unavailable (warns and omits `-R`).
3. Script executes load generator with `--latency`, writes:
   - raw output: `results/raw/<impl>-<endpoint>.txt`
4. Script then invokes `wrk2_summary.sh` to emit:
   - summary output: `results/summaries/<impl>-<endpoint>.json`
5. `--dry-run` mode prints resolved command/paths without execution.

## Inputs, outputs, and constraints

- Inputs:
  - endpoint name (`ping|decode|users-post|users-get`),
  - implementation label (`IMPL`),
  - base URL (default `http://127.0.0.1:8080`).
- Outputs:
  - one raw txt + one summary json per profile run.
- Constraints:
  - requires `wrk2` or `wrk` for real execution.
  - service must be running and implement endpoint contract.

## Failure modes and diagnostics

- unsupported endpoint -> usage error and exit 2.
- missing load generator/service connectivity -> command failure from `wrk2`/`wrk`.
- summary parse failure -> `wrk2_summary.sh` failure.

## Example usage

Dry-run command resolution:

```bash
benchmark-suite/scripts/run_profile.sh --dry-run sec4 ping
```

Real profile run:

```bash
make -C benchmark-suite bench-profile IMPL=sec4 ENDPOINT=decode
```

## Tradeoffs and next steps

- Tradeoff:
  - fixed profile rates are intentionally static; hardware calibration tuning is a separate step.
- Next:
  - add optional fanout profile,
  - add consolidated per-run report bundling across multiple profile outputs.
