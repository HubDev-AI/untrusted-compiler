---
status: complete
priority: p1
issue_id: "025"
tags: [post-alpha, lasm, runtime, db, hotpath]
dependencies: ["024"]
---

# Post-Alpha Sequence State + Header Normalization

This slice moves multi-op DB sequence tx-state ownership into `lasm_db_client` and removes an avoidable response-header map preclone in LASM response writing.

# Task Board

- [x] T1: Extract sequence tx-state tracking/cleanup core from dispatch into client layer
- [x] T2: Rewire sequence failure/success cleanup paths to use client-owned sequence state
- [x] T3: Remove dispatch-local sequence cleanup wrappers made redundant by extraction
- [x] T4: Remove full response-header preclone in LASM HTTP writer normalization path
- [x] T5: Run focused validation and package merge-ready entry

# Work Log

### 2026-03-10 - Completed

**By:** Codex

**Actions:**
- Updated `compiler/sec4-cli/src/lasm_db_client/operations.rs`:
  - added `LasmInternalDbSequenceState` for sequence tx tracking and adapter-aware cleanup.
- Updated `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - replaced dispatch-local `BTreeMap/BTreeSet` sequence tx bookkeeping with `LasmInternalDbSequenceState`.
  - rewired sequence failure/success cleanup to call client-layer sequence-state cleanup.
  - removed now-redundant dispatch-local sequence cleanup wrappers.
- Updated `compiler/sec4-cli/src/main.rs`:
  - changed LASM response header normalization to avoid cloning the full response header map before normalization.

**Validation:**
- `cargo check -p sec4` ✅
- `cargo test -p sec4 --bin sec4 exec_tx_sequence -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 operation_count_marker -- --nocapture` ✅
- `cargo test -p sec4 --bin sec4 indexed_internal_db_markers_without_operation_count -- --nocapture` ✅
