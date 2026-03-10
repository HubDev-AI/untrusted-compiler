---
status: complete
priority: p1
issue_id: "030"
tags: [post-alpha, lasm, db-client, runtime, config]
dependencies: ["029"]
---

# Post-Alpha DB Limit Config Batch Extraction

This slice moves all remaining DB runtime limit resolver/override state (except dispatch wrappers) from `lasm_db_runtime_dispatch` into `lasm_db_client::config`, tightening package boundaries without changing external behavior.

# Task Board

- [x] T1: Add client-config limit resolvers/overrides for SQL template, params bytes, params entries, query-one row bytes, and query-one row columns
- [x] T2: Rewire dispatch public wrapper APIs to delegate to client-config implementations
- [x] T3: Remove dispatch-local duplicated limit constants/statics/resolver internals
- [x] T4: Keep runtime enforcement/error envelope behavior unchanged
- [x] T5: Validate focused marker/runtime compilation checks

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/lasm_db_client/config.rs`:
  - added limit constants/state + resolver/setter functions for:
    - `sql_template_max_bytes`
    - `params_max_bytes`
    - `params_max_entries`
    - `query_one_row_max_bytes`
    - `query_one_row_max_columns`
  - retained existing op-sequence max resolver/setter in the same config layer.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - switched public limit wrappers to delegate to client-config functions for all above limits.
  - removed dispatch-local env constants, `OnceLock` caches, and override atomics for these limits.
  - preserved enforcement callsites and deterministic error-envelope mapping.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --bin sec4 operation_count_marker -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 indexed_internal_db_markers_without_operation_count -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 classify_db_runtime_error_ -- --nocapture` ✅ (0 matched; compile + filter path clean)
