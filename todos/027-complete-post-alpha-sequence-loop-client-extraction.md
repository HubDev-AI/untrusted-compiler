---
status: complete
priority: p1
issue_id: "027"
tags: [post-alpha, lasm, db-client, runtime, extraction]
dependencies: ["026"]
---

# Post-Alpha Sequence Loop Client Extraction

This slice moves more LASM DB internal-sequence `execTx` validation/state tracking out of dispatch and into `lasm_db_client` helpers to keep dispatch focused on HTTP/materialization orchestration.

# Task Board

- [x] T1: Add reusable sequence failure/preparation types in client layer
- [x] T2: Extract `execTx` sequence pre-validation/state prep into `LasmInternalDbSequenceState`
- [x] T3: Extract runtime tx marker tracking helpers for `tx` and allocated `execTx` flows
- [x] T4: Replace duplicated dispatch logic with client helper calls + shared failure bridge
- [x] T5: Validate compile + focused LASM DB sequence marker tests

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/lasm_db_client/operations.rs`:
  - added `LasmInternalDbSequenceFailure` with deterministic envelope metadata (`code`, `kind`, `message`, `status`).
  - added `LasmSequenceExecTxPreparation` for prepared sequence state transfer (`tx_header`, `tx_db_header`, retain flag, allocated source).
  - added `prepare_exec_tx_sequence_operation(...)` to centralize `execTx` dual-source validation, handle parsing, db-cap handle checks, and reuse/allocate decisions.
  - added `track_sequence_tx_runtime_result(...)` for `db.tx` runtime marker tracking.
  - added `track_sequence_allocated_exec_tx_runtime_result(...)` for allocated `execTx` marker tracking.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - imported new client-layer sequence failure type.
  - added `fail_lasm_internal_db_sequence_failure(...)` bridge to map client failures into existing deterministic HTTP error envelope path.
  - replaced inline `execTx` sequence validation/state prep with `sequence_state.prepare_exec_tx_sequence_operation(...)`.
  - replaced inline runtime tx marker tracking branches with client helper calls.
  - preserved existing behavior/headers/error contract while shrinking dispatch-side branch complexity.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --bin sec4 exec_tx_sequence -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 operation_count_marker -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 indexed_internal_db_markers_without_operation_count -- --nocapture` ✅
