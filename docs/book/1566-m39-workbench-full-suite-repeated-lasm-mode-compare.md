# M39: Workbench Full Suite Repeated LASM Mode Compare

## What changed

The canonical workbench full-suite runner now knows how to execute repeated sequential LASM mode compares and publish that artifact alongside the normal fixed-target matrix, step matrix, and report outputs.

Files:

- `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
- `benchmark-suite/scripts/run_workbench_lasm_mode_compare_repeats.sh`
- `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`

## Why this mattered

The short 1-second LASM mode compare was useful for quick direction, but it was too noisy to treat as operator-grade evidence. We already had a repeated mode-compare runner, but it sat off to the side as a manual command.

That was the wrong place for it. The canonical Postgres workbench workload is the thing we are tuning, so the canonical full-suite runner should be able to emit the repeated LASM mode-compare artifact directly.

## What the full-suite runner does now

New options:

- `--lasm-mode-compare-repeats <n>`
- `--out-mode-compare-repeats <path>`

When repeats are enabled:

1. the normal workbench fixed-target matrix still runs,
2. the normal workbench step matrix still runs,
3. the LASM repeated sequential mode compare runs on the same endpoint set and DB/runtime knobs,
4. the aggregate repeated artifact is threaded into the full-suite run summary,
5. the report publish path receives that artifact as the mode-compare input.

The runner fails deterministically if repeats are requested without `sec4-lasm` present in `--impls`.

## Current repeated result

On the real Postgres explicit transaction endpoint:

- endpoint: `wb-tasks-with-comment-tx`
- artifact: `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`

Current medians:

- `single`: `347.595 req/s`, `p99 34.205ms`, `rss 45280 KB`
- `fixed`: `350.79 req/s`, `p99 32.945ms`, `rss 95408 KB`
- `proxy`: `351.205 req/s`, `p99 27.335ms`, `rss 102376 KB`

Current recommendation:

- `proxy`
- reason: highest median requests/sec in the repeated sequential artifact

## Why this is the right shape

This keeps the benchmark contract honest:

- one command can now produce the canonical DB-backed comparison artifacts,
- the noisy mode choice gets a repeated sequential version,
- the next runtime/scaling pass can be judged against a stable baseline instead of a single short burst.

It also keeps scope tight. This change does not invent a new benchmark system. It just moves the already-built repeated LASM compare into the normal operator path.
