# M39: LASM Request-Line Leading Whitespace Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parsing now rejects request lines that begin with whitespace before the method token.
- Deterministic parser failure message:
  - `invalid request line: leading whitespace is not allowed`
- Existing parser-envelope behavior remains intact (`HTTP.BAD_REQUEST`, `kind=validation`, trace metadata).

## Why

Using permissive `split_whitespace()` alone can accept malformed request lines with leading spaces. This hardening keeps request-line framing strict and deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_leading_whitespace`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_method_token`
