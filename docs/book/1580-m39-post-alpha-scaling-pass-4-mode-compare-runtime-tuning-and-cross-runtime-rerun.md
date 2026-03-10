# M39: Post-Alpha Scaling Pass 4 (Mode Compare, Runtime Tuning, Cross-Runtime Rerun)

## What changed

1. Ran repeated LASM mode compare on the canonical six-endpoint Postgres workload (`repeats=2`) and published:
   - `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats-postgres-full-pass4.json`
2. Synced latest recommendation to the default mode-compare file:
   - `benchmark-suite/results/summaries/workbench-lasm-mode-compare-repeats.json`
3. Applied one runtime tuning delta:
   - `compiler/sec4-cli/src/lasm_cluster_runtime_config.rs`
   - auto proxy relay-worker sizing now keeps floor `3` for `instance_hint >= 4`.
4. Re-ran cross-runtime canonical matrix (`sec4-lasm,node,go,rust`) on the same DB-backed workload and refreshed canonical publication artifacts.

## Why

The pass closes the next post-alpha execution lane:
mode recommendation refresh -> one runtime tuning delta -> publishable cross-runtime rerun.

This keeps benchmark publication tied to the same workload contract and prevents stale mode recommendations from driving publication runs.

## Evidence

1. Mode compare recommendation:
   - `mode=proxy`
   - reason: `medianRequestsPerSec=5335.07`
2. Cross-runtime rerun totals:
   - `passed=4 failed=0 skipped=0`
3. Canonical publication files refreshed:
   - `benchmark-suite/results/workbench-benchmark-report.md`
   - `benchmark-suite/results/workbench-benchmark-report.html`
   - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
   - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`

## Notes

1. This pass is benchmark-contract and runtime-tuning focused; no language semantics changed.
2. Workbench workload parity is preserved across all compared runtimes in the rerun command family.
