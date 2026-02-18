# M39 - LASM Run Benchmark Contract Response Materialization

## Summary

Replaced LASM run-path placeholder JSON response envelopes with request-aware contract materialization for benchmark routes. The LASM backend now derives `/decode`, `/users`, and `/users/:id` JSON bodies from request payloads and path params, with in-process user state for deterministic read-after-write behavior.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - Added shared LASM dynamic response state (`users_by_id`) across worker threads and oneshot flow.
   - Extended `process_lasm_connection_with_runtime(...)` to post-process matched LASM responses using request body and path params.
   - Added deterministic materialization helpers:
     - decode schema -> `{"ok":true,"id":"<payload.id>"}`
     - create schema -> stores payload by id + `{"ok":true,"userId":"<payload.id>"}`
     - user schema -> returns stored payload by `:id`, with deterministic `400` UUID-invalid and `404` not-found envelopes.

2. `benchmark-suite/services/sec4-lasm/smoke.sh`
   - Updated smoke assertions to verify benchmark contract bodies (via `jq`) instead of generic schema envelopes.

3. `benchmark-suite/services/sec4-lasm/README.md`
   - Documented that LASM benchmark service now materializes benchmark JSON contracts from request context.

## Why

This closes a core LASM parity gap: route status/content-type were already deterministic, but response bodies were still static extraction markers. Request-aware materialization makes LASM benchmark output behavior materially closer to real backend semantics and keeps the benchmark lane useful for backend evolution decisions.

## Validation

1. `cargo test -p sec4 --test commands lasm_smoke_command_extracts_res_ok_status_and_body`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok`
3. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
4. `bash benchmark-suite/scripts/test_test_service_contracts.sh`
5. `bash benchmark-suite/scripts/test_run_comparison_matrix.sh`
