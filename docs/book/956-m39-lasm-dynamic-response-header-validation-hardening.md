# M39: LASM Dynamic Response-Header Validation Hardening

## Why

LASM dynamic placeholder materialization for response headers enabled powerful runtime behavior, but materialized header names/values could include control characters when derived from request query/path/header data.

Without validation on the materialized output path, this opened response-header injection risk (for example CRLF in dynamic header names/values).

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Applied HTTP token validation (`is_lasm_http_token`) to materialized response header names.
2. Applied HTTP header-value validation (`is_lasm_http_header_value`) to materialized response header values.
3. Dropped invalid dynamic headers deterministically instead of emitting malformed wire output.
4. Kept Set-Cookie repeatable handling and applied value validation to each emitted cookie line.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_drops_invalid_dynamic_response_headers`
   - injects CRLF via encoded query placeholders in dynamic header name/value,
   - verifies response remains valid (`200` + body),
   - verifies injected header artifacts are not emitted.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_overrides_user_content_length_with_actual_body_size`
   - sanity check for adjacent response-writer hardening path.
