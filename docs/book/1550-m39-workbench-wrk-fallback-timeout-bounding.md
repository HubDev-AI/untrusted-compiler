# M39 - Workbench `wrk` Fallback Timeout Bounding

## What changed

- Updated:
  - `benchmark-suite/scripts/run_workbench_profile.sh`
  - `benchmark-suite/scripts/run_profile.sh`
- In `wrk` fallback mode (when `wrk2` is unavailable), benchmark commands now include:
  - `--timeout <value>`
  - default timeout: `10s`
  - override: `BENCH_WRK_FALLBACK_TIMEOUT`
- Fallback warning text now includes the effective timeout value.
- Raw benchmark artifact headers now include:
  - `# sec4-bench-wrk-fallback-timeout=<value>`

## Why it exists

Without explicit fallback timeout caps, stalled endpoints can inflate short benchmark windows into multi-minute runs, making step-matrix execution unstable and difficult to compare.

## Behavior contract

- `wrk2` mode is unchanged.
- `wrk` fallback remains non-constant-rate, but now has bounded request stall behavior.
- Operators can tune timeout using `BENCH_WRK_FALLBACK_TIMEOUT` for local environments with slower response paths.

## Operator docs

- `benchmark-suite/README.md`
- `examples/lasm-alpha-full/README.md`
