# 103 M7 Slice: CSRF IssueToken Intrinsic Bootstrap

This chapter documents the next M7 bootstrap slice: bridging `csrf.issueToken` through semantic analysis, C lowering, and runtime ABI stubs.

## `csrf.issueToken` Helper Intrinsic

### What it is

Added intrinsic support for:
- `csrf.issueToken` / `csrf_issue_token`

It lowers to runtime ABI stub:
- `ailang_rt_csrf_issue_token`

### Why it exists

The CSRF middleware spec includes token issuance helpers. Without bridge support, bootstrap services cannot exercise this API shape through `c-bin` builds.

### How it works internally

- Added semantic intrinsic spec with `net` effect in `semantic.rs`.
- Added C backend rewrite mapping in `c_backend.rs`.
- Added runtime C declaration/definition in `runtime/c/`.
- Added CLI and core tests validating runtime symbol lowering.
- Added security-map call tagging for `effect.net` visibility.
- Updated `examples/hello-api` handler flow to call `csrf.issueToken`.

### Inputs, outputs, and constraints

- Input: context-like argument payloads (bootstrap currently uses placeholders).
- Output: runtime stub return lowered into C output.
- Constraints:
  - runtime CSRF token semantics remain placeholder in M7 bootstrap mode.
  - this slice validates compile-path continuity, not final token generation behavior.

### Failure modes and diagnostics

- Missing `net` effect declarations now fail semantic checks when `csrf.issueToken` is used.
- Misspelled helper names can fail semantic/C compile stages.

### Example usage

```ailang
fn issue(ctx: Int) effects { net } -> Int {
  csrf.issueToken(ctx);
  0
}
```

### Tradeoffs and next steps

- This keeps CSRF helper APIs available in the bootstrapped toolchain while runtime remains stubbed.
- Next steps are real token generation/storage semantics and middleware enforcement integration.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - added `c_backend_rewrites_csrf_issue_token_intrinsic_to_runtime_symbol`
  - runtime ABI assertions include `ailang_rt_csrf_issue_token`
- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_csrf_issue_token_intrinsic_when_clang_available`
- `compiler/ailang-core/src/security_map.rs`
  - added `csrf.issueToken` call-tag mapping to `effect.net`
