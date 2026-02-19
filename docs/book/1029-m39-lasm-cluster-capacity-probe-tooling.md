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

This provides a reproducible entry point for 1M-target LASM cluster capacity verification loops.
