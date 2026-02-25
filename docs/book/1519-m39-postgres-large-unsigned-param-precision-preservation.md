# 1519 M39 Slice: Postgres Large Unsigned Param Precision Preservation

## What changed

1. Updated `parse_lasm_postgres_query_param_value(...)` in `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`:
   - JSON numeric params now check `as_u64()` before `as_f64()`,
   - values above signed 64-bit range are preserved as exact text params.
2. Added unit coverage:
   - `positional_object_params_preserve_large_unsigned_integer_as_text`.

## Why

Large unsigned integers could previously flow through `f64` conversion, which can lose integer precision. Preserving these values as exact text avoids lossy conversion while keeping deterministic runtime parameterization.

## Validation

1. `cargo test -p sec4 --bin sec4 positional_object_params_preserve_large_unsigned_integer_as_text -- --nocapture`
