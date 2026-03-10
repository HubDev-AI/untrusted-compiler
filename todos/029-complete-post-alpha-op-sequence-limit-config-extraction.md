---
status: complete
priority: p1
issue_id: "029"
tags: [post-alpha, lasm, db-client, runtime, config]
dependencies: ["028"]
---

# Post-Alpha Op-Sequence Limit Config Extraction

This slice moves `db` operation-sequence max-limit configuration ownership from dispatch into `lasm_db_client::config`, improving client-layer boundary ownership while preserving CLI/runtime behavior.

# Task Board

- [x] T1: Add op-sequence max env/override resolver in `lasm_db_client::config`
- [x] T2: Rewire dispatch `lasm_db_op_sequence_max_limit` and override setter to delegate to client config
- [x] T3: Remove dispatch-local duplicated op-sequence max constants/statics/resolver
- [x] T4: Validate focused marker-limit tests after extraction

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/lasm_db_client/config.rs`:
  - added op-sequence limit config constants/state:
    - `SEC4_RT_LASM_DB_OP_SEQUENCE_MAX`
    - default/min/max and override cache.
  - added `resolve_lasm_db_op_sequence_max()` and `set_lasm_db_op_sequence_max_override(...)`.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - switched op-sequence limit API wrappers to delegate to `lasm_db_client::config`.
  - removed dispatch-local op-sequence max env constants, statics, and resolver implementation.
  - kept existing public dispatch-facing limit/override functions and all error-envelope semantics unchanged.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --bin sec4 operation_count_marker -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 indexed_internal_db_markers_without_operation_count -- --nocapture` ✅
