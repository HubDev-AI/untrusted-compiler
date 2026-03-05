# 1537 M39 Slice: Workbench Full Benchmark Suite Runner

This slice adds a single workbench orchestrator that runs both fixed-target and step-load benchmark lanes, then publishes one combined report.

## What changed

1. Added full-suite workbench orchestrator:
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
   - runs:
     - `run_workbench_benchmark_matrix.sh` (fixed-target lane)
     - `run_workbench_step_matrix.sh` (step-load lane)
   - republishes combined markdown report using `publish_report.sh` with step-matrix input.
2. Added make targets:
   - `make -C benchmark-suite workbench-full-bench-dry`
   - `make -C benchmark-suite workbench-full-bench`
3. Added deterministic full-suite artifacts:
   - summary: `benchmark-suite/results/summaries/workbench-full-runs.json`
   - report: `benchmark-suite/results/workbench-full-benchmark-report.md`
4. Kept LASM workbench DB adapter controls aligned with other workbench runners:
   - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
   - `WORKBENCH_LASM_DB_BASE=<path>` (sqlite mode)
   - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=<path>` (postgres mode)
5. Updated operator docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`

## Why

Workbench benchmark lanes had separate commands for fixed-target and step-load runs. Operators needed manual chaining to produce one final report that included both throughput/latency matrix data and knee-signal step data. The full-suite runner removes that manual composition step and keeps artifact paths deterministic for CI/operator handoff.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
2. `make -C benchmark-suite workbench-full-bench-dry`
3. `BENCH_THREADS=2 BENCH_CONNECTIONS=16 BENCH_DURATION=2s BENCH_STEP_DURATION=2s BENCH_STEP_RATES=80,120 benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh --impls sec4 --endpoints wb-task-get --port 18097`
