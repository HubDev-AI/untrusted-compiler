---
status: complete
priority: p1
issue_id: "020"
tags: [post-alpha, lasm, benchmark, automation, reliability]
dependencies: ["019"]
---

# Post-Alpha Auto Mode-Compare Recovery

This board tracks reliability hardening for LASM benchmark auto-mode selection on the canonical workbench workload.

# Task Board

- [x] T1: Add explicit task board and execution scope
  - Goal: keep this slice scoped to benchmark auto-mode recovery and deterministic fallback behavior.
- [x] T2: Implement benchmark matrix auto-recovery path
  - Goal: when `--lasm-mode auto` has missing/mismatched recommendation artifacts, attempt deterministic mode-compare regeneration before fallback.
- [x] T3: Lock behavior with script-level coverage
  - Goal: add/refresh benchmark script test coverage for auto-generate + fallback semantics.
- [x] T4: Finalize docs/board status and package PR
  - Goal: close board status with changed files + validation summary.

# Work Log

### 2026-03-10 - Board created

**By:** Codex

**Actions:**
- Created execution board for post-alpha benchmark auto-mode recovery.
- Started implementation in `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`.

### 2026-03-10 - T2/T3 implemented

**By:** Codex

**Actions:**
- Added `--lasm-auto-mode-compare-generate 0|1` (default `1`) to `run_workbench_benchmark_matrix.sh`.
- Implemented `auto` recommendation resolver with deterministic checks on:
  - artifact existence,
  - endpoint workload match,
  - db-adapter match,
  - recommendation mode validity.
- Added auto-generation path for missing/mismatched recommendation artifacts via delegated:
  - `benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh`
- Kept deterministic fallback to `single` mode when recommendation stays unavailable.
- Extended `test_run_workbench_benchmark_matrix.sh` to cover:
  - auto-mode generation delegation in dry-run,
  - fallback-to-single behavior after missing artifact,
  - expected warning markers.
- Validation run:
  - `benchmark-suite/scripts/test_run_workbench_benchmark_matrix.sh` ✅
  - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh` ✅

### 2026-03-10 - T4 finalized

**By:** Codex

**Actions:**
- Synced board status to complete.
- Recorded branch-gate and shell-subshell corrections in `.claude/napkin.md`.
- Prepared the slice for PR packaging on `codex/post-alpha-auto-mode-compare-generation`.
