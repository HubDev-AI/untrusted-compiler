---
status: complete
priority: p1
issue_id: "023"
tags: [post-alpha, lasm, benchmark, step-matrix, automation]
dependencies: ["022"]
---

# Post-Alpha Step-Matrix Auto-Mode Recovery

This slice aligns standalone step-load benchmarking with the same LASM auto-mode recovery semantics already used by matrix/full-suite orchestration.

# Task Board

- [x] T1: Add step-matrix option for auto recommendation artifact generation
- [x] T2: Implement step-matrix auto recommendation regeneration/fallback behavior
- [x] T3: Update step-matrix tests for fallback warning + delegated generation markers
- [x] T4: Package complete board entry

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Added `--lasm-auto-mode-compare-generate 0|1` (default `1`) to:
  - `benchmark-suite/scripts/run_workbench_step_matrix.sh`
- Implemented `--lasm-mode auto` behavior in step-matrix runner:
  - resolve mode from artifact when workload/adapter match
  - optionally regenerate via `run_workbench_lasm_mode_compare.sh`
  - deterministic fallback to `single` with warning when recommendation remains unavailable
- Updated:
  - `benchmark-suite/scripts/test_run_workbench_step_matrix.sh`
    - now accepts modern fallback warning wording
    - verifies delegated mode-compare dry-run generation marker

**Validation:**
- `benchmark-suite/scripts/test_run_workbench_step_matrix.sh` ✅
- `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh` ✅
