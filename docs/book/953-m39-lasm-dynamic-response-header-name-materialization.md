# M39: LASM Dynamic Response-Header Name Materialization

## Why

LASM dynamic placeholder support already covered response bodies, response-header values, and cookie emission.

But response-header names were still effectively static: typed flows like
`headers.name(validate.nonEmpty(req.query("header_name")))` compiled, yet LASM extraction/materialization did not resolve dynamic header keys.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Extended header-gate extraction so both `headers.name(...)` and `headers.value(...)` can resolve request-derived placeholders (including `validate.nonEmpty(...)` wrappers).
2. Updated LASM header materialization to process both header keys and header values.
3. Added shared helper `materialize_lasm_request_placeholders(...)` and reused it across body/header materialization paths to keep key/value placeholder resolution consistent.
4. Preserved repeatable `Set-Cookie` handling when header key materialization yields `Set-Cookie`.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - route now uses dynamic header key from `req.query("header_name")`,
   - verifies emitted dynamic header key/value (`X-Trace-Echo: q+7 ok`) and existing dynamic body/header behavior.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie`
   - sanity check for dynamic cookie + multi-`Set-Cookie` behavior after key-materialization changes.
