# M39 Slice: LASM DB Multi-Op `execTx` Sequence TX Reuse

Date: 2026-02-25  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Added internal sequence orchestration for multi-op DB dispatch to reuse tx handles across repeated `execTx` steps sourced from the same `db.tx(dbCap)` plan input.
- Added sequence-retain internal marker (`X-Sec4-Internal-Db-Tx-Sequence-Retain`) so single-op `execTx` behavior remains unchanged while sequence mode can retain/reuse tx handles.
- Added deterministic cleanup for retained tx handles after sequence completion (success and failure paths).

## Why

- Multi-op DB dispatch executes operations in-order, but `execTx` previously allocated and immediately cleaned tx handles per step, which prevented same-sequence transaction-handle reuse.
- Reuse semantics are required for deterministic multi-step transaction flows generated from one handler.

## Behavioral contract

- In multi-op sequences, repeated `execTx` operations derived from the same `db.tx(dbCap)` source reuse the same tx handle.
- Runtime still stops on first operation failure with deterministic envelopes.
- Retained tx handles do not leak beyond the sequence lifecycle; cleanup runs on success and failure exits.
- Non-sequence and single-operation `execTx` paths keep prior semantics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_reuses_tx_handle_across_multi_op_exec_tx_sequence -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_backend_executes_multiple_db_intrinsic_ops_in_single_handler -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_db_operation_sequence_over_limit -- --exact`
