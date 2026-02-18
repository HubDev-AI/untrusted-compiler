# M39: LASM Dynamic User Store Persistence

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- Added optional disk-backed persistence for LASM dynamic benchmark user state.
- New runtime env knob:
  - `SEC4_RT_LASM_DB_BASE=<dir>`
  - persistence file: `<dir>/users.json`
- Behavior:
  - startup: loads existing users store (if present/valid)
  - create flow (`CreateUserResponse`): updates in-memory map and writes store file deterministically
  - read flow (`UserResponse`): serves from loaded/persisted map

## Why

LASM benchmark-style user materialization previously lived only in memory for one process lifetime. This adds practical database-like behavior for restart continuity without changing route contracts.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_user_store_when_lasm_db_base_is_set`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok`
