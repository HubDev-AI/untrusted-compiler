# M39: LASM Run Latest Response-Write Parity

## Why

LASM run response-plan extraction returned the first discovered `res.*` helper in a handler body.

For sequential handlers, that diverged from imperative execution semantics where the latest response write should win.

## What Changed

1. Updated response-plan extraction in blocks to track and return the latest discovered response write, not first-match early exit.
2. Updated expression traversal to preserve last-write precedence across sequential expression evaluation (`call` arguments/callee and binary operands).
3. Added LASM oneshot integration coverage proving:
   - sequential `res.text(...)` calls in one handler now yield the final status/body.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_uses_latest_response_write_in_handler`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_uses_latest_duplicate_route_registration`
