# 1538 M39 Slice: Workbench Full-Suite Repeated-Run Wrapper

This slice adds a repeated-run orchestrator for the workbench full benchmark suite so benchmark evidence can be captured across multiple same-condition runs with deterministic artifact naming.

## What changed

1. Added repeated-run wrapper:
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh`
   - behavior:
     - runs `run_workbench_full_benchmark_suite.sh` N times (`--runs`, default `3`),
     - writes run-scoped artifacts for each run,
     - emits one aggregate run-manifest JSON.
2. Added make targets:
   - `make -C benchmark-suite workbench-full-bench-repeats-dry`
   - `make -C benchmark-suite workbench-full-bench-repeats`
3. Added deterministic repeated-run artifacts:
   - aggregate summary:
     - `benchmark-suite/results/summaries/workbench-full-benchmark-repeats.json`
   - run-scoped artifacts:
     - `benchmark-suite/results/summaries/workbench-full-benchmark-runs/`
4. Added repeat control knob:
   - `WORKBENCH_REPEAT_RUNS` (default `3`)
5. Updated operator docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`

## Why

Single benchmark runs can be noisy. The repeated wrapper standardizes batch execution while keeping artifact paths deterministic, making it easier to compare runs and hand off reproducible benchmark evidence.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh`
2. `make -C benchmark-suite workbench-full-bench-repeats-dry`
3. `BENCH_THREADS=2 BENCH_CONNECTIONS=16 BENCH_DURATION=2s BENCH_STEP_DURATION=2s BENCH_STEP_RATES=80,120 benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh --runs 2 --impls sec4 --endpoints wb-task-get --port 18098`
