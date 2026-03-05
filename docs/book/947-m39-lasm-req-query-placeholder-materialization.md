# M39: LASM `req.query` Placeholder Materialization

## Why

LASM dynamic `res.text(...)` materialization already supported request-derived placeholders for:

- `req.pathParam("...")`
- `req.header("...")`

`req.query("...")` was still missing in the same flow, so simple query-driven handler responses could not be materialized by the LASM backend.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added query-param capture to parsed LASM request state:
   - request target now splits into normalized path + query map.
2. Extended `res.text(...)` placeholder planning to include:
   - `req.query("key")`
   - wrapped through `validate.nonEmpty(...)` / `validate_non_empty(...)`.
3. Extended response placeholder materialization to replace:
   - `{{req.query:key}}` from current request query map.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_supports_bound_res_text_arguments`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached`
