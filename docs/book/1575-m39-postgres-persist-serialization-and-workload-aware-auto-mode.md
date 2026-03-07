# M39: Postgres Persist Serialization And Workload-Aware Auto Mode

## What It Is

This slice closed two benchmark-stability bugs exposed by the canonical LASM/Postgres workbench run:

1. Postgres record-persist workers now serialize persistence per config key.
2. LASM benchmark `--lasm-mode auto` now falls back to `fixed` when its recommendation artifact was produced for a different workload.

## Why It Exists

The fresh alpha benchmark rerun exposed two distinct failures:

1. Background Postgres record compaction and append batches were running concurrently for the same DSN/config, producing `23505`, `55P03`, and `40P01` failures under load.
2. The benchmark runner reused a `proxy` recommendation from a mode-compare artifact built only for `wb-tasks-with-comment-tx`, then applied it to the full mixed workload, where `wb-tasks-list` became unstable.

Those were not acceptable alpha failures because they came from runtime/orchestration seams, not from the public app contract itself.

## How It Works Internally

### 1. Per-config Postgres persist serialization

`compiler/sec4-cli/src/lasm_db_runtime_postgres_persist.rs` now keeps a lock per Postgres persist config key.

Each persist worker still batches tasks, but once tasks are grouped by config key:

- only one worker at a time may run:
  - full-sync compaction
  - append batch
  - single append
- for that exact config key

That preserves background persistence while removing same-config races between:

- `persist_lasm_postgres_records_full_sync_thread_local(...)`
- `persist_lasm_postgres_record_append_batch_thread_local(...)`
- `persist_lasm_postgres_record_append_thread_local(...)`

### 2. Workload-aware auto-mode fallback

`benchmark-suite/scripts/run_workbench_benchmark_matrix.sh` used to resolve:

- `--lasm-mode auto`

only from:

- `.recommendation.mode`

inside the mode-compare artifact.

It now also checks:

- requested endpoint set
- artifact endpoint set
- LASM DB adapter

If the recommendation artifact does not match the requested workload, the runner prints a warning and falls back to:

- `fixed`

instead of silently reusing the wrong recommendation.

## Evidence

Observed root-cause evidence from the failing rerun:

1. Persist path failures disappeared after per-config serialization replaced same-config worker races.
2. `wb-tasks-list` with `--lasm-mode auto` now resolves to `cluster-fixed` when the available recommendation artifact only covers `wb-tasks-with-comment-tx`.

That turns the previous unstable auto-selected mixed-workload run into a stable benchmark path again.

## Tradeoffs

1. Per-config serialization lowers persistence concurrency for the same DSN/config.
   - This is intentional.
   - For the alpha workload, correctness and benchmark stability matter more than maximizing background record-store parallelism.
2. Auto-mode fallback is conservative.
   - It prefers a stable `fixed` topology over an invalid workload recommendation.
   - A future improvement can add per-workload or mixed-workload mode-compare artifacts instead of using fallback.
