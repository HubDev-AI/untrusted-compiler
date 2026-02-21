# 1243 M39 Slice: LASM Capacity Probe Cluster-Status Resolved Fields

This slice makes LASM capacity probe summaries report resolved runtime relay/accept settings directly from cluster status snapshots.

## What changed

1. `run_lasm_cluster_capacity_probe.sh` now starts LASM cluster runs with:
   - `--cluster-status-json <raw status path>`
2. Probe summary JSON now includes resolved runtime values under `run`:
   - `clusterRelayWorkersResolved`
   - `clusterAcceptWorkersResolved`
   - `clusterRelayAcceptBatchMaxResolved`
   - `clusterRelayQueueCapacityResolved`
   - `clusterRelayQueueShardCapacityResolved`
3. Probe artifacts now include `clusterStatusJson` path in the output JSON.
4. Probe dry-run plan output now prints resolved status-json artifact path (`clusterStatusJson=...`).
5. Updated probe contract test to validate dry-run status-json path rendering.

## Why

Auto-mode tuning previously recorded only requested values (`auto`) but not the resolved runtime envelope. This made capacity artifacts harder to compare and slowed scaling iterations. Capturing resolved cluster status fields closes that observability gap.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
2. `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --project-path examples/lasm-alpha-full --request-path /health --request-header 'Authorization: Bearer token123' --duration 5s --threads 4 --connections 64 --target-requests 150000 --port 18096 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out results/summaries/sec4-lasm-cluster-capacity-probe-status-fields-smoke.json`
5. `jq '.run | {clusterRelayWorkersResolved,clusterAcceptWorkersResolved,clusterRelayAcceptBatchMaxResolved,clusterRelayQueueCapacityResolved,clusterRelayQueueShardCapacityResolved}' benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-status-fields-smoke.json`
