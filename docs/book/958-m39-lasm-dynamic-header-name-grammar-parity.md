# M39: LASM Dynamic Header-Name Grammar Parity

## Why

LASM dynamic response-header validation initially used generic HTTP token checks for materialized header names.

That was broader than the Sec4 header-name gate/runtime contract (`[A-Za-z0-9-]+`), allowing dynamic names such as `X_Trace_Echo` that C runtime would reject.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added `is_lasm_response_header_name_valid(...)` mirroring header-name gate/runtime grammar (`ASCII alphanumeric` or `-`).
2. Switched dynamic response-header name validation from HTTP-token rules to the stricter header-name grammar parity rule.
3. Kept request parser behavior unchanged (request headers still use HTTP token parsing), while response sink materialization now follows sink-level header-name constraints.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - existing compose route now sends a second request with `header_name=X_Trace_Echo`,
   - verifies invalid dynamic response header name is dropped,
   - verifies valid static headers and dynamic body materialization still work.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_drops_invalid_dynamic_response_headers`
   - sanity check for output-side validation hardening behavior.
