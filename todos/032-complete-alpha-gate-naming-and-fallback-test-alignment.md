---
status: complete
priority: p0
issue_id: "032"
tags: [alpha, release-gate, naming-lock, tests]
dependencies: ["031"]
---

# Alpha Gate Naming + Fallback Test Alignment

This slice removes one release-gate naming-lock blocker and aligns two DB success-shaping tests with current flat-params parser behavior.

# Task Board

- [x] T1: Fix naming-lock failure from legacy absolute path in final alpha readiness plan
- [x] T2: Fix failing `sec4` test assertions that assumed nested params must bypass flat parser
- [x] T3: Re-run strict alpha gate and confirm green

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `/Users/vladimirtrifonov/src/ai/AILang/docs/plans/2026-03-07-final-alpha-readiness-summary.md`:
  - replaced machine-specific `BENCH_WRK2_BIN=/Users/.../AILang/...` with relative path `./benchmark-suite/bin/wrk2` to satisfy naming-lock rules.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` tests:
  - renamed:
    - `derive_exec_like_success_data_uses_json_fallback_when_flat_params_fail`
    - `derive_query_one_success_data_uses_json_fallback_when_flat_params_fail`
  - new names assert behavior contract on nested params payloads without requiring flat-parser failure implementation detail.
  - removed unused test import from test module.
- Ran strict gate and confirmed pass:
  - `scripts/check-naming-lock.sh` ✅
  - `scripts/check-milestone-closure.sh --fail-on-pending` ✅
  - `scripts/release-alpha-gate.sh` ✅

