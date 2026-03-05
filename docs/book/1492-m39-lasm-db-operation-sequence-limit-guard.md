# M39 Slice: LASM DB Operation Sequence Limit Guard

Date: 2026-02-25  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Added a deterministic DB operation-sequence max bound for LASM handler planning and execution:
  - `max 64` DB intrinsic operations per handler.
- `sec4 run --backend lasm` now fails fast during route-plan validation when a handler exceeds this bound.
- `sec4 lasm-smoke` applies the same bound when resolving a route plan for smoke execution.
- LASM runtime dispatch now also enforces the bound when materializing internal DB operation markers.

## Why

- Multi-operation dispatch is now real, so a deterministic upper bound prevents pathological handler expansion and keeps runtime behavior predictable.
- Shared planner/run/smoke/runtime guard alignment avoids split-brain behavior where one path accepts oversized plans and another path rejects them.

## Behavioral contract

- Handlers with `<= 64` DB operations execute as planned.
- Handlers with `> 64` DB operations fail deterministically before serving requests:
  - `run failed: route <METHOD> <PATH> resolves <N> DB intrinsic operations; maximum supported per handler is 64`.
- Runtime dispatch returns deterministic `DB.OPERATION_INVALID` when internal marker sequences exceed the max.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_db_operation_sequence_over_limit -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_backend_executes_multiple_db_intrinsic_ops_in_single_handler -- --exact`
- `cargo test -p sec4 --test commands lasm_smoke_command_materializes_db_list_records_response -- --exact`
