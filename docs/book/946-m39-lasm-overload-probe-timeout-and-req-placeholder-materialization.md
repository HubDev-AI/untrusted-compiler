# M39: LASM Overload Probe Timeout and Request Placeholder Materialization

## Why

Two runtime gaps remained in LASM `sec4 run --backend lasm`:

1. On queue saturation, overflow head parsing could block up to the full serve timeout while probing slow clients.
2. `res.text(...)` extraction supported only static literal/bound strings, so simple request-derived values from `req.pathParam` / `req.header` were not materialized in LASM responses.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Overload path now applies a bounded read-probe timeout (`min(effective_timeout_ms, 50ms)`) before parsing overflow request heads, keeping accept-loop saturation behavior responsive.
2. `res.text(...)` extraction now supports request-derived placeholder planning:
   - `req.pathParam("...")`
   - `req.header("...")`
   - wrapped through `validate.nonEmpty(...)` / `validate_non_empty(...)`
3. Response materialization path now resolves planned placeholders at runtime using current request/path params before response write.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_supports_bound_res_text_arguments`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_header_value_character`
4. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request`
5. `cargo test -p sec4 --test commands run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached`
