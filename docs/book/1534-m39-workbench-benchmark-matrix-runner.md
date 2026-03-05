# 1534 M39 Slice: Workbench Benchmark Matrix Runner

This slice adds a dedicated load-benchmark execution path for workbench lanes, separate from smoke-only parity checks.

## What changed

1. Added workbench endpoint load profiles:
   - `benchmark-suite/load/wrk2/post_wb_tasks.lua`
   - `benchmark-suite/load/wrk2/post_wb_tasks_with_comment.lua`
   - `benchmark-suite/load/wrk2/post_wb_task_comment.lua`
   - `benchmark-suite/load/wrk2/get_wb_task.lua`
   - `benchmark-suite/load/wrk2/get_wb_tasks_list.lua`
2. Added profile runner for workbench endpoints:
   - `benchmark-suite/scripts/run_workbench_profile.sh`
   - supports:
     - `wb-tasks-post`
     - `wb-tasks-with-comment`
     - `wb-task-comment-post`
     - `wb-task-get`
     - `wb-tasks-list`
3. Added full workbench matrix orchestrator:
   - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
   - behavior:
     - reads `benchmark-suite/workbench/matrix.backends.json`,
     - starts each implemented workbench service lane,
     - executes `/wb/setup` + deterministic seed task/comment creation,
     - runs endpoint profiles with `wrk2`/`wrk`,
     - emits per-impl summaries/reports and cross-impl compare/analysis/report artifacts.
4. Added make targets:
   - `make -C benchmark-suite workbench-bench-dry`
   - `make -C benchmark-suite workbench-bench`
5. Updated benchmark docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`

## Why

Workbench lanes were smoke-validated but had no dedicated comparable load runner under the workbench contract. This slice closes that gap so prompt-generated feature-app lanes can be benchmarked under one deterministic orchestration path.

## Validation

1. `make -C benchmark-suite workbench-bench-dry`
2. `BENCH_DURATION=5s BENCH_THREADS=2 BENCH_CONNECTIONS=16 benchmark-suite/scripts/run_workbench_benchmark_matrix.sh --impls sec4 --endpoints wb-task-get --port 18094`
