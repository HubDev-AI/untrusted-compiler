# M38-S51 Outbound HTTP Wrapper Invalid-Handle Parity

## What it is

M38-S51 adds direct runtime harness coverage for invalid capability/handle diagnostics at outbound HTTP wrapper boundaries.

The harness validates deterministic contract parity for:

- `sec4_rt_http_get` (public wrapper) invalid-handle path
- `sec4_rt_http_get_internal` (internal wrapper) invalid-handle path

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S50 locked wrapper invalid-URL pre-parser envelopes. The adjacent contract is missing/zero-handle behavior.

This slice ensures invalid-handle failures remain deterministic and wrapper-local:

- public wrapper -> `NET.GET_INVALID`
- internal wrapper -> `NET.GET_INTERNAL_INVALID`

without drifting into internal-policy deny or parser-class diagnostics.

## How it works

1. Added a new clang-gated harness test:
   - `c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
2. The harness asserts both wrappers for:
   - missing net capability handle (`net=0`)
   - missing URL handle (`url=0`)
3. The harness verifies deterministic envelope shape:
   - `kind=validation`
   - expected wrapper code (`NET.GET_INVALID` or `NET.GET_INTERNAL_INVALID`)
   - no `NET.INTERNAL_DENIED`
   - no parser-class `NET.REQUEST_*` token drift.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused wrapper-contract harness without runtime behavior changes.
- Improves regression localization by separating invalid-handle wrappers from invalid-URL and parser-class contracts.

## Next

1. Add direct precedence coverage for policy-denied internal requests with otherwise valid internal URL handles (`NET.INTERNAL_DENIED`).
2. Keep wrapper invalid-handle, invalid-URL, parser-class, and policy-denial contracts in separate focused harnesses.
