---
status: complete
priority: p1
issue_id: "021"
tags: [post-alpha, lasm, benchmark, observability, automation]
dependencies: ["020"]
---

# Post-Alpha Auto Mode Source Telemetry

This slice finalizes observability for LASM benchmark auto-mode resolution and keeps matrix auto-mode non-blocking under stale/missing recommendation artifacts.

# Task Board

- [x] T1: Add deterministic auto-mode recommendation resolver and generation path
- [x] T2: Add mode-resolution source visibility in dry-run start markers and run summary JSON
- [x] T3: Extend script tests for auto-mode generation/fallback/source markers
- [x] T4: Package merge-ready changes

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- `run_workbench_benchmark_matrix.sh` now supports:
  - `--lasm-auto-mode-compare-generate 0|1` (default `1`)
  - deterministic auto-mode recommendation resolution checks (artifact existence/workload+adapter match/recommendation validity)
  - delegated recommendation artifact generation via `run_workbench_lasm_mode_compare.sh` when needed
  - deterministic fallback to `single` with warnings when recommendation remains unavailable
  - explicit `lasmModeSource` reporting (`explicit`, `auto-artifact`, `auto-generated-artifact`, `auto-fallback-single`, `auto-no-lasm-fallback-single`)
- `test_run_workbench_benchmark_matrix.sh` now verifies:
  - delegated mode-compare generation in dry-run auto mode
  - fallback-to-single behavior for missing recommendation artifacts
  - `lasmModeSource=auto-fallback-single` marker

**Validation:**
- `benchmark-suite/scripts/test_run_workbench_benchmark_matrix.sh` ✅
- `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh` ✅
