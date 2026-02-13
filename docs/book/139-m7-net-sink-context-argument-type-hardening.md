# 139 M7 Slice: Net Sink Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for net sink intrinsics.

## What it is

Added semantic type enforcement for context-first net sink calls:
- `httpClient.get(ctx, netCap, url)` requires argument 1 to be `Ctx`.
- `httpClient.getInternal(ctx, internalNetCap, url)` requires argument 1 to be `Ctx`.

Violations now emit `E4001` with `security` + `sink` tags.

## Why it exists

Net sink call-shape contracts already constrained arity, but context-first forms still accepted non-context placeholders in slot 1. This update keeps outbound network pathways consistent with explicit typed context semantics.

## How it works internally

In `enforce_net_sink_call_shapes(...)`:
1. kept existing arity checks,
2. added context-first branch for 3-argument net sink calls,
3. validated `arg_types[0]` is `Ctx`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_net_public_context_argument_type.ut`
    - `invalid_net_internal_context_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first net sink calls,
  - sink-tagged diagnostics aligned with security-first tooling surfaces.
- Constraint:
  - this slice tightens semantic checks only; runtime ABI and C lowering stay unchanged.

## Failure modes and diagnostics

Examples:
- `httpClient.get(1, netCap, url)` ->
  - `E4001`: net sink context argument must be `Ctx`.
- `httpClient.getInternal(1, internalNetCap, url)` ->
  - `E4001`: net sink context argument must be `Ctx`.

## Example usage

```ut
fn fetch(ctx: Ctx, net: NetCap, url: PublicUrl) effects { net } -> Int {
  httpClient.get(ctx, net, url);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: context-first net sink calls with non-`Ctx` slot-1 values now fail semantic analysis.
- Next:
  - apply the same context-first type enforcement to FS sink and secret-source/reveal helpers for full capability-family parity.
