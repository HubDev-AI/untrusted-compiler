# 294 M32 Follow-up Slice: Runtime Gate Handle De-stub

This chapter documents the first concrete runtime de-stub implementation after the M32 closure checkpoint.

## What it is

Updated:
- `runtime/c/sec4_runtime.h`
- `runtime/c/sec4_runtime.c`
- `compiler/sec4-core/tests/c_backend.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

Added behavior:
- Request source intrinsics now accept explicit runtime arguments:
  - `sec4_rt_req_query(int64_t name)`
  - `sec4_rt_req_path_param(int64_t name)`
  - `sec4_rt_req_header(int64_t name)`
- Trust/sanitize/url/path/header gate intrinsics now accept typed inputs and return deterministic opaque handles instead of constant zero stubs.
- `path.under(base, input)` now enforces a minimal runtime guard: returns `0` if either `base` or `input` is missing.

## Why it exists

M32 produced closure/gating orchestration. This slice starts converting that orchestration into real runtime behavior so gate APIs are no longer pure no-op placeholders.

## How it works internally

1. Added deterministic handle derivation helpers in runtime:
   - `sec4_rt_hash_token(...)`
   - `sec4_rt_gate_handle_from_input(...)`
2. Request-source calls derive stable untrusted tokens from call inputs.
3. Validation/sanitization/url/path/header constructors derive stable typed handles from input tokens.
4. `path.under` combines `base` and `input` tokens into a derived path-safe handle and hard-fails on missing operands.

## Tests and validation

- Updated C backend runtime-asset expectations for the new runtime signatures:
  - `cargo test -p sec4-core --test c_backend`
- Added new clang-gated integration test proving runtime gate outputs are non-stub (distinct handles across distinct inputs):
  - `c_bin_runtime_gate_handles_are_non_stub_when_clang_available`

## Trade-offs and next steps

- Trade-off:
  - This is still token-level runtime behavior (not full semantic validation of real strings/URLs/headers).
- Next:
  - Extend gate runtime to perform real content checks (header CRLF, URL class checks, path normalization) while preserving deterministic `c-bin` integration coverage.
