# M39 - Benchmark Default Implementation Set Includes sec4-lasm

## Summary

Promoted `sec4-lasm` from opt-in to default in benchmark orchestration so LASM is exercised in normal benchmark runs without extra flags.

## What Changed

1. `benchmark-suite/Makefile`
   - Default `IMPLS` now includes `sec4-lasm`:
     - from `sec4,node,go,rust`
     - to `sec4,sec4-lasm,node,go,rust`

2. `benchmark-suite/scripts/preflight.sh`
   - Default implementation set now includes `sec4-lasm`.

3. `benchmark-suite/scripts/run_comparison_matrix.sh`
   - Default `--impls` list now includes `sec4-lasm`.

4. `benchmark-suite/scripts/run_step_matrix.sh`
   - Default `--impls` list now includes `sec4-lasm`.

5. `benchmark-suite/scripts/run_full_benchmark_suite.sh`
   - Default `--impls` list now includes `sec4-lasm`.

6. `benchmark-suite/README.md`
   - Updated workflow examples and default-run notes to include `sec4-lasm`.

## Why

M39-S2B requires LASM lane visibility in baseline concurrency/benchmark signals. Making LASM default removes drift risk where regular benchmark runs accidentally skip LASM.

## Validation

1. `bash benchmark-suite/scripts/test_preflight.sh`
2. `bash benchmark-suite/scripts/test_run_comparison_matrix.sh`
3. `bash benchmark-suite/scripts/test_run_step_matrix.sh`
4. `bash benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
