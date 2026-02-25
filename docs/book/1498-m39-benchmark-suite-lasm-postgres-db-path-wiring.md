# M39 Slice: Benchmark Suite LASM Postgres DB-Path Wiring

Date: 2026-02-26  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Extended benchmark endpoint support in profile/step runners:
  - `db-hot-write`
  - `db-hot-write-tx`
  - `db-hot-query-one`
  - `db-records`
- Added LASM DB runtime wiring options to matrix/full orchestrators:
  - `--lasm-db-adapter records-log|sqlite|postgres`
  - `--lasm-db-postgres-dsn-file <path>`
- Wired sec4-lasm startup in matrix/step runners to forward adapter mode and Postgres DSN (file or env fallback).
- Added benchmark make entrypoints for repeatable LASM Postgres loops:
  - `bench-matrix-lasm-postgres`
  - `bench-matrix-lasm-postgres-dry`
  - `bench-full-lasm-postgres`
  - `bench-full-lasm-postgres-dry`
- Updated benchmark docs and dry-run contract tests for new endpoint/wiring surfaces.

## Why

- Postgres runtime implementation is now real and needs repeatable benchmark loops under the same orchestrator contracts.
- Existing matrix/full wrappers only targeted baseline endpoints and did not expose LASM adapter/DSN configuration.
- DB hot-path endpoints are needed for deterministic DB-path throughput/latency comparisons in CI/local runs.

## Behavioral contract

- Benchmark matrix/step/full dry-runs now emit explicit sec4-lasm adapter markers when LASM DB mode is configured.
- Postgres adapter mode requires a DSN source:
  - `--lasm-db-postgres-dsn-file`, or
  - `SEC4_RT_LASM_DB_POSTGRES_DSN`.
- DB hot-path endpoints are first-class benchmark endpoints across profile and step runners.

## Validation

- `benchmark-suite/scripts/test_run_profile.sh`
- `benchmark-suite/scripts/test_run_step_profile.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_step_matrix.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `make -C benchmark-suite bench-matrix-lasm-postgres-dry BENCH_LASM_DB_POSTGRES_DSN_FILE=<temp-file>`
