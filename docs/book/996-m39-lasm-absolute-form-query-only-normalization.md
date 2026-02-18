# M39: LASM Absolute-Form Query-Only Normalization

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-target normalization for absolute-form requests now preserves query-only targets with no explicit path.
- `http://localhost?expand=full` is now normalized as `/?expand=full` (instead of `/`), keeping query parameters available downstream.
- Added command integration coverage that materializes `req.query("expand")` from this request form.

## Why

Without this normalization, absolute-form query-only requests dropped query data before route/query extraction, which broke query-driven handlers even when the request line was otherwise valid.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_preserves_query_for_absolute_form_without_explicit_path`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target_with_uppercase_scheme`
