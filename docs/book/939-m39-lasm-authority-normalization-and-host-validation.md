# M39: LASM Authority Normalization and Host Validation

## Why

After adding absolute-form request target support and host parity checks, LASM still accepted malformed `Host` values (for example comma-joined hosts) and treated default-port authority pairs too strictly.

That left avoidable ambiguity in request routing and proxy-style request parsing.

## What Changed

Runtime parser updates in `compiler/sec4-cli/src/main.rs`:

1. Added explicit authority parser for:
   - `Host` header values
   - absolute-form request target authorities (`http://host[:port]/...`)
2. Enforced deterministic `400 invalid host header` when host authority is malformed:
   - host lists (comma-separated)
   - userinfo (`user@host`)
   - malformed or zero port values
   - whitespace/invalid authority forms
3. Kept duplicate-host conflict enforcement, but now based on parsed normalized authority tuples.
4. Upgraded absolute-form host parity matching:
   - compares normalized host names
   - treats default ports as equivalent (`http` -> `80`, `https` -> `443`)
5. Rejects malformed absolute-form authorities deterministically with:
   - `400 invalid request target: malformed absolute-form`

## Behavior Impact

Accepted:

- `GET http://localhost:80/health HTTP/1.1` with `Host: localhost`

Rejected:

- `Host: localhost,example.com` -> `400 invalid host header`
- `GET http://user@localhost/health HTTP/1.1` -> `400 invalid request target: malformed absolute-form`

## Validation

Targeted command integration tests:

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_default_port_equivalence`
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_invalid_host_header_format`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_absolute_form_userinfo_authority`
4. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`
5. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_absolute_form_host_mismatch`
