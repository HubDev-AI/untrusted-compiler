# 162 M7 Slice: CSP Builder Signature Hardening

This chapter documents a focused M7 hardening step for CSP builder helper signatures.

## What it is

Added semantic checks for CSP builders:
- `sec.csp()` must be called with zero arguments,
- `sec.cspAdd(policy, directive, sources)` now enforces:
  - `policy` is `CspPolicy`,
  - `directive` is `String`,
  - `sources` is `String` (bridge-stage representation).

Violations emit `E4001` with `security` tags.

## Why it exists

CSP configuration is security-critical. Permissive placeholder arguments can hide broken policy construction and reduce confidence in default security-header posture.

## How it works internally

In semantic trust-gate enforcement:
1. detect `sec.csp` and `sec.cspAdd` calls,
2. enforce `sec.csp` no-arg shape,
3. enforce `sec.cspAdd` arity and typed arguments,
4. emit deterministic diagnostics for malformed CSP builder usage.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_sec_csp_argument_shape.ut`
    - `invalid_sec_csp_add_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed CSP builder calls,
  - stable diagnostics for tooling and CI.
- Constraint:
  - `sources` remains a bridge-stage string descriptor until richer CSP source-list types are introduced.

## Failure modes and diagnostics

Examples:
- `sec.csp(1)` ->
  - `E4001`: sec.csp expects no arguments.
- `sec.cspAdd(csp, 1, 2)` ->
  - `E4001`: directive/sources arguments must be `String`.

## Example usage

```ut
fn cspPolicy() -> Int {
  let p = sec.csp();
  sec.cspAdd(p, "default-src", "'self'");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder CSP builder arguments used in early bootstrap fixtures now fail semantic checks.
- Next:
  - evolve `sources` from string descriptors toward typed CSP source-list builders.
