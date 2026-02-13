# 110 M7 Slice: Log Event Helper Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: bridging structured log-event helper APIs from the security logging spec into semantic analysis, C lowering, and runtime ABI.

## Structured Log Event Helpers

### What it is

Added intrinsic support for:
- `log.attrRedacted` / `log_attr_redacted`
- `log.withAttr` / `log_with_attr`
- `log.withHttp` / `log_with_http`
- `log.withError` / `log_with_error`

Lowered runtime ABI stubs:
- `ailang_rt_log_attr_redacted`
- `ailang_rt_log_with_attr`
- `ailang_rt_log_with_http`
- `ailang_rt_log_with_error`

Also added primitive type names:
- `LogAttr`
- `LogEvent`

### Why it exists

The v0 logging model uses structured events and explicit redaction helpers. Without these bridges, documented log-event composition patterns cannot compile through `c-bin`, and bootstrap examples cannot exercise the intended API shape.

### How it works internally

- Added semantic intrinsic entries in `semantic.rs`:
  - `log.attrRedacted` returns `LogAttr`
  - `log.withAttr`, `log.withHttp`, and `log.withError` return `LogEvent`
- Added `LogAttr` and `LogEvent` to the primitive type catalog.
- Added C rewrite mappings in `c_backend.rs` for dotted and underscore forms.
- Added runtime declarations/definitions in `runtime/c/`.
- Extended log-builder tests (core + CLI) to assert these helper calls lower and link through `c-bin`.

### Inputs, outputs, and constraints

- Input: structured log helper calls in bootstrap service code.
- Output: generated C calls to runtime symbols `ailang_rt_log_*`.
- Constraints:
  - runtime behavior remains stubbed in M7.
  - this slice validates compile/link and API-shape coverage, not final runtime log serialization behavior.

### Failure modes and diagnostics

- Misspelled helper names fail semantic/C compile phases.
- Strict policy-level checks for log field content (for example secret and untrusted data constraints) remain enforced by existing sink-flow diagnostics; richer log-shape validation is deferred.
- Signature hardening for helper argument types/call-shapes is covered in a later slice (`179-m7-log-event-helper-signature-hardening.md`).
- Builder-label typing hardening (`log.event`/`log.redacted`) is covered in a later slice (`181-m7-log-value-builder-label-signature-hardening.md`).

### Example usage

```ailang
fn main() -> Int {
  let event = log.event("user.created");
  let attr = log.attrRedacted("token");
  let withAttr = log.withAttr(event, "token", attr);
  let withHttp = log.withHttp(withAttr, "POST", "/users", 200, 42);
  let err = err.internal("boom");
  log.withError(withHttp, err);
  0
}
```

### Tradeoffs and next steps

- This keeps runtime bridge coverage aligned with the security logging spec while avoiding early over-design.
- Next steps are behavior-level runtime logging semantics (structured output shape, policy-controlled field filtering, and correlation-id propagation).

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - runtime ABI assertions include new `ailang_rt_log_*` helper symbols
  - log-builder rewrite test now asserts `attrRedacted/withAttr/withHttp/withError` lowering
- `compiler/ailang-cli/tests/json_output.rs`
  - log-builder `c-bin` integration fixture now includes new helper calls
  - generated C assertions cover all new runtime symbols
