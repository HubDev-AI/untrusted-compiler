# M39: LASM Duplicate Query Key First-Value Parity

## Why

The C runtime request-query map keeps the first value for duplicate keys, but LASM parsing was overwriting with the last value.

That created backend divergence for handlers using `req.query("...")`.

## What Changed

1. Updated LASM query parsing in `split_lasm_path_and_query` to keep the first observed value for each query key.
2. Extended the existing LASM req-placeholder integration test to send a duplicate-key query request (`trace=first&trace=second`) and assert deterministic first-value behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
