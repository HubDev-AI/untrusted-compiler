# 404 M16 Slice: Runtime Trace Correlation Header and Error Envelope Sync

This chapter documents M16-S9: adding deterministic runtime trace correlation across HTTP headers and structured error payloads.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` that introduces request-scoped trace IDs and propagates them through:

- response header: `X-Trace-Id`
- standard error envelope field: `error.traceId`

## Why it exists

Structured error envelopes were already available in M16-S8, but trace IDs were static placeholders. That prevented practical correlation between response headers and error payloads during debugging and deterministic replay workflows.

## Implementation details

1. Added request trace-id state to runtime request context.
2. Added deterministic per-request trace-id assignment (`rt-<n>`) in the HTTP handler path.
3. Updated response serialization to include:
   - `X-Trace-Id: <trace-id>`
4. Updated standard error response helper to inject current request trace-id into error JSON payload.

## Validation

Integration assertions in `compiler/sec4-cli/tests/json_output.rs` now verify:

- success response contains header `X-Trace-Id: rt-1`,
- invalid JSON error response contains `\"traceId\":\"rt-1\"`.

Targeted runtime test suite remains green with trace-correlation checks enabled.

## Tradeoffs and next steps

- Current trace IDs are deterministic runtime-local counters, which is ideal for tests.
- Future slices can map these to externally provided request IDs and propagate them into logging/capture artifacts for full observability parity.
