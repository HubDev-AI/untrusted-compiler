# M38-S48 Outbound HTTP Parser Diagnostics Envelope Consistency

## What it is

M38-S48 locks parser diagnostics envelope consistency for `NET.REQUEST_*` errors.

Parser diagnostics now use a unified runtime rendering path with deterministic details ordering.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Earlier slices added many parser-class diagnostics. Without a single renderer path, detail ordering and payload shape could drift over time.

This slice hardens consistency and makes parser diagnostics easier for tooling to consume deterministically.

## How it works

1. Refactored parser diagnostic emission to a table-driven request-parser diagnostic renderer.
2. Unified all supported parser statuses into one rendering function with deterministic detail ordering:
   - first `phase=parse`
   - second `component=<...>`
3. Added dedicated harness coverage to validate detail ordering across fallback, non-IPv6, and IPv6 parser statuses.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_request_parser_diagnostics_use_unified_details_order_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Parser diagnostic code path adds a small table lookup for status-to-diagnostic mapping.
- Improves maintainability and deterministic payload contract stability for downstream tooling.

## Next

1. Add direct malformed-input sink-bridge parity assertions between `sec4_rt_http_get` and `sec4_rt_http_get_internal` wrapper boundaries.
2. Continue reducing generic net-wrapper fallback branches where parser context is available.
