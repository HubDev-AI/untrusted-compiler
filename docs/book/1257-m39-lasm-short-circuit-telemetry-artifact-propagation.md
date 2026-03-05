# 1257 M39 Slice: LASM Short-Circuit Telemetry Artifact Propagation

This slice propagates relay saturation short-circuit telemetry from runtime status into benchmark artifacts and operator-facing summaries.

## What changed

1. Capacity probe summary enrichment:
   - `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh` now captures and persists:
     - `run.clusterRelayDispatchSaturationShortCircuitTotal`
     - `run.clusterRelayDispatchSaturationShortCircuitPerSec`
   - values are sourced from cluster status JSON when available (proxy mode).
2. Saturation matrix/analyzer propagation:
   - `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh` now forwards short-circuit telemetry into each run item.
   - `benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh` now validates and carries those fields in `rankedRuns`.
   - `benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh` now renders short-circuit values in ranked and verification sections.
3. Mode compare/report propagation:
   - `benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh` now includes short-circuit telemetry in `comparison.*` output.
   - `benchmark-suite/scripts/publish_report.sh` now renders short-circuit totals in `LASM Mode Comparison` report section.
4. Test fixtures/contracts updated:
   - `benchmark-suite/scripts/testdata/sample-mode-compare.json`
   - `benchmark-suite/scripts/test_publish_report.sh`
   - `benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`

## Why

The short-circuit optimization was measurable at runtime but not visible across benchmark summaries. Propagating it through probe/matrix/mode-compare/report artifacts gives one consistent view for tuning and regression tracking.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh benchmark-suite/scripts/run_lasm_cluster_mode_compare.sh benchmark-suite/scripts/publish_report.sh benchmark-suite/scripts/test_publish_report.sh benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`
2. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_analyze_lasm_cluster_saturation_boost_matrix.sh`
4. `benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`
5. `benchmark-suite/scripts/test_run_lasm_cluster_mode_compare.sh`
6. `benchmark-suite/scripts/test_publish_report.sh`
7. `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
