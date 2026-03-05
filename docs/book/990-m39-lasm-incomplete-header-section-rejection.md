# M39: LASM Incomplete Header-Section Rejection

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-head parser now rejects EOF while reading header lines when the header terminator blank line is missing.
- Deterministic failure mapping:
  - `400 Bad Request`
  - message: `incomplete request while reading header line`

## Why

Previously, EOF during header parsing could be treated as normal header-section completion, which accepted truncated request framing. This hardening makes request framing strict and deterministic.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_incomplete_header_section`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_431_when_header_limit_is_exceeded`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_rejects_chunk_size_line_exceeding_header_limit`
