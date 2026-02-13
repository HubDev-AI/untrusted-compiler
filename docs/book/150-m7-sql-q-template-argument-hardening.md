# 150 M7 Slice: sql.q Template-Argument Hardening

This chapter documents a focused M7 hardening step for `sql.q` signature enforcement.

## What it is

Added semantic checks for `sql.q`:
- call shape must be exactly `sql.q(template, params)`,
- template argument must be `String`.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`sql.q` is the construction boundary for typed SQL. Allowing non-string template placeholders (for example numeric literals) weakens the API contract and makes malformed query construction look valid at compile time.

## How it works internally

In semantic call enforcement:
1. detect `sql.q` intrinsic calls,
2. enforce exact two-argument shape,
3. enforce `String` type for the template argument,
4. emit deterministic `E4001` diagnostics for invalid shape or template type.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_sql_q_template_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - CLI/C backend integration fixture alignment:
    - `compiler/ailang-cli/tests/json_output.rs`
    - `compiler/ailang-core/tests/c_backend.rs`
- Outputs:
  - compile-time rejection of malformed `sql.q` template arguments,
  - stable schema-tagged diagnostics for tooling.
- Constraint:
  - this slice hardens semantic contracts only; runtime SQL ABI remains unchanged.

## Failure modes and diagnostics

Example:
- `sql.q(1, params)` ->
  - `E4001`: sql.q template argument must be `String`.

## Example usage

```ailang
fn buildQuery() -> SqlQuery {
  sql.q("SELECT 1", 2)
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder-style `sql.q(1, 2)` examples no longer pass semantic checks.
- Next:
  - reject `Secret<_>`/`Untrusted<_>` SQL param payloads (`173-m7-sql-q-params-value-safety-hardening.md`).
  - tighten second-argument typing toward a dedicated parameter-list/`DbParam` surface once that type model is finalized.
