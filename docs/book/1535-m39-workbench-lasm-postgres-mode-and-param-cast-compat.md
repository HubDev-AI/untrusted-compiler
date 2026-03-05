# 1535 M39 Slice: Workbench LASM Postgres Mode + Param Cast Compatibility

This slice makes the workbench benchmark runner configurable for LASM DB adapter mode and removes a Postgres write-path incompatibility in workbench SQL templates.

## What changed

1. Added LASM DB mode controls to workbench benchmark orchestration:
   - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh` now accepts:
     - `--lasm-db-adapter sqlite|postgres`
     - `--lasm-db-base <path>` (sqlite mode)
     - `--lasm-postgres-dsn-file <path>` (postgres mode)
2. Wired make-level knobs for those flags:
   - `WORKBENCH_LASM_DB_ADAPTER`
   - `WORKBENCH_LASM_DB_BASE`
   - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE`
3. Fixed workbench SQL insert templates for cross-adapter numeric param compatibility:
   - `benchmark-suite/services/sec4-lasm-workbench/src/main.ut`
   - `benchmark-suite/services/sec4-workbench/src/main.ut`
   - task-insert paths now cast priority placeholder via `cast($5 as bigint)` so LASM Postgres parameter serialization no longer fails on write paths.
4. Updated benchmark docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`

## Why

Workbench benchmark orchestration needed explicit LASM adapter control so LASM could be benchmarked in sqlite or postgres modes on demand. During validation, LASM Postgres seed writes failed with deterministic `DB.EXEC_FAILED` serialization errors on the priority placeholder. Aligning SQL placeholder casts removed that incompatibility while preserving the same alpha array payload contract across lanes.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
2. `make -C benchmark-suite workbench-bench-dry WORKBENCH_LASM_DB_ADAPTER=postgres`
3. `BENCH_DURATION=5s BENCH_THREADS=2 BENCH_CONNECTIONS=16 benchmark-suite/scripts/run_workbench_benchmark_matrix.sh --impls sec4-lasm --endpoints wb-task-get --lasm-db-adapter postgres --port 18095`
