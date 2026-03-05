# M39: LASM Request-Line Tab Separator Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parsing now rejects request lines that use tab separators between tokens.
- Deterministic parser failure message:
  - `invalid request line: tab separators are not allowed`
- Parser-envelope parity remains unchanged (`HTTP.BAD_REQUEST`, `kind=validation`, trace metadata).

## Why

Request-line grammar requires explicit spacing between method, target, and version. Accepting tab separators is unnecessarily permissive and can hide malformed framing.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_tab_separator`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_line_trailing_whitespace`
