# 106 M7 Slice: Path/Header/Redact Helper Bridge

This chapter documents the next M7 bootstrap slice: bridging additional safe helper APIs through semantic analysis, C lowering, runtime ABI, and security metadata.

## Added Helper Intrinsics

### What it is

Added intrinsic support for:
- `path.base` / `path_base`
- `headers.name` / `headers_name`
- `headers.value` / `headers_value`
- `secrets.redact` / `secret_redact`

Lowered runtime ABI stubs:
- `ailang_rt_path_base`
- `ailang_rt_headers_name`
- `ailang_rt_headers_value`
- `ailang_rt_secret_redact`

### Why it exists

These helpers are part of the v0 security-first stdlib surface. Without runtime bridge coverage, spec-shaped code using path/header builders and secret redaction would not compile cleanly through the `c-bin` path.

### How it works internally

- Added semantic intrinsic entries and return types in `semantic.rs`:
  - `path.base` -> `PathSafe`
  - `headers.name` -> `HeaderName`
  - `headers.value` -> `HeaderValue`
  - `secrets.redact` -> `String`
- Added `headers` intrinsic namespace recognition.
- Extended C backend intrinsic rewriting in `c_backend.rs`.
- Added runtime C declarations/definitions in `runtime/c/`.
- Extended `security_map` tag mapping:
  - `gate.path.base`
  - `gate.header.name`
  - `gate.header.value`
  - `gate.secret.redact`
- Extended core and CLI integration tests.

### Inputs, outputs, and constraints

- Input: helper calls in compile-path AILang code.
- Output: generated C calling runtime helper stubs and successful `c-bin` build/run.
- Constraints:
  - helper runtime semantics are still placeholder stubs in M7 bootstrap mode.
  - this slice validates API-shape and bridge completeness, not final runtime validation behavior.

### Failure modes and diagnostics

- Misspelled helper names can fail semantic/C compile stages.
- Security semantics for these helpers are still partial until behavior-level runtime slices land.

### Example usage

```ailang
fn createUser(origin: Untrusted<String>) -> Int {
  let base = path.base(1);
  headers.name(1);
  headers.value(1);
  secrets.redact(1);
  path.under(base, origin);
  0
}
```

### Tradeoffs and next steps

- This closes another API-surface gap without blocking on full runtime behavior.
- Next steps are behavior-level validation semantics and stricter sink/gate argument typing.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - header/source runtime symbol assertions include new helper symbols
  - gate and secret intrinsic rewrite tests include new helper calls
- `compiler/ailang-cli/tests/json_output.rs`
  - gate and secret `c-bin` integration fixtures assert new helper symbol lowering
