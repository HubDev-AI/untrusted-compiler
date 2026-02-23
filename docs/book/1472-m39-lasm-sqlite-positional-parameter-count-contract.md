# M39: LASM sqlite positional parameter-count contract hardening

## What changed

LASM sqlite positional parameter validation now enforces exact parameter count.

Behavior:

- keeps existing “too few params” validation (`requires at least N`),
- for SQL templates with placeholders, now also rejects “too many params” with deterministic runtime error:
  - `sqlite query expects exactly N sql parameters but received M`.
- for zero-placeholder SQL templates, existing compatibility behavior is preserved (extra params are ignored).

DB runtime classification maps this arity error to deterministic `400 DB.*_INVALID`.

## Why

Previously, extra positional params could be accepted and ignored when sqlite statement placeholder count was lower than provided payload size. Strict exact-count validation removes this silent mismatch and keeps DB contracts explicit.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 positional_arity_validation_rejects_extra_params`
- `cargo test -p sec4 --bin sec4 classify_db_runtime_exact_arity_errors_as_validation`
