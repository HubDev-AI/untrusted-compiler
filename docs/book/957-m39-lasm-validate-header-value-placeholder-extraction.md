# M39: LASM Validate-HeaderValue Placeholder Extraction

## Why

LASM dynamic header extraction already supported request-derived values through `headers.value(...)` and `validate.nonEmpty(...)` unwrapping.

But direct typed-header flows using `validate.headerValue(req.query(...))` were not unwrapped in the extraction path, causing valid typed sink usage to miss placeholder materialization in LASM run mode.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added explicit detection for `validate.headerValue(...)` wrappers (`validate_header_value` alias included).
2. Extended response-header gate extraction so `value`-gate parsing unwraps `validate.headerValue(...)` and continues placeholder extraction from the wrapped argument.
3. Kept existing `validate.nonEmpty(...)` behavior unchanged and scoped `validate.headerValue(...)` unwrapping to header-value gate paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - route now uses `let trace_value = validate.headerValue(req.query("trace"));` and passes it directly to `res.setHeader(...)`,
   - verifies dynamic header/body placeholder materialization remains correct.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_drops_invalid_dynamic_response_headers`
   - sanity check that output-side header validation hardening remains intact with the new extraction path.
