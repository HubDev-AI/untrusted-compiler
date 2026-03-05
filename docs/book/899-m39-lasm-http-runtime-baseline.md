# M39 - LASM HTTP Runtime Baseline

## What Was Added

Implemented a minimal HTTP request/response runtime path on top of the LASM async scheduler.

Files:

1. `compiler/sec4-core/src/lasm_http_runtime.rs`
2. `compiler/sec4-core/src/lasm_runtime.rs` (added `drain_completed`)
3. `compiler/sec4-core/src/lib.rs` (exports)

## Runtime Behavior

`LasmHttpRuntime` now supports:

1. route registration (`method` + `path`),
2. request submission,
3. async scripted handler execution (`Yield`, `SleepMs`, `Complete`),
4. deterministic response emission.

Deterministic error mapping in baseline:

1. missing route -> `404` text response (`route not found`),
2. non-zero handler exit -> `500` text response with exit detail,
3. successful handler exit (`0`) -> configured route response.

## Why

This slice closes `M39-S2B` bootstrap acceptance by proving request/response execution over the LASM scheduler lane before socket/network integration.

## Validation

1. `cargo test -p sec4-core lasm_http_runtime::tests::`

Covered tests:

1. missing route 404,
2. successful scripted route response,
3. non-zero handler exit -> deterministic 500,
4. sleep-based completion ordering (fast route before slow route).
