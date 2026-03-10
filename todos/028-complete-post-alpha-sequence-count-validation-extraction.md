---
status: complete
priority: p1
issue_id: "028"
tags: [post-alpha, lasm, db-client, runtime, extraction]
dependencies: ["027"]
---

# Post-Alpha Sequence Count Validation Extraction

This slice moves internal DB operation-sequence count validation rules out of dispatch and into `lasm_db_client`, keeping runtime behavior unchanged while tightening client-layer ownership.

# Task Board

- [x] T1: Add client-layer sequence-count validation API + typed validation errors
- [x] T2: Replace dispatch-local marker/limit parsing with the new client validator
- [x] T3: Keep deterministic HTTP envelope mapping in dispatch via one centralized mapper
- [x] T4: Validate focused sequence marker + `execTx` sequence tests

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/lasm_db_client/operations.rs`:
  - added `LasmInternalDbOperationSequenceValidationError` (`InvalidMarker`, `MarkerValueTooSmall`, `IndexedMarkersRequireCount`, `ExceedsMaximum`).
  - added `validate_lasm_internal_db_operation_sequence_count(...)` to own:
    - operation-count marker parsing,
    - minimum marker value check (`>= 2`),
    - indexed-marker-without-count rejection,
    - max-sequence-size guard.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - switched to `validate_lasm_internal_db_operation_sequence_count(...)` for sequence-count gate logic.
  - added `respond_lasm_internal_db_sequence_validation_error(...)` to map typed client validation errors into existing deterministic `DB.OPERATION_INVALID` envelopes.
  - removed duplicated inline count/limit validation branches.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --bin sec4 operation_count_marker -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 indexed_internal_db_markers_without_operation_count -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 exec_tx_sequence -- --nocapture` ✅
