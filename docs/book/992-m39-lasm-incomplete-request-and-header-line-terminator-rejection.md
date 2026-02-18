# M39: LASM Incomplete Request/Header Line Terminator Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parsing now rejects request lines that do not end with a newline terminator.
- LASM request-head parsing now rejects header lines that do not end with a newline terminator.
- Deterministic failure mapping:
  - `400 Bad Request`
  - message: `incomplete request while reading request line`
  - message: `incomplete request while reading header line`

## Why

Without explicit newline-terminator checks, EOF-truncated start-lines and header lines could be parsed as complete tokens before a later read failed. This hardening rejects truncated framing at the exact stage where truncation occurs.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_incomplete_request_line_terminator`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_incomplete_header_line_terminator`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_incomplete_header_section`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_incomplete_chunk_size_line`
