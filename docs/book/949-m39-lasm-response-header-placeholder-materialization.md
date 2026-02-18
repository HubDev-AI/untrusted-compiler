# M39: LASM Response-Header Placeholder Materialization

## Why

LASM already materialized request-derived placeholders in `res.text(...)` response bodies, but response headers were still emitted as raw placeholder tokens.

That created a behavior gap for handlers using header sinks like `res.setHeader(headers.name(...), headers.value("{{req.query:key}}"))`.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. `apply_lasm_dynamic_response_materialization(...)` now materializes placeholders in both body and response headers.
2. Added `apply_lasm_header_placeholder_materialization(...)` to resolve:
   - `{{req.pathParam:...}}`
   - `{{req.header:...}}`
   - `{{req.query:...}}`
   across all response header values.
3. Added shared placeholder detection helper (`contains_lasm_request_placeholder_tokens(...)`) so body/header paths use the same detection rules.
4. Extended LASM command integration coverage so a route can emit dynamic headers and body from the same request-derived placeholder set.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - verifies dynamic body materialization for path/header/query values,
   - verifies dynamic response-header materialization for query/header placeholders (`X-Trace-Echo`, `X-Request-Id-Echo`),
   - verifies query decode behavior remains intact (`%XX`, `+`, invalid escape fallback).
