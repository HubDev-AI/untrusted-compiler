# M39: LASM Capacity Probe Build-Profile + Multi-Sample Support

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Updated `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`:
  - added `--build-profile <debug|release>` (default `release`) so probe runs can target the intended sec4 binary profile,
  - added `--samples <n>` (default `1`) to execute multiple wrk passes in one probe run,
  - probe now records per-sample metrics and reports:
    - `observed.samples[]` (all sample runs),
    - `observed.selectedSample` and best-sample observed values (`requests`, `requestsPerSec`, `p99`),
    - sample success/failure counters.
- Updated `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`:
  - dry-run assertions now cover `buildProfile` + `samples`,
  - added validation errors for invalid `--samples` and invalid `--build-profile`.

## Why

Proxy-mode throughput measurements showed high short-run variance on the local machine. Single-sample probes made tuning decisions noisy and easy to misread. This slice makes capacity probes repeatable and explicit about binary profile, so runtime tuning changes can be judged on stronger signal.

## Result

- Operators can run deterministic probe batches without external wrappers:
  - `--build-profile release --samples 3` for normal tuning runs,
  - `--build-profile debug --samples 1` for quick local iteration.
- Summary artifacts now preserve both:
  - chosen best sample for pass/fail gating,
  - full sample set for variance inspection.

## Validation

- `bash -n benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh`
- `bash -n benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_capacity_probe.sh`
- `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh --duration 4s --target-requests 1 --threads 4 --connections 64 --samples 2 --build-profile debug --skip-build --instances 2 --autoscale-max-instances 3 --out results/summaries/sec4-lasm-cluster-capacity-probe-samples-smoke.json`
