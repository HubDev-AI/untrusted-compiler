# 92 M6 Slice: Validator/Sanitizer/URL/Path Intrinsic Runtime Lowering

This chapter documents the final M6 intrinsic-lowering coverage slice for validation and trust-gate APIs.

## Trust-Gate Intrinsic Rewriting

### What it is

C emission now rewrites:
- `validate.headerValue`
- `validate.email`
- `validate.uuid`
- `validate.int64`
- `validate.nonEmpty`
- `sanitize.html`
- `url.public`
- `url.internal`
- `path.under` / `validate.pathUnder`

and underscore aliases to runtime symbols:
- `ailang_rt_validate_*`
- `ailang_rt_sanitize_html`
- `ailang_rt_url_public` / `ailang_rt_url_internal`
- `ailang_rt_path_under`

Runtime ABI stubs were added for all these functions.

### Why it exists

These gates define trusted-boundary flow in AILang. Completing their backend lowering means the C pipeline can now compile representative programs that exercise the full security-gate surface, not just effectful sinks.

### How it works internally

- Added mapping rules in `lower_c_expr(...)` for dotted/underscore gate names.
- Added corresponding runtime stubs in `runtime/c/ailang_runtime.h/.c`.
- Added C-backend and CLI integration tests that validate emitted symbols and runnable `c-bin` outputs.

### Inputs, outputs, and constraints

- Input: MIR expression text containing supported gate intrinsic names.
- Output: C-valid runtime-gate call symbols.
- Constraints:
  - URL gate intrinsics still consume `net` effect in semantic checks and require `effects { net }` where used.
  - runtime implementations remain placeholders in M6.

### Failure modes and diagnostics

- Missing `net` effects on URL gates continue to fail semantic checks (`E4002`).
- Gate argument contract mismatches continue to fail semantic checks (`E4002` family) before backend emission.
- Unsupported spellings remain unlowered.

### Example usage

```ailang
fn gates(input: Untrusted<String>, base: PathSafe) effects { net } -> Int {
  validate.headerValue(input);
  validate.email(input);
  sanitize.html(input);
  url.public(input);
  path.under(base, input);
  0
}
```

### Tradeoffs and next steps

- Current runtime gate behavior is stubbed to keep M6 focused on backend ABI coverage.
- This completes broad intrinsic symbol lowering coverage in the C emitter.
- Next step: start replacing stubs with concrete runtime behavior in M7 HTTP/JSON and policy-aware runtime slices.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - new `c_backend_rewrites_gate_intrinsics_to_runtime_symbols`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_gate_intrinsics_when_clang_available`
