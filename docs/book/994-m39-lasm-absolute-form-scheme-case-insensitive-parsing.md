# M39: LASM Absolute-Form Scheme Case-Insensitive Parsing

Date: 2026-02-18
Milestone: M39-S2B (LASM async backend bootstrap)

## What changed

- LASM request-target absolute-form parsing now accepts case-insensitive `http` and `https` schemes.
- Requests like `GET HTTP://host/path HTTP/1.1` and `GET HTTPS://host/path HTTP/1.1` now parse through the absolute-form path instead of being rejected as invalid targets.
- Existing authority/`Host` parity checks remain unchanged and still reject mismatches deterministically.

## Why

URI scheme matching is case-insensitive. Lowercase-only matching caused valid absolute-form requests with uppercase or mixed-case schemes to fail before route resolution.

## Validation

- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target_with_uppercase_scheme`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_accepts_absolute_form_request_target`
- `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_absolute_form_host_mismatch`
