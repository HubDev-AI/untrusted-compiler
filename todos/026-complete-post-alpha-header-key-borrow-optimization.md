---
status: complete
priority: p1
issue_id: "026"
tags: [post-alpha, lasm, runtime, hotpath]
dependencies: ["025"]
---

# Post-Alpha Header-Key Borrow Optimization

This slice reduces avoidable header-key allocations in LASM runtime header lookup paths while keeping behavior unchanged.

# Task Board

- [x] T1: Switch case-insensitive header-key lookup helper to borrowed-key return (`&str`)
- [x] T2: Rewire hot-path call sites to avoid unnecessary owned-key cloning
- [x] T3: Keep mutable-removal paths safe by cloning key only where borrow rules require it
- [x] T4: Validate compile + focused oneshot command path

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/main.rs`:
  - `find_lasm_header_key_case_insensitive(...)` now returns borrowed keys (`Option<&str>`) with explicit lifetime.
  - adjusted lookup call sites (`runtime_error_code`, normalization/upsert/trace-id helpers) to use borrowed keys and clone only when required for map mutation.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - updated internal header removal helpers to align with borrowed-key lookup, cloning only at remove boundaries required by mutable borrows.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --test commands run_command_oneshot_serves_request_and_exits -- --exact` ✅
