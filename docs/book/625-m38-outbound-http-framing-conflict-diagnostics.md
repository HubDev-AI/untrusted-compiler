# M38-S5 Outbound HTTP Framing-Conflict Diagnostics

## What it is

M38-S5 hardens outbound runtime HTTP parsing by rejecting conflicting response framing headers.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Responses that combine incompatible framing signals (for example `Transfer-Encoding: chunked` with `Content-Length`) are ambiguous and can cause parser inconsistencies.

This slice adds deterministic rejection and a dedicated error code so failures are explicit and auditable.

## How it works

1. While parsing response headers, runtime tracks framing metadata:
   - transfer-encoding presence and chunked token detection,
   - content-length presence/value.
2. Runtime returns a dedicated read status when:
   - `Transfer-Encoding: chunked` and `Content-Length` appear together, or
   - duplicate `Content-Length` values conflict.
3. Outbound read-error mapping translates that status into:
   - code: `NET.RESPONSE_FRAMING_CONFLICT`
   - message: `outbound http response contains conflicting framing headers`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_conflicting_framing_headers_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Runtime now fails fast on ambiguous framing even if some servers would have been accepted before.
- This is intentional to keep parser behavior deterministic and reduce request-smuggling style ambiguity on outbound dependencies.

## Next

1. Continue M38 with stricter status-line and framing-token diagnostics.
2. Keep outbound runtime hardening covered by focused clang-gated harness tests.
