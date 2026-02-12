# 87 M6 Slice: Log Intrinsic Runtime Lowering

This chapter documents the next M6 vertical slice: lowering logging intrinsics to runtime ABI symbols so logging-effect programs compile and run through `c-bin`.

## Log Intrinsic Rewriting

### What it is

C emission now rewrites these intrinsic spellings:
- `log.info(...)`
- `log.warn(...)`
- `log.error(...)`
- `log.emit(...)`

and underscore aliases to:
- `ailang_rt_log_any(...)`

Runtime ABI now exposes:
- `void ailang_rt_log_any();`

with a no-op stub implementation in `runtime/c/ailang_runtime.c`.

### Why it exists

Without rewriting, dotted logging calls produce invalid C symbol names and fail native compilation. This slice extends intrinsic lowering beyond `time.now` so common effectful logging paths are runnable in the current backend.

### How it works internally

- `lower_c_expr(...)` now rewrites supported log intrinsic names using placeholder-based substitution (same pattern used for `time.now`) to avoid replacement collisions.
- Runtime ABI header/source were extended with `ailang_rt_log_any`.
- Existing C emission paths (`let`, `eval`, `return`, `branch`, `switch`) all pass expressions through `lower_c_expr(...)`, so rewritten calls apply consistently.

### Inputs, outputs, and constraints

- Input: MIR expression text containing supported log intrinsic call names.
- Output: C-valid runtime call sites (`ailang_rt_log_any(...)`).
- Constraints:
  - runtime log behavior is currently no-op.
  - rewrite coverage is currently limited to listed log intrinsic families.

### Failure modes and diagnostics

- Unsupported intrinsic spellings still surface as invalid/unknown C calls during compile.
- ABI signature drift between runtime header/source is caught by existing runtime content tests.

### Example usage

```ailang
fn main() effects { log } -> Int {
  log.info(1);
  0
}
```

Generated C includes:

```c
(void)(ailang_rt_log_any(1));
```

### Tradeoffs and next steps

- This is ABI plumbing, not final structured logging semantics.
- Runtime log function currently discards inputs intentionally to keep backend iteration fast.
- Next step: evolve runtime log ABI toward structured event payloads aligned with the v0 log-event spec.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - runtime content assertions include `ailang_rt_log_any`
  - new `c_backend_rewrites_log_intrinsics_to_runtime_symbol`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_log_intrinsic_when_clang_available`
