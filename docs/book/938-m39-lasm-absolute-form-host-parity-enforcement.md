# M39 - LASM Absolute-Form Host Parity Enforcement

## Summary

Hardened LASM absolute-form request parsing by enforcing parity between request-target authority and `Host` header.

## What Changed

1. `compiler/sec4-cli/src/main.rs`
   - `read_lasm_http_request(...)` now captures absolute-form authority (`http://authority/path`).
   - When both absolute-form authority and `Host` header are present and differ, parser now fails deterministically:
     - status: `400`
     - message: `host header does not match request target authority`
   - Existing absolute-form normalization remains intact for matching authority inputs.

2. `compiler/sec4-cli/tests/commands.rs`
   - Added `run_command_oneshot_lasm_backend_returns_400_for_absolute_form_host_mismatch`.

## Why

Absolute-form targets are common in proxy-style traffic. Enforcing authority/host parity prevents ambiguous host-resolution semantics and keeps LASM parser behavior deterministic.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_absolute_form_host_mismatch`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_host_headers`
4. `bash benchmark-suite/services/sec4-lasm/smoke.sh`
