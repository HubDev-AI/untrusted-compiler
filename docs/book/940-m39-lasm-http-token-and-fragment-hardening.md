# M39: LASM HTTP Token and Fragment Hardening

## Why

LASM request parsing already validated version, host, and framing contracts, but still accepted a few malformed HTTP shapes:

1. Invalid method tokens in the request line.
2. Header names with surrounding whitespace or invalid token characters.
3. Request targets containing URI fragments (`#...`), which should not appear in HTTP request targets.

Those are parser-level contracts and should fail deterministically before route/runtime execution.

## What Changed

`compiler/sec4-cli/src/main.rs`:

1. Added token validation helper for HTTP method/header-name tokens (RFC token-compatible character set).
2. Added deterministic method token rejection:
   - `400 invalid request line: invalid method token`
3. Hardened header line parsing:
   - rejects whitespace around header names before `:`
   - rejects invalid header-name token characters
   - deterministic diagnostics:
     - `invalid header line: whitespace around header name`
     - `invalid header line: invalid header name token`
4. Added request target fragment rejection (except `*` form):
   - `400 invalid request target: fragment is not allowed`

## Validation

Focused command integration tests:

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_method_token`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_header_name_whitespace`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_request_target_fragment`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_request_target`
