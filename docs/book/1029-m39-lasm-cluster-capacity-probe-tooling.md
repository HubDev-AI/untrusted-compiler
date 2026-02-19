# 1029 M39 Slice: LASM Cluster Capacity Probe Tooling

This slice adds dedicated benchmark tooling for LASM cluster scaling measurements.

## What changed

1. Added `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`.
2. Added dry-run contract test `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`.
3. Integrated benchmark make target:
   - `make -C benchmark-suite lasm-cluster-capacity-probe`.
4. Updated benchmark docs with usage and notes.

## Why

Existing capacity probe tooling targets the benchmark service binary path. For LASM cluster scaling work, we also need repeatable measurements against `sec4 run --backend lasm` directly, including cluster and autoscale tuning flags.

## What the probe captures

- starts `sec4 run --backend lasm` on a configurable project path/port,
- applies cluster controls (`--instances`, `--autoscale-*`, relay tuning),
- drives load with `wrk` (including required request headers),
- samples peak RSS from the running sec4 process,
- writes a deterministic JSON summary with:
  - observed request count / req/s,
  - peak RSS,
  - p99 (when present in wrk output),
  - pass/fail against `targetRequests`.

## Validation

- `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --dry-run ...` argument plan checks
- 1M-target probe execution example:
  - command: `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 20s --target-requests 1000000 --threads 8 --connections 256 --instances 4 --autoscale-max-instances 8 --autoscale-target-connections 256 --cluster-relay-workers 32 --cluster-relay-queue 4096 --out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-1m.json`
  - result: pass with `1,278,004` requests (`~63,573 req/s`), p99 about `883ms`, peak RSS `10,464 KB`.

This provides a reproducible entry point for 1M-target LASM cluster capacity verification loops.
