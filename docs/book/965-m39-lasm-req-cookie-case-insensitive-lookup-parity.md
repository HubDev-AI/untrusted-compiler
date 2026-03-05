# M39: LASM `req.cookie` Case-Insensitive Lookup Parity

## Why

The C runtime cookie parser matches cookie names case-insensitively, but LASM cookie lookup used exact case-sensitive matching.

That created backend divergence for handlers and placeholders reading cookie names with different casing.

## What Changed

1. Updated LASM cookie-header parsing to match cookie names case-insensitively (`eq_ignore_ascii_case`) after trimming.
2. Added guard for empty cookie keys after trim in the parser helper.
3. Extended existing LASM req-placeholder integration coverage with a mixed-case cookie-name request (`Cookie: Session=...`) and assertions for header/body materialization.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
