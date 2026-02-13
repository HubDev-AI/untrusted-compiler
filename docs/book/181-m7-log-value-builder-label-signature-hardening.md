# 181 M7 Slice: Log Value Builder Label Signature Hardening

This chapter documents a focused M7 hardening step for log value-builder label arguments.

## What it is

Added compile-time signature checks so:
- `log.event(name)` requires a `String` argument.
- `log.redacted(label)` requires a `String` argument.

Violations emit `E4001` diagnostics tagged with `security`.

## Why it exists

Typed log sinks (`log.info/warn/error/emit`) now require `LogValue` payloads. Hardening core builders for event names and redaction labels prevents numeric placeholder drift and keeps structured logging APIs predictable.

## How it works internally

In `semantic.rs`, `enforce_log_value_builder_signatures(...)` now validates:
1. `log.event` arity/type contract,
2. `log.redacted` arity/type contract.

The helper runs in the same semantic pass as other intrinsic signature checks.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_log_event_argument_type.ai`
    - `invalid_log_redacted_argument_type.ai`
  - tag tests in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - log-intrinsic and log-builder integration fixtures in:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - non-string event/redaction labels are rejected at compile time.
  - log integration fixtures use string labels and continue to pass through `c-bin`.
- Constraint:
  - this slice hardens label typing only; richer `log.field/log.obj` structural validation remains deferred.

## Failure modes and diagnostics

Examples:
- `log.event(1)` -> `E4001`: log.event argument must be `String`.
- `log.redacted(1)` -> `E4001`: log.redacted argument must be `String`.

## Example usage

```ailang
fn main() -> Int {
  let event = log.event("user.created");
  let redacted = log.redacted("token");
  event;
  redacted;
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge fixtures/examples using numeric labels now fail and must use strings.
- Next:
  - payload signature hardening for `log.str`/`log.i64`/`log.bool`/`log.field`/`log.obj` is implemented in `182-m7-log-value-builder-payload-signature-hardening.md`.
