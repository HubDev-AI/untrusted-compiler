# 1250 M39 Slice: LASM Saturation Summary Relay Pump-Batch Columns

This slice extends saturation analysis/summary outputs so relay pump-batch requested/resolved values are visible in top-level operator artifacts.

## What changed

1. Analyzer propagation:
   - `analyze_lasm_cluster_saturation_boost_matrix.sh` now carries `clusterRelayPumpBatchMaxResolved` from matrix runs into `rankedRuns`.
2. Summary rendering:
   - `render_lasm_cluster_saturation_boost_summary.sh` now includes:
     - Probe profile line: requested `clusterRelayPumpBatchMax`.
     - Ranked-runs table column: `Pump Batch (resolved)`.
     - Recommendation line field: `resolvedPumpBatch`.
     - Verification section line: `Relay pump batch max (resolved)`.
3. Contract coverage:
   - `test_render_lasm_cluster_saturation_boost_summary.sh` fixtures/assertions now validate the new pump-batch fields in profile, ranked table, and verification sections.

## Why

Relay pump-batch is now part of the runtime tuning surface. Without first-class summary visibility, operators still need raw JSON inspection to compare saturation runs. This keeps recommended-step artifacts directly actionable from one markdown report.

## Validation

1. `bash -n benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`
2. `benchmark-suite/scripts/test_analyze_lasm_cluster_saturation_boost_matrix.sh`
3. `benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`
4. `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`
5. `benchmark-suite/scripts/test_publish_report.sh`
