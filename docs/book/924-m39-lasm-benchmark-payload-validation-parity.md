# M39 - LASM Benchmark Payload Validation Parity

## Summary

Extended LASM benchmark response materialization to validate full benchmark user payload shape before returning decode/create success responses.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - Added `validate_lasm_benchmark_user_payload(...)` for benchmark contract checks:
     - `id` must be UUID v4
     - `email` must be a valid email-like value
     - `age` must be integer `0..=150`
     - `tags` must be array length `<= 16` with each string length `1..32`
     - `address.zip` must be digits length `4..10`
     - `meta.flags` must include boolean `a`, `b`, `c`
   - Wired decode/create dynamic materialization to return deterministic `400 VALIDATION.INVALID` (or `VALIDATION.UUID_INVALID`) when validation fails.
   - Kept compatibility fallback for empty-body non-JSON fixture traffic.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added focused integration test `run_command_oneshot_lasm_backend_returns_400_for_invalid_user_payload`.

## Why

Benchmark comparability requires LASM lane to reject malformed user payloads consistently with the benchmark endpoint contract, not only malformed JSON syntax.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_json_payload`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_user_payload`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
5. `bash benchmark-suite/scripts/test_run_comparison_matrix.sh`
