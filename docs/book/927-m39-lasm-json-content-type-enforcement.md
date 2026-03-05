# M39 - LASM JSON Content-Type Enforcement

## Summary

Added deterministic content-type enforcement for JSON-schema benchmark handlers in LASM run materialization.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - For `DecodeResponse` and `CreateUserResponse` materialization paths:
     - when request body is non-empty and `Content-Type` is not JSON, LASM now returns
       - `400`
       - `HTTP.BAD_REQUEST`
       - message: `content-type must be application/json`
   - Existing compatibility fallback is preserved for empty-body legacy fixture traffic.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added integration test:
     - `run_command_oneshot_lasm_backend_returns_400_for_non_json_content_type_payload`

## Why

This aligns LASM JSON handler behavior with explicit request-content expectations and removes silent parsing for non-JSON payload declarations.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_non_json_content_type_payload`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_user_payload`
3. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
