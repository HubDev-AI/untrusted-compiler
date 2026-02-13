# 131 M7 Slice: FS Sink Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for filesystem sink calls.

## What it is

Added semantic argument-shape enforcement for:
- `fs.read`:
  - allowed forms: `(fsCap, path)` or `(ctx, fsCap, path)`.
- `fs.write`:
  - allowed forms: `(fsCap, path, value)` or `(ctx, fsCap, path, value)`.

Malformed calls now emit `E4001` with `security` + `sink` tags.

## Why it exists

FS sinks previously tolerated missing path/value arguments in compact forms. Tightening call shape first makes path-safety hardening and capability auditing more deterministic.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_fs_sink_call_shapes(...)`,
2. recognized `fs.read` and `fs.write` sink families via canonical helper predicates,
3. validated compact and context-first arities,
4. emitted deterministic `E4001` sink diagnostics with canonical fix notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_fs_write_missing_args.ut`
    - updated `valid_capabilities_effects.ut`
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - db/fs/net CLI integration test (`compiler/sec4-cli/tests/json_output.rs`)
- Outputs:
  - compile-time rejection of malformed FS sink argument shapes,
  - updated semantic capability fixture to pass explicit path/value arguments.
- Constraint:
  - this slice enforces call-shape only; strict `PathSafe` argument typing remains staged for later hardening.

## Failure modes and diagnostics

Examples:
- `fs.write(fsCap)` ->
  - `E4001`: fs sink call has invalid argument shape.
- `fs.read(ctxOnly)` ->
  - shape diagnostic when required path argument is missing.

## Example usage

```ut
fn writeFile(fs: FsCap) effects { fs.write, fs.read } -> Int {
  fs.write(fs, "/tmp/a", "payload");
  fs.read(fs, "/tmp/a");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: compact placeholder calls without path/payload now fail and must be updated.
- Next:
  - enforce typed `PathSafe` sink arguments for full M8 typed-sink guarantees,
  - align code actions with canonical FS sink call shapes.
