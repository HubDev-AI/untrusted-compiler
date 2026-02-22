# M39: LASM Capacity Probe wrk-Process Fanout

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`:
  - added `--wrk-processes <n>` (env `LASM_CAPACITY_WRK_PROCESSES`, default `1`),
  - each sample can now launch multiple `wrk` processes in parallel,
  - per-sample metrics are aggregated across processes:
    - `requests` = sum,
    - `requestsPerSec` = sum,
    - `p99` = conservative max across process p99 values (normalized to `ms`),
  - sample metadata now keeps per-process raw/result details in:
    - `observed.samples[].wrkProcessRuns[]`,
  - run metadata now records:
    - `run.wrkProcesses`,
    - `run.wrkTotalConnections`.
- Updated `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`:
  - dry-run assertions now verify `wrkProcesses`,
  - added deterministic invalid-input checks for `--wrk-processes`:
    - integer-only,
    - `>= 1`.

## Why

Single-process `wrk` can become the load generator bottleneck before the LASM runtime is saturated. This slice adds deterministic multi-process fanout so probe runs can push higher request pressure without rewriting orchestration wrappers.

## Result

- Capacity probe can now scale client-side load per sample directly:
  - `--wrk-processes 1` preserves prior behavior,
  - `--wrk-processes > 1` runs N parallel `wrk` clients and reports one aggregated sample result.
- Artifacts retain process-level details for diagnosis while keeping one pass/fail surface.

## Validation

- `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --skip-build --duration 4s --threads 4 --connections 64 --samples 2 --wrk-processes 2 --target-requests 1 --port 18124 --out results/summaries/sec4-lasm-cluster-capacity-probe-wrk2-smoke.json`
