# M39: LASM Request-Line Trailing Whitespace Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parsing now rejects request lines with trailing whitespace after the HTTP version token.
- Deterministic parser failure message:
  - `invalid request line: trailing whitespace is not allowed`
- Parser envelope parity remains unchanged (`HTTP.BAD_REQUEST`, `kind=validation`, trace metadata).

## Why

Permissive request-line tokenization can accept malformed framing with trailing spaces. This hardening keeps request-line syntax strict and deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_trailing_whitespace`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_leading_whitespace`
