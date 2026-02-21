# 1245 M39 Slice: LASM Capacity Probe Status Artifact Cleanup Default

This slice prevents default LASM capacity probe runs from leaving raw status-json files behind in the repo worktree.

## What changed

1. Added new probe flag:
   - `--keep-cluster-status-json`
2. Default behavior now removes the per-run raw status snapshot file after summary emission.
3. Dry-run plan output now includes explicit keep-mode marker:
   - `keepClusterStatusJson=true|false`
4. Probe contract test now validates:
   - default keep mode (`false`),
   - explicit keep mode (`true`) dry-run markers.
5. Added short smoke validation to confirm default cleanup leaves no status-json artifact file.

## Why

After adding cluster-status telemetry capture (`1243`), repeated probe runs could leave raw status files in `benchmark-suite/results/raw/` and pollute local git status. Cleanup-by-default keeps the workspace deterministic while preserving an explicit opt-in keep mode for debugging.

## Validation

1. `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
2. `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
3. `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
4. `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --project-path examples/lasm-alpha-full --request-path /health --request-header 'Authorization: Bearer token123' --duration 3s --threads 2 --connections 32 --target-requests 30000 --port 18096 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --out results/summaries/sec4-lasm-cluster-capacity-probe-status-cleanup-smoke.json`
5. `test ! -f benchmark-suite/results/raw/sec4-lasm-cluster-capacity-status-18096.json`
