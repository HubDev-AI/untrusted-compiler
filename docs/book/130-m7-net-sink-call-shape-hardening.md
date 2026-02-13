# 130 M7 Slice: Net Sink Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for outbound network sink calls.

## What it is

Added semantic argument-shape enforcement for:
- `httpClient.get`:
  - allowed forms: `(netCap, url)` or `(ctx, netCap, url)`.
- `httpClient.getInternal`:
  - allowed forms: `(internalNetCap, url)` or `(ctx, internalNetCap, url)`.

Malformed calls now emit `E4001` with `security` + `sink` tags.

## Why it exists

Net sinks previously tolerated missing URL arguments in compact-form calls. That weakens sink intent and makes SSRF-oriented URL typing harder to enforce incrementally.

This slice locks call shape first, while preserving current URL type permissiveness for staged hardening.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_net_sink_call_shapes(...)`,
2. recognized public/internal net sink call families via canonical helper predicates,
3. validated compact vs context-first arities,
4. emitted deterministic `E4001` sink diagnostics with fix notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures for missing-URL net calls
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - DB/fs/net CLI integration test fixture (`compiler/sec4-cli/tests/json_output.rs`)
- Outputs:
  - compile-time rejection of malformed net sink argument shapes,
  - updated semantic capability fixture to include explicit URL argument.
- Constraint:
  - this slice is call-shape hardening only; strict `PublicUrl`/`InternalUrl` sink typing remains staged separately.

## Failure modes and diagnostics

Examples:
- `httpClient.get(netCap)` ->
  - `E4001`: net sink call has invalid argument shape.
- `httpClient.getInternal(ctxOnly, capOnly)` ->
  - shape diagnostic when URL argument is missing.

## Example usage

```ut
fn fetch(net: NetCap, internal: InternalNetCap) effects { net } -> Int {
  httpClient.get(net, "https://example.com");
  httpClient.getInternal(internal, "http://internal.local");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive fixtures without URL arguments now fail and need explicit sink inputs.
- Next:
  - enforce typed URL sink arguments (`PublicUrl`/`InternalUrl`) for full M8 sink-type guarantees,
  - align code actions with canonical net sink call shapes.
