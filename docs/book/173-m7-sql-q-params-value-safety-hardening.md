# 173 M7 Slice: sql.q Params Value-Safety Hardening

This chapter documents a focused M7 hardening step for `sql.q` parameter safety.

## What it is

Extended `sql.q` checks so:
- call shape remains `sql.q(template, params)`,
- params argument cannot contain `Secret<_>` or `Untrusted<_>`.

Violations emit `E4001` with security tags (`secret`/`taint`).

## Why it exists

`sql.q` is the query-construction boundary before values become `SqlQuery`. Rejecting secret/untrusted payload wrappers at this point prevents silent trust-boundary bypass through SQL parameter packaging.

## How it works internally

In `enforce_sql_q_signature`:
1. preserve existing template/arity checks,
2. inspect param argument type for `Secret`/`Untrusted` wrappers,
3. emit tagged diagnostics with canonical remediation notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_sql_q_params_secret.ut`
    - `invalid_sql_q_params_untrusted.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of secret/untrusted SQL params in `sql.q`,
  - stable diagnostics for CLI/editor feedback.
- Constraint:
  - this slice does not yet introduce a dedicated `DbParam` aggregate type.

## Failure modes and diagnostics

Examples:
- `sql.q("SELECT 1", secret)` ->
  - `E4001`: sql.q params argument cannot be `Secret<_>`.
- `sql.q("SELECT 1", untrusted)` ->
  - `E4001`: sql.q params argument cannot be `Untrusted<_>`.

## Example usage

```ut
fn buildQuery(userId: String) -> SqlQuery {
  sql.q("SELECT * FROM users WHERE id = $1", userId)
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage code that forwarded wrapped secret/untrusted values directly into `sql.q` now fails semantic checks.
- Next:
  - define stricter second-argument typing around explicit SQL param container types.
