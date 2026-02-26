# 1539 M39 Slice: Workbench Full-Suite Repeats Stats + Report Renderer

This slice upgrades the repeated workbench full-suite lane with aggregate cross-run stats and a markdown renderer.

## What changed

1. Extended repeated-run summary generation:
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh`
   - now computes and persists:
     - `compareStats` (requests/sec, p99 ms, RSS stats grouped by impl+endpoint across runs)
     - `stepStats` (knee/achieved-ratio and step p99 stats grouped by impl+endpoint across runs)
2. Added markdown renderer:
   - `benchmark-suite/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh`
   - converts repeated summary JSON into markdown tables with:
     - run artifact index
     - aggregate compare stats
     - aggregate step stats
3. Added make target:
   - `make -C benchmark-suite workbench-full-bench-repeats-report`
4. Added output path defaults:
   - summary JSON: `benchmark-suite/results/summaries/workbench-full-benchmark-repeats.json`
   - markdown report: `benchmark-suite/results/workbench-full-benchmark-repeats.md`
5. Updated operator docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`

## Why

Run-manifest paths alone are not enough to reason about consistency across repeated runs. Aggregate grouped stats and a rendered markdown summary provide quick, deterministic evidence for whether throughput/latency/memory and step-knee signals are stable across run batches.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh`
2. `bash -n benchmark-suite/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh`
3. `make -C benchmark-suite workbench-full-bench-repeats-dry`
4. `BENCH_THREADS=2 BENCH_CONNECTIONS=16 BENCH_DURATION=2s BENCH_STEP_DURATION=2s BENCH_STEP_RATES=80,120 benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh --runs 2 --impls sec4 --endpoints wb-task-get --port 18098`
5. `make -C benchmark-suite workbench-full-bench-repeats-report`
