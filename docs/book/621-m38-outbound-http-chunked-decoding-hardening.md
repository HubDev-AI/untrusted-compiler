# M38-S1 Outbound HTTP Chunked Decoding Hardening

## What it is

M38-S1 hardens runtime outbound HTTP response parsing so `Transfer-Encoding: chunked` responses are decoded to plain body bytes before being tracked and returned.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, outbound response extraction treated payload bytes as raw body data.  
For chunked responses, that meant chunk framing could leak into returned bodies instead of decoded content.

## How it works

`sec4_rt_extract_outbound_http_body(...)` now:

1. Parses headers for:
   - `Transfer-Encoding`
   - `Content-Length`
   - `Location` (existing behavior preserved).
2. If `Transfer-Encoding` contains `chunked`, it decodes chunk frames into output buffer bytes.
3. Enforces limits and malformed-response handling deterministically:
   - invalid chunk framing -> parse failure path,
   - chunk/body size overflow -> resource-limit path.
4. Keeps non-chunked flow unchanged, with `Content-Length` respected when present.

Because both HTTP and HTTPS read paths route through this extractor, chunked decoding applies to both transport variants.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_roundtrip_succeeds_with_env_override_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Decoder currently targets standard chunk framing for v0.1 paths and prioritizes deterministic failure over permissive parsing.
- This improves real interoperability without expanding protocol scope beyond current runtime architecture.

## Next

1. Add M38-S2 coverage for malformed chunk edge cases and trailer-heavy responses.
2. Keep redirect and TLS paths under targeted regression checks during further outbound parser changes.
