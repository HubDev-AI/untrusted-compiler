# 91 M6 Slice: Secrets Intrinsic Runtime Lowering

This chapter documents the next M6 vertical slice: lowering secrets intrinsics to runtime ABI symbols.

## Secrets Intrinsic Rewriting

### What it is

C emission now rewrites:
- `secrets.get(...)` / `secret_read(...)`
- `secrets.reveal(...)` / `secret_reveal(...)`

to runtime symbols:
- `ailang_rt_secret_get(...)`
- `ailang_rt_secret_reveal(...)`

Runtime ABI now includes stubs for both functions.

### Why it exists

Secrets APIs are part of AILang’s security-first core model. This slice keeps backend intrinsic coverage aligned with semantic capabilities/effects, even while runtime behavior remains stubbed in M6.

### How it works internally

- Added replacements in `lower_c_expr(...)` for dotted and underscore secrets intrinsic spellings.
- Added `ailang_rt_secret_get` and `ailang_rt_secret_reveal` declarations/definitions under `runtime/c/`.
- Rewritten calls flow through existing emission paths automatically.

### Inputs, outputs, and constraints

- Input: MIR expressions containing secrets intrinsic call names.
- Output: C-valid runtime calls for secrets APIs.
- Constraints:
  - runtime behavior is placeholder.
  - policy still governs semantic allowability (for example `secrets.reveal` is forbidden by default policy).

### Failure modes and diagnostics

- `secrets.reveal` remains policy-gated and may fail semantic checks before backend.
- Missing `SecretsCap` or missing effect declarations still fail semantic validation (`E2003`, `E2001`, policy diagnostics).

### Example usage

```ailang
fn readSecret(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec, 1);
  0
}
```

### Tradeoffs and next steps

- This slice focuses on C backend call lowering, not secret-store integration.
- `secrets.reveal` runtime stub exists for backend completeness, while policy continues to control real use.
- Next step: wire runtime secret adapters and preserve redaction/reveal policy boundaries in M7+.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - new `c_backend_rewrites_secret_intrinsics_to_runtime_symbols`
- `compiler/ailang-cli/tests/json_output.rs`
  - new `build_emit_c_bin_handles_secret_read_intrinsic_when_clang_available`
