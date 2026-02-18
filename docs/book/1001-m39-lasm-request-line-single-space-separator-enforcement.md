# M39: LASM Request-Line Single-Space Separator Enforcement

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parsing now requires exactly single-space separators between method, request-target, and HTTP version tokens.
- Deterministic parser failure message for malformed spacing:
  - `invalid request line: expected single-space separators`
- Existing parser-envelope behavior remains unchanged (`HTTP.BAD_REQUEST`, `kind=validation`, trace metadata).

## Why

Permissive whitespace tokenization can accept malformed request lines with extra separators. Enforcing single-space separation keeps request-line framing deterministic and strict.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_double_space_separator`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_missing_http_version`
