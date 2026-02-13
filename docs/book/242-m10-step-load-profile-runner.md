# 242 M10 Slice: Step-Load Profile Runner

This chapter documents adding step-load benchmark execution to the M10 harness.

## What it is

Updated:
- `benchmark-suite/scripts/run_step_profile.sh`
- `benchmark-suite/scripts/test_run_step_profile.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added `run_step_profile.sh` for endpoint step-load execution,
- default per-endpoint rate ladders are built in,
- outputs per-step tagged raw/summary files plus one aggregated step summary bundle,
- added Make target `bench-step-profile`.

## Why it exists

M10 fairness guidance requires more than one fixed rate. Step-load runs help identify saturation knees and tail-latency behavior as load increases.

## How it works internally

1. Script selects default step rates by endpoint unless overridden.
2. For each rate:
   - runs `run_profile.sh` with `BENCH_TARGET=<rate>` and step duration,
   - copies raw/summary outputs to rate-tagged files.
3. Aggregates step summaries into:
   - `results/summaries/<impl>-<endpoint>-step.json`.

## Inputs, outputs, and constraints

- Inputs:
  - `impl`, `endpoint`, optional `base_url`,
  - env overrides:
    - `BENCH_STEP_RATES`
    - `BENCH_STEP_DURATION`
- Outputs:
  - `results/raw/<impl>-<endpoint>-r<rate>.txt`
  - `results/summaries/<impl>-<endpoint>-r<rate>.json`
  - `results/summaries/<impl>-<endpoint>-step.json`
- Constraints:
  - non-dry-run uses `run_profile.sh` dependencies (`wrk2`, etc.).

## Failure modes and diagnostics

- unsupported endpoint -> explicit usage failure.
- empty step-rate list -> explicit configuration failure.
- no generated step summaries -> explicit execution failure.

## Example usage

Default step profile:

```bash
make -C benchmark-suite bench-step-profile IMPL=ailang ENDPOINT=decode
```

Custom rates:

```bash
BENCH_STEP_RATES=500,1000,1500 BENCH_STEP_DURATION=20s make -C benchmark-suite bench-step-profile IMPL=node ENDPOINT=decode
```

## Tradeoffs and next steps

- Tradeoff:
  - step runs are slower than single-target profile runs.
- Next:
  - wire step-summary analysis into publish report for automatic knee-point commentary.
