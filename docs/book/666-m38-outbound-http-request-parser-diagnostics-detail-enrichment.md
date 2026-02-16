# M38-S45 Outbound HTTP Request-Parser Diagnostics Detail Enrichment

## What it is

M38-S45 enriches split request-parser diagnostics with deterministic structured details.

All `NET.REQUEST_*` parser errors now include:

- `{"key":"phase","value":"parse"}`
- `{"key":"component","value":"..."}` where component is `scheme|host|port|target|ipv6`.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S44 introduced split non-IPv6 parser codes, but parser diagnostics still lacked stable detail context for downstream tooling and deterministic triage.

This slice adds structured parser metadata while keeping code-level diagnostics unchanged.

## How it works

1. Added runtime helper to emit standard error payloads with two deterministic detail entries.
2. Updated split request-parser diagnostics to use detail-enriched payloads with `phase=parse` and component-specific values.
3. Updated redirect parser-failure path to reuse the same detail-enriched request-parser diagnostics.
4. Expanded runtime harness assertions to verify both diagnostic code and deterministic detail entries.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime error rendering path is slightly larger due to two-detail JSON helper.
- Diagnostic payloads are more explicit, which improves replay/audit/tooling stability.

## Next

1. Normalize remaining generic parser fallback (`NET.URL_INVALID`) into deterministic parser-specific fallback code.
2. Add harness coverage for parser fallback normalization path with detail checks.
