# M39: LASM Content-Length Upsert Correctness

## Why

LASM response writing previously inserted `Content-Length` only when missing.

If handler code set `content-length` manually (including invalid values), LASM could emit incorrect framing headers that did not match the actual body size.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Switched `Content-Length` handling from insert-if-missing to case-insensitive upsert.
2. Response writer now always emits `Content-Length` derived from `response.body.len()` regardless of user-supplied header casing/value.
3. Existing case-insensitive header normalization/merge remains in place, so output has one deterministic content-length header.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_overrides_user_content_length_with_actual_body_size`
   - handler sets `content-length: 999`,
   - verifies LASM emits a single content-length header with actual body size (`4` for `pong`).
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_merges_header_names_case_insensitively`
   - verifies broader case-insensitive header merge behavior remains correct.
