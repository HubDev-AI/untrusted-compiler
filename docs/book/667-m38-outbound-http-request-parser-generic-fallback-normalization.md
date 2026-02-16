# M38-S46 Outbound HTTP Request-Parser Generic Fallback Normalization

## What it is

M38-S46 normalizes unknown parser-status fallback to a deterministic request-parser diagnostic.

Instead of generic `NET.URL_INVALID`, parser-fallback path now emits:

- `NET.REQUEST_PARSE_INVALID`
- details:
  - `{"key":"phase","value":"parse"}`
  - `{"key":"component","value":"unknown"}`

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

After split parser diagnostics (IPv6 + non-IPv6), remaining generic fallback branches could still collapse to non-parser generic errors.

This slice makes fallback behavior explicit and deterministic for parser-context failures.

## How it works

1. Added request-parser fallback branch in `sec4_rt_store_request_parse_error_from_outbound_parse_status(...)` for `SEC4_RT_OUTBOUND_URL_PARSE_INVALID`.
2. Added redirect fallback status `SEC4_RT_REDIRECT_RESOLVE_REQUEST_PARSE_INVALID` in parser-status mapping.
3. Reused the same detail-enriched parser error emission path so fallback errors preserve `phase=parse` and `component=unknown`.
4. Added dedicated harness tests for direct and redirect unknown-fallback mapping behavior.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_unknown_fallback_is_deterministic_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_request_parser_unknown_fallback_is_deterministic_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one more explicit fallback code path to runtime diagnostics surface.
- Improves determinism and parser-context error consistency for downstream tooling.

## Next

1. Add parser-diagnostic parity assertions between `sec4_rt_http_get` and `sec4_rt_http_get_internal` sink paths.
2. Keep reducing generic net error fallback branches where parser-class context is available.
