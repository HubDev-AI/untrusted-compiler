# M39: LASM Deterministic Read Error Mapping

## Why

LASM request parsing used OS-dependent I/O error strings in some request-read failures.

That risked non-deterministic diagnostics across machines and made malformed-byte handling less explicit.

## What Changed

In `compiler/sec4-cli/src/main.rs` (`read_lasm_http_request`):

1. Replaced generic `could not {stage}: {err}` mapping with deterministic error buckets:
   - timeout / would-block -> `408 request read timeout while {stage}`
   - invalid-data -> `400 invalid request encoding while {stage}`
   - unexpected EOF -> `400 incomplete request while {stage}`
   - all other read errors -> `400 could not {stage}`
2. Preserved existing timeout behavior contract and status code semantics.

## Behavior Impact

Malformed non-UTF8 header bytes now fail deterministically with:

- `400 Bad Request`
- body message containing `invalid request encoding while reading header line`

instead of platform-specific OS error strings.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_header_encoding`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_408_when_request_read_times_out`
