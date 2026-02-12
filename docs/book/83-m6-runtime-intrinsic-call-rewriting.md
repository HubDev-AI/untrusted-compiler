# 83 M6 Slice: Runtime Intrinsic Call Rewriting

This chapter documents the next M6 vertical slice: rewriting selected intrinsic call spellings in emitted C so they resolve to runtime ABI symbols.

## Intrinsic Call Rewriting (`time.now`)

### What it is

The C emitter now rewrites:
- `time.now(...)`
- `time_now(...)`

to:
- `ailang_rt_time_now(...)`

A runtime stub for `ailang_rt_time_now` is now part of `ailang_runtime.h/.c`.

### Why it exists

AILang source-level intrinsic names can contain dotted namespaces (`time.now`). Those names are not valid C symbols. This slice introduces a minimal intrinsic-call lowering rule so generated C remains valid while preserving runtime ABI wiring.

### How it works internally

- `emit_runtime_header()` now declares:
  - `int64_t ailang_rt_time_now(void);`
- `emit_runtime_source()` now defines a minimal stub:
  - returns `0` for now.
- C emission now runs expression strings through `lower_c_expr(...)` before writing them in:
  - `let` assignments
  - eval statements
  - return values
  - branch conditions
  - switch scrutinees
- `lower_c_expr(...)` uses a stable placeholder pass to avoid accidental double-rewrite collisions.

### Inputs, outputs, and constraints

- Input: MIR expression text containing intrinsic call spellings.
- Output: C-valid runtime call spellings for currently supported intrinsic patterns.
- Constraints:
  - rewrite coverage is intentionally narrow in this slice (`time.now` family only).
  - runtime implementation remains a deterministic stub until real clock/runtime policy integration is added.

### Failure modes and diagnostics

- If rewrite coverage is missing for a dotted intrinsic, generated C may still contain invalid symbols.
- Runtime ABI declaration/definition mismatches would fail at compile/link time through existing clang diagnostics.

### Example usage

AILang:

```ailang
fn current() effects { time.now } -> Int64 {
  time.now()
}
```

Generated C return (simplified):

```c
return ailang_rt_identity_i64(ailang_rt_time_now());
```

### Tradeoffs and next steps

- Current behavior is deterministic and safe for compiler iteration, but not yet semantically complete clock behavior.
- Rewriting is string-based; future slices should move toward structured call lowering to runtime symbols for broader intrinsic coverage.
- Next step: expand intrinsic mapping beyond `time.now` and align with typed runtime capability/effect surfaces.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - validates runtime header/source include `ailang_rt_time_now`
  - validates `time.now()` lowers to `ailang_rt_time_now()` in emitted C
