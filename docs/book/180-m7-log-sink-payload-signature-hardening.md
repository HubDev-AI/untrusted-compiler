# 180 M7 Slice: Log Sink Payload Signature Hardening

This chapter documents a focused M7 hardening step for typed log sink payloads.

## What it is

Added compile-time signature checks so:
- `log.info(payload)`
- `log.warn(payload)`
- `log.error(payload)`
- `log.emit(payload)`

require exactly one payload argument and that payload must be `LogValue`.

Violations emit `E4001` with `security` tags.

## Why it exists

The runtime bridge allowed loosely typed payloads for log sinks. This slice enforces the security baseline rule that logging APIs are structured and typed by default, reducing accidental stringly/non-structured logging paths.

## How it works internally

In `semantic.rs`, `enforce_log_sink_signatures(...)` now runs for log sinks:
1. enforces exactly one sink argument,
2. requires `LogValue` payload type,
3. skips signature mismatch diagnostics for `Secret<_>` / `Untrusted<_>` payloads so existing sink-flow diagnostics remain the single primary error.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture `invalid_log_sink_payload_argument_type.ai`
  - tag test coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - log intrinsic integration fixtures in:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection for non-`LogValue` log sink payloads.
  - existing secret/taint sink diagnostics remain unchanged.
- Constraint:
  - this slice hardens sink signatures only; runtime logging behavior remains bridge-mode/no-op.

## Failure modes and diagnostics

Examples:
- `log.info(1)` -> `E4001`: log sink argument must be `LogValue`.
- `log.info(secret)` still reports existing secret sink diagnostic (`E1003`) rather than duplicate signature errors.

## Example usage

```ailang
fn main() effects { log } -> Int {
  log.info(log.event(1));
  0
}
```

## Tradeoffs and next steps

- Tradeoff: older placeholder fixtures that logged numeric literals now require structured `LogValue` construction.
- Next:
  - tighten `log.event`/`log.field` payload typing once event-shape constructors are fully typed.
