---
status: complete
priority: p1
issue_id: "022"
tags: [post-alpha, lasm, benchmark, full-suite, automation]
dependencies: ["021"]
---

# Post-Alpha Full-Suite Auto-Mode Recovery

This slice aligns full-suite orchestration with benchmark matrix auto-mode recovery so `--lasm-mode auto` does not hard-stop on stale or missing recommendation artifacts.

# Task Board

- [x] T1: Add full-suite option wiring for auto-mode artifact generation control
- [x] T2: Implement full-suite auto recommendation regeneration/fallback behavior
- [x] T3: Update full-suite script tests for fallback warning behavior
- [x] T4: Package complete board entry

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Added `--lasm-auto-mode-compare-generate 0|1` (default `1`) to:
  - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
- Implemented full-suite `--lasm-mode auto` recovery flow:
  - resolve recommendation from artifact when workload+adapter match
  - optionally regenerate recommendation artifact via `run_workbench_lasm_mode_compare.sh`
  - fallback to `single` mode with warning when recommendation remains unavailable
- Forwarded `--lasm-auto-mode-compare-generate` to delegated benchmark matrix runs.
- Updated fallback-warning assertion in:
  - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`

**Validation:**
- `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh` ✅
- `benchmark-suite/scripts/test_run_workbench_benchmark_matrix.sh` ✅
