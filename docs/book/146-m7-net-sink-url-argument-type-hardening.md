# 146 M7 Slice: Net Sink URL-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces typed URL payloads for net sinks.

## What it is

Added semantic type enforcement for net sink URL arguments:
- `httpClient.get` URL argument must be `PublicUrl`.
- `httpClient.getInternal` URL argument must be `InternalUrl`.

This applies to both compact and context-first call forms.

Violations emit `E4001` with `security` + `sink` tags.

## Why it exists

Net sink call-shape and context typing were already hardened, but URL payloads could still be generic trusted values. Enforcing typed URL payloads aligns net sinks with the security-first typed-sink baseline.

## How it works internally

In `enforce_net_sink_call_shapes(...)`:
1. kept existing arity and context checks,
2. selected URL index by call form (`1` compact, `2` context-first),
3. selected expected type by sink (`PublicUrl` vs `InternalUrl`),
4. skipped duplicate diagnostics when URL value already contains `Untrusted<_>` or `Secret<_>` (covered by sink-flow diagnostics),
5. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_net_public_url_argument_type.ai`
    - `invalid_net_internal_url_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - semantic fixture alignment updates replacing raw URL literals with typed URL parameters in valid capability fixtures.
- Outputs:
  - compile-time rejection of non-typed net sink URL values,
  - sink-tagged diagnostics aligned with URL trust-boundary rules.
- Constraint:
  - this slice updates semantic checks and fixtures only; runtime ABI and C lowering remain unchanged.

## Failure modes and diagnostics

Examples:
- `httpClient.get(netCap, 1)` ->
  - `E4001`: net sink URL argument must be `PublicUrl`.
- `httpClient.getInternal(internalCap, 1)` ->
  - `E4001`: net sink URL argument must be `InternalUrl`.

## Example usage

```ailang
fn fetch(ctx: Ctx, net: NetCap, url: PublicUrl) effects { net } -> Int {
  httpClient.get(ctx, net, url);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted trusted-but-untyped URL payloads now fail semantic analysis.
- Next:
  - enforce typed `PathSafe` payloads for filesystem sinks to complete sink payload type parity across DB/net/fs surfaces.
