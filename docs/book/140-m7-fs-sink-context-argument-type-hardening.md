# 140 M7 Slice: FS Sink Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for FS sink intrinsics.

## What it is

Added semantic type enforcement for context-first FS sink calls:
- `fs.read(ctx, fsCap, path)` requires argument 1 to be `Ctx`.
- `fs.write(ctx, fsCap, path, value)` requires argument 1 to be `Ctx`.

Violations now emit `E4001` with `security` + `sink` tags.

## Why it exists

FS sink call-shape checks already constrained arity, but context-first forms still accepted non-context placeholders in slot 1. This update keeps filesystem boundary calls consistent with typed context semantics used across the security-first surface.

## How it works internally

In `enforce_fs_sink_call_shapes(...)`:
1. kept existing arity checks,
2. added context-first branches for 3/4-argument FS sink calls,
3. validated `arg_types[0]` is `Ctx`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_fs_read_context_argument_type.ai`
    - `invalid_fs_write_context_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first FS sink calls,
  - sink-tagged diagnostics aligned with tooling and audit expectations.
- Constraint:
  - this slice updates semantic validation only; runtime ABI and C lowering remain unchanged.

## Failure modes and diagnostics

Examples:
- `fs.read(1, fsCap, path)` ->
  - `E4001`: fs sink context argument must be `Ctx`.
- `fs.write(1, fsCap, path, value)` ->
  - `E4001`: fs sink context argument must be `Ctx`.

## Example usage

```ailang
fn load(ctx: Ctx, fs: FsCap, path: PathSafe) effects { fs.read } -> Int {
  fs.read(ctx, fs, path);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: context-first FS sink calls with placeholder slot-1 values now fail semantic checks.
- Next:
  - extend the same context-first type enforcement to secret-source/reveal helper families for full capability-call parity.
