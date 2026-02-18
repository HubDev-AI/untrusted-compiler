# 889 M38 Slice: `path.base` Handle Runtime Compatibility

This slice removes a temporary compatibility workaround and restores real `path.base(...)` support for non-literal string values.

## Problem

`path.base(...)` previously compiled correctly for string literals, but non-literal string values could be emitted as handle expressions into C call sites, causing runtime instability.

A temporary semantic restriction was introduced to force literal-only usage.

## Implementation

Runtime ABI now supports both call forms through C11 generic dispatch in `runtime/c/sec4_runtime.h`:

- literal input path:
  - `sec4_rt_path_base_literal(const char *input)`
- tracked-handle input path:
  - `sec4_rt_path_base_handle(int64_t input)`
- compatibility macro:
  - `sec4_rt_path_base(input)` dispatches to the correct implementation.

Runtime logic in `runtime/c/sec4_runtime.c`:

- keeps existing literal normalization and validation behavior,
- adds tracked-handle resolution + deterministic validation for handle-based inputs.

## Compiler/Semantic alignment

The temporary semantic literal-only guard was removed.

`path.base(...)` now accepts `String` values again (including validated non-literal strings), matching runtime capability.

## Validation

Focused validations executed:

- `cargo test -p sec4-core --test c_backend`
- `cargo test -p sec4-core --test golden_semantic`
- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_gate_intrinsics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_path_and_header_guards_when_clang_available`
- `cargo run -q -p sec4 -- check --path examples/showcase-api`
- `cargo run -q -p sec4 -- build --path examples/showcase-api --emit c-bin`

All passed.
