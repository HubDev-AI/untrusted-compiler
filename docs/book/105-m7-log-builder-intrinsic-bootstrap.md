# 105 M7 Slice: Log Builder Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: supporting structured log-value builder intrinsics through semantic analysis and C runtime lowering.

## Structured Log Builder Intrinsics

### What it is

Added intrinsic support for log-value constructors:
- `log.event` / `log_event`
- `log.field` / `log_field`
- `log.obj` / `log_obj`
- `log.str` / `log_str`
- `log.i64` / `log_i64`
- `log.bool` / `log_bool`
- `log.redacted` / `log_redacted`

Runtime ABI stubs added:
- `sec4_rt_log_event`
- `sec4_rt_log_field`
- `sec4_rt_log_obj`
- `sec4_rt_log_str`
- `sec4_rt_log_i64`
- `sec4_rt_log_bool`
- `sec4_rt_log_redacted`

### Why it exists

Untrusted<T>’s logging model is structured and redaction-first. Before this slice, only sink calls (`log.info`/`warn`/`error`/`emit`) compiled through the runtime bridge, while log-value constructor APIs in the spec were missing.

### How it works internally

- Added semantic intrinsic entries returning `LogValue` with no effects.
- Extended C intrinsic rewrite mappings for dotted and underscore aliases.
- Added runtime C declarations/definitions for all builder stubs.
- Added core and CLI integration tests to verify symbol lowering and runnable `c-bin` output.

### Inputs, outputs, and constraints

- Input: Untrusted<T> expressions building structured log payload fragments.
- Output: generated C calling runtime log-builder stubs.
- Constraints:
  - runtime value semantics are placeholder in M7 bootstrap mode.
  - this slice validates bridge completeness and API shape, not final log serialization behavior.

### Failure modes and diagnostics

- Misspelled builder names can remain unresolved and fail semantic/C compile stages.
- Logging sink security rules still apply separately at `log.*` sink call sites.

### Example usage

```ut
fn build() -> Int {
  let event = log.event(1);
  let field = log.field(1, 2);
  let redacted = log.redacted(1);
  event;
  field;
  redacted;
  0
}
```

### Tradeoffs and next steps

- This closes a major bootstrap gap in structured logging API coverage.
- Next steps are to enforce richer value-shape checks and integrate these constructors with runtime log-event schema emission.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - added `c_backend_rewrites_log_builder_intrinsics_to_runtime_symbols`
  - runtime ABI assertions include log builder symbols
- `compiler/sec4-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_log_builder_intrinsics_when_clang_available`
