# M39: LASM Dynamic Response-Header Non-Empty Value Parity

## Why

LASM dynamic response-header materialization already rejected control characters, but still allowed empty materialized values.

That diverged from Sec4 response-header sink expectations where empty dynamic header values should be dropped deterministically.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added `is_lasm_response_header_value_valid(...)` to enforce both:
   - valid header-value characters (existing rule), and
   - non-empty materialized values.
2. Switched dynamic response-header materialization to use this stricter helper.
3. Applied the same helper to dynamic `Set-Cookie` split handling so empty cookie lines are dropped consistently.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_serves_request_and_exits`
   - verifies existing LASM oneshot behavior remains intact.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_drops_invalid_dynamic_response_headers`
   - confirms invalid control-character dynamic headers are still rejected.
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_drops_empty_dynamic_response_header_values`
   - verifies materialized empty dynamic response-header values are dropped while response status/body stay correct.
