# M38-S4 Outbound HTTP Oversized-Header Diagnostics

## What it is

M38-S4 hardens outbound runtime HTTP parsing by splitting oversized-response-header failures from generic parse failures.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Before this slice, oversized response headers were folded into generic `NET.RESPONSE_INVALID`, which made runtime triage noisy and less actionable.

This slice introduces a deterministic error code for header-size overflow while keeping existing parser behavior stable.

## How it works

1. Outbound response extraction now returns a dedicated read status when response headers exceed `SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES`.
2. Outbound read-error mapping converts that status into:
   - code: `NET.RESPONSE_HEADERS_TOO_LARGE`
   - message: `outbound http response headers exceed runtime header limit`
3. Existing status mappings for chunked framing, timeout, redirect, and generic invalid responses remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_oversized_headers_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- The runtime still enforces a single global outbound-header byte limit; it does not expose per-request overrides yet.
- New deterministic code improves observability, but callers still need policy/logging wiring for incident aggregation.

## Next

1. Continue M38 with conflicting framing-header diagnostics (`Content-Length` + `Transfer-Encoding`).
2. Keep targeted outbound runtime matrix green (chunked/redirect/tls/header-limit paths).
