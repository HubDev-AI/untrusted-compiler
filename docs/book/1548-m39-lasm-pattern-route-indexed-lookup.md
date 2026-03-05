# 1548 M39 Slice: LASM pattern-route indexed lookup fast path

## What it is

This slice adds indexed pattern-route lookup in LASM HTTP runtime:

- file: `compiler/sec4-core/src/lasm_http_runtime.rs`
- new runtime indexes:
  - `pattern_route_indexes_by_method`
  - `pattern_route_indexes_by_len`

## Why it exists

Pattern-route matching previously scanned all registered pattern routes for each request.  
Under larger route sets this added avoidable per-request routing overhead on LASM hot paths.

## How it works internally

1. On pattern-route registration, runtime now records the new route index in:
   - method + segment-count index,
   - segment-count-only index.
2. Route resolution now:
   - resolves exact routes first (unchanged),
   - then checks only method/segment-count candidate route indexes (latest-registration-wins preserved via reverse index traversal).
3. `Allow`-header method discovery for path mismatches now:
   - checks exact routes as before,
   - checks only pattern candidates with matching segment count.

## Inputs, outputs, and constraints

- Inputs/outputs are unchanged:
  - same route registration API,
  - same request/response envelopes.
- Constraints preserved:
  - deterministic latest pattern registration wins,
  - deterministic `HEAD -> GET` fallback behavior.

## Failure modes and diagnostics

No new diagnostics were introduced. Existing routing diagnostics and response contracts remain unchanged.

## Validation

- `cargo test -p sec4-core lasm_http_runtime`
  - route runtime unit coverage passed (including pattern override and parameterized-route tests).

## Tradeoffs and next steps

- Tradeoff: small memory overhead for route index maps.
- Benefit: lower pattern-route candidate scan cost on request hot path.
- Next step: measure route-lookup contribution in workbench/cluster capacity probes and continue incremental runtime hot-path profiling.
