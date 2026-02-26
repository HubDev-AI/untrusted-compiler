# Workbench: Feature-Rich Cross-Backend Benchmark

This folder defines a prompt-first workflow for generating and benchmarking equivalent real-DB backends across implementations.

## Purpose

1. Keep one canonical app contract.
2. Generate equivalent services for each backend.
3. Benchmark all of them under identical load/profile settings.

## Layout

1. `spec/feature-app-v1.md`:
   - canonical API, DB schema usage, deterministic envelopes.
2. `prompts/generate-feature-app-v1.md`:
   - generation prompt used for each backend target.
3. `matrix.backends.json`:
   - implementation targets and expected service roots.

## Workflow

1. Freeze contract (`spec/feature-app-v1.md`).
2. Generate per-backend services from `prompts/generate-feature-app-v1.md`.
3. Validate parity against contract.
4. Run smoke parity matrix:
   - `make -C benchmark-suite workbench-smoke`
5. Run load benchmark matrix:
   - `make -C benchmark-suite workbench-bench-dry`
   - `make -C benchmark-suite workbench-bench`
   - optional `sec4-lasm` DB mode overrides:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode)
   - outputs:
     - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
     - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
     - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
     - `benchmark-suite/results/workbench-benchmark-report.md`
6. Run step-load benchmark matrix (knee detection):
   - `make -C benchmark-suite workbench-step-bench-dry`
   - `make -C benchmark-suite workbench-step-bench`
   - optional scope overrides:
     - `WORKBENCH_IMPLS=sec4,sec4-lasm`
     - `WORKBENCH_ENDPOINTS=wb-task-get,wb-tasks-list`
   - optional `sec4-lasm` DB mode overrides:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode)
   - outputs:
     - `benchmark-suite/results/summaries/workbench-step-runs.json`
     - `benchmark-suite/results/summaries/workbench-step-matrix.json`
     - `benchmark-suite/results/summaries/<impl>-<endpoint>-step.json`
     - `benchmark-suite/results/summaries/<impl>-<endpoint>-step-analysis.json`
7. Run full workbench benchmark suite (fixed-target + step-load + combined report):
   - `make -C benchmark-suite workbench-full-bench-dry`
   - `make -C benchmark-suite workbench-full-bench`
   - optional scope overrides:
     - `WORKBENCH_IMPLS=sec4,sec4-lasm`
     - `WORKBENCH_ENDPOINTS=wb-task-get,wb-tasks-list`
   - optional `sec4-lasm` DB mode overrides:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode)
   - outputs:
     - `benchmark-suite/results/summaries/workbench-full-runs.json`
     - `benchmark-suite/results/workbench-full-benchmark-report.md`

## Current implementation lanes

1. `sec4-lasm-workbench` - `implemented-alpha`
2. `node-workbench` - `implemented-alpha`
3. `go-workbench` - `implemented-alpha`
4. `rust-workbench` - `implemented-alpha`
5. `sec4-workbench` - `implemented-alpha`

## Constraints

1. No placeholder/stub behavior in DB paths.
2. Mutating endpoints require auth.
3. Error/success envelope contract must be deterministic across all implementations.
4. Alpha wire-format parity is enforced across lanes for benchmark calls:
   - `params`, `task_params`, `comment_params` JSON-array query payloads.
