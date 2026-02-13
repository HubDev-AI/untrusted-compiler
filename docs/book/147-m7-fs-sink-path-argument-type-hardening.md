# 147 M7 Slice: FS Sink Path-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces typed filesystem paths at FS sinks.

## What it is

Added semantic type enforcement for FS sink path arguments:
- `fs.read` path argument must be `PathSafe`.
- `fs.write` path argument must be `PathSafe`.

This applies to compact and context-first call forms.

Violations emit `E4001` with `security` + `sink` tags.

## Why it exists

FS sink call-shape and context checks were already hardened, but path payloads could still be generic trusted values. Enforcing `PathSafe` path payloads aligns FS sinks with the typed-boundary security model.

## How it works internally

In `enforce_fs_sink_call_shapes(...)`:
1. kept existing arity and context checks,
2. selected path index by call form (`1` compact, `2` context-first),
3. skipped duplicate diagnostics when path value already contains `Untrusted<_>` or `Secret<_>` (covered by sink-flow diagnostics),
4. validated path argument is `PathSafe`,
5. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_fs_read_path_argument_type.ut`
    - `invalid_fs_write_path_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - semantic fixture alignment updates replacing raw path literals with typed `PathSafe` parameters in valid capability fixtures.
- Outputs:
  - compile-time rejection of non-`PathSafe` FS path values,
  - sink-tagged diagnostics aligned with path traversal boundary rules.
- Constraint:
  - this slice updates semantic checks/fixtures only; runtime ABI and C lowering remain unchanged.

## Failure modes and diagnostics

Examples:
- `fs.read(fsCap, 1)` ->
  - `E4001`: fs sink path argument must be `PathSafe`.
- `fs.write(fsCap, 1, value)` ->
  - `E4001`: fs sink path argument must be `PathSafe`.

## Example usage

```ut
fn store(ctx: Ctx, fs: FsCap, path: PathSafe) effects { fs.write } -> Int {
  fs.write(ctx, fs, path, "payload");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted trusted-but-untyped path values now fail semantic analysis.
- Next:
  - continue tightening remaining sink payload contracts (for example header/cookie/value-side strict typing where still permissive) for complete M7 parity.
