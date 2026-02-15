# 298 M32 Follow-up Slice: Runtime Request-Source Extraction

This chapter documents the fifth post-closure runtime stabilization slice.

## What it is

Updated:
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/README.md`

Key runtime extraction changes:
- Request state now keeps raw header bytes and full request target (path + query).
- `req.query` now extracts value from query string when request context is present.
- `req.header` now extracts concrete header values from captured raw request headers.
- Both query/header extracted values are fed into tracked-string mapping so later gates (`url.public`, `url.internal`) can inspect real payloads.

## Why it exists

URL content guards from `M32-R4` need inspectable string payloads. Before this slice, request-source functions mostly hashed function arguments without extracting request data. That made runtime checks less realistic for real HTTP traffic.

## How it works internally

1. Added tracked value registry (`sec4_rt_tracked_values`) keyed by deterministic handles.
2. Added query parser helper (`sec4_rt_extract_query_value`) and request-source storage helper (`sec4_rt_track_string_value`).
3. Preserved full request target before route matching strips query suffix.
4. Captured raw headers during HTTP parsing and reused existing header parser helper for `req.header` lookups.
5. Added URL parser/host classification helpers to validate `url.public/internal` against extracted tracked values.

## Validation

Validated with deterministic runtime harness and integration tests:
- `c_bin_runtime_url_guards_when_clang_available`
- `c_bin_runtime_gate_handles_are_non_stub_when_clang_available`
- `c_bin_http_runtime_serves_health_route_in_oneshot_mode_when_clang_available`

## Trade-offs and next steps

- Trade-off:
  - Path-param extraction is not implemented yet; `req.pathParam` still uses fallback behavior.
- Next:
  - implement route-template-aware path param extraction and normalization (`M32-R6`).
