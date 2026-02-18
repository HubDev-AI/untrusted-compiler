# M39: LASM Query Percent-Decoding Parity

## Why

LASM `req.query(...)` placeholder materialization existed, but query parsing still used raw key/value segments.

That diverged from established runtime behavior where query keys/values decode percent escapes and `+` to space, while invalid escapes fall back to raw values.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added `decode_lasm_query_component(...)` for deterministic query decoding:
   - `%XX` hex decode,
   - `+` -> space.
2. `split_lasm_path_and_query(...)` now decodes query keys/values before insertion.
3. Invalid percent escapes keep prior practical behavior by falling back to raw key/value segments.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - verifies percent-decoded key (`tra%63e` -> `trace`),
   - verifies mixed value decode (`q%2B7+ok` -> `q+7 ok`),
   - verifies invalid escape fallback (`bad%zz` remains raw).
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_supports_bound_res_text_arguments`
3. `cargo test -p sec4 --test commands run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request`
