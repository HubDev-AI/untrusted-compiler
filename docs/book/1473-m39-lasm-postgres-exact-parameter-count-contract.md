# M39: LASM Postgres exact parameter-count contract

## What changed

LASM Postgres runtime arity validation now enforces exact SQL parameter count for:

- `db.exec`
- `db.execTx`
- `db.queryOne`

Behavior:

- keeps existing “too few params” validation (`requires at least N`),
- for SQL templates with placeholders, now also rejects extra params with deterministic runtime error:
  - `postgres query expects exactly N sql parameters but received M`.
- for zero-placeholder SQL templates, existing compatibility behavior is preserved (extra params are ignored).

DB runtime classifier maps this arity error to deterministic `400 DB.*_INVALID`.

## Why

Before this change, extra Postgres params depended on downstream driver/server mismatch behavior. Exact runtime validation keeps parameter contracts explicit and deterministic across adapters.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 postgres_parameter_arity_rejects_extra_params`
