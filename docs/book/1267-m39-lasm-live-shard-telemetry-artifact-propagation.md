# 1267 M39 Slice: LASM Live-Shard Telemetry Artifact Propagation

This slice propagates LASM relay live-shard telemetry from runtime status into benchmark artifacts and operator-facing summaries.

## What changed

1. Capacity probe summary enrichment:
   - `run_lasm_cluster_capacity_probe.sh` now captures `relayLiveSenderCount` from cluster status JSON.
   - Probe summary output now includes `run.clusterRelayLiveSenderCountResolved`.
2. Saturation matrix propagation:
   - `run_lasm_cluster_saturation_boost_matrix.sh` now forwards `clusterRelayLiveSenderCountResolved` into each run row.
   - `analyze_lasm_cluster_saturation_boost_matrix.sh` validates and carries the field in `rankedRuns`.
   - `render_lasm_cluster_saturation_boost_summary.sh` now renders live-shard values in ranked/recommendation/verification output.
3. Mode compare/report propagation:
   - `run_lasm_cluster_mode_compare.sh` now emits relay live-shard proxy/fixed values and delta in `comparison`.
   - `publish_report.sh` now renders those live-shard values in `LASM Mode Comparison` section.
4. Fixtures/contracts updated:
   - `scripts/testdata/sample-mode-compare.json`
   - `scripts/test_publish_report.sh`
   - `scripts/test_render_lasm_cluster_saturation_boost_summary.sh`

## Why

Short-circuit counters are useful, but they do not directly show relay shard health. Propagating live-shard telemetry through artifacts makes degraded relay states visible during saturation tuning and mode comparison.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh benchmark-suite/scripts/publish_report.sh benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh benchmark-suite/scripts/test_publish_report.sh`
2. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_analyze_lasm_cluster_saturation_boost_matrix.sh`
4. `benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`
5. `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
6. `benchmark-suite/scripts/test_publish_report.sh`
7. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
