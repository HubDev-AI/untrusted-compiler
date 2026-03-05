# M39 - LASM Run Benchmark Response Validation Hardening

## Summary

Hardened LASM benchmark-response materialization so JSON-based handlers fail deterministically when request payloads are invalid instead of silently falling back to static schema envelopes.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - Extended `apply_lasm_dynamic_response_materialization(...)` with validation gates for `DecodeResponse` and `CreateUserResponse`:
     - if JSON is expected and parse fails -> `400` with `JSON.INVALID_SYNTAX` envelope
     - if `id` is missing/invalid UUID v4 -> `400` with `VALIDATION.UUID_INVALID` envelope
   - Kept compatibility fallback for older fixture paths: empty body + no JSON content-type preserves previous static behavior.
   - Added `lasm_request_expects_json(...)` helper to detect JSON-required request shape from headers.

## Why

This removes a correctness gap in LASM run behavior: request-aware benchmark endpoints now enforce basic input validity instead of producing false-positive success envelopes on malformed payloads.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok`
2. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
3. `bash benchmark-suite/scripts/test_run_comparison_matrix.sh`
