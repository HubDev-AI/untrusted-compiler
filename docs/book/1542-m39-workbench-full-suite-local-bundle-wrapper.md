# 1542 M39 Slice: Workbench Full-Suite Local Bundle Wrapper

This slice adds a one-command local-infra operator wrapper for workbench full-suite repeated runs plus markdown summary rendering.

## What changed

1. Added wrapper script:
   - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_bundle.sh`
2. Wrapper behavior:
   - delegates repeated suite execution to:
     - `run_workbench_full_benchmark_suite_local_repeats.sh`
   - uses deterministic output defaults:
     - summary JSON: `results/summaries/workbench-full-benchmark-repeats.json`
     - markdown report: `results/workbench-full-benchmark-repeats.md`
   - in non-dry-run mode, renders markdown automatically via:
     - `render_workbench_full_benchmark_suite_repeats_summary.sh`
   - in dry-run mode, prints the renderer command plan and skips rendering.
3. Added make targets:
   - `workbench-full-bench-local-bundle`
   - `workbench-full-bench-local-bundle-dry`
4. Added smoke test:
   - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_bundle.sh`
5. Wired smoke test into:
   - `benchmark-suite/Makefile` (`test-scripts`),
   - `.github/workflows/benchmark-smoke.yml`.

## Why

Operators already had local wrappers for full-suite and repeats, but still needed a second explicit step to render markdown summary. The bundle wrapper reduces that to one command while preserving dry-run transparency.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_bundle.sh`
2. `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_bundle.sh`
3. `benchmark-suite/scripts/test_makefile_profile_targets.sh`
4. `make -C benchmark-suite workbench-full-bench-local-bundle-dry WORKBENCH_IMPLS=sec4 WORKBENCH_ENDPOINTS=wb-task-get WORKBENCH_REPEAT_RUNS=1`
