# M39: LASM ExecTx Parse After Handle Validation

Date: 2026-02-22  
Milestone: M39 (DB runtime dispatch overhead reduction)

## What Changed

- Reordered `db.execTx` runtime dispatch flow:
  - tx/db handle source validation now runs before adapter SQL param pre-parse,
  - adapter pre-parse (`postgres`/`sqlite`) now runs only after deterministic handle validation succeeds.

## Why

`db.execTx` previously parsed adapter params before validating tx/db handles, so invalid-handle requests paid unnecessary parse cost before returning deterministic validation errors.

## Result

- Invalid `db.execTx` handle paths now fail earlier without extra adapter parse work.
- Success-path behavior and envelopes are unchanged.
- Keeps adapter-aware preparse model while reducing wasted work on validation failures.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
