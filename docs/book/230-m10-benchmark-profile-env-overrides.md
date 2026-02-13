# 230 M10 Slice: Benchmark Profile Environment Overrides

This chapter documents configurable benchmark profile overrides for faster validation loops.

## What it is

Updated:
- `benchmark-suite/scripts/run_profile.sh`
- `benchmark-suite/scripts/test_run_profile.sh`
- `benchmark-suite/README.md`

`run_profile.sh` now supports environment-driven overrides for wrk2 profile parameters while preserving existing defaults.

## Why it exists

M10 workflows need both reproducible default profiles and short local validation runs. This slice enables quick iteration without editing scripts.

## How it works internally

`run_profile.sh` reads these optional environment variables:
- `BENCH_THREADS`
- `BENCH_CONNECTIONS`
- `BENCH_DURATION`
- `BENCH_TARGET` (global)
- `BENCH_TARGET_PING`
- `BENCH_TARGET_DECODE`
- `BENCH_TARGET_USERS_POST`

Resolution order for target RPS:
1. endpoint default,
2. endpoint-specific override,
3. global `BENCH_TARGET` override.

## Inputs, outputs, and constraints

- Inputs:
  - profile args (`impl`, `endpoint`, optional `base_url`),
  - optional env overrides above.
- Outputs:
  - unchanged raw + summary artifact paths,
  - wrk2 command adjusted by overrides.
- Constraints:
  - defaults remain stable when no env vars are set.

## Failure modes and diagnostics

- unsupported endpoint -> usage error.
- invalid wrk2 arguments from bad env values -> wrk2 failure propagated.

## Example usage

Default profile:

```bash
benchmark-suite/scripts/run_profile.sh ailang ping
```

Fast local profile:

```bash
BENCH_THREADS=2 \
BENCH_CONNECTIONS=16 \
BENCH_DURATION=7s \
BENCH_TARGET=1234 \
benchmark-suite/scripts/run_profile.sh --dry-run ailang decode
```

## Tradeoffs and next steps

- Tradeoff:
  - overrides are shell-env based; no persisted profile files yet.
- Next:
  - add named profile presets (for example `ci-fast`, `local-debug`, `full-report`) and stamp them into report metadata.
