# M38-S47 Outbound HTTP Request-Parser Public/Internal Sink Parity

## What it is

M38-S47 adds explicit runtime harness coverage that parser diagnostics parity holds between:

- `sec4_rt_http_get` (public sink)
- `sec4_rt_http_get_internal` (internal sink)

for equivalent malformed redirect-target parser failures.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Request-parser diagnostics were hardened in previous slices, but parity between public/internal sink wrappers was only implicit via shared runtime paths.

This slice makes that parity an explicit, locked contract.

## How it works

1. Added public-sink redirect parser parity harness:
   - `c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`
2. Harness runs malformed redirect-target cases through `sec4_rt_http_get` with loopback SSRF blocking toggled off for local test execution.
3. Assertions validate both code and structured details (`phase=parse`, matching `component`) for the same parser-failure classes already covered by internal sink tests.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`

## Trade-offs

- Adds one more clang-gated runtime harness test and local-loopback test setup assumptions.
- Increases confidence that sink wrappers remain behaviorally aligned while runtime evolves.

## Next

1. Lock parser diagnostics envelope consistency checks for all `NET.REQUEST_*` parser diagnostics.
2. Add deterministic detail-order assertions for parser diagnostic payload rendering.
